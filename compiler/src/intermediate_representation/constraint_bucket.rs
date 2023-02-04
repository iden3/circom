use code_producers::c_elements::CProducer;
use code_producers::llvm_elements::{LLVMAdapter, LLVMInstruction, LLVMProducer};
use code_producers::wasm_elements::WASMProducer;
use crate::intermediate_representation::{Instruction, InstructionPointer};
use crate::intermediate_representation::ir_interface::{Allocate, IntoInstruction, ObtainMeta};
use crate::translating_traits::{WriteC, WriteLLVMIR, WriteWasm};

#[derive(Clone)]
pub struct ConstraintBucket {
    pub wrapped: InstructionPointer
}

impl IntoInstruction for ConstraintBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Constraint(self)
    }
}

impl Allocate for ConstraintBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for ConstraintBucket {
    fn get_line(&self) -> usize {
        self.wrapped.get_line()
    }
    fn get_message_id(&self) -> usize {
        self.wrapped.get_message_id()
    }
}

impl ToString for ConstraintBucket {
    fn to_string(&self) -> String {
        format!(
            "CONSTRAINT:{}",
            self.wrapped.to_string()
        )
    }
}

impl WriteLLVMIR for ConstraintBucket {
    fn produce_llvm_ir<'a>(&self, producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) -> Option<LLVMInstruction<'a>> {
        // TODO: Create the constraint call
        self.wrapped.produce_llvm_ir(producer, llvm)
    }
}

impl WriteWasm for ConstraintBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        self.wrapped.produce_wasm(producer)
    }
}

impl WriteC for ConstraintBucket {
    fn produce_c(&self, producer: &CProducer, is_parallel: Option<bool>) -> (Vec<String>, String) {
        self.wrapped.produce_c(producer, is_parallel)
    }
}