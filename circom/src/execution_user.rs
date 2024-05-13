use ansi_term::Colour;
use compiler::hir::very_concrete_program::VCP;
use constraint_writers::ConstraintExporter;
use program_structure::program_archive::ProgramArchive;
use virtual_fs::{FileSystem, VPath};

pub struct ExecutionConfig {
    pub r1cs: String,
    pub sym: String,
    pub json_constraints: String,
    pub json_substitutions: String,
    pub no_rounds: usize,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_old_heuristics:bool,
    pub flag_verbose: bool,
    pub inspect_constraints_flag: bool,
    pub sym_flag: bool,
    pub r1cs_flag: bool,
    pub json_substitution_flag: bool,
    pub json_constraint_flag: bool,
    pub prime: String,
}

pub fn execute_project(
    fs: &mut dyn FileSystem,
    program_archive: ProgramArchive,
    config: ExecutionConfig,
) -> Result<VCP, ()> {
    use constraint_generation::{build_circuit, BuildConfig};
    let json_constraints_path: VPath = config.json_constraints.into();
    let build_config = BuildConfig {
        no_rounds: config.no_rounds,
        flag_json_sub: config.json_substitution_flag,
        json_substitutions: config.json_substitutions,
        flag_s: config.flag_s,
        flag_f: config.flag_f,
        flag_p: config.flag_p,
        flag_verbose: config.flag_verbose,
        inspect_constraints: config.inspect_constraints_flag,
        flag_old_heuristics: config.flag_old_heuristics,
        prime : config.prime,
    };
    let custom_gates = program_archive.custom_gates;
    let (exporter, vcp) = build_circuit(fs, program_archive, build_config)?;
    if config.r1cs_flag {
        generate_output_r1cs(fs, &config.r1cs, exporter.as_ref(), custom_gates)?;
    }
    if config.sym_flag {
        generate_output_sym(fs, &config.sym, exporter.as_ref())?;
    }
    if config.json_constraint_flag {
        generate_json_constraints(fs, &json_constraints_path, exporter.as_ref())?;
    }
    Result::Ok(vcp)
}

fn generate_output_r1cs(fs: &mut dyn FileSystem, file: &str, exporter: &dyn ConstraintExporter, custom_gates: bool) -> Result<(), ()> {
    if let Result::Ok(()) = exporter.r1cs(fs, file, custom_gates) {
        println!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
        Result::Err(())
    }
}

fn generate_output_sym(fs: &mut dyn FileSystem, file: &str, exporter: &dyn ConstraintExporter) -> Result<(), ()> {
    if let Result::Ok(()) = exporter.sym(fs, file) {
        println!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
        Result::Err(())
    }
}

fn generate_json_constraints(
    fs: &mut dyn FileSystem,
    json_constraints_path: &VPath,
    exporter: &dyn ConstraintExporter,
) -> Result<(), ()> {
    if let Ok(()) = exporter.json_constraints(fs, json_constraints_path) {
        println!("{} {}", Colour::Green.paint("Constraints written in:"), json_constraints_path);
        Result::Ok(())
    } else {
        eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
        Result::Err(())
    }
}
