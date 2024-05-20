use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

pub trait WriteC {
    /*
        returns (x, y) where:
            x: c instructions produced.
            y: if the instructions in x compute some value, that value is stored in y.
    */
    fn produce_c(&self, producer: &CProducer, is_parallel: Option<bool>) -> (Vec<String>, String);
    fn write_c(&self, data: &mut Vec<u8>, producer: &CProducer) {
        use code_producers::wasm_elements::wasm_code_generator::merge_code;
        let (c_instructions, _) = self.produce_c(producer, None);
        let code = merge_code(c_instructions);
        data.extend_from_slice(code.as_bytes());
    }
}

pub trait WriteWasm {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String>;
    fn write_wasm(&self, data: &mut Vec<u8>, producer: &WASMProducer) {
        let wasm_instructions = self.produce_wasm(producer);
        let code = wasm_code_generator::merge_code(wasm_instructions);
        data.extend_from_slice(code.as_bytes());
    }
}
