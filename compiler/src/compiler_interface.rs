use vfs::VfsResult;

pub use crate::circuit_design::circuit::{Circuit, CompilationFlags};
pub use crate::hir::very_concrete_program::VCP;
use std::io::BufWriter;

pub struct Config {
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub wat_flag: bool,
}

pub fn run_compiler(fs: &dyn vfs::FileSystem, vcp: VCP, config: Config, version: &str) -> Result<Circuit, ()> {
    let flags = CompilationFlags { main_inputs_log: config.produce_input_log, wat_flag: config.wat_flag };
    let circuit = Circuit::build(fs, vcp, flags, version);
    if config.debug_output {
        produce_debug_output(fs, &circuit)?;
    }
    Ok(circuit)
}

pub fn write_wasm(fs: &dyn vfs::FileSystem, circuit: &Circuit, js_folder: &str, wasm_name: &str, file: &str) -> Result<(), ()> {
    fs_rimraf(fs, js_folder).map_err(|_err| {})?;
    fs.create_dir(js_folder).map_err(|_err| {})?;
    let file = fs.create_file(file).map_err(|_err| {})?;
    let mut writer = BufWriter::new(file);
    circuit.produce_wasm(fs, js_folder, wasm_name, &mut writer)
}

pub fn write_c(fs: &dyn vfs::FileSystem, circuit: &Circuit, c_folder: &str, c_run_name: &str, c_file: &str, dat_file: &str) -> Result<(), ()> {
    fs_rimraf(fs, c_folder).map_err(|_err| {})?;
    fs.create_dir(c_folder).map_err(|_err| {})?;
    let dat_file = fs.create_file(dat_file).map_err(|_err| {})?;
    let c_file = fs.create_file(c_file).map_err(|_err| {})?;
    let mut c_file = BufWriter::new(c_file);
    let mut dat_file = BufWriter::new(dat_file);
    circuit.produce_c(fs, c_folder, c_run_name, &mut c_file, &mut dat_file)
}

fn produce_debug_output(fs: &dyn vfs::FileSystem, circuit: &Circuit) -> Result<(), ()> {
    use std::io::Write;
    use std::path::Path;
    let path = format!("ir_log");
    fs_rimraf(fs, &path).map_err(|_err| {})?;
    fs.create_dir(&path).map_err(|_err| {})?;
    for id in 0..circuit.templates.len() {
        let file = Path::new(&format!("ir_log/template_{}.txt", id)).canonicalize().unwrap().to_str().unwrap().to_string();
        let file_signals = fs.create_file(&file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file_signals);
        let body = circuit.produce_ir_string_for_template(id);
        writer.write_all(body.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    for id in 0..circuit.functions.len() {
        let file = Path::new(&format!("ir_log/function_{}.txt", id)).canonicalize().unwrap().to_str().unwrap().to_string();
        let file_signals = fs.create_file(&file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file_signals);
        let body = circuit.produce_ir_string_for_function(id);
        writer.write_all(body.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    Result::Ok(())
}

fn fs_rimraf(fs: &dyn vfs::FileSystem, path: &str) -> VfsResult<()> {
    if path == "" || path == "/" {
        panic!("Refused `rm -rf /` catastrophe");
    }

    if !fs.exists(path)? {
        return Ok(());
    }

    match fs.metadata(path)?.file_type {
        vfs::VfsFileType::File => fs.remove_file(path),
        vfs::VfsFileType::Directory => {
            for child in fs.read_dir(path)? {
                fs_rimraf(fs, &format!("{}/{}", path, child))?;
            }

            fs.remove_dir(path)
        }
    }
}
