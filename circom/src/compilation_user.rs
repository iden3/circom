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
    pub no_asm_flag: bool,
    pub sanity_check_style: usize,

    pub prime: String,
}

pub fn compile(config: CompilerConfig) -> Result<(), ()> {


    if config.c_flag || config.wat_flag || config.wasm_flag{
        let circuit = compiler_interface::run_compiler(
            config.vcp,
            Config { 
                debug_output: config.debug_output, 
                produce_input_log: config.produce_input_log, 
                wat_flag: config.wat_flag,
                no_asm_flag: config.no_asm_flag,
                sanity_check_style: config.sanity_check_style,

            },
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
            if config.no_asm_flag {                
                println!(
                    "{} {}/{}, {}, {}, {}, {}, {} and {}",
                    Colour::Green.paint("Written successfully:"),
                    &config.c_folder,
                    "main.cpp".to_string(),
                    "circom.hpp".to_string(),
                    "calcwit.hpp".to_string(),
                    "calcwit.cpp".to_string(),
                    "fr.hpp".to_string(),
                    "fr.cpp".to_string(),
                    "Makefile".to_string()
                );
            } else {
                if config.prime == "goldilocks" {
                    println!(
                        "{} {}/{}, {}, {}, {}, {}, {} and {}",
                        Colour::Green.paint("Written successfully:"),
                        &config.c_folder,
                        "main.cpp".to_string(),
                        "circom.hpp".to_string(),
                        "calcwit.hpp".to_string(),
                        "calcwit.cpp".to_string(),
                        "fr.hpp".to_string(),
                        "Makefile".to_string(),
                        "json2bin64.cpp".to_string()
                    );
                } else {
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
            }
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

    let wat_contents = read_to_string(wat_file).map_err(|err| Report::error(
        format!("Error reading generated WAT file '{}': {}", wat_file, err),
        ReportCode::ErrorWat2Wasm,
    ))?;
    let buf = ParseBuffer::new(&wat_contents).map_err(|err| Report::error(
        format!("Error translating the circuit from wat to wasm.\n\nException encountered when parsing WAT: {}", err),
        ReportCode::ErrorWat2Wasm,
    ))?;
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
                    let file = File::create(wasm_file).map_err(|err| Report::error(
                        format!("Error creating WASM file '{}': {}", wasm_file, err),
                        ReportCode::ErrorWat2Wasm,
                    ))?;
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

#[cfg(test)]
mod tests {
    use super::wat_to_wasm;
    use program_structure::error_code::ReportCode;
    use program_structure::error_definition::Report;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "circom-wat-to-wasm-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn convert(&self) -> Result<(), Report> {
            wat_to_wasm(
                self.0.join("input.wat").to_str().unwrap(),
                self.0.join("output.wasm").to_str().unwrap(),
            )
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn assert_wat_error(result: Result<(), Report>, message: &str) {
        let report = result.err().expect("expected a WAT-to-WASM error report");
        assert!(matches!(report.get_code(), ReportCode::ErrorWat2Wasm));
        assert!(report.is_error());
        assert!(
            report.get_message().contains(message),
            "{}",
            report.get_message()
        );
    }

    #[test]
    fn wat_to_wasm_reports_missing_input() {
        let dir = TestDir::new();
        assert_wat_error(dir.convert(), "Error reading generated WAT file");
        assert!(!dir.0.join("output.wasm").exists());
    }

    #[test]
    fn wat_to_wasm_reports_lexer_error() {
        let dir = TestDir::new();
        fs::write(dir.0.join("input.wat"), "(; unterminated comment").unwrap();
        assert_wat_error(dir.convert(), "Exception encountered when parsing WAT");
        assert!(!dir.0.join("output.wasm").exists());
    }

    #[test]
    fn wat_to_wasm_reports_output_creation_error() {
        let dir = TestDir::new();
        fs::write(dir.0.join("input.wat"), "(module)").unwrap();
        fs::create_dir(dir.0.join("output.wasm")).unwrap();
        assert_wat_error(dir.convert(), "Error creating WASM file");
        assert!(dir.0.join("output.wasm").is_dir());
    }

    #[test]
    fn wat_to_wasm_converts_valid_module() {
        let dir = TestDir::new();
        fs::write(dir.0.join("input.wat"), "(module)").unwrap();
        assert!(dir.convert().is_ok());
        assert_eq!(
            fs::read(dir.0.join("output.wasm")).unwrap(),
            b"\0asm\x01\0\0\0"
        );
    }
}
