//! Static "certifiability" checker for `--cbc`: verifies that a compiled
//! circuit, as actually instantiated with its concrete template parameters,
//! satisfies (H1) No Free Assignments, (H2) No Bare Tags and (H3) Trusted
//! Composition. See the plan for the full design; this file is filled in
//! incrementally, one hypothesis at a time.

use crate::execution_data::{ExecutedTemplate, NodePointer};
use program_structure::ast::{Access, AssignOp, Expression, Meta, SignalType, Statement, VariableType};
use program_structure::wire_data::{WireData, WireType};
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::generate_file_location;
use program_structure::program_archive::ProgramArchive;
use program_structure::utils::constants::UsefulConstants;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct CertificationResult {
    pub errors: ReportCollection,
    pub warnings: ReportCollection,
    // Non-trusted instances that passed H1+H2 on their own body.
    pub certified: Vec<String>,
    // Instances skipped entirely (H3 black box), by report name.
    pub trusted: Vec<String>,
    // Non-trusted instances that failed, with their own violations.
    pub failed: Vec<(String, ReportCollection)>,
    // report_name -> that template's declared output signal names, snapshotted
    // here because `program`/`model` are consumed by `export()` before the
    // structural warnings `absorb_structural_warnings` reads even exist.
    output_signals: HashMap<String, HashSet<String>>,
}

