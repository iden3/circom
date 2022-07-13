mod input_user;
mod parser_user;
mod type_analysis_user;
mod execution_user;
mod verification_user;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");


use ansi_term::Colour;
use input_user::Input;
fn main() {
    let result = start();
    if result.is_err() {
        eprintln!("{}", Colour::Red.paint("previous errors were found"));
    } else {
        println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
    }
}

fn start() -> Result<(), ()> {
    use execution_user::ExecutionConfig;
    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    let config = ExecutionConfig {
        flag_verbose: user_input.flag_verbose(),
        inspect_constraints_flag: user_input.inspect_constraints_flag(),
        r1cs_flag: user_input.r1cs_flag(),
        json_constraint_flag: user_input.json_constraints_flag(),
        sym_flag: user_input.sym_flag(),
        sym: user_input.sym_file().to_string(),
        r1cs: user_input.r1cs_file().to_string(),
        json_constraints: user_input.json_constraints_file().to_string(),
    };
    let (_circuit, tree_constraints) = execution_user::execute_project(program_archive, config)?;
    verification_user::execute_verification(tree_constraints, user_input.witness_file().clone());
    Result::Ok(())
}
