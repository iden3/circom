use code_producers::llvm_elements::{LLVMInstruction, LLVMIRProducer};
use crate::intermediate_representation::{Instruction, InstructionList, InstructionPointer};
use crate::intermediate_representation::ir_interface::{Allocate, IntoInstruction, LoopBucket, ObtainMeta};
use crate::translating_traits::WriteLLVMIR;

#[derive(Clone, Debug)]
pub struct UnrolledLoopBucket {
    pub original_loop: InstructionPointer,
    pub line: usize,
    pub message_id: usize,
    pub body: Vec<InstructionList>
}

impl IntoInstruction for UnrolledLoopBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::UnrolledLoop(self)
    }
}

impl Allocate for UnrolledLoopBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for UnrolledLoopBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for UnrolledLoopBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let mut body = "".to_string();
        body = format!("{}[", body);
        for iter in &self.body {
            for i in iter {
                body = format!("{}{};", body, i.to_string());
            }
            body = format!("{}, ", body);
        }
        body = format!("{}]", body);
        format!("UNROLLED_LOOP(line:{},template_id:{},n_iterations:{},body:{})", line, template_id, self.body.len(), body)
    }
}

impl WriteLLVMIR for UnrolledLoopBucket {
    fn produce_llvm_ir<'a, 'b>(&self, producer: &'b dyn LLVMIRProducer<'a>) -> Option<LLVMInstruction<'a>> {
        let mut last = None;
        for iteration in &self.body {
            for inst in iteration {
                last = inst.produce_llvm_ir(producer);
            }
        }
        last
    }
}