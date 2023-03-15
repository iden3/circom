use ansi_term::Colour;
use compiler::compiler_interface;
use compiler::compiler_interface::{Config, VCP};
use program_structure::error_definition::Report;
use program_structure::error_code::ReportCode;
use program_structure::file_definition::FileLibrary;
use crate::VERSION;


pub struct CompilerConfig {
    pub js_folder: String,
    pub wasm_name: String,
    pub wat_file: String,
    pub wasm_file: String,
    pub c_folder: String,
    pub c_run_name: String,
    pub c_file: String,
    pub dat_file: String,
    pub wat_flag: bool,
    pub wasm_flag: bool,
    pub c_flag: bool,
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub vcp: VCP,
}

pub fn compile(config: CompilerConfig) -> Result<(), ()> {


    if config.c_flag || config.wat_flag || config.wasm_flag{
        let circuit = compiler_interface::run_compiler(
            config.vcp,
            Config { debug_output: config.debug_output, produce_input_log: config.produce_input_log, wat_flag: config.wat_flag },
            VERSION
        )?;
    
        if config.c_flag {
            compiler_interface::write_c(&circuit, &config.c_folder, &config.c_run_name, &config.c_file, &config.dat_file)?;
            println!(
                "{} {} and {}",
                Colour::Green.paint("Written successfully:"),
                config.c_file,
                config.dat_file
            );
            println!(
                "{} {}/{}, {}, {}, {}, {}, {}, {} and {}",
                Colour::Green.paint("Written successfully:"),
            &config.c_folder,
                "main.cpp".to_string(),
                "circom.hpp".to_string(),
                "calcwit.hpp".to_string(),
                "calcwit.cpp".to_string(),
                "fr.hpp".to_string(),
                "fr.cpp".to_string(),
                "fr.asm".to_string(),
                "Makefile".to_string()
            );
        }
    
        match (config.wat_flag, config.wasm_flag) {
            (true, true) => {
                compiler_interface::write_wasm(&circuit, &config.js_folder, &config.wasm_name, &config.wat_file)?;
                println!("{} {}", Colour::Green.paint("Written successfully:"), config.wat_file);
                let result = wat_to_wasm(&config.wat_file, &config.wasm_file);
                match result {
                    Result::Err(report) => {
                        Report::print_reports(&[report], &FileLibrary::new());
                        return Err(());
                    }
                    Result::Ok(()) => {
                        println!("{} {}", Colour::Green.paint("Written successfully:"), config.wasm_file);
                    }
                }
            }
            (false, true) => {
                compiler_interface::write_wasm(&circuit,  &config.js_folder, &config.wasm_name, &config.wat_file)?;
                let result = wat_to_wasm(&config.wat_file, &config.wasm_file);
                std::fs::remove_file(&config.wat_file).unwrap();
                match result {
                    Result::Err(report) => {
                        Report::print_reports(&[report], &FileLibrary::new());
                        return Err(());
                    }
                    Result::Ok(()) => {
                        println!("{} {}", Colour::Green.paint("Written successfully:"), config.wasm_file);
                    }
                }
            }
            (true, false) => {
                compiler_interface::write_wasm(&circuit,  &config.js_folder, &config.wasm_name, &config.wat_file)?;
                println!("{} {}", Colour::Green.paint("Written successfully:"), config.wat_file);
            }
            (false, false) => {}
        }
    }
    

    Ok(())
}


fn wat_to_wasm(wat_file: &str, wasm_file: &str) -> Result<(), Report> {
    use std::fs::read_to_string;
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;
    use wast::Wat;
    use wast::parser::{self, ParseBuffer};

    let wat_contents = read_to_string(wat_file).unwrap();
    let buf = ParseBuffer::new(&wat_contents).unwrap();
    let result_wasm_contents = parser::parse::<Wat>(&buf);
    match result_wasm_contents {
        Result::Err(error) => {
            Result::Err(Report::error(
                format!("Error translating the circuit from wat to wasm.\n\nException encountered when parsing WAT: {}", error),
                ReportCode::ErrorWat2Wasm,
            ))
        }
        Result::Ok(mut wat) => {
            let wasm_contents = wat.module.encode();
            match wasm_contents {
                Result::Err(error) => {
                    Result::Err(Report::error(
                        format!("Error translating the circuit from wat to wasm.\n\nException encountered when encoding WASM: {}", error),
                        ReportCode::ErrorWat2Wasm,
                    ))
                }
                Result::Ok(wasm_contents) => {
                    let file = File::create(wasm_file).unwrap();
                    let mut writer = BufWriter::new(file);
                    writer.write_all(&wasm_contents).map_err(|_err| Report::error(
                        format!("Error writing the circuit. Exception generated: {}", _err),
                        ReportCode::ErrorWat2Wasm,
                    ))?;
                    writer.flush().map_err(|_err| Report::error(
                        format!("Error writing the circuit. Exception generated: {}", _err),
                        ReportCode::ErrorWat2Wasm,
                    ))?;
                    Ok(())
                }
            }
        }
    }
}
