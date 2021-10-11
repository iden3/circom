use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Clone)]
pub struct CreateCmpBucket {
    pub line: usize,
    pub message_id: usize,
    pub template_id: usize,
    pub cmp_unique_id: usize,
    pub symbol: String,
    pub sub_cmp_id: InstructionPointer,
    // signal offset with respect to the start of the father's signals
    pub signal_offset: usize,
    pub signal_offset_jump: usize,
    // component number offset with respect to the father's component id
    pub component_offset: usize,
    pub component_offset_jump:usize, 
    pub number_of_cmp: usize,
    pub is_parallel: bool,
}

impl IntoInstruction for CreateCmpBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::CreateCmp(self)
    }
}

impl Allocate for CreateCmpBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for CreateCmpBucket {
    fn get_line(&self) -> usize {
        self.line
    }

    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for CreateCmpBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let id_no = self.sub_cmp_id.to_string();
        format!(
            "CREATE_CMP(line:{},template_id:{},name:{},id_no:{})",
            line, template_id, self.symbol, id_no
        )
    }
}

impl WriteWasm for CreateCmpBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        instructions.push(";; create component bucket".to_string());
        //obtain address of the subcomponent inside the component
        instructions.push(get_local(producer.get_offset_tag()));
        instructions
            .push(set_constant(&producer.get_sub_component_start_in_component().to_string()));
        instructions.push(add32());
        let mut instructions_sci = self.sub_cmp_id.produce_wasm(producer);
        instructions.append(&mut instructions_sci);
        instructions.push(set_constant("4")); //size in byte of i32
        instructions.push(mul32());
        instructions.push(add32()); // address of the subcomponent in the component
        if self.number_of_cmp > 1 {
            instructions.push(set_local(producer.get_create_loop_sub_cmp_tag()));
        }
        let signal_full_offset = self.signal_offset * producer.get_size_32_bits_in_memory() * 4;
        instructions.push(set_constant(&signal_full_offset.to_string())); //offset of the signals in the subcomponent
        instructions.push(get_local(producer.get_signal_start_tag()));
        instructions.push(add32());
        if self.number_of_cmp > 1 {
            instructions.push(set_local(producer.get_create_loop_offset_tag()));
        }
        if self.number_of_cmp > 1 {
            instructions.push(set_constant(&self.number_of_cmp.to_string()));
            instructions.push(set_local(producer.get_create_loop_counter_tag()));
            instructions.push(add_block());
            instructions.push(add_loop());
            instructions.push(get_local(producer.get_create_loop_sub_cmp_tag())); //sub_component address in component
            instructions.push(get_local(producer.get_create_loop_offset_tag()));
            //sub_component signal address start
        }
        instructions.push(call(&format!("${}_create", self.symbol)));
        instructions.push(store32(None)); //store the offset given by create in the subcomponent address
        if self.number_of_cmp > 1 {
            instructions.push(get_local(producer.get_create_loop_counter_tag()));
            instructions.push(set_constant("1"));
            instructions.push(sub32());
            instructions.push(set_local(producer.get_create_loop_counter_tag()));
            instructions.push(get_local(producer.get_create_loop_counter_tag()));
            instructions.push(eqz32());
            instructions.push(br_if("1"));
            // next sub_cmp  is 4 bytes later
            instructions.push(get_local(producer.get_create_loop_sub_cmp_tag()));
            instructions.push(set_constant("4"));
            instructions.push(add32());
            instructions.push(set_local(producer.get_create_loop_sub_cmp_tag()));
            // next signal offset
            instructions.push(get_local(producer.get_create_loop_offset_tag()));
            let jump_in_bytes = self.signal_offset_jump * producer.get_size_32_bits_in_memory() * 4;
            instructions.push(set_constant(&jump_in_bytes.to_string()));
            instructions.push(add32());
            instructions.push(set_local(producer.get_create_loop_offset_tag()));
            //back to loop, to create next component
            instructions.push(br("0"));
            instructions.push(add_end());
            instructions.push(add_end());
        }
        instructions.push(";; end create component bucket".to_string());
        instructions
    }
}

impl WriteC for CreateCmpBucket {
    fn produce_c(&self, producer: &CProducer) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut instructions = vec![];
        let (mut scmp_idx_instructions, scmp_idx) = self.sub_cmp_id.produce_c(producer);
        instructions.append(&mut scmp_idx_instructions);
        std::mem::drop(scmp_idx_instructions);
        instructions.push("{".to_string());
        instructions.push(format!("uint aux_create = {};", scmp_idx));
     //instructions.push(format!("int aux_cmp_num = {}+{}+{};", self.component_offset, CTX_INDEX, scmp_idx));
     instructions.push(format!("int aux_cmp_num = {}+{}+1;", self.component_offset, CTX_INDEX));
        instructions.push(format!("uint csoffset = {}+{};", MY_SIGNAL_START.to_string(), self.signal_offset));
        instructions.push(format!("for (uint i = 0; i < {}; i++) {{", self.number_of_cmp));
        let sub_cmp_template_create = format!("{}_create", self.symbol);
    let create_args = vec!["csoffset".to_string(), "aux_cmp_num".to_string(), CIRCOM_CALC_WIT.to_string()];
        //let create_args = vec!["csoffset".to_string(), CIRCOM_CALC_WIT.to_string()]; // comment
        let create_call = build_call(sub_cmp_template_create, create_args);
	instructions.push(format!("{}[aux_create+i] = aux_cmp_num;", MY_SUBCOMPONENTS));
	// instructions.push(format!("printf(\"cmp_id: %d\\n\",{}[aux_create+i]);",MY_SUBCOMPONENTS));

    instructions.push(format!("{};", create_call));
        //instructions.push(format!("{}[aux_create+i] = {};", MY_SUBCOMPONENTS, create_call)); //comment
        instructions.push(format!("csoffset += {} ;", self.signal_offset_jump));
	instructions.push(format!("aux_cmp_num += {};",self.component_offset_jump));
        instructions.push("}".to_string());
        instructions.push("}".to_string());
        (instructions, "".to_string())
    }
}