impl CertificationResult {
    pub fn is_certifiable(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn print_summary(&self) {
        use ansi_term::Colour;
        println!();
        println!("{}", Colour::White.bold().paint("Certification report"));
        println!("  {} {}", Colour::Green.paint("Certified:"), self.certified.len());
        println!("  {} {}", Colour::Cyan.paint("Trusted (circomlib):"), self.trusted.len());
        for name in &self.trusted {
            println!("    - {}", name);
        }
        println!("  {} {}", Colour::Red.paint("Not certifiable:"), self.failed.len());
        for (name, errs) in &self.failed {
            println!("    - {} ({} error(s)):", name, errs.len());
            for err in errs {
                println!("        [{}] {}", err.get_code(), err.get_message());
            }
        }
    }

    // Promotes DAG-level "unconstrained signal" warnings (the same analysis
    // `--inspect` reports, re-run here regardless of whether `--inspect` was
    // passed) into certification failures. Needed because H1/H2 alone only
    // catch a signal that was touched dishonestly -- not one that is simply
    // never touched at all, which they cannot see without knowing an array's
    // concrete size (see certifier_examples/arrays/known_limitation_*).
    //
    // No DAG/AST index plumbing is needed to attribute a warning to a node:
    // `insert_in_dag` calls `dag.add_node(self.report_name.clone(), ...)`
    // (executed_template.rs), so `Node.template_name` IS the exact
    // `report_name` string already used as the key here, and every one of
    // `constraint_correctness_analysis`'s four message shapes starts with
    // `In template "<report_name>": ...` -- so a warning's owner is read
    // directly off its own message, not out of some parallel index.
    pub fn absorb_structural_warnings(&mut self, warnings: &[Report]) {
        for warning in warnings {
            let Some(template) = extract_template_name(warning.get_message()) else { continue };
            if self.trusted.iter().any(|t| t == template) {
                // Same black-box exemption as H1/H2: we don't check inside a
                // trusted node's own body at all.
                continue;
            }
            if matches!(warning.get_code(), ReportCode::UnconstrainedIOSignal) {
                // A father not using every one of a subcomponent's outputs is
                // legitimate and common: never a certification failure.
                continue;
            }
            if matches!(warning.get_code(), ReportCode::UnconstrainedSignal) {
                let Some(signal) = extract_local_signal_name(warning.get_message()) else { continue };
                // `arr[1].x` -> `arr`: both the bus field and the array
                // index have to go before comparing with declared outputs.
                let top_level = signal.split(|c| c == '.' || c == '[').next().unwrap_or(signal);
                let is_output =
                    self.output_signals.get(template).map(|o| o.contains(top_level)).unwrap_or(false);
                if !is_output {
                    // Unconstrained input or intermediate signal: allowed,
                    // treated as verified. Only an unconstrained OUTPUT is a
                    // real gap.
                    continue;
                }
            }
            let promoted = Report::error(warning.get_message().clone(), *warning.get_code());
            self.errors.push(promoted.clone());
            if let Some((_, errs)) = self.failed.iter_mut().find(|(name, _)| name == template) {
                errs.push(promoted);
            } else {
                self.certified.retain(|name| name != template);
                self.failed.push((template.to_string(), vec![promoted]));
            }
        }
    }
}

fn extract_template_name(message: &str) -> Option<&str> {
    let rest = message.strip_prefix("In template \"")?;
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn extract_local_signal_name(message: &str) -> Option<&str> {
    let rest = message.strip_prefix("In template \"")?;
    let end = rest.find('"')?;
    let rest = rest[end + 1..].strip_prefix(": ")?;
    if let Some(rest) = rest.strip_prefix("Local signal ") {
        return rest.strip_suffix(" does not appear in any constraint");
    }
    if let Some(rest) = rest.strip_prefix("Array of local signals ") {
        return rest.split(" contains a total of ").next();
    }
    None
}

pub fn certify(model: &[ExecutedTemplate], program: &ProgramArchive) -> CertificationResult {
    let mut errors = ReportCollection::new();
    let warnings = ReportCollection::new();
    let trusted_nodes = trusted_instances(model, program, &known_template_hashes());

    let mut certified = Vec::new();
    let mut trusted = Vec::new();
    let mut failed = Vec::new();
    let mut output_signals = HashMap::new();
    for (node, exe) in model.iter().enumerate() {
        output_signals.insert(
            exe.report_name.clone(),
            program.get_template_data(&exe.template_name).get_outputs().keys().cloned().collect(),
        );
        if trusted_nodes[node] {
            // (H3) Trusted subtree root: treated as a black box, exactly like
            // a circomlib2 `TrustedComponent` axiom -- H1/H2 are never
            // checked on its own body (nor, since every descendant is
            // trusted too, on any of theirs).
            trusted.push(exe.report_name.clone());
            continue;
        }
        let mut node_errors = ReportCollection::new();
        check_h1(&exe.code, &exe.report_name, &mut node_errors);
        check_h2(exe, model, program, &mut node_errors);
        if node_errors.is_empty() {
            certified.push(exe.report_name.clone());
        } else {
            errors.extend(node_errors.iter().cloned());
            failed.push((exe.report_name.clone(), node_errors));
        }
    }
    CertificationResult { errors, warnings, certified, trusted, failed, output_signals }
}

// ---------------------------------------------------------------------------
// (H3) Trusted Composition
// ---------------------------------------------------------------------------
//
// A template is a trusted (H3) black box iff its own DEFINITION -- the
// `template Name(...) { ... }` text, normalized by `normalize_definition` --
// hashes to the same SHA-256 as the circomlib2 template of the same name. No
// opt-in flag needed (no `--trusted-path`): every template that is part of the
// compilation is eligible, automatically, wherever `include` pulled it from.
// This is deliberately per-TEMPLATE, not per-file: editing one template of a
// vendored circomlib2 file leaves every other template of that file trusted,
// and a template is never trusted just because of the file it lives in.
//
// Matching its own hash is necessary but not sufficient: an instance is
// trusted only if its definition matches AND every subcomponent it
// instantiates is trusted too, recursively -- i.e. iff its whole subtree of
// descendants is pristine circomlib2. A circomlib2 template is audited
// together with the circomlib2 templates it composes, so a `LessThan` whose
// `Num2Bits` has been replaced is no longer the audited `LessThan`, even
// though its own text is untouched: both lose trust and both are checked
// under H1/H2. An instance that isn't trusted falls through to ordinary
// H1/H2 checking (fail closed: a real circomlib2 template body almost always
// uses '<--', so a tampered or non-vendored one will simply fail H1 on its
// own -- no separate rejection logic needed).
//
// Generated with `--cbc --cbc-refresh` from
// https://github.com/iden3/circomlib.git, branch `circomlib_buses`, commit
// aef239c53279535128b86303dc571d9750bcfa0e (2026-10-22), `circuits/`
// directory. Same `<hash>  <path>::<template>` format as
// `LOCAL_HASH_CACHE_FILE`, so regenerating it is just copying that file here.
const KNOWN_CIRCOMLIB2_TEMPLATE_HASHES: &str = include_str!("circomlib2_template_hashes.txt");

// (template name, hash) pairs. The name is part of the key because circomlib2
// reuses names across files (e.g. `Pedersen` in pedersen.circom and in
// pedersen_old.circom), and a body should only be trusted under its own name.
type KnownHashes = HashSet<(String, String)>;

// Once a local refresh cache exists, it is used EXCLUSIVELY, never merged
// with the embedded table: the whole point of `--cbc-refresh` is to replace a
// possibly stale/compromised embedded snapshot with the current canonical
// truth, so an old embedded hash for a template that has since been patched
// upstream (e.g. a security fix) must stop being trusted, not stay trusted
// forever as a silent fallback. Only when no refresh has ever happened for
// this project (no cache file at all) do we fall back to the hashes baked
// into the compiler at build time.
fn known_template_hashes() -> KnownHashes {
    match std::fs::read_to_string(LOCAL_HASH_CACHE_FILE) {
        Ok(refreshed) => parse_hash_table(&refreshed),
        Err(_) => parse_hash_table(KNOWN_CIRCOMLIB2_TEMPLATE_HASHES),
    }
}

fn parse_hash_table(content: &str) -> KnownHashes {
    content
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let hash = fields.next()?;
            let (_, name) = fields.next()?.rsplit_once("::")?;
            Some((name.to_string(), hash.to_string()))
        })
        .collect()
}

