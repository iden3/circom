use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::llvm_elements::{
    any_value_wraps_basic_value, LLVMInstruction, LLVMIRProducer, AnyValue,
};
use code_producers::llvm_elements::{AnyValueEnum, InstructionOpcode}; //from inkwell via "pub use" in mod.rs
use code_producers::llvm_elements::functions::create_bb;
use code_producers::llvm_elements::instructions::{create_br, create_conditional_branch};
use code_producers::wasm_elements::*;
use crate::intermediate_representation::BucketId;


#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BranchBucket {
    pub id: BucketId,
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
    fn produce_llvm_ir<'a, 'b>(
        &self,
        producer: &'b dyn LLVMIRProducer<'a>,
    ) -> Option<LLVMInstruction<'a>> {
        println!("{}\n", self.to_string());
        // Necessary basic blocks
        let current_function = producer.current_function();
        let then_bb = create_bb(producer, current_function, "if.then");
        let else_bb = create_bb(producer, current_function, "if.else");
        let merge_bb = create_bb(producer, current_function, "if.merge");

        // Generate check of the condition and the conditional jump in the current block
        let cond_code =
            self.cond.produce_llvm_ir(producer).expect("Cond instruction must produce a value!");
        create_conditional_branch(producer, cond_code.into_int_value(), then_bb, else_bb);

        // Define helper to process the body of the given branch of the if-statement.
        // If needed, it will produce an unconditional jump to the "merge" basic block.
        // Returns true iff the unconditional jump was produced.
        let process_body = |branch_body: &InstructionList| {
            let mut last_inst = None;
            for inst in branch_body {
                last_inst = inst.produce_llvm_ir(producer);
            }
            if let Some(inst) = last_inst {
                //The final instruction will never be a BasicValueEnum. Even the
                //  ternary conditional operator will be desugared to a normal
                //  if-statement that stores to a temporary variable.
                assert!(!any_value_wraps_basic_value(inst));
                //Special case: Should not branch after a branch, return, or unreachable
                if let AnyValueEnum::InstructionValue(v) = inst {
                    match v.get_opcode() {
                        InstructionOpcode::Unreachable
                        | InstructionOpcode::Return
                        | InstructionOpcode::Br => {
                            return false;
                        }
                        _ => {}
                    }
                }
            }
            //Any other case
            create_br(producer, merge_bb);
            return true;
        };

        // Then branch
        producer.set_current_bb(then_bb);
        let jump_from_if = process_body(&self.if_branch);
        // Else branch
        producer.set_current_bb(else_bb);
        let jump_from_else = process_body(&self.else_branch);
        // Merge block (where the function body continues)
        producer.set_current_bb(merge_bb);


        //If there are no jumps to the merge block, it is unreachable.
        if !jump_from_if && !jump_from_else {
            let u = producer.builder().build_unreachable();
            Some(u.as_any_value_enum())
        } else {
            None //merge block is empty
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
