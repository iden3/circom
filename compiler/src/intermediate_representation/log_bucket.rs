use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;


#[derive(Clone)]
pub enum LogBucketArg {
    LogExp(InstructionPointer),
    LogStr(usize)
}

#[derive(Clone)]
pub struct LogBucket {
    pub line: usize,
    pub message_id: usize,
    pub argsprint: Vec<LogBucketArg>,
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
        let mut ret = String::new();
        for print in self.argsprint.clone() {
            if let LogBucketArg::LogExp(exp) = print {
                let print = exp.to_string();
                let log = format!("LOG(line: {},template_id: {},evaluate: {})", line, template_id, print);
                ret = ret + &log;
            }
        }
        ret
    }
}

impl WriteWasm for LogBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; log bucket".to_string());
	    }
        for logarg in self.argsprint.clone() {
	    match &logarg {
                LogBucketArg::LogExp(exp) => {
                    let mut instructions_print = exp.produce_wasm(producer);
                    instructions.append(&mut instructions_print);
                    instructions.push(call("$copyFr2SharedRWMemory"));
                    instructions.push(call("$showSharedRWMemory"));
	        }
		LogBucketArg::LogStr(stringid) => {
                    let pos = producer.get_string_list_start() +
                              stringid * producer.get_size_of_message_in_bytes();
                    instructions.push(set_constant(&pos.to_string()));
                    instructions.push(call("$buildLogMessage"));
                    instructions.push(call("$writeBufferMessage"));                    
	        }
            }
	}
	// add nl
        instructions.push(set_constant(&producer.get_message_buffer_start().to_string()));
        instructions.push(set_constant("0x0000000a"));
        instructions.push(store32(None)); // stores \n000 
        instructions.push(set_constant(&producer.get_message_buffer_counter_position().to_string()));
        instructions.push(set_constant("0"));
        instructions.push(store32(None));
        instructions.push(call("$writeBufferMessage"));
        if producer.needs_comments() {
            instructions.push(";; end of log bucket".to_string());
	}
        instructions
    }
}

impl WriteC for LogBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut log_c = Vec::new();
        let mut index = 0;
        for logarg in &self.argsprint {
            if let LogBucketArg::LogExp(exp) = logarg {
                let (mut argument_code, argument_result) = exp.produce_c(producer, parallel);
                let to_string_call = build_call("Fr_element2str".to_string(), vec![argument_result]);
                let temp_var = "temp".to_string();
                let into_temp = format!("char* temp = {}", to_string_call);
                let print_c =
                    build_call("printf".to_string(), vec!["\"%s\"".to_string(), temp_var.clone()]);
                let delete_temp = format!("delete [] {}", temp_var);
                log_c.append(&mut argument_code);
                log_c.push("{".to_string());
                log_c.push(format!("{};", into_temp));
                log_c.push(format!("{};", print_c));
                log_c.push(format!("{};", delete_temp));
                log_c.push("}".to_string());
            }
            else if let LogBucketArg::LogStr(string_id) = logarg {
                let string_value = &producer.get_string_table()[*string_id];

                let print_c =
                    build_call(
                        "printf".to_string(), 
                        vec![format!("\"{}\"", string_value)]
                    );
                log_c.push("{".to_string());
                log_c.push(format!("{};", print_c));
                log_c.push("}".to_string());
            }
            else{
                unreachable!();
            }
            if index != self.argsprint.len() - 1 { 
                let print_c =
                    build_call(
                        "printf".to_string(), 
                        vec![format!("\" \"")]
                    );
                log_c.push("{".to_string());
                log_c.push(format!("{};", print_c));
                log_c.push("}".to_string());
            }
            index += 1;
        }
        let print_end_line = build_call(
            "printf".to_string(), 
            vec![format!("\"\\n\"")]
        );
        log_c.push("{".to_string());
        log_c.push(format!("{};", print_end_line));
        log_c.push("}".to_string());
        (log_c, "".to_string())
    }
}
