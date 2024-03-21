use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Clone)]
pub struct ReturnBucket {
    pub line: usize,
    pub message_id: usize,
    pub with_size: InstrContext,
    pub value: InstructionPointer,
}

impl IntoInstruction for ReturnBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Return(self)
    }
}

impl Allocate for ReturnBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for ReturnBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for ReturnBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let value = self.value.to_string();
        format!("RETURN(line: {},template_id: {},value: {})", line, template_id, value)
    }
}

impl WriteWasm for ReturnBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; return bucket".to_string());
	}
        if self.with_size.size == 1 {
            instructions.push(get_local(producer.get_result_address_tag())); //result address
            let mut instructions_value = self.value.produce_wasm(producer);
            instructions.append(&mut instructions_value);
            instructions.push(call("$Fr_copy"));
        } else {
            let mut instructions_value = self.value.produce_wasm(producer);
            instructions.append(&mut instructions_value); // value evaluation address
            instructions.push(set_local(producer.get_store_aux_2_tag())); // value evaluation temp address
            instructions.push(add_block());
            instructions.push(add_loop());
            instructions.push(get_local(producer.get_result_size_tag()));
            instructions.push(eqz32());
            instructions.push(br_if("1"));
            instructions.push(get_local(producer.get_result_address_tag())); //result address
            instructions.push(get_local(producer.get_store_aux_2_tag()));
            instructions.push(call("$Fr_copy"));
            instructions.push(get_local(producer.get_result_size_tag())); // update get_result_size
            instructions.push(set_constant("1"));
            instructions.push(sub32());
            instructions.push(set_local(producer.get_result_size_tag())); // update get_result_size
            instructions.push(get_local(producer.get_result_address_tag())); //prepare next result address
            let s = (producer.get_size_32_bit() + 2) * 4;
            instructions.push(set_constant(&s.to_string()));
            instructions.push(add32());
            instructions.push(set_local(producer.get_result_address_tag()));
            instructions.push(get_local(producer.get_store_aux_2_tag()));
            instructions.push(set_constant(&s.to_string()));
            instructions.push(add32());
            instructions.push(set_local(producer.get_store_aux_2_tag()));
            instructions.push(br("0"));
            instructions.push(add_end());
            instructions.push(add_end());
        }
        let mut free_stack_code = free_stack(producer);
        instructions.append(&mut free_stack_code);
        instructions.push(set_constant("0"));	
        instructions.push(add_return());
        if producer.needs_comments() {
            instructions.push(";; end of return bucket".to_string());
	}
        instructions
    }
}

impl WriteC for ReturnBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut instructions = vec![];
        instructions.push("// return bucket".to_string());
        let (mut instructions_value, src) = self.value.produce_c(producer, parallel);
        instructions.append(&mut instructions_value);
        
        let real_size = if self.with_size.variable_size.is_none() {
            self.with_size.size.to_string()
        } else{
            self.with_size.variable_size.as_ref().unwrap().clone()
        };

        let warning_condition = format!("{} > {}", FUNCTION_DESTINATION_SIZE, real_size); 
        let check_correct_size_1 = format!(
            "if({}){{
                {}
            }}", 
            warning_condition,
            build_warning_return_message(self.line, &real_size)
        );
        
        let error_condition = format!("{} >= {}", FUNCTION_DESTINATION_SIZE, real_size); 

        let check_correct_size_2 = format!(
            "if(!({})){{
                {}
            }}", 
            error_condition,
            build_failed_return_message(self.line, &real_size)
        );
        let assertion = format!("{};", build_call("assert".to_string(), vec![error_condition]));

        instructions.push(check_correct_size_1);
        instructions.push(check_correct_size_2);
        instructions.push(assertion);



        if real_size != "1" {   
            let copy_arguments =
                vec![FUNCTION_DESTINATION.to_string(), src, real_size.clone()];
            instructions.push(format!("{};", build_call("Fr_copyn".to_string(), copy_arguments)));
        } else {
            let copy_arguments = vec![FUNCTION_DESTINATION.to_string(), src];
            instructions.push(format!("{};", build_call("Fr_copy".to_string(), copy_arguments)));
        }
        // to update the length of the stored value
        instructions.push(format!("{} = {};", FUNCTION_RETURN_SIZE, real_size));
        instructions.push(add_return());
        (instructions, "".to_string())
    }
}
