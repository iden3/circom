use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Clone)]
pub struct AssertBucket {
    pub line: usize,
    pub message_id: usize,
    pub evaluate: InstructionPointer,
}

impl IntoInstruction for AssertBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Assert(self)
    }
}

impl Allocate for AssertBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for AssertBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for AssertBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let evaluate = self.evaluate.to_string();
        format!("ASSERT(line: {},template_id: {},evaluate: {})", line, template_id, evaluate)
    }
}

impl WriteWasm for AssertBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; assert bucket".to_string());
	}
        let mut instructions_eval = self.evaluate.produce_wasm(producer);
        instructions.append(&mut instructions_eval);
        instructions.push(call("$Fr_isTrue"));
        instructions.push(eqz32());
        instructions.push(add_if());
        instructions.push(set_constant(&self.message_id.to_string()));
        instructions.push(set_constant(&self.line.to_string()));
        instructions.push(call("$buildBufferMessage"));
        instructions.push(call("$printErrorMessage"));
        instructions.push(set_constant(&exception_code_assert_fail().to_string()));
        instructions.push(add_return());
        instructions.push(add_end());
        if producer.needs_comments() {
            instructions.push(";; end of assert bucket".to_string());
	}
        instructions
    }
}

impl WriteC for AssertBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let (prologue, value) = self.evaluate.produce_c(producer, parallel);
        let is_true = build_call("Fr_isTrue".to_string(), vec![value]);
        let if_condition = format!("if (!{}) {};", is_true, build_failed_assert_message(self.line));    
        let assertion = format!("{};", build_call("assert".to_string(), vec![is_true]));
        let mut assert_c = prologue;
        assert_c.push(if_condition);
        assert_c.push(assertion);
        (assert_c, "".to_string())
    }
}
