use code_producers::c_elements::CProducer;
use code_producers::llvm_elements::{LLVMAdapter, LLVMInstruction, new_constraint, to_basic_metadata_enum, LLVMIRProducer};
use code_producers::llvm_elements::instructions::{create_call, create_load, get_instruction_arg};
use code_producers::llvm_elements::llvm_code_generator::{CONSTRAINT_VALUE_FN_NAME, CONSTRAINT_VALUES_FN_NAME};
use code_producers::wasm_elements::WASMProducer;
use crate::intermediate_representation::{Instruction, InstructionPointer};
use crate::intermediate_representation::ir_interface::{Allocate, IntoInstruction, ObtainMeta};
use crate::translating_traits::{WriteC, WriteLLVMIR, WriteWasm};

#[derive(Clone)]
pub enum ConstraintBucket {
    Substitution(InstructionPointer),
    Equality(InstructionPointer)
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
        match self {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }.get_line()
    }
    fn get_message_id(&self) -> usize {
        match self {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }.get_message_id()
    }
}

impl ToString for ConstraintBucket {
    fn to_string(&self) -> String {
        format!(
            "CONSTRAINT:{}",
            match self {
                ConstraintBucket::Substitution(i) => i,
                ConstraintBucket::Equality(i) => i
            }.to_string()
        )
    }
}

impl WriteLLVMIR for ConstraintBucket {
    fn produce_llvm_ir<'a, 'b>(&self, producer: &'b dyn LLVMIRProducer<'a>) -> Option<LLVMInstruction<'a>> {
        // TODO: Create the constraint call
        let prev = match self {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }.produce_llvm_ir(producer).expect("A constrained instruction MUST produce a value!");

        const STORE_SRC_IDX: u32 = 1;
        const STORE_DST_IDX: u32 = 0;
        const ASSERT_IDX: u32 = 0;

        match self {
            ConstraintBucket::Substitution(_) => {
                let lhs = get_instruction_arg(prev.into_instruction_value(), STORE_DST_IDX);
                let rhs_ptr = get_instruction_arg(prev.into_instruction_value(), STORE_SRC_IDX);
                let rhs = create_load(producer,rhs_ptr.into_pointer_value());
                let constr = new_constraint(producer);
                let call = create_call(producer,CONSTRAINT_VALUES_FN_NAME, &[
                    to_basic_metadata_enum(lhs),
                    to_basic_metadata_enum(rhs),
                    to_basic_metadata_enum(constr)]);
                Some(call)
            }
            ConstraintBucket::Equality(_) => {
                let bool = get_instruction_arg(prev.into_instruction_value(), ASSERT_IDX);
                let constr = new_constraint(producer);
                let call = create_call(producer, CONSTRAINT_VALUE_FN_NAME, &[
                    to_basic_metadata_enum(bool),
                    to_basic_metadata_enum(constr)]);
                Some(call)
            }
        }
    }
}

impl WriteWasm for ConstraintBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        match self {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }.produce_wasm(producer)
    }
}

impl WriteC for ConstraintBucket {
    fn produce_c(&self, producer: &CProducer, is_parallel: Option<bool>) -> (Vec<String>, String) {
        match self {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }.produce_c(producer, is_parallel)
    }
}