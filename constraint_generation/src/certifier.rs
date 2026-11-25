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
            if matches!(warning.get_code(), ReportCode::UnconstrainedIOSignal) {
                // A father not using every one of a subcomponent's outputs is
                // legitimate and common: never a certification failure.
                continue;
            }
            let is_trusted = self.trusted.iter().any(|t| t == template);
            let is_output = matches!(warning.get_code(), ReportCode::UnconstrainedSignal)
                && extract_local_signal_name(warning.get_message()).map_or(false, |signal| {
                    // `arr[1].x` -> `arr`: both the bus field and the array
                    // index have to go before comparing with declared outputs.
                    let top_level = signal.split(|c| c == '.' || c == '[').next().unwrap_or(signal);
                    self.output_signals.get(template).map(|o| o.contains(top_level)).unwrap_or(false)
                });
            if !is_output {
                // An unconstrained input or intermediate cannot make any
                // output non-deterministic: allowed. Anything else inside a
                // trusted node is part of the black box (same exemption as
                // H1/H2).
                if is_trusted || matches!(warning.get_code(), ReportCode::UnconstrainedSignal) {
                    continue;
                }
            }
            // An output that appears in no constraint is never acceptable,
            // not even inside a trusted circomlib2 template: no audit can
            // vouch for a wire that the constraint system does not mention.
            let mut promoted = Report::error(warning.get_message().clone(), *warning.get_code());
            if is_output {
                promoted.add_note(
                    "An output that appears in no constraint is not determined by the inputs: \
                     any value satisfies the circuit, so it is NOT safe (under-constrained)."
                        .to_string(),
                );
            }
            if is_trusted {
                promoted.add_note(
                    "This happens inside a template trusted as circomlib2: trust never covers an \
                     unconstrained output, so it is reported anyway."
                        .to_string(),
                );
                self.trusted.retain(|name| name != template);
            }
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
    let mut warnings = ReportCollection::new();
    let trusted_nodes = trusted_instances(model, program, &known_template_hashes(), &mut warnings);

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
// A template is also audited together with the FUNCTIONS it calls and the
// BUS types it uses (e.g. `LessThan`'s guard `assert(n <= maxbits()-2)`, or
// the fields `Num2Bin` writes in `BinaryNumber`): editing `maxbits()` or
// `BinaryNumber` changes what the template does without touching its text.
// So the table also holds one hash per circomlib2 function and bus, and a
// template only matches if, besides its own text, every function and bus in
// its static closure (identifiers in its text, transitively through those
// functions' and buses' own texts) matches too. Subtemplates are covered by
// the per-instance recursion above. When that is what breaks trust, a CW01
// warning names the modified dependency.
//
// Generated with `CBC_CIRCOMLIB2_DIR=<checkout> circom ... --cbc --cbc-refresh`
// from https://github.com/iden3/circomlib.git, branch `circomlib_buses`,
// checked out at commit aef239c53279535128b86303dc571d9750bcfa0e
// (2026-10-22), `circuits/` directory: 111 templates, 20 functions, 5 buses. Same format as `LOCAL_HASH_CACHE_FILE` (see `HASH_TABLE_HEADER`),
// so regenerating it is just copying that file here.
const KNOWN_CIRCOMLIB2_TEMPLATE_HASHES: &str = include_str!("circomlib2_template_hashes.txt");

// (key, hash) pairs, where the key is `Name` for a template, `function:name`
// for a function and `bus:Name` for a bus. The name is part of the key
// because circomlib2 reuses names across files (e.g. `Pedersen` in
// pedersen.circom and in pedersen_old.circom), and a body should only be
// trusted under its own name.
type KnownHashes = HashSet<(String, String)>;

