use std::path::PathBuf;

pub struct Input {
    pub input_program: PathBuf,
    pub out_r1cs: PathBuf,
    pub out_json_constraints: PathBuf,
    pub out_json_substitutions: PathBuf,
    pub out_wat_code: PathBuf,
    pub out_wasm_code: PathBuf,
    pub out_wasm_name: String,
    pub out_js_folder: PathBuf,
    pub out_c_run_name: String,
    pub out_c_folder: PathBuf,
    pub out_c_code: PathBuf,
    pub out_c_dat: PathBuf,
    pub out_sym: PathBuf,
    //pub field: &'static str,
    pub c_flag: bool,
    pub wasm_flag: bool,
    pub wat_flag: bool,
    pub r1cs_flag: bool,
    pub sym_flag: bool,
    pub json_constraint_flag: bool,
    pub json_substitution_flag: bool,
    pub main_inputs_flag: bool,
    pub print_ir_flag: bool,
    pub fast_flag: bool,
    pub reduced_simplification_flag: bool,
    pub parallel_simplification_flag: bool,
    pub flag_old_heuristics: bool,
    pub inspect_constraints_flag: bool,
    pub no_rounds: usize,
    pub flag_verbose: bool,
    pub prime: String,
    pub link_libraries : Vec<PathBuf>
}


const R1CS: &'static str = "r1cs";
const WAT: &'static str = "wat";
const WASM: &'static str = "wasm";
const CPP: &'static str = "cpp";
const JS: &'static str = "js";
const DAT: &'static str = "dat";
const SYM: &'static str = "sym";
const JSON: &'static str = "json";


impl Input {
    pub fn new() -> Result<Input, ()> {
        use ansi_term::Colour;
        use input_processing::SimplificationStyle;
        let matches = input_processing::view();
        let input = input_processing::get_input(&matches)?;
        let mut file_name = input.file_stem().unwrap().to_str().unwrap().to_string();
        let output_path = input_processing::get_output_path(&matches)?;

        let c_flag = input_processing::get_c(&matches);

        if c_flag && (file_name == "main" || file_name == "fr" || file_name == "calcwit"){
            println!("{}", Colour::Yellow.paint(format!("The name {} is reserved in Circom when using de --c flag. The files generated for your circuit will use the name {}_c instead of {}.", file_name, file_name, file_name)));
            file_name = format!("{}_c", file_name)
        };
        let output_c_path = Input::build_folder(&output_path, &file_name, CPP);
        let output_js_path = Input::build_folder(&output_path, &file_name, JS);
        let o_style = input_processing::get_simplification_style(&matches)?;
        let link_libraries = input_processing::get_link_libraries(&matches);
        Result::Ok(Input {
            //field: P_BN128,
            input_program: input,
            out_r1cs: Input::build_output(&output_path, &file_name, R1CS),
            out_wat_code: Input::build_output(&output_js_path, &file_name, WAT),
            out_wasm_code: Input::build_output(&output_js_path, &file_name, WASM),
	        out_js_folder: output_js_path.clone(),
	        out_wasm_name: file_name.clone(),
	        out_c_folder: output_c_path.clone(),
	        out_c_run_name: file_name.clone(),
            out_c_code: Input::build_output(&output_c_path, &file_name, CPP),
            out_c_dat: Input::build_output(&output_c_path, &file_name, DAT),
            out_sym: Input::build_output(&output_path, &file_name, SYM),
            out_json_constraints: Input::build_output(
                &output_path,
                &format!("{}_constraints", file_name),
                JSON,
            ),
            out_json_substitutions: Input::build_output(
                &output_path,
                &format!("{}_substitutions", file_name),
                JSON,
            ),
            wat_flag:input_processing::get_wat(&matches),
            wasm_flag: input_processing::get_wasm(&matches),
            c_flag: c_flag,
            r1cs_flag: input_processing::get_r1cs(&matches),
            sym_flag: input_processing::get_sym(&matches),
            main_inputs_flag: input_processing::get_main_inputs_log(&matches),
            json_constraint_flag: input_processing::get_json_constraints(&matches),
            json_substitution_flag: input_processing::get_json_substitutions(&matches),
            print_ir_flag: input_processing::get_ir(&matches),
            no_rounds: if let SimplificationStyle::O2(r) = o_style { r } else { 0 },
            fast_flag: o_style == SimplificationStyle::O0,
            reduced_simplification_flag: o_style == SimplificationStyle::O1,
            parallel_simplification_flag: input_processing::get_parallel_simplification(&matches),
            inspect_constraints_flag: input_processing::get_inspect_constraints(&matches),
            flag_old_heuristics: input_processing::get_flag_old_heuristics(&matches),
            flag_verbose: input_processing::get_flag_verbose(&matches), 
            prime: input_processing::get_prime(&matches)?,
            link_libraries
        })
    }