// Trust of every instance in `model` (indexed like it). The instance graph
// is acyclic, and memoized per node so shared subtrees are only walked once.
fn trusted_instances(model: &[ExecutedTemplate], program: &ProgramArchive, known: &KnownHashes) -> Vec<bool> {
    let mut own_match = HashMap::new();
    let mut memo = vec![None; model.len()];
    for node in 0..model.len() {
        subtree_is_trusted(node, model, program, known, &mut own_match, &mut memo);
    }
    memo.into_iter().map(|trusted| trusted.unwrap_or(false)).collect()
}

fn subtree_is_trusted(
    node: NodePointer,
    model: &[ExecutedTemplate],
    program: &ProgramArchive,
    known: &KnownHashes,
    own_match: &mut HashMap<String, bool>,
    memo: &mut Vec<Option<bool>>,
) -> bool {
    if let Some(trusted) = memo[node] {
        return trusted;
    }
    let exe = &model[node];
    let trusted = definition_matches(exe, program, known, own_match)
        && exe
            .subcomponent_edges()
            .all(|(_, child)| subtree_is_trusted(child, model, program, known, own_match, memo));
    memo[node] = Some(trusted);
    trusted
}

// The same template is often instantiated with different parameters --
// cache per template name so each definition is only hashed once.
fn definition_matches(
    exe: &ExecutedTemplate,
    program: &ProgramArchive,
    known: &KnownHashes,
    cache: &mut HashMap<String, bool>,
) -> bool {
    if let Some(matches) = cache.get(&exe.template_name) {
        return *matches;
    }
    let matches = match definition_hash(program, &exe.template_name) {
        Some(hash) => known.contains(&(exe.template_name.clone(), hash)),
        None => false,
    };
    cache.insert(exe.template_name.clone(), matches);
    matches
}

// Hashes the definition straight from the source the parser already read
// (`file_library`), not from disk again, so what is hashed is exactly what
// was compiled.
fn definition_hash(program: &ProgramArchive, template_name: &str) -> Option<String> {
    let data = program.get_template_data(template_name);
    let source = program.file_library.to_storage().get(data.get_file_id())?.source();
    Some(hash_definition(source.get(data.get_definition_location())?))
}

fn hash_definition(definition: &str) -> String {
    format!("{:x}", Sha256::digest(normalize_definition(definition).as_bytes()))
}

