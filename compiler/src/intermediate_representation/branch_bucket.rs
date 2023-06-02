use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::llvm_elements::{LLVMInstruction, any_value_wraps_basic_value, any_value_to_basic, to_enum, LLVMIRProducer};
use code_producers::llvm_elements::functions::create_bb;
use code_producers::llvm_elements::instructions::{create_br, create_conditional_branch, create_phi};
use code_producers::wasm_elements::*;
use program_structure::ast::Statement;

#[derive(Clone)]
pub struct BranchBucket {
    pub stmt: Statement,
    pub line: usize,
    pub message_id: usize,
    pub cond: InstructionPointer,
    pub if_branch: InstructionList,
    pub else_branch: InstructionList,
}

impl IntoInstruction for BranchBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Branch(self)
    }
}
impl Allocate for BranchBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for BranchBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for BranchBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let cond = self.cond.to_string();
        let mut if_body = "".to_string();
        for i in &self.if_branch {
            if_body = format!("{}{};", if_body, i.to_string());
        }
        let mut else_body = "".to_string();
        for i in &self.else_branch {
            else_body = format!("{}{};", else_body, i.to_string());
        }
        format!(
            "IF(line:{},template_id:{},cond:{},if:{},else{})",
            line, template_id, cond, if_body, else_body
        )
    }
}

impl WriteLLVMIR for BranchBucket {
    fn produce_llvm_ir<'a, 'b>(&self, producer: &'b dyn LLVMIRProducer<'a>) -> Option<LLVMInstruction<'a>> {
        println!("{}\n", self.to_string());
        // Necessary basic blocks
        let current_function = producer.current_function();
        let then_bb = create_bb(producer, current_function, "if.then");
        let else_bb = create_bb(producer, current_function, "if.else");
        let merge_bb = create_bb(producer, current_function, "if.merge");
        // Check of the condition
        let cond_code = self.cond.produce_llvm_ir(producer)
            .expect("Cond instruction must produce a value!");
        create_conditional_branch(producer, cond_code.into_int_value(), then_bb, else_bb);
        // Then branch
        producer.set_current_bb(then_bb);
        let mut then_last_inst = None;
        for inst in &self.if_branch {
            then_last_inst = inst.produce_llvm_ir(producer);
            if let Some(inst) = then_last_inst {
                if !any_value_wraps_basic_value(inst) {
                    then_last_inst = None
                }
            }
        }
        create_br(producer, merge_bb);
        // Else branch
        producer.set_current_bb(else_bb);
        let mut else_last_inst = None;
        for inst in &self.else_branch {
            else_last_inst = inst.produce_llvm_ir(producer);
            if let Some(inst) = else_last_inst {
                if !any_value_wraps_basic_value(inst) {
                    else_last_inst = None
                }
            }
        }
        create_br(producer, merge_bb);
        // Merge results
        producer.set_current_bb(merge_bb);
        match (then_last_inst, else_last_inst) {
            (None, None) => {
                None
            }
            (Some(then), None) => {
                let phi = create_phi(producer,then.get_type(), "if.merge.phi");
                let then = any_value_to_basic(then);
                phi.add_incoming_as_enum(&[(then, then_bb)]);
                Some(to_enum(phi))
            }
            (None, Some(else_)) => {
                let phi = create_phi(producer, else_.get_type(), "if.merge.phi");
                let else_ = any_value_to_basic(else_);
                phi.add_incoming_as_enum(&[(else_, else_bb)]);
                Some(to_enum(phi))
            }
            (Some(then), Some(else_)) => {
                assert_eq!(then.get_type(), else_.get_type(), "Types of the two branches of if statement must be of the same type!");
                let phi = create_phi(producer,then.get_type(), "if.merge.phi");
                let then = any_value_to_basic(then);
                let else_ = any_value_to_basic(else_);
                phi.add_incoming_as_enum(&[(then, then_bb), (else_, else_bb)]);
                Some(to_enum(phi))
            }
        }
    }
}

impl WriteWasm for BranchBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; branch bucket".to_string());
	}
        if self.if_branch.len() > 0 {
            let mut instructions_cond = self.cond.produce_wasm(producer);
            instructions.append(&mut instructions_cond);
            instructions.push(call("$Fr_isTrue"));
            instructions.push(add_if());
            for ins in &self.if_branch {
                let mut instructions_if = ins.produce_wasm(producer);
                instructions.append(&mut instructions_if);
            }
            if self.else_branch.len() > 0 {
                instructions.push(add_else());
                for ins in &self.else_branch {
                    let mut instructions_else = ins.produce_wasm(producer);
                    instructions.append(&mut instructions_else);
                }
            }
	    instructions.push(add_end());
        } else {
            if self.else_branch.len() > 0 {
                let mut instructions_cond = self.cond.produce_wasm(producer);
                instructions.append(&mut instructions_cond);
                instructions.push(call("$Fr_isTrue"));
                instructions.push(eqz32());
                instructions.push(add_if());
                for ins in &self.else_branch {
                    let mut instructions_else = ins.produce_wasm(producer);
                    instructions.append(&mut instructions_else);
                }
		instructions.push(add_end());
            }
        }
        if producer.needs_comments() {
            instructions.push(";; end of branch bucket".to_string());
	}
        instructions
    }
}

impl WriteC for BranchBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::merge_code;
        let (condition_code, condition_result) = self.cond.produce_c(producer, parallel);
        let condition_result = format!("Fr_isTrue({})", condition_result);
        let mut if_body = Vec::new();
        for instr in &self.if_branch {
            let (mut instr_code, _) = instr.produce_c(producer, parallel);
            if_body.append(&mut instr_code);
        }
        let mut else_body = Vec::new();
        for instr in &self.else_branch {
            let (mut instr_code, _) = instr.produce_c(producer, parallel);
            else_body.append(&mut instr_code);
        }
        let mut conditional = format!("if({}){{\n{}}}", condition_result, merge_code(if_body));
        if !else_body.is_empty() {
            conditional.push_str(&format!("else{{\n{}}}", merge_code(else_body)));
        }
        let mut c_branch = condition_code;
        c_branch.push(conditional);
        (c_branch, "".to_string())
    }
}
