use vfs::FileSystem;
use vfs_utils::{rimraf, SimplePath};

pub use crate::circuit_design::circuit::{Circuit, CompilationFlags};
pub use crate::hir::very_concrete_program::VCP;
use std::io::BufWriter;

pub struct Config {
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub wat_flag: bool,
}

pub fn run_compiler(fs: &dyn FileSystem, cwd: &str, vcp: VCP, config: Config, version: &str) -> Result<Circuit, ()> {
    let flags = CompilationFlags { main_inputs_log: config.produce_input_log, wat_flag: config.wat_flag };
    let circuit = Circuit::build(fs, cwd, vcp, flags, version);
    if config.debug_output {
        produce_debug_output(fs, cwd, &circuit)?;
    }
    Ok(circuit)
}

pub fn write_wasm(fs: &dyn FileSystem, circuit: &Circuit, js_folder: &str, wasm_name: &str, file: &str) -> Result<(), ()> {
    rimraf(fs, js_folder).map_err(|_err| {})?;
    fs.create_dir(js_folder).map_err(|_err| {})?;
    let file = fs.create_file(file).map_err(|_err| {})?;
    let mut writer = BufWriter::new(file);
    circuit.produce_wasm(fs, js_folder, wasm_name, &mut writer)
}

pub fn write_c(fs: &dyn FileSystem, circuit: &Circuit, c_folder: &str, c_run_name: &str, c_file: &str, dat_file: &str) -> Result<(), ()> {
    rimraf(fs, c_folder).map_err(|_err| {})?;
    fs.create_dir(c_folder).map_err(|_err| {})?;
    let dat_file = fs.create_file(dat_file).map_err(|_err| {})?;
    let c_file = fs.create_file(c_file).map_err(|_err| {})?;
    let mut c_file = BufWriter::new(c_file);
    let mut dat_file = BufWriter::new(dat_file);
    circuit.produce_c(fs, c_folder, c_run_name, &mut c_file, &mut dat_file)
}

fn produce_debug_output(fs: &dyn FileSystem, cwd: &str, circuit: &Circuit) -> Result<(), ()> {
    use std::io::Write;
    let path = format!("ir_log");
    rimraf(fs, &path).map_err(|_err| {})?;
    fs.create_dir(&path).map_err(|_err| {})?;
    for id in 0..circuit.templates.len() {
        let mut file = SimplePath::new(cwd);
        file.push(&format!("ir_log/template_{}.txt", id));
        let file = file.to_string();
        let file_signals = fs.create_file(&file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file_signals);
        let body = circuit.produce_ir_string_for_template(id);
        writer.write_all(body.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    for id in 0..circuit.functions.len() {
        let mut file = SimplePath::new(cwd);
        file.push(&format!("ir_log/function_{}.txt", id));
        let file = file.to_string();
        let file_signals = fs.create_file(&file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file_signals);
        let body = circuit.produce_ir_string_for_function(id);
        writer.write_all(body.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    Result::Ok(())
}