// Comments are dropped (with the same rules as the parser's `preprocess`, so
// `//` inside a string is a comment here too) and every run of whitespace,
// comments included, becomes a single space. Re-indenting, reflowing or
// re-commenting a circomlib2 template keeps it trusted; any change to its
// tokens does not.
fn normalize_definition(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut pending_space = false;
    while let Some(c) = chars.next() {
        let is_space = if c == '/' && chars.peek() == Some(&'/') {
            for c in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
            true
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = ' ';
            for c in chars.by_ref() {
                if previous == '*' && c == '/' {
                    break;
                }
                previous = c;
            }
            true
        } else {
            c.is_whitespace()
        };
        if is_space {
            pending_space = true;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Refreshing the embedded hashes from the real repository (`--cbc
// --cbc-refresh`)
// ---------------------------------------------------------------------------
//
// The embedded hashes are frozen at whatever commit was vendored when this
// binary was built; `circomlib_buses` will keep moving. `--cbc-refresh`
// re-clones it fresh, parses every current file with the compiler's own
// parser, hashes every template definition it finds, and writes them to a
// local cache file next to wherever `circom` is invoked from --
// `known_template_hashes` then uses that cache instead of the embedded table,
// so refreshed hashes take effect immediately without recompiling the
// compiler itself.
const CIRCOMLIB2_REPO_URL: &str = "https://github.com/iden3/circomlib.git";
const CIRCOMLIB2_BRANCH: &str = "circomlib_buses";
const LOCAL_HASH_CACHE_FILE: &str = ".circomlib2_template_hashes";

// Shells out to `git` (required -- fails loudly if it isn't installed, since
// this was explicitly requested), clones `circomlib_buses` into a scratch
// directory, hashes every template defined in a `.circom` file under
// `circuits/`, and overwrites `LOCAL_HASH_CACHE_FILE` with the fresh set, in a
// `sha256sum`-style `<hash>  <path>::<template>` format for easy manual
// inspection/diffing.
pub fn refresh_circomlib2_cache(prime: &str) -> Result<(), String> {
    let scratch_dir = std::env::temp_dir().join(format!("circomlib2_refresh_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch_dir);

    let status = std::process::Command::new("git")
        .args(["clone", "--quiet", "--depth", "1", "--branch", CIRCOMLIB2_BRANCH, CIRCOMLIB2_REPO_URL])
        .arg(&scratch_dir)
        .status();
    let status = match status {
        Ok(status) => status,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(
                "`git` was not found in PATH -- it is required by --cbc-refresh to fetch \
                 the real circomlib2 repository."
                    .to_string(),
            );
        }
        Err(err) => return Err(format!("failed to run `git clone`: {}", err)),
    };
    if !status.success() {
        let _ = std::fs::remove_dir_all(&scratch_dir);
        return Err(format!(
            "`git clone {} --branch {}` failed",
            CIRCOMLIB2_REPO_URL, CIRCOMLIB2_BRANCH
        ));
    }

    let circuits_dir = scratch_dir.join("circuits");
    let mut files = Vec::new();
    let collected = collect_circom_files(&circuits_dir, &circuits_dir, &mut files);
    if collected.is_none() {
        let _ = std::fs::remove_dir_all(&scratch_dir);
        return Err(format!("could not read {} after cloning", circuits_dir.display()));
    }
    files.sort();

    let result = hash_circomlib_files(&files, prime);
    let _ = std::fs::remove_dir_all(&scratch_dir);
    let (cache_content, template_count) = result?;
    std::fs::write(LOCAL_HASH_CACHE_FILE, cache_content)
        .map_err(|err| format!("failed to write {}: {}", LOCAL_HASH_CACHE_FILE, err))?;

    println!(
        "{}",
        ansi_term::Colour::Green.paint(format!(
            "Refreshed {} with {} circomlib2 ({}) template hashes from {} files.",
            LOCAL_HASH_CACHE_FILE,
            template_count,
            CIRCOMLIB2_BRANCH,
            files.len()
        ))
    );
    Ok(())
}

fn hash_circomlib_files(files: &[(String, PathBuf)], prime: &str) -> Result<(String, usize), String> {
    let field = UsefulConstants::new(&prime.to_string()).get_p().clone();
    let mut cache_content = String::new();
    let mut template_count = 0;
    for (relative_path, absolute_path) in files {
        let source = std::fs::read_to_string(absolute_path)
            .map_err(|err| format!("failed to read {}: {}", absolute_path.display(), err))?;
        let definitions = parser::template_definitions(&source, &field)
            .map_err(|_| format!("failed to parse circuits/{}", relative_path))?;
        for (name, location) in definitions {
            let definition = source
                .get(location)
                .ok_or_else(|| format!("bad template range in circuits/{}", relative_path))?;
            cache_content.push_str(&format!("{}  circuits/{}::{}\n", hash_definition(definition), relative_path, name));
            template_count += 1;
        }
    }
    Ok((cache_content, template_count))
}

fn collect_circom_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Option<()> {
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            collect_circom_files(root, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("circom") {
            let relative = path.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");
            out.push((relative, path));
        }
    }
    Some(())
}

// ---------------------------------------------------------------------------
// (H1) No Free Assignments
// ---------------------------------------------------------------------------

// `exe.code` is already pruned to the branches this concrete instantiation
// actually takes (`filters::apply_unused`), so walking it directly respects
// param-dependent `if`s without needing to know which branch was taken.
fn check_h1(stmt: &Statement, instance_name: &str, errors: &mut ReportCollection) {
    use Statement::*;
    match stmt {
        Substitution { meta, op: AssignOp::AssignSignal, var, .. } => {
            errors.push(free_assignment_report(instance_name, var, meta));
        }
        UnderscoreSubstitution { meta, op: AssignOp::AssignSignal, .. } => {
            errors.push(free_assignment_report(instance_name, "_", meta));
        }
        // Not reachable in practice post-execution (tuple substitutions are
        // rewritten away before this point), kept for defensive completeness.
        MultSubstitution { meta, op: AssignOp::AssignSignal, .. } => {
            errors.push(free_assignment_report(instance_name, "<tuple>", meta));
        }
        IfThenElse { if_case, else_case, .. } => {
            check_h1(if_case, instance_name, errors);
            if let Some(else_case) = else_case {
                check_h1(else_case, instance_name, errors);
            }
        }
        While { stmt, .. } => check_h1(stmt, instance_name, errors),
        InitializationBlock { initializations, .. } => {
            for s in initializations {
                check_h1(s, instance_name, errors);
            }
        }
        Block { stmts, .. } => {
            for s in stmts {
                check_h1(s, instance_name, errors);
            }
        }
        _ => {}
    }
}

fn free_assignment_report(instance_name: &str, signal: &str, meta: &Meta) -> Report {
    let mut report = Report::error(
        format!(
            "(H1) In {}: signal '{}' is fixed with the unconstrained operator '<--'/'-->'; \
             a certifiable circuit must fix every signal with '<=='/'==>' instead.",
            instance_name, signal
        ),
        ReportCode::FreeAssignmentDetected,
    );
    let location = generate_file_location(meta.start, meta.end);
    report.add_primary(location, meta.get_file_id(), "unconstrained assignment here".to_string());
    report
}

// ---------------------------------------------------------------------------
// (H2) No Bare Tags
// ---------------------------------------------------------------------------
//
// Single sequential walk over `exe.code`, in the same order the compiler's own
// runtime tag propagation (`execute.rs`) would process it. A tagged
// declaration starts out "pending"; it only becomes a legitimate ("verified")
// tag source once its own `<==`/`==>` is shown to copy from an already
// verified source (mirrors the syntactic restriction the compiler itself
// applies at runtime: only a bare `Variable` RHS on a `<==` propagates tags).
// Anything left pending at the end of the walk never had a legitimate source
// and is reported as a bare tag.

struct H2Context<'a> {
    model: &'a [ExecutedTemplate],
    program: &'a ProgramArchive,
    instance_name: &'a str,
    subcomponents: HashMap<&'a str, NodePointer>,
    // Any known bus-typed path (a local top-level variable, or a dotted
    // nested-field path like "p.inner") -> the name of the bus type it holds.
    // Lets us resolve `path.field` against `BusData` however deep it goes.
    bus_types: HashMap<String, String>,
    pending: HashMap<String, (Vec<String>, Meta)>,
    verified: HashMap<String, HashSet<String>>,
    // Paths permanently disqualified by at least one dishonest touch. Circom
    // itself only confirms an array's tag "if it inherits the tag in all
    // positions" (`assignment_utils.rs`), and a bus-typed local variable's
    // fields are tracked per DECLARED NAME, not per array cell -- so several
    // `Substitution`s can legitimately target the same collapsed path (each
    // element of an array of signals, or of bus instances, or of a nested
    // bus-typed field). Once ANY of them is dishonest, the whole path is
    // poisoned for good, regardless of order; it is reported exactly once,
    // at the first offending touch, and never re-verified afterwards.
    dishonest: HashSet<String>,
    errors: &'a mut ReportCollection,
}

fn check_h2<'a>(
    exe: &'a ExecutedTemplate,
    model: &'a [ExecutedTemplate],
    program: &'a ProgramArchive,
    errors: &'a mut ReportCollection,
) {
    let subcomponents: HashMap<&str, NodePointer> = exe.subcomponent_edges().collect();
    let mut ctx = H2Context {
        model,
        program,
        instance_name: &exe.report_name,
        subcomponents,
        bus_types: HashMap::new(),
        pending: HashMap::new(),
        verified: HashMap::new(),
        dishonest: HashSet::new(),
        errors,
    };
    ctx.walk(&exe.code);
    ctx.finalize();
}

impl<'a> H2Context<'a> {
    fn walk(&mut self, stmt: &Statement) {
        use Statement::*;
        match stmt {
            Declaration { meta, xtype, name, .. } => self.on_declaration(meta, xtype, name),
            Substitution { meta, var, access, op, rhe } => {
                self.on_substitution(meta, var, access, *op, rhe)
            }
            IfThenElse { if_case, else_case, .. } => {
                self.walk(if_case);
                if let Some(else_case) = else_case {
                    self.walk(else_case);
                }
            }
            While { stmt, .. } => self.walk(stmt),
            InitializationBlock { initializations, .. } => {
                for s in initializations {
                    self.walk(s);
                }
            }
            Block { stmts, .. } => {
                for s in stmts {
                    self.walk(s);
                }
            }
            _ => {}
        }
    }

    fn on_declaration(&mut self, meta: &Meta, xtype: &VariableType, name: &str) {
        let (bus_type, stype, tags) = match xtype {
            VariableType::Signal(stype, tags) => (None, *stype, tags.as_slice()),
            VariableType::Bus(bus_type, stype, tags) => (Some(bus_type.as_str()), *stype, tags.as_slice()),
            _ => return, // Var/Component: not tag-bearing.
        };

        if tags.is_empty() {
            self.verified.insert(name.to_string(), HashSet::new());
        } else if stype == SignalType::Input {
            // Inputs carry their declared tags axiomatically (this is exactly
            // the paper's `x |= Req` hypothesis) -- nothing to verify here.
            self.verified.insert(name.to_string(), tags.iter().cloned().collect());
        } else {
            self.pending.insert(name.to_string(), (tags.to_vec(), meta.clone()));
        }

        // A bus carries not just its own top-level tags but a whole nested
        // structure of per-field tags declared once in its TYPE (`BusData`,
        // not in this `Declaration`) -- seed/track that whole subtree too,
        // exactly mirroring the top-level treatment above.
        if let Some(bus_type) = bus_type {
            self.bus_types.insert(name.to_string(), bus_type.to_string());
            if stype == SignalType::Input {
                self.seed_verified_subtree(name, bus_type);
            } else {
                self.seed_pending_subtree(name, bus_type, meta);
            }
        }
    }

    // Recursively marks every (possibly nested) field of `bus_type` as
    // axiomatically verified, carrying exactly its own declared tags -- used
    // for input buses (trusted wholesale, same axiom as a plain tagged input)
    // and for whole-bus copies from a trusted subcomponent's output.
    fn seed_verified_subtree(&mut self, base_path: &str, bus_type: &str) {
        let bus_data = self.program.get_bus_data(bus_type);
        for (field_name, wire) in bus_data.get_fields() {
            let path = format!("{}.{}", base_path, field_name);
            self.verified.insert(path.clone(), wire.get_tags().clone());
            if let WireType::Bus(nested_type) = wire.get_type() {
                self.bus_types.insert(path.clone(), nested_type.clone());
                self.seed_verified_subtree(&path, &nested_type);
            }
        }
    }

    // Mirrors `seed_verified_subtree` for a NON-input bus: every tagged field
    // starts out pending its own honest `<==`, exactly like a plain tagged
    // signal declaration, since there is no separate per-field `Declaration`
    // node to hook into (field tags live once in the bus TYPE, not per use).
    fn seed_pending_subtree(&mut self, base_path: &str, bus_type: &str, meta: &Meta) {
        let bus_data = self.program.get_bus_data(bus_type);
        for (field_name, wire) in bus_data.get_fields() {
            let path = format!("{}.{}", base_path, field_name);
            if !wire.get_tags().is_empty() {
                self.pending.insert(path.clone(), (wire.get_tags().iter().cloned().collect(), meta.clone()));
            }
            if let WireType::Bus(nested_type) = wire.get_type() {
                self.bus_types.insert(path.clone(), nested_type.clone());
                self.seed_pending_subtree(&path, &nested_type, meta);
            }
        }
    }

    fn on_substitution(
        &mut self,
        meta: &Meta,
        var: &str,
        access: &[Access],
        op: AssignOp,
        rhe: &Expression,
    ) {
        if first_component_access(access).is_some() {
            if self.subcomponents.contains_key(var) {
                // `var.field <== rhe` feeding a subcomponent's declared input:
                // circom's own runtime tag propagation already enforces that
                // `rhe` carries whatever tags the callee requires there -- a
                // hard compile error otherwise (`execute.rs:1734`,
                // `AssignmentMissingTags`), so a circuit that reached this
                // checker already satisfies it. Nothing left to check.
                return;
            }
            // Otherwise `var` is a local bus and this is `var.field <== rhe`
            // (possibly nested further, `var.field.subfield <== rhe`) -- a
            // genuine local tag claim on that field, not covered by circom's
            // subcomponent-input check above.
            let Some((path, _, _)) = self.resolve_access(var, access) else { return };
            self.handle_target(meta, path, op, rhe);
            return;
        }
        // Plain signal, whole-bus, or array-indexed assignment: tags apply to
        // the whole declared variable, regardless of which element.
        self.handle_target(meta, var.to_string(), op, rhe);
    }

    // Checks (and records) the honesty of an assignment into `path`, which
    // may be a plain signal, a whole bus, or one specific (possibly nested)
    // bus field. `path` collapses array indices (an array of signals, of bus
    // instances, or a nested array-typed bus field all share one path), so
    // this can run more than once for the same `path` -- see `dishonest` for
    // why that is handled by accumulating, not by only looking at the first.
    fn handle_target(&mut self, meta: &Meta, path: String, op: AssignOp, rhe: &Expression) {
        if self.dishonest.contains(&path) {
            // Already permanently disqualified by an earlier touch (e.g. a
            // different array index of the same declared path); already
            // reported once, and no later touch can rescue it.
            return;
        }

        // The tags this exact touch must justify: either still-untouched
        // (`pending`, first touch ever) or already provisionally established
        // by an earlier touch to the same collapsed path (`verified`) --
        // re-checked here in case THIS touch is the one that breaks it.
        let tags = self
            .pending
            .get(&path)
            .map(|(t, _)| t.clone())
            .or_else(|| self.verified.get(&path).filter(|t| !t.is_empty()).map(|t| t.iter().cloned().collect()));

        let Some(tags) = tags else {
            // No tag requirement recorded at all for `path` -- either
            // untagged, or (if `path` is itself a bus) still needs the
            // propagation-only treatment below, exactly as circom's own
            // `TagWire` propagation carries a whole nested `fields` map, not
            // just the top-level tags, through a bare-Variable `<==`.
            if op == AssignOp::AssignConstraintSignal && self.bus_types.contains_key(&path) {
                self.record_legitimate_copy(&path, HashSet::new(), rhe);
            }
            return;
        };

        self.pending.remove(&path);
        let legitimate = op == AssignOp::AssignConstraintSignal && self.honest_source(rhe, &tags);
        if legitimate {
            self.record_legitimate_copy(&path, tags.into_iter().collect(), rhe);
        } else {
            self.verified.remove(&path);
            self.dishonest.insert(path.clone());
            self.errors.push(bare_tag_report(self.instance_name, &path, &tags, meta));
        }
    }

    // Records that `path` (a plain signal, a whole bus, or a specific nested
    // bus field) just earned its tags honestly, AND -- since circom's own
    // runtime tag propagation copies a bus's whole nested structure on a
    // bare-Variable `<==`, not just its top-level tags -- propagates that
    // same structure into `path`'s own field subtree, if it has one.
    fn record_legitimate_copy(&mut self, path: &str, tags: HashSet<String>, rhe: &Expression) {
        self.verified.insert(path.to_string(), tags);
        let Some(bus_type) = self.bus_types.get(path).cloned() else { return };
        // Whatever this copy is about to establish supersedes any of `path`'s
        // own fields still waiting on a (now moot) direct field assignment.
        let prefix = format!("{}.", path);
        self.pending.retain(|k, _| !k.starts_with(&prefix));

        let Expression::Variable { name: src, access, .. } = rhe else { return };
        if access.is_empty() {
            // Local bus-to-bus copy: propagate the SOURCE's own already-PROVEN
            // sub-tree (not just what its type declares -- `src` may itself
            // still have unresolved fields at this point in the walk).
            let src_prefix = format!("{}.", src);
            let copies: Vec<(String, HashSet<String>)> = self
                .verified
                .iter()
                .filter_map(|(k, v)| k.strip_prefix(src_prefix.as_str()).map(|suf| (suf.to_string(), v.clone())))
                .collect();
            for (suffix, carried) in copies {
                self.verified.insert(format!("{}.{}", path, suffix), carried);
            }
        } else if self.subcomponents.contains_key(src.as_str()) {
            // Whole-bus output of a subcomponent: trust the callee's declared
            // structure recursively, exactly like any other component promise
            // (Assumption 1) -- its own honesty is checked independently.
            self.seed_verified_subtree(path, &bus_type);
        }
    }

    // Walks `base.access[0].access[1]...` (skipping array indices) against
    // whichever bus type is in scope at each step -- the callee template's
    // own declared port for the first hop through a subcomponent, `BusData`
    // for every hop through a (possibly nested) local bus field. Returns the
    // dotted key used for `pending`/`verified`, the tags found there, and
    // whether the chain started at a subcomponent (its tags are a DECLARED
    // promise to trust directly, per Assumption 1) or a local bus (its tags
    // are only a requirement -- honesty still has to come from `verified`).
    fn resolve_access(&self, base: &str, access: &[Access]) -> Option<(String, HashSet<String>, bool)> {
        let mut hops = access.iter().filter_map(|a| match a {
            Access::ComponentAccess(name) => Some(name.as_str()),
            Access::ArrayAccess(_) => None,
        });
        let first = hops.next()?;

        let (mut path, mut tags, mut nested_bus, is_component) =
            if let Some(node) = self.subcomponents.get(base) {
                let template = self.program.get_template_data(&self.model[*node].template_name);
                let wire = template.get_output_info(first).or_else(|| template.get_input_info(first))?;
                (format!("{}.{}", base, first), wire.get_tags().clone(), bus_type_of(wire), true)
            } else if let Some(bus_type) = self.bus_types.get(base) {
                let wire = self.program.get_bus_data(bus_type).get_field_info(first)?;
                (format!("{}.{}", base, first), wire.get_tags().clone(), bus_type_of(wire), false)
            } else {
                return None;
            };

        for hop in hops {
            let wire = self.program.get_bus_data(&nested_bus?).get_field_info(hop)?;
            tags = wire.get_tags().clone();
            nested_bus = bus_type_of(wire);
            path.push('.');
            path.push_str(hop);
        }

        Some((path, tags, is_component))
    }

    // A tag claim on `rhe` is honest iff it is a bare `Variable` (no
    // arithmetic, no call) pointing at a source that already, legitimately,
    // carries every tag being claimed.
    fn honest_source(&self, rhe: &Expression, tags: &[String]) -> bool {
        let Expression::Variable { name, access, .. } = rhe else { return false };
        let required: HashSet<&str> = tags.iter().map(String::as_str).collect();

        if first_component_access(access).is_none() {
            // Plain variable (or array-indexed): same declaration-level tags
            // regardless of which element.
            return match self.verified.get(name) {
                Some(carried) => required.iter().all(|t| carried.contains(*t)),
                None => false,
            };
        }

        let Some((path, declared_or_verified, is_component)) = self.resolve_access(name, access) else {
            return false;
        };
        if is_component {
            // Assumption 1: trust the callee's own declared promise directly.
            required.iter().all(|t| declared_or_verified.contains(*t))
        } else {
            match self.verified.get(&path) {
                Some(carried) => required.iter().all(|t| carried.contains(*t)),
                None => false,
            }
        }
    }

    fn finalize(&mut self) {
        // Declared with tags, but never legitimately (re)assigned at all.
        for (name, (tags, meta)) in std::mem::take(&mut self.pending) {
            self.errors.push(bare_tag_report(self.instance_name, &name, &tags, &meta));
        }
    }
}

fn first_component_access<'a>(access: &'a [Access]) -> Option<&'a str> {
    access.iter().find_map(|a| match a {
        Access::ComponentAccess(name) => Some(name.as_str()),
        Access::ArrayAccess(_) => None,
    })
}

fn bus_type_of(wire: &WireData) -> Option<String> {
    match wire.get_type() {
        WireType::Bus(name) => Some(name),
        WireType::Signal => None,
    }
}

fn bare_tag_report(instance_name: &str, signal: &str, tags: &[String], meta: &Meta) -> Report {
    let mut report = Report::error(
        format!(
            "(H2) In {}: signal '{}' declares tag(s) {:?} without an honest source \
             (must be copied via '<=='/'==>' from an already-tagged signal or from the \
             tagged output of a trusted/certifiable subcomponent).",
            instance_name, signal, tags
        ),
        ReportCode::BareTagDetected,
    );
    let location = generate_file_location(meta.start, meta.end);
    report.add_primary(location, meta.get_file_id(), "bare tag declared here".to_string());
    report
}
