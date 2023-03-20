pub use super::address_type::{AddressType, InputInformation, StatusInput};
pub use super::assert_bucket::AssertBucket;
pub use super::branch_bucket::BranchBucket;
pub use super::call_bucket::{CallBucket, FinalData, ReturnType};
pub use super::compute_bucket::{ComputeBucket, OperatorType};
pub use super::create_component_bucket::CreateCmpBucket;
pub use super::load_bucket::LoadBucket;
pub use super::location_rule::LocationRule;
pub use super::log_bucket::LogBucket;
pub use super::loop_bucket::LoopBucket;
pub use super::return_bucket::ReturnBucket;
pub use super::store_bucket::StoreBucket;
pub use super::log_bucket::LogBucketArg;
pub use super::types::{InstrContext, ValueType};
pub use super::value_bucket::ValueBucket;

use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

pub trait IntoInstruction {
    fn into_instruction(self) -> Instruction;
}
pub trait Allocate {
    fn allocate(self) -> InstructionPointer;
}

pub trait ObtainMeta {
    fn get_line(&self) -> usize;
    fn get_message_id(&self) -> usize;
}

pub trait CheckCompute {
    fn has_compute_in(&self) -> bool;
}

pub type InstructionList = Vec<InstructionPointer>;
pub type InstructionPointer = Box<Instruction>;

#[derive(Clone)]
pub enum Instruction {
    Value(ValueBucket),
    Load(LoadBucket),
    Store(StoreBucket),
    Compute(ComputeBucket),
    Call(CallBucket),
    Branch(BranchBucket),
    Return(ReturnBucket),
    Assert(AssertBucket),
    Log(LogBucket),
    Loop(LoopBucket),
    CreateCmp(CreateCmpBucket),
}

impl Allocate for Instruction {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self)
    }
}

impl ObtainMeta for Instruction {
    fn get_line(&self) -> usize {
        use Instruction::*;
        match self {
            Value(v) => v.get_line(),
            Load(v) => v.get_line(),
            Store(v) => v.get_line(),
            Compute(v) => v.get_line(),
            Call(v) => v.get_line(),
            Branch(v) => v.get_line(),
            Return(v) => v.get_line(),
            Loop(v) => v.get_line(),
            Assert(v) => v.get_line(),
            CreateCmp(v) => v.get_line(),
            Log(v) => v.get_line(),
        }
    }

    fn get_message_id(&self) -> usize {
        use Instruction::*;
        match self {
            Value(v) => v.get_message_id(),
            Load(v) => v.get_message_id(),
            Store(v) => v.get_message_id(),
            Compute(v) => v.get_message_id(),
            Call(v) => v.get_message_id(),
            Branch(v) => v.get_message_id(),
            Return(v) => v.get_message_id(),
            Loop(v) => v.get_message_id(),
            Assert(v) => v.get_message_id(),
            CreateCmp(v) => v.get_message_id(),
            Log(v) => v.get_message_id(),
        }
    }
}

impl CheckCompute for Instruction {
    fn has_compute_in(&self) -> bool {
        use Instruction::*;
        match self {
	    Load(_v) => {true
	    },
	    Compute(_) => true,
	    _ => false,
        }
    }
}

impl WriteWasm for Instruction {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use Instruction::*;
        match self {
            Value(v) => v.produce_wasm(producer),
            Load(v) => v.produce_wasm(producer),
            Store(v) => v.produce_wasm(producer),
            Compute(v) => v.produce_wasm(producer),
            Call(v) => v.produce_wasm(producer),
            Branch(v) => v.produce_wasm(producer),
            Return(v) => v.produce_wasm(producer),
            Loop(v) => v.produce_wasm(producer),
            Assert(v) => v.produce_wasm(producer),
            CreateCmp(v) => v.produce_wasm(producer),
            Log(v) => v.produce_wasm(producer),
        }
    }
}

impl WriteC for Instruction {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use Instruction::*;
        assert!(parallel.is_some());
        match self {
            Value(v) => v.produce_c(producer, parallel),
            Load(v) => v.produce_c(producer, parallel),
            Store(v) => v.produce_c(producer, parallel),
            Compute(v) => v.produce_c(producer, parallel),
            Call(v) => v.produce_c(producer, parallel),
            Branch(v) => v.produce_c(producer, parallel),
            Return(v) => v.produce_c(producer, parallel),
            Loop(v) => v.produce_c(producer, parallel),
            Assert(v) => v.produce_c(producer, parallel),
            CreateCmp(v) => v.produce_c(producer, parallel),
            Log(v) => v.produce_c(producer, parallel),
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        use Instruction::*;
        match self {
            Value(v) => v.to_string(),
            Load(v) => v.to_string(),
            Store(v) => v.to_string(),
            Compute(v) => v.to_string(),
            Call(v) => v.to_string(),
            Branch(v) => v.to_string(),
            Return(v) => v.to_string(),
            Loop(v) => v.to_string(),
            Assert(v) => v.to_string(),
            CreateCmp(v) => v.to_string(),
            Log(v) => v.to_string(),
        }
    }
}