// First line of every hash table. A table without it predates function and
// bus hashes: with it, every template that calls a function or uses a bus
// would silently lose trust, so such a stale cache is ignored (with a
// warning) instead of used.
const HASH_TABLE_HEADER: &str = "# cbc-hashes v2: templates (Name), functions (function:name), buses (bus:Name)";

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
        Ok(refreshed) if refreshed.lines().next() == Some(HASH_TABLE_HEADER) => parse_hash_table(&refreshed),
        Ok(_) => {
            eprintln!(
                "{}",
                ansi_term::Colour::Yellow.paint(format!(
                    "warning: {} was written by an older -cbc (no function/bus hashes); \
                     ignoring it and using the embedded table. Run --cbc-refresh again.",
                    LOCAL_HASH_CACHE_FILE
                ))
            );
            parse_hash_table(KNOWN_CIRCOMLIB2_TEMPLATE_HASHES)
        }
        Err(_) => parse_hash_table(KNOWN_CIRCOMLIB2_TEMPLATE_HASHES),
    }
}

fn parse_hash_table(content: &str) -> KnownHashes {
    content
        .lines()
        .filter(|line| !line.starts_with('#'))
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
fn trusted_instances(
    model: &[ExecutedTemplate],
    program: &ProgramArchive,
    known: &KnownHashes,
    warnings: &mut ReportCollection,
) -> Vec<bool> {
    let mut own_match = HashMap::new();
    let mut memo = vec![None; model.len()];
    for node in 0..model.len() {
        subtree_is_trusted(node, model, program, known, &mut own_match, &mut memo, warnings);
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
    warnings: &mut ReportCollection,
) -> bool {
    if let Some(trusted) = memo[node] {
        return trusted;
    }
    let exe = &model[node];
    let trusted = definition_matches(exe, program, known, own_match, warnings)
        && exe
            .subcomponent_edges()
            .all(|(_, child)| subtree_is_trusted(child, model, program, known, own_match, memo, warnings));
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
    warnings: &mut ReportCollection,
) -> bool {
    if let Some(matches) = cache.get(&exe.template_name) {
        return *matches;
    }
    let own = match definition_hash(program, &exe.template_name) {
        Some(hash) => known.contains(&(exe.template_name.clone(), hash)),
        None => false,
    };
    let matches = own
        && match first_modified_dependency(program, known, &exe.template_name) {
            None => true,
            Some(dependency) => {
                warnings.push(Report::warning(
                    format!(
                        "(H3) Template '{}' matches circomlib2, but it uses {}, which does not \
                         match circomlib2: the template is not trusted and is checked as \
                         ordinary code (H1/H2), together with every instance above it.",
                        exe.template_name, dependency
                    ),
                    ReportCode::TrustLostDependency,
                ));
                false
            }
        };
    cache.insert(exe.template_name.clone(), matches);
    matches
}

// Walks the static closure of functions and buses a template depends on
// (identifiers in its normalized text, then transitively in theirs) and
// returns the first one whose definition is not in `known`, as a readable
// description ("function maxbits()", "bus BinaryNumber"). Identifiers are
// over-approximated (a name in a dead branch, or inside a log string, also
// counts), which can only make trust stricter. Circom forbids reusing a name
// across templates, functions and buses, so a name identifies its definition.
fn first_modified_dependency(program: &ProgramArchive, known: &KnownHashes, template: &str) -> Option<String> {
    let template_data = program.get_template_data(template);
    let own_text = definition_text(program, template_data.get_file_id(), template_data.get_definition_location())?;
    let mut queue: Vec<String> = identifiers(&normalize_definition(&own_text));
    let mut seen: HashSet<String> = HashSet::new();
    while let Some(ident) = queue.pop() {
        if !seen.insert(ident.clone()) {
            continue;
        }
        let (key, description, text) = if program.contains_function(&ident) {
            let data = program.get_function_data(&ident);
            (
                format!("function:{}", ident),
                format!("function {}()", ident),
                definition_text(program, data.get_file_id(), data.get_definition_location()),
            )
        } else if program.contains_bus(&ident) {
            let data = program.get_bus_data(&ident);
            (
                format!("bus:{}", ident),
                format!("bus {}", ident),
                definition_text(program, data.get_file_id(), data.get_definition_location()),
            )
        } else {
            continue; // a variable, signal, tag, template, keyword...
        };
        let Some(text) = text else { return Some(description) };
        if !known.contains(&(key, hash_definition(&text))) {
            return Some(description);
        }
        queue.extend(identifiers(&normalize_definition(&text)));
    }
    None
}

fn definition_text(
    program: &ProgramArchive,
    file_id: usize,
    location: program_structure::file_definition::FileLocation,
) -> Option<String> {
    let source = program.file_library.to_storage().get(file_id)?.source();
    source.get(location).map(str::to_string)
}

fn identifiers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
            current.push(c);
        } else if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
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
    // Reproducibility aid: hash an existing local checkout (e.g. pinned at the
    // commit the embedded table must describe) instead of cloning the moving
    // branch head. This is how `circomlib2_template_hashes.txt` is regenerated.
    if let Ok(local) = std::env::var("CBC_CIRCOMLIB2_DIR") {
        let circuits_dir = PathBuf::from(&local).join("circuits");
        let mut files = Vec::new();
        collect_circom_files(&circuits_dir, &circuits_dir, &mut files)
            .ok_or_else(|| format!("could not read {}", circuits_dir.display()))?;
        files.sort();
        let (cache_content, count) = hash_circomlib_files(&files, prime)?;
        std::fs::write(LOCAL_HASH_CACHE_FILE, cache_content)
            .map_err(|err| format!("failed to write {}: {}", LOCAL_HASH_CACHE_FILE, err))?;
        println!(
            "{}",
            ansi_term::Colour::Green.paint(format!(
                "Refreshed {} with {} circomlib2 template/function/bus hashes from {} files of {} (CBC_CIRCOMLIB2_DIR).",
                LOCAL_HASH_CACHE_FILE, count, files.len(), local
            ))
        );
        return Ok(());
    }
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
            "Refreshed {} with {} circomlib2 ({}) template/function/bus hashes from {} files.",
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
        let definitions = parser::callable_definitions(&source, &field)
            .map_err(|_| format!("failed to parse circuits/{}", relative_path))?;
        for (kind, name, location) in definitions {
            let definition = source
                .get(location)
                .ok_or_else(|| format!("bad definition range in circuits/{}", relative_path))?;
            let key = if kind == "template" { name } else { format!("{}:{}", kind, name) };
            cache_content.push_str(&format!("{}  circuits/{}::{}\n", hash_definition(definition), relative_path, key));
            template_count += 1;
        }
    }
    Ok((format!("{}\n{}", HASH_TABLE_HEADER, cache_content), template_count))
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
    // The instance being checked: its concrete signal dimensions and
    // constraints tell which positions of an array were actually assigned.
    exe: &'a ExecutedTemplate,
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
    // Plain (non-bus) intermediate/output signals declared WITHOUT tags.
    // Circom lets such a signal INHERIT the tags of whatever it is copied
    // from with a bare `<==` ("Tag not defined by user" in
    // `assignment_utils.rs::perform_tag_propagation`): the first assignment
    // adds the source's tags, every later one (another array position) keeps
    // only the tags the new source carries too. `-cbc` mirrors that, so
    // `t <== IsZero()(a); o <== t;` with `signal t;` and
    // `signal output {binary} o;` is honest, as it is for circom.
    untagged: HashSet<String>,
    // Every assignment to an `untagged` path (array indices collapse), as the
    // source its tags come from. The inherited tags of a path are the
    // intersection over ALL of them, resolved once the walk is over (see
    // `resolve_inherited`), so a later dishonest position (or `<--`, or an
    // expression) still removes a tag that an earlier claim relied on.
    inherit_edges: HashMap<String, Vec<InheritSource>>,
    // Tag claims whose source is an `untagged` signal: (claimed path, claimed
    // tags, source path, meta). Provisionally accepted during the walk and
    // re-checked in `finalize` against the final inherited tags.
    deferred: Vec<(String, Vec<String>, String, Meta)>,
    errors: &'a mut ReportCollection,
}

