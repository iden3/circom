use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Clone)]
pub struct LogBucket {
    pub line: usize,
    pub message_id: usize,
    pub print: InstructionPointer,
    pub is_parallel: bool,
}

impl IntoInstruction for LogBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Log(self)
    }
}

impl Allocate for LogBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for LogBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for LogBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let print = self.print.to_string();
        format!("LOG(line: {},template_id: {},evaluate: {})", line, template_id, print)
    }
}

impl WriteWasm for LogBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; log bucket".to_string());
	}
        let mut instructions_print = self.print.produce_wasm(producer);
        instructions.append(&mut instructions_print);
        instructions.push(call("$copyFr2SharedRWMemory"));
        instructions.push(call("$showSharedRWMemory"));
        if producer.needs_comments() {
            instructions.push(";; end of log bucket".to_string());
	}
        instructions
    }
}

impl WriteC for LogBucket {
    fn produce_c(&self, producer: &CProducer) -> (Vec<String>, String) {
        use c_code_generator::*;
        let (argument_code, argument_result) = self.print.produce_c(producer);
        let to_string_call = build_call("Fr_element2str".to_string(), vec![argument_result]);
        let temp_var = "temp".to_string();
        let into_temp = format!("char* temp = {}", to_string_call);
        let print_c =
            build_call("printf".to_string(), vec!["\"%s\\n\"".to_string(), temp_var.clone()]);
        let delete_temp = format!("delete [] {}", temp_var);
        let mut log_c = argument_code;
        log_c.push("{".to_string());
        log_c.push(format!("{};", into_temp));
        log_c.push(format!("{};", print_c));
        log_c.push(format!("{};", delete_temp));
        log_c.push("}".to_string());
        (log_c, "".to_string())
    }
}
