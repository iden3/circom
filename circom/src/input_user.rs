use std::path::PathBuf;

pub struct Input {
    pub input_program: PathBuf,
    pub out_r1cs: PathBuf,
    pub out_json_constraints: PathBuf,
    pub out_sym: PathBuf,
    pub field: &'static str,
    pub r1cs_flag: bool,
    pub sym_flag: bool,
    pub json_constraint_flag: bool,
    pub main_inputs_flag: bool,
    pub inspect_constraints_flag: bool,
    pub flag_verbose: bool,
    pub witness_file: String,
}

const P_0: &'static str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const R1CS: &'static str = "r1cs";
const SYM: &'static str = "sym";
const JSON: &'static str = "json";

impl Input {
    pub fn new() -> Result<Input, ()> {
        let matches = input_processing::view();
        let input = input_processing::get_input(&matches)?;
        let file_name = input.file_stem().unwrap().to_str().unwrap().to_string();
        let output_path = input_processing::get_output_path(&matches)?;
        Result::Ok(Input {
            field: P_0,
            input_program: input,
            out_r1cs: Input::build_output(&output_path, &file_name, R1CS),
            out_sym: Input::build_output(&output_path, &file_name, SYM),
            out_json_constraints: Input::build_output(
                &output_path,
                &format!("{}_constraints", file_name),
                JSON,
            ),
            r1cs_flag: input_processing::get_r1cs(&matches),
            sym_flag: input_processing::get_sym(&matches),
            main_inputs_flag: input_processing::get_main_inputs_log(&matches),
            json_constraint_flag: input_processing::get_json_constraints(&matches),
            inspect_constraints_flag: input_processing::get_inspect_constraints(&matches),
            flag_verbose: input_processing::get_flag_verbose(&matches),
            witness_file: input_processing::get_witness_file(&matches),
        })
    }
    
    fn build_output(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
        file.push(format!("{}.{}",filename,ext));
        file
    }

    pub fn input_file(&self) -> &str {
        &self.input_program.to_str().unwrap()
    }
    pub fn r1cs_file(&self) -> &str {
        self.out_r1cs.to_str().unwrap()
    }
    pub fn sym_file(&self) -> &str {
        self.out_sym.to_str().unwrap()
    }
    pub fn json_constraints_file(&self) -> &str {
        self.out_json_constraints.to_str().unwrap()
    }
    pub fn r1cs_flag(&self) -> bool {
        self.r1cs_flag
    }
    pub fn json_constraints_flag(&self) -> bool {
        self.json_constraint_flag
    }
    pub fn sym_flag(&self) -> bool {
        self.sym_flag
    }
    pub fn inspect_constraints_flag(&self) -> bool {
        self.inspect_constraints_flag
    }
    pub fn flag_verbose(&self) -> bool {
        self.flag_verbose
    }
    pub fn witness_file(&self) -> String{
        self.witness_file.clone()
    }
}
mod input_processing {
    use ansi_term::Colour;
    use clap::{App, Arg, ArgMatches};
    use std::path::{Path, PathBuf};
    use crate::VERSION;

    pub fn get_input(matches: &ArgMatches) -> Result<PathBuf, ()> {
        let route = Path::new(matches.value_of("input").unwrap()).to_path_buf();
        if route.is_file() {
            Result::Ok(route)
        } else {
            Result::Err(eprintln!("{}", Colour::Red.paint("invalid input file")))
        }
    }

    pub fn get_output_path(matches: &ArgMatches) -> Result<PathBuf, ()> {
        let route = Path::new(matches.value_of("output").unwrap()).to_path_buf();
        if route.is_dir() {
            Result::Ok(route)
        } else {
            Result::Err(eprintln!("{}", Colour::Red.paint("invalid output path")))
        }
    }

    pub fn get_json_constraints(matches: &ArgMatches) -> bool {
        matches.is_present("print_json_c")
    }

    pub fn get_sym(matches: &ArgMatches) -> bool {
        matches.is_present("print_sym")
    }

    pub fn get_r1cs(matches: &ArgMatches) -> bool {
        matches.is_present("print_r1cs")
    }

    pub fn get_main_inputs_log(matches: &ArgMatches) -> bool {
        matches.is_present("main_inputs_log")
    }

    pub fn get_inspect_constraints(matches: &ArgMatches) -> bool {
        matches.is_present("inspect_constraints")
    }

    pub fn get_flag_verbose(matches: &ArgMatches) -> bool {
        matches.is_present("flag_verbose")
    }

    pub fn get_witness_file(matches: &ArgMatches) -> String{
        matches.value_of("witness").unwrap().to_string()
    }

    pub fn view() -> ArgMatches<'static> {
        App::new("circom compiler")
            .version(VERSION)
            .author("IDEN3")
            .about("Compiler for the circom programming language")
            .arg(
                Arg::with_name("input")
                    .multiple(false)
                    .default_value("./circuit.circom")
                    .help("Path to a circuit with a main component"),
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .default_value(".")
                    .help("Path to the directory where the output will be written"),
            )
            .arg(
                Arg::with_name("print_json_c")
                    .long("json")
                    .takes_value(false)
                    .help("outputs the constraints in json format"),
            )
            .arg(
                Arg::with_name("inspect_constraints")
                    .long("inspect")
                    .takes_value(false)
                    .help("Does an additional check over the constraints produced"),
            )
            .arg(
                Arg::with_name("print_sym")
                    .long("sym")
                    .takes_value(false)
                    .help("outputs witness in sym format"),
            )
            .arg(
                Arg::with_name("print_r1cs")
                    .long("r1cs")
                    .takes_value(false)
                    .help("outputs the constraints in r1cs format"),
            )
            .arg(
                Arg::with_name("flag_verbose")
                    .long("verbose")
                    .takes_value(false)
                    .help("Shows logs during compilation"),
            )
            .get_matches()
    }
}
