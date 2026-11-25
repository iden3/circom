extern crate num_bigint_dig as num_bigint;
extern crate num_traits;

mod compute_constants;
mod environment_utils;
mod execute;
mod execution_data;
mod assignment_utils;
mod certifier;

use ansi_term::Colour;
use circom_algebra::algebra::{ArithmeticError, ArithmeticExpression};
use compiler::hir::very_concrete_program::VCP;
use constraint_list::ConstraintList;
use constraint_writers::ConstraintExporter;
use dag::DAG;
use execution_data::executed_program::ExportResult;
use execution_data::ExecutedProgram;
use program_structure::ast::{self};
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::FileID;
use program_structure::program_archive::ProgramArchive;
use std::rc::Rc;

pub struct BuildConfig {
    pub no_rounds: usize,
    pub flag_json_sub: bool,
    pub json_substitutions: String,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_verbose: bool,
    pub flag_old_heuristics: bool,
    pub inspect_constraints: bool,
    pub certifiable: bool,
    pub refresh_circomlib2: bool,
    pub prime: String,
}

#[derive(Debug, Copy, Clone)]
pub struct FlagsExecution{
    pub verbose: bool,
    pub inspect: bool,
}

pub type ConstraintWriter = Box<dyn ConstraintExporter>;
type BuildResponse = Result<(ConstraintWriter, VCP), ()>;
pub fn build_circuit(program: ProgramArchive, config: BuildConfig) -> BuildResponse {
    let files = program.file_library.clone();
    let flags = FlagsExecution{
        verbose: config.flag_verbose,
        inspect: config.inspect_constraints,
    };
    let (exe, warnings) = instantiation(&program, flags, &config.prime).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    Report::print_reports(&warnings, &files);

    // H1/H2/H3 need `exe.model`'s AST, which `export()` consumes -- so they
    // must run here, before it. The verdict isn't final yet even if they
    // already fail: we still proceed to `export()` so the structural
    // (unconstrained-signal) check below also runs and gets merged into the
    // SAME report, rather than hiding behind whichever kind of violation
    // happened to be found first. `<--`/bare-tag usage is ordinary, valid
    // circom that `export()` handles fine outside of `--cbc`, so
    // there is nothing unsafe about continuing past a failing `cert_result`.
    let mut cert_result = None;
    if config.certifiable {
        if config.refresh_circomlib2 {
            certifier::refresh_circomlib2_cache(&config.prime).map_err(|err| {
                eprintln!("{}", Colour::Red.paint(format!("--cbc-refresh failed: {}", err)));
            })?;
        }
        let result = certifier::certify(&exe.model, &program);
        Report::print_reports(&result.warnings, &files);
        cert_result = Some(result);
    }

    let (mut dag, mut vcp, warnings) = export(exe, program, flags).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    if config.inspect_constraints {
        Report::print_reports(&warnings, &files);
    }

    if let Some(mut cert_result) = cert_result {
        // Re-run the same structural analysis `--inspect` uses, regardless
        // of whether `--inspect` was separately passed: a signal that is
        // never touched at all (not even dishonestly) is invisible to H1/H2.
        let structural_warnings = dag.constraint_analysis().map_err(|errs| {
            Report::print_reports(&errs, &files);
        })?;
        cert_result.absorb_structural_warnings(&structural_warnings);
        if cert_result.is_certifiable() {
            cert_result.print_summary();
            println!("{}", Colour::Green.paint("Certifiable: PASS (H1, H2, H3, fully constrained)"));
        } else {
            Report::print_reports(&cert_result.errors, &files);
            cert_result.print_summary();
            println!("{}", Colour::Red.paint("Certifiable: FAIL"));
            return Result::Err(());
        }
    }
    if config.flag_f {
        sync_dag_and_vcp(&mut vcp, &mut dag);
        if config.flag_json_sub { 
            use constraint_writers::json_writer::SubstitutionJSON;
            let substitution_log = SubstitutionJSON::new(&config.json_substitutions).unwrap();
            let _ = substitution_log.end();
            println!("{} {}", Colour::Green.paint("Written successfully:"), config.json_substitutions);
        };

        Result::Ok((Box::new(dag), vcp))
    } else {
        let list = simplification_process(&mut vcp, dag, &config);
        if config.flag_json_sub { 
            println!("{} {}", Colour::Green.paint("Written successfully:"), config.json_substitutions);
        };
        Result::Ok((Box::new(list), vcp))
    }
}

type InstantiationResponse = Result<(ExecutedProgram, ReportCollection), ReportCollection>;
fn instantiation(program: &ProgramArchive, flags: FlagsExecution, prime: &String) -> InstantiationResponse {
    let execution_result = execute::constraint_execution(&program, flags, prime);
    match execution_result {
        Ok((program_exe, warnings)) => {
            let no_nodes = program_exe.number_of_nodes();
            let success = Colour::Green.paint("template instances");
            let nodes_created = format!("{}: {}", success, no_nodes);
            println!("{}", &nodes_created);
            InstantiationResponse::Ok((program_exe,warnings))
        }
        Err(reports) => InstantiationResponse::Err(reports),
    }
}

fn export(exe: ExecutedProgram, program: ProgramArchive, flags: FlagsExecution) -> ExportResult {
    let exported = exe.export(program, flags);
    exported
}

fn sync_dag_and_vcp(vcp: &mut VCP, dag: &mut DAG) {
    let witness = Rc::new(DAG::produce_witness(dag));
    VCP::add_witness_list(vcp, Rc::clone(&witness));
}

fn simplification_process(vcp: &mut VCP, dag: DAG, config: &BuildConfig) -> ConstraintList {
    use dag::SimplificationFlags;
    let flags = SimplificationFlags {
        flag_s: config.flag_s,
        parallel_flag: config.flag_p,
        port_substitution: config.flag_json_sub,
        json_substitutions: config.json_substitutions.clone(),
        no_rounds: config.no_rounds,
        flag_old_heuristics: config.flag_old_heuristics,
        prime : config.prime.clone(),
    };
    let list = DAG::map_to_list(dag, flags);
    VCP::add_witness_list(vcp, Rc::new(list.get_witness_as_vec()));
    list
}
