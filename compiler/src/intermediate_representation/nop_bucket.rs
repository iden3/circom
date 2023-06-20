use code_producers::llvm_elements::{LLVMInstruction, LLVMIRProducer};
use crate::intermediate_representation::{Instruction, InstructionPointer};
use crate::intermediate_representation::ir_interface::{Allocate, IntoInstruction, ObtainMeta};
use crate::translating_traits::WriteLLVMIR;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NopBucket;

impl IntoInstruction for NopBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Nop(self)
    }
}

impl Allocate for NopBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for NopBucket {
    fn get_line(&self) -> usize {
        0
    }
    fn get_message_id(&self) -> usize {
        0
    }
}

impl ToString for NopBucket {
    fn to_string(&self) -> String {
        "NOP".to_string()
    }
}

impl WriteLLVMIR for NopBucket {
    fn produce_llvm_ir<'a, 'b>(&self, _producer: &'b dyn LLVMIRProducer<'a>) -> Option<LLVMInstruction<'a>> {
        None
    }
}