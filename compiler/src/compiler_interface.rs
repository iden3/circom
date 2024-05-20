use virtual_fs::{FileSystem, FsResult};

pub use crate::circuit_design::circuit::{Circuit, CompilationFlags};
pub use crate::hir::very_concrete_program::VCP;

pub struct Config {
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub wat_flag: bool,
}

pub fn run_compiler(fs: &mut dyn FileSystem, vcp: VCP, config: Config, version: &str) -> FsResult<Circuit> {
    let flags = CompilationFlags { main_inputs_log: config.produce_input_log, wat_flag: config.wat_flag };
    let circuit = Circuit::build(fs, vcp, flags, version);
    if config.debug_output {
        produce_debug_output(fs, &circuit)?;
    }
    Ok(circuit)
}

pub fn write_wasm(fs: &mut dyn FileSystem, circuit: &Circuit, js_folder: &str, wasm_name: &str, file: &str) -> FsResult<()> {
    fs.rimraf(&js_folder.into())?;
    fs.create_dir(&js_folder.into())?;
    let mut data = Vec::<u8>::new();
    circuit.produce_wasm(fs, js_folder, wasm_name, &mut data)?;
    fs.write(&file.into(), &data)
}

pub fn write_c(fs: &mut dyn FileSystem, circuit: &Circuit, c_folder: &str, c_run_name: &str, c_file: &str, dat_file: &str) -> FsResult<()> {
    fs.rimraf(&c_folder.into())?;
    fs.create_dir(&c_folder.into())?;
    let mut c_data = Vec::<u8>::new();
    let mut dat_data = Vec::<u8>::new();
    circuit.produce_c(fs, c_folder, c_run_name, &mut c_data, &mut dat_data)?;
    fs.write(&c_file.into(), &c_data)?;
    fs.write(&dat_file.into(), &dat_data)
}

fn produce_debug_output(fs: &mut dyn FileSystem, circuit: &Circuit) -> FsResult<()> {
    let path = format!("ir_log");
    fs.rimraf(&path.clone().into())?;
    fs.create_dir(&path.into())?;
    for id in 0..circuit.templates.len() {
        let mut file = fs.cwd()?;
        file.push(&format!("ir_log/template_{}.txt", id));
        let body = circuit.produce_ir_string_for_template(id);
        fs.write(&file, body.as_bytes())?;
    }
    for id in 0..circuit.functions.len() {
        let mut file = fs.cwd()?;
        file.push(&format!("ir_log/function_{}.txt", id));
        let body = circuit.produce_ir_string_for_function(id);
        fs.write(&file, body.as_bytes())?;
    }
    Result::Ok(())
}