// Where an assignment to an `untagged` path takes its tags from.
enum InheritSource {
    // A local signal or local bus field, resolved at the end of the walk:
    // recursively inherited if it is `untagged` itself, otherwise its own
    // honestly established tags (none if it ended up dishonest).
    Local(String),
    // Already known: a subcomponent port's declared tags, or nothing at all
    // (`<--`, an arithmetic expression, a constant).
    Fixed(HashSet<String>),
}

// Outcome of checking a tag claim's source.
enum Honesty {
    Honest,
    Dishonest,
    // The source is an `untagged` signal: decided in `finalize`.
    Deferred(String),
}

fn check_h2<'a>(
    exe: &'a ExecutedTemplate,
    model: &'a [ExecutedTemplate],
    program: &'a ProgramArchive,
    errors: &'a mut ReportCollection,
) {
    let subcomponents: HashMap<&str, NodePointer> = exe.subcomponent_edges().collect();
    let mut ctx = H2Context {
        exe,
        model,
        program,
        instance_name: &exe.report_name,
        subcomponents,
        bus_types: HashMap::new(),
        pending: HashMap::new(),
        verified: HashMap::new(),
        dishonest: HashSet::new(),
        untagged: HashSet::new(),
        inherit_edges: HashMap::new(),
        deferred: Vec::new(),
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
            // Inputs never inherit (their tags are exactly the declared
            // interface); intermediates and outputs do, inside this template.
            // Whole buses already propagate their field structure through
            // `record_legitimate_copy`.
            if bus_type.is_none() && stype != SignalType::Input {
                self.untagged.insert(name.to_string());
            }
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
        // `=` (AssignVar) never moves a value into a signal. On a signal/bus
        // path it only shows up as (a) the bus "constructor" `p = Point()` the
        // parser inserts right after EVERY bus declaration
        // (`split_bus_declaration_into_single_nodes`), or (b) a tag-value
        // assignment such as `out.maxbit = n`. Neither establishes nor breaks
        // a tag's honesty; treating (a) as a tag claim made every bus with a
        // bus-level tag fail CE02 at its own declaration, inputs included.
        if op == AssignOp::AssignVar {
            // (b) is not neutral, though: circom keeps a value set by hand
            // and IGNORES the value the source brings (`value_defined` in
            // `perform_tag_propagation`), so `m.maxbit = 4; m <== x;` with
            // `x.maxbit = 8` claims a bound nothing proves. A tag value can
            // only be honest if it comes from an honest source, never typed in.
            self.check_tag_value_assignment(meta, var, access);
            return;
        }
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
            if self.untagged.contains(&path) {
                // Same rule as circom: only a bare `<==` carries tags; `<--`
                // or an expression contributes none (and so, intersected with
                // the other positions, removes every inherited tag).
                let source = if op == AssignOp::AssignConstraintSignal {
                    self.inherit_source(rhe)
                } else {
                    InheritSource::Fixed(HashSet::new())
                };
                self.inherit_edges.entry(path).or_default().push(source);
            }
            return;
        };

        self.pending.remove(&path);
        let honesty = if op == AssignOp::AssignConstraintSignal {
            self.honest_source(rhe, &tags)
        } else {
            Honesty::Dishonest
        };
        match honesty {
            Honesty::Honest => self.record_legitimate_copy(&path, tags.into_iter().collect(), rhe),
            Honesty::Deferred(src) => {
                self.deferred.push((path.clone(), tags.clone(), src, meta.clone()));
                self.record_legitimate_copy(&path, tags.into_iter().collect(), rhe);
            }
            Honesty::Dishonest => {
                self.verified.remove(&path);
                self.dishonest.insert(path.clone());
                self.errors.push(bare_tag_report(self.instance_name, &path, &tags, meta));
            }
        }
    }

    // `path.tag = value`: if `tag` is one of the tags `path` declares, its
    // value is being set by hand -> CE02, and the path is disqualified for
    // good (no later copy can make that value honest).
    fn check_tag_value_assignment(&mut self, meta: &Meta, var: &str, access: &[Access]) {
        let hops: Vec<&str> = access
            .iter()
            .filter_map(|a| match a {
                Access::ComponentAccess(name) => Some(name.as_str()),
                Access::ArrayAccess(_) => None,
            })
            .collect();
        let Some((tag, fields)) = hops.split_last() else { return };
        if self.subcomponents.contains_key(var) {
            return; // circom does not allow setting a subcomponent's tags
        }
        let path = std::iter::once(var).chain(fields.iter().copied()).collect::<Vec<_>>().join(".");
        let declares = self.pending.get(&path).map_or(false, |(tags, _)| tags.iter().any(|t| t == tag))
            || self.verified.get(&path).map_or(false, |tags| tags.contains(*tag));
        if !declares || self.dishonest.contains(&path) {
            return;
        }
        let tags = self.pending.remove(&path).map(|(tags, _)| tags).unwrap_or_else(|| vec![tag.to_string()]);
        self.verified.remove(&path);
        self.dishonest.insert(path.clone());
        let mut report = Report::error(
            format!(
                "(H2) In {}: the value of tag '{}' of signal '{}' is set by hand; a certifiable \
                 circuit must obtain tag values from an honest source (a bare copy of a signal, \
                 or the output of a trusted/certifiable subcomponent), never assign them.",
                self.instance_name, tag, path
            ),
            ReportCode::BareTagDetected,
        );
        report.add_primary(
            generate_file_location(meta.start, meta.end),
            meta.get_file_id(),
            "tag value set by hand here".to_string(),
        );
        report.add_note(format!(
            "circom keeps a hand-set value and ignores the one the source carries, so this \
             value is an unchecked claim (declared tags: {:?}).",
            tags
        ));
        self.errors.push(report);
    }

    // True iff every position of the `untagged` signal `path` was assigned:
    // each `<==` produces a constraint mentioning its position, so a position
    // that appears in no constraint of this instance was never written. Same
    // rule as circom, which only propagates inherited tags out of a signal
    // once it is complete (`remaining_inserts == 0` in
    // `compute_propagated_tags`).
    fn is_complete(&self, path: &str, constrained: &HashSet<&str>) -> bool {
        let Some(wire) = self.exe.intermediates.iter().chain(self.exe.outputs.iter()).find(|w| w.name == path) else {
            return false;
        };
        let mut positions = vec![path.to_string()];
        for &dim in &wire.length {
            positions = positions
                .into_iter()
                .flat_map(|prefix| (0..dim).map(move |i| format!("{}[{}]", prefix, i)))
                .collect();
        }
        positions.iter().all(|p| constrained.contains(p.as_str()))
    }

    // The source an assignment `<== rhe` into an `untagged` path inherits
    // from. Only a bare `Variable` carries tags (as in circom).
    fn inherit_source(&self, rhe: &Expression) -> InheritSource {
        let Expression::Variable { name, access, .. } = rhe else {
            return InheritSource::Fixed(HashSet::new());
        };
        if first_component_access(access).is_none() {
            return InheritSource::Local(name.clone());
        }
        match self.resolve_access(name, access) {
            // A subcomponent port: its declared tags (Assumption 1).
            Some((_, tags, true)) => InheritSource::Fixed(tags),
            // A local bus field: whatever it ends up honestly carrying.
            Some((path, _, false)) => InheritSource::Local(path),
            None => InheritSource::Fixed(HashSet::new()),
        }
    }

    // Tags a NON-`untagged` local path honestly carries at the end of the
    // walk: none if it was ever disqualified or never resolved.
    fn final_tags_of(&self, path: &str) -> HashSet<String> {
        if self.dishonest.contains(path) {
            return HashSet::new();
        }
        self.verified.get(path).cloned().unwrap_or_default()
    }

    // Inherited tags of every `untagged` path: for each path, the
    // intersection over all its assignments of the tags of their sources.
    // Computed as a GREATEST fixpoint (`None` = not yet constrained, the
    // identity of intersection), so self-references such as `t[1] <== t[0]`
    // resolve to what the other positions bring instead of to nothing.
    // Sound because circom never lets a signal be read before it is written,
    // so no value can depend on itself. A path that is never assigned, not
    // completely assigned (`is_complete`), or only through such cycles,
    // inherits nothing.
    fn resolve_inherited(&self) -> HashMap<String, HashSet<String>> {
        let constrained: HashSet<&str> = self
            .exe
            .constraints
            .iter()
            .flat_map(|c| c.take_signals().into_iter().map(String::as_str))
            .collect();
        // Never assigned, or not completely (some array position never
        // written): inherits nothing, exactly as in circom.
        let incomplete: HashSet<&String> = self
            .untagged
            .iter()
            .filter(|p| !self.inherit_edges.contains_key(*p) || !self.is_complete(p, &constrained))
            .collect();
        let mut current: HashMap<String, Option<HashSet<String>>> = self
            .untagged
            .iter()
            .map(|p| (p.clone(), if incomplete.contains(p) { Some(HashSet::new()) } else { None }))
            .collect();
        loop {
            let mut changed = false;
            for (path, edges) in &self.inherit_edges {
                if incomplete.contains(path) {
                    continue;
                }
                let mut acc: Option<HashSet<String>> = None;
                for edge in edges {
                    let carried = match edge {
                        InheritSource::Fixed(tags) => Some(tags.clone()),
                        InheritSource::Local(src) if self.untagged.contains(src) => current[src].clone(),
                        InheritSource::Local(src) => Some(self.final_tags_of(src)),
                    };
                    acc = match (acc, carried) {
                        (None, other) | (other, None) => other,
                        (Some(a), Some(b)) => Some(a.intersection(&b).cloned().collect()),
                    };
                }
                if current[path] != acc {
                    current.insert(path.clone(), acc);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        current.into_iter().map(|(p, t)| (p, t.unwrap_or_default())).collect()
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
    fn honest_source(&self, rhe: &Expression, tags: &[String]) -> Honesty {
        let Expression::Variable { name, access, .. } = rhe else { return Honesty::Dishonest };
        let required: HashSet<&str> = tags.iter().map(String::as_str).collect();
        let covers = |carried: Option<&HashSet<String>>| match carried {
            Some(carried) if required.iter().all(|t| carried.contains(*t)) => Honesty::Honest,
            _ => Honesty::Dishonest,
        };

        if first_component_access(access).is_none() {
            // Declared without tags: whatever it inherits is only known once
            // every assignment to it has been seen.
            if self.untagged.contains(name.as_str()) {
                return Honesty::Deferred(name.clone());
            }
            // Plain variable (or array-indexed): same declaration-level tags
            // regardless of which element.
            return covers(self.verified.get(name));
        }

        let Some((path, declared_or_verified, is_component)) = self.resolve_access(name, access) else {
            return Honesty::Dishonest;
        };
        if is_component {
            // Assumption 1: trust the callee's own declared promise directly.
            covers(Some(&declared_or_verified))
        } else {
            covers(self.verified.get(&path))
        }
    }

    fn finalize(&mut self) {
        // Claims copied from an `untagged` signal: honest iff that signal
        // inherited every claimed tag on ALL of its assignments.
        let inherited = self.resolve_inherited();
        let mut reported: HashSet<String> = HashSet::new();
        for (path, tags, src, meta) in std::mem::take(&mut self.deferred) {
            if self.dishonest.contains(&path) || reported.contains(&path) {
                continue; // already reported once for this collapsed path
            }
            let carried = inherited.get(&src).cloned().unwrap_or_default();
            if !tags.iter().all(|t| carried.contains(t)) {
                reported.insert(path.clone());
                let mut report = bare_tag_report(self.instance_name, &path, &tags, &meta);
                report.add_note(format!(
                    "'{}' is declared without tags; it only inherits a tag if every assignment \
                     to it (every array position) is a bare '<==' copy of a source carrying it. \
                     Inherited here: {:?}.",
                    src,
                    {
                        let mut v: Vec<_> = carried.into_iter().collect();
                        v.sort();
                        v
                    }
                ));
                self.errors.push(report);
            }
        }
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
