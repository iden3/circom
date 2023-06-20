use code_producers::llvm_elements::{LLVMInstruction, LLVMIRProducer};
use crate::intermediate_representation::{Instruction, InstructionList, InstructionPointer};
use crate::intermediate_representation::ir_interface::{Allocate, IntoInstruction, ObtainMeta};
use crate::translating_traits::WriteLLVMIR;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BlockBucket {
    pub line: usize,
    pub message_id: usize,
    pub body: InstructionList
}

impl IntoInstruction for BlockBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::UnrolledLoop(self)
    }
}

impl Allocate for BlockBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for BlockBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for BlockBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let mut body = "".to_string();
        body = format!("{}[", body);
        for i in &self.body {
            body = format!("{}{};", body, i.to_string());
        }
        body = format!("{}]", body);
        format!("UNROLLED_LOOP(line:{},template_id:{},n_iterations:{},body:{})", line, template_id, self.body.len(), body)
    }
}

impl WriteLLVMIR for BlockBucket {
    fn produce_llvm_ir<'a, 'b>(&self, producer: &'b dyn LLVMIRProducer<'a>) -> Option<LLVMInstruction<'a>> {
        let mut last = None;
        for inst in &self.body {
            last = inst.produce_llvm_ir(producer);
        }
        last
    }
}