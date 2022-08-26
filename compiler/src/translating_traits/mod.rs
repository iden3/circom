use code_producers::c_elements::*;
use code_producers::wasm_elements::*;
use std::io::Write;

pub trait WriteC {
    /*
        returns (x, y) where:
            x: c instructions produced.
            y: if the instructions in x compute some value, that value is stored in y.
    */
    fn produce_c(&self, producer: &CProducer, is_parallel: Option<bool>) -> (Vec<String>, String);
    fn write_c<T: Write>(&self, writer: &mut T, producer: &CProducer) -> Result<(), ()> {
        use code_producers::wasm_elements::wasm_code_generator::merge_code;
        let (c_instructions, _) = self.produce_c(producer, None);
        let code = merge_code(c_instructions);
        writer.write_all(code.as_bytes()).map_err(|_| {})?;
        writer.flush().map_err(|_| {})
    }
}

pub trait WriteWasm {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String>;
    fn write_wasm<T: Write>(&self, writer: &mut T, producer: &WASMProducer) -> Result<(), ()> {
        let wasm_instructions = self.produce_wasm(producer);
        let code = wasm_code_generator::merge_code(wasm_instructions);
        writer.write_all(code.as_bytes()).map_err(|_| {})?;
        writer.flush().map_err(|_| {})
    }
}
