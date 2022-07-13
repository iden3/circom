use ansi_term::Colour;
use compiler::hir::very_concrete_program::VCP;
use constraint_writers::debug_writer::DebugWriter;
use constraint_writers::ConstraintExporter;
use program_structure::program_archive::ProgramArchive;
use dag::TreeConstraints;


pub struct ExecutionConfig {
    pub r1cs: String,
    pub sym: String,
    pub json_constraints: String,
    pub flag_verbose: bool,
    pub inspect_constraints_flag: bool,
    pub sym_flag: bool,
    pub r1cs_flag: bool,
    pub json_constraint_flag: bool,
}

pub fn execute_project(
    program_archive: ProgramArchive,
    config: ExecutionConfig,
) -> Result<(VCP, TreeConstraints), ()> {
    use constraint_generation::{build_circuit, BuildConfig};
    let debug = DebugWriter::new(config.json_constraints).unwrap();
    let build_config = BuildConfig {
        flag_verbose: config.flag_verbose,
        inspect_constraints: config.inspect_constraints_flag,
    };
    let (exporter, vcp, tree_constraints) = build_circuit(program_archive, build_config)?;
    if config.r1cs_flag {
        generate_output_r1cs(&config.r1cs, exporter.as_ref())?;
    }
    if config.sym_flag {
        generate_output_sym(&config.sym, exporter.as_ref())?;
    }
    if config.json_constraint_flag {
        generate_json_constraints(&debug, exporter.as_ref())?;
    }
    Result::Ok((vcp, tree_constraints))
}

fn generate_output_r1cs(file: &str, exporter: &dyn ConstraintExporter) -> Result<(), ()> {
    if let Result::Ok(()) = exporter.r1cs(file) {
        println!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
        Result::Err(())
    }
}

fn generate_output_sym(file: &str, exporter: &dyn ConstraintExporter) -> Result<(), ()> {
    if let Result::Ok(()) = exporter.sym(file) {
        println!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
        Result::Err(())
    }
}

fn generate_json_constraints(
    debug: &DebugWriter,
    exporter: &dyn ConstraintExporter,
) -> Result<(), ()> {
    if let Ok(()) = exporter.json_constraints(&debug) {
        println!("{} {}", Colour::Green.paint("Constraints written in:"), debug.json_constraints);
        Result::Ok(())
    } else {
        eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
        Result::Err(())
    }
}
