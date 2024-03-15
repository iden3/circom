use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Clone)]
pub struct ValueBucket {
    pub line: usize,
    pub message_id: usize,
    pub parse_as: ValueType,
    pub op_aux_no: usize,
    pub value: usize,
}

impl IntoInstruction for ValueBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Value(self)
    }
}

impl Allocate for ValueBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for ValueBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for ValueBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let parse_as = self.parse_as.to_string();
        let op_aux_number = self.op_aux_no.to_string();
        let value = self.value;
        format!(
            "VALUE(line:{},template_id:{},as:{},op_number:{},value:{})",
            line, template_id, parse_as, op_aux_number, value
        )
    }
}

impl WriteWasm for ValueBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; value bucket".to_string());
	}
        match &self.parse_as {
            ValueType::U32 => {
                instructions.push(set_constant(&self.value.to_string()));
            }
            ValueType::BigInt => {
                let mut const_pos = self.value;
                const_pos *= (producer.get_size_32_bit() + 2) * 4;
                const_pos += producer.get_constant_numbers_start();
                instructions.push(set_constant(&const_pos.to_string()));
            }
        }
        if producer.needs_comments() {
            instructions.push(";; end of value bucket".to_string());
	}
        instructions
    }
}

impl WriteC for ValueBucket {
    fn produce_c(&self, _producer: &CProducer, _parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let index = self.value.to_string();
        match self.parse_as {
            ValueType::U32 => (vec![], index),
            ValueType::BigInt => {
                let access = format!("&{}", circuit_constants(index));
                (vec![], access)
            }
        }
    }
}
