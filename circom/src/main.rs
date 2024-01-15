mod compilation_user;
mod execution_user;
mod input_user;
mod parser_user;
mod type_analysis_user;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");


use ansi_term::Colour;
use input_user::Input;
fn main() {
    let result = start();
    if result.is_err() {
        eprintln!("{}", Colour::Red.paint("previous errors were found"));
        std::process::exit(1);
    } else {
        println!("{}", Colour::Green.paint("Everything went okay"));
        //std::process::exit(0);
    }
}

fn start() -> Result<(), ()> {
    use compilation_user::CompilerConfig;
    use execution_user::ExecutionConfig;
    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    let config = ExecutionConfig {
        no_rounds: user_input.no_rounds(),
        flag_p: user_input.parallel_simplification_flag(),
        flag_s: user_input.reduced_simplification_flag(),
        flag_f: user_input.unsimplified_flag(),
        flag_old_heuristics: user_input.flag_old_heuristics(),
        flag_verbose: user_input.flag_verbose(),
        inspect_constraints_flag: user_input.inspect_constraints_flag(),
        r1cs_flag: user_input.r1cs_flag(),
        json_constraint_flag: user_input.json_constraints_flag(),
        json_substitution_flag: user_input.json_substitutions_flag(),
        sym_flag: user_input.sym_flag(),
        sym: user_input.sym_file().to_string(),
        r1cs: user_input.r1cs_file().to_string(),
        json_constraints: user_input.json_constraints_file().to_string(),
        json_substitutions: user_input.json_substitutions_file().to_string(),
        prime: user_input.prime(),        
    };
    let circuit = execution_user::execute_project(program_archive, config)?;
    let compilation_config = CompilerConfig {
        vcp: circuit,
        debug_output: user_input.print_ir_flag(),
        c_flag: user_input.c_flag(),
        wasm_flag: user_input.wasm_flag(),
        wat_flag: user_input.wat_flag(),
	    js_folder: user_input.js_folder().to_string(),
	    wasm_name: user_input.wasm_name().to_string(),
	    c_folder: user_input.c_folder().to_string(),
	    c_run_name: user_input.c_run_name().to_string(),
        c_file: user_input.c_file().to_string(),
        dat_file: user_input.dat_file().to_string(),
        wat_file: user_input.wat_file().to_string(),
        wasm_file: user_input.wasm_file().to_string(),
        produce_input_log: user_input.main_inputs_flag(),
    };
    compilation_user::compile(compilation_config)?;
    Result::Ok(())
}
