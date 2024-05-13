pub use crate::circuit_design::circuit::{Circuit, CompilationFlags};
pub use crate::hir::very_concrete_program::VCP;
use std::fs::File;
use std::io::BufWriter;

pub struct Config {
    pub print_ir: bool,
    pub produce_input_log: bool,
    pub wat_flag: bool,
}

pub fn run_compiler(vcp: VCP, config: Config, version: &str) -> Result<Circuit, ()> {
    let flags = CompilationFlags { main_inputs_log: config.produce_input_log, wat_flag: config.wat_flag };
    let circuit = Circuit::build(vcp, flags, version);
    if config.print_ir {
        print_ir_representation(&circuit)?;
    }
    Ok(circuit)
}

pub fn write_wasm(circuit: &Circuit, js_folder: &str, wasm_name: &str, file: &str) -> Result<(), ()> {
    use std::path::Path;
    if Path::new(js_folder).is_dir() {
        std::fs::remove_dir_all(js_folder).map_err(|_err| {})?;
    }
    std::fs::create_dir(js_folder).map_err(|_err| {})?;
    let file = File::create(file).map_err(|_err| {})?;
    let mut writer = BufWriter::new(file);
    circuit.produce_wasm(js_folder, wasm_name, &mut writer)
}

pub fn write_c(circuit: &Circuit, c_folder: &str, c_run_name: &str, c_file: &str, dat_file: &str) -> Result<(), ()> {
    use std::path::Path;
    if Path::new(c_folder).is_dir() {
        std::fs::remove_dir_all(c_folder).map_err(|_err| {})?;
    }
    std::fs::create_dir(c_folder).map_err(|_err| {})?;
    let dat_file = File::create(dat_file).map_err(|_err| {})?;
    let c_file = File::create(c_file).map_err(|_err| {})?;
    let mut c_file = BufWriter::new(c_file);
    let mut dat_file = BufWriter::new(dat_file);
    circuit.produce_c(c_folder, c_run_name, &mut c_file, &mut dat_file)
}

fn print_ir_representation(circuit: &Circuit) -> Result<(), ()> {

    // To obtain the information about the number of inputs

    use std::io::Write;
    use std::path::Path;
    let path = format!("ir_log");
    if Path::new(&path).is_dir() {
        std::fs::remove_dir_all(&path).map_err(|_err| {})?;
    }
    {
	let main_inputs = circuit.c_producer.get_main_input_list();
	let signals_in_witness = circuit.c_producer.get_witness_to_signal_list();
	let main_header = circuit.c_producer.get_main_header();
	std::fs::create_dir(&path).map_err(|_err| {})?;
	let gen_file = format!("ir_log/global_info.json");
	let file_global = File::create(gen_file).map_err(|_err| {})?;
	let mut writer = BufWriter::new(file_global);
	let mut lbody = vec![];
	lbody.push("{".to_string());
	lbody.push(format!("\"main\":\"{}\",",main_header.to_string()));
	let s_main_inputs: Vec<String> = main_inputs.into_iter().map(|(x1,x2,x3)| format!("[{},{},{}]",x1.to_string(),x2.to_string(),x3.to_string())).collect();
	lbody.push(format!("\"inputs\":[{}],",s_main_inputs.join(",\n")));
	let s_signals_in_witness: Vec<String> = signals_in_witness.into_iter().map(|x| x.to_string()).collect();
	lbody.push(format!("\"signals_in_witness\":[{}]",s_signals_in_witness.join(",\n")));	
        lbody.push("}".to_string());
	let body = lbody.join("\n");
	writer.write_all(body.as_bytes()).map_err(|_err| {})?;
	writer.flush().map_err(|_err| {})?;
    }
    for id in 0..circuit.templates.len() {
        let file = format!("ir_log/template_{}.json", id);
        let file_signals = File::create(file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file_signals);
        let body = circuit.produce_ir_string_for_template(id);
        writer.write_all(body.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    for id in 0..circuit.functions.len() {
        let file = format!("ir_log/function_{}.json", id);
        let file_signals = File::create(file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file_signals);
        let body = circuit.produce_ir_string_for_function(id);
        writer.write_all(body.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    Result::Ok(())
}