    fn build_folder(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
	    let folder_name = format!("{}_{}",filename,ext);
	    file.push(folder_name);
	    file
    }
    
    fn build_output(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
        file.push(format!("{}.{}",filename,ext));
        file
    }

    pub fn get_link_libraries(&self) -> &Vec<PathBuf> {
        &self.link_libraries
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
    pub fn wat_file(&self) -> &str {
        self.out_wat_code.to_str().unwrap()
    }
    pub fn wasm_file(&self) -> &str {
        self.out_wasm_code.to_str().unwrap()
    }
    pub fn js_folder(&self) -> &str {
        self.out_js_folder.to_str().unwrap()
    }
    pub fn wasm_name(&self) -> String {
        self.out_wasm_name.clone()
    }

    pub fn c_folder(&self) -> &str {
        self.out_c_folder.to_str().unwrap()
    }
    pub fn c_run_name(&self) -> String {
        self.out_c_run_name.clone()
    }

    pub fn c_file(&self) -> &str {
        self.out_c_code.to_str().unwrap()
    }
    pub fn dat_file(&self) -> &str {
        self.out_c_dat.to_str().unwrap()
    }
    pub fn json_constraints_file(&self) -> &str {
        self.out_json_constraints.to_str().unwrap()
    }
    pub fn json_substitutions_file(&self) -> &str {
        self.out_json_substitutions.to_str().unwrap()
    }
    pub fn wasm_flag(&self) -> bool {
        self.wasm_flag
    }
    pub fn wat_flag(&self) -> bool {
        self.wat_flag
    }
    pub fn c_flag(&self) -> bool {
        self.c_flag
    }
    pub fn unsimplified_flag(&self) -> bool {
        self.fast_flag
    }
    pub fn r1cs_flag(&self) -> bool {
        self.r1cs_flag
    }
    pub fn json_constraints_flag(&self) -> bool {
        self.json_constraint_flag
    }
    pub fn json_substitutions_flag(&self) -> bool {
        self.json_substitution_flag
    }
    pub fn main_inputs_flag(&self) -> bool {
        self.main_inputs_flag
    }
    pub fn sym_flag(&self) -> bool {
        self.sym_flag
    }
    pub fn print_ir_flag(&self) -> bool {
        self.print_ir_flag
    }
    pub fn inspect_constraints_flag(&self) -> bool {
        self.inspect_constraints_flag
    }
    pub fn flag_verbose(&self) -> bool {
        self.flag_verbose
    }
    pub fn reduced_simplification_flag(&self) -> bool {
        self.reduced_simplification_flag
    }
    pub fn parallel_simplification_flag(&self) -> bool {
        self.parallel_simplification_flag
    }
    pub fn flag_old_heuristics(&self) -> bool {
        self.flag_old_heuristics
    }
    pub fn no_rounds(&self) -> usize {
        self.no_rounds
    }
    pub fn prime(&self) -> String{
        self.prime.clone()
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
            let route = if route.to_str().is_some() { ": ".to_owned() + route.to_str().unwrap()} else { "".to_owned() };
            Result::Err(eprintln!("{}", Colour::Red.paint("Input file does not exist".to_owned() + &route)))
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

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub enum SimplificationStyle { O0, O1, O2(usize) }
    pub fn get_simplification_style(matches: &ArgMatches) -> Result<SimplificationStyle, ()> {

        let o_0 = matches.is_present("no_simplification");
        let o_1 = matches.is_present("reduced_simplification");
        let o_2 = matches.is_present("full_simplification");
        let o_2round = matches.is_present("simplification_rounds");
        match (o_0, o_1, o_2round, o_2) {
            (true, _, _, _) => Ok(SimplificationStyle::O0),
            (_, true, _, _) => Ok(SimplificationStyle::O1),
            (_, _, true,  _) => {
                let o_2_argument = matches.value_of("simplification_rounds").unwrap();
                let rounds_r = usize::from_str_radix(o_2_argument, 10);
                if let Result::Ok(no_rounds) = rounds_r { 
                    if no_rounds == 0 { Ok(SimplificationStyle::O1) }
                    else {Ok(SimplificationStyle::O2(no_rounds))}} 
                else { Result::Err(eprintln!("{}", Colour::Red.paint("invalid number of rounds"))) }
            },
            
            (false, false, false, true) => Ok(SimplificationStyle::O2(usize::MAX)),
            (false, false, false, false) => Ok(SimplificationStyle::O2(usize::MAX)),
        }
    }

    pub fn get_json_constraints(matches: &ArgMatches) -> bool {
        matches.is_present("print_json_c")
    }

    pub fn get_json_substitutions(matches: &ArgMatches) -> bool {
        matches.is_present("print_json_sub")
    }

    pub fn get_sym(matches: &ArgMatches) -> bool {
        matches.is_present("print_sym")
    }

    pub fn get_r1cs(matches: &ArgMatches) -> bool {
        matches.is_present("print_r1cs")
    }

    pub fn get_wasm(matches: &ArgMatches) -> bool {
        matches.is_present("print_wasm")
    }

    pub fn get_wat(matches: &ArgMatches) -> bool {
        matches.is_present("print_wat")
    }

    pub fn get_c(matches: &ArgMatches) -> bool {
        matches.is_present("print_c")
    }

    pub fn get_main_inputs_log(matches: &ArgMatches) -> bool {
        matches.is_present("main_inputs_log")
    }

    pub fn get_parallel_simplification(matches: &ArgMatches) -> bool {
        matches.is_present("parallel_simplification")
    }

    pub fn get_ir(matches: &ArgMatches) -> bool {
        matches.is_present("print_ir")
    }
    pub fn get_inspect_constraints(matches: &ArgMatches) -> bool {
        matches.is_present("inspect_constraints")
    }

    pub fn get_flag_verbose(matches: &ArgMatches) -> bool {
        matches.is_present("flag_verbose")
    }

    pub fn get_flag_old_heuristics(matches: &ArgMatches) -> bool {
        matches.is_present("flag_old_heuristics")
    }
    pub fn get_prime(matches: &ArgMatches) -> Result<String, ()> {
        
        match matches.is_present("prime"){
            true => 
               {
                   let prime_value = matches.value_of("prime").unwrap();
                   if prime_value == "bn128"
                      || prime_value == "bls12381"
                      || prime_value == "goldilocks"
                      || prime_value == "grumpkin"
                      || prime_value == "pallas"
                      || prime_value == "vesta"
                      || prime_value == "secq256r1"
                      {
                        Ok(String::from(matches.value_of("prime").unwrap()))
                    }
                    else{
                        Result::Err(eprintln!("{}", Colour::Red.paint("invalid prime number")))
                    }
               }
               
            false => Ok(String::from("bn128")),
        }
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
                Arg::with_name("no_simplification")
                    .long("O0")
                    .hidden(false)
                    .takes_value(false)
                    .help("No simplification is applied")
                    .display_order(420)
            )
            .arg(
                Arg::with_name("reduced_simplification")
                    .long("O1")
                    .hidden(false)
                    .takes_value(false)
                    .help("Only applies signal to signal and signal to constant simplification")
                    .display_order(460)
            )
            .arg(
                Arg::with_name("full_simplification")
                    .long("O2")
                    .takes_value(false)
                    .hidden(false)
                    .help("Full constraint simplification")
                    .display_order(480)
            )
            .arg(
                Arg::with_name("simplification_rounds")
                    .long("O2round")
                    .takes_value(true)
                    .hidden(false)
                    .help("Maximum number of rounds of the simplification process")
                    .display_order(500)
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .default_value(".")
                    .display_order(1)
                    .help("Path to the directory where the output will be written"),
            )
            .arg(
                Arg::with_name("print_json_c")
                    .long("json")
                    .takes_value(false)
                    .display_order(120)
                    .help("Outputs the constraints in json format"),
            )
            .arg(
                Arg::with_name("print_ir")
                    .long("irout")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(360)
                    .help("Outputs the low-level IR of the given circom program"),
            )
            .arg(
                Arg::with_name("inspect_constraints")
                    .long("inspect")
                    .takes_value(false)
                    .display_order(801)
                    .help("Does an additional check over the constraints produced"),
            )
            .arg(
                Arg::with_name("print_json_sub")
                    .long("simplification_substitution")
                    .takes_value(false)
                    .display_order(980)
                    .help("Outputs the substitution applied in the simplification phase in json format"),
            )
            .arg(
                Arg::with_name("print_sym")
                    .long("sym")
                    .takes_value(false)
                    .display_order(60)
                    .help("Outputs witness in sym format"),
            )
            .arg(
                Arg::with_name("print_r1cs")
                    .long("r1cs")
                    .takes_value(false)
                    .display_order(30)
                    .help("Outputs the constraints in r1cs format"),
            )
            .arg(
                Arg::with_name("print_wasm")
                    .long("wasm")
                    .takes_value(false)
                    .display_order(90)
                    .help("Compiles the circuit to wasm"),
            )
            .arg(
                Arg::with_name("print_wat")
                    .long("wat")
                    .takes_value(false)
                    .display_order(120)
                    .help("Compiles the circuit to wat"),
            )
            .arg(
                Arg::with_name("link_libraries")
                .short("l")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)   
                .display_order(330) 
                .help("Adds directory to library search path"),
            )
            .arg(
                Arg::with_name("print_c")
                    .long("c")
                    .short("c")
                    .takes_value(false)
                    .display_order(150)
                    .help("Compiles the circuit to c"),
            )
            .arg(
                Arg::with_name("parallel_simplification")
                    .long("parallel")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(180)
                    .help("Runs non-linear simplification in parallel"),
            )
            .arg(
                Arg::with_name("main_inputs_log")
                    .long("inputs")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(210)
                    .help("Produces a log_inputs.txt file"),
            )
            .arg(
                Arg::with_name("flag_verbose")
                    .long("verbose")
                    .takes_value(false)
                    .display_order(800)
                    .help("Shows logs during compilation"),
            )
            .arg(
                Arg::with_name("flag_old_heuristics")
                    .long("use_old_simplification_heuristics")
                    .takes_value(false)
                    .display_order(980)
                    .help("Applies the old version of the heuristics when performing linear simplification"),
            )
            .arg (
                Arg::with_name("prime")
                    .short("prime")
                    .long("prime")
                    .takes_value(true)
                    .default_value("bn128")
                    .display_order(300)
                    .help("To choose the prime number to use to generate the circuit. Receives the name of the curve (bn128, bls12381, goldilocks, grumpkin, pallas, vesta, secq256r1)"),
            )
            .get_matches()
    }

    pub fn get_link_libraries(matches: &ArgMatches) -> Vec<PathBuf> {
        let mut link_libraries = Vec::new();
        let m = matches.values_of("link_libraries");
        if let Some(paths) = m {
            for path in paths.into_iter() {
                link_libraries.push(Path::new(path).to_path_buf());
            }
        }
        link_libraries
    }
}
