use inkwell::types::{IntType, StructType, VoidType};
use crate::llvm_elements::{LLVM, LLVMAdapter, LLVMContext, LLVMIRProducer};

pub type BigIntType<'a> = IntType<'a>; // i256

pub fn bigint_type<'a>(producer: & dyn LLVMIRProducer<'a>) -> BigIntType<'a> {
    producer.context().custom_width_int_type(256)
}

    pub fn opaque_struct_type<'a>(producer: & dyn LLVMIRProducer<'a>, name: &str) -> StructType<'a> {
        producer.context().opaque_struct_type(name)
    }



pub fn subcomponent_type<'a>(producer: & dyn LLVMIRProducer<'a>) -> StructType<'a> {
        opaque_struct_type(producer, "subcomponent")
    }
pub fn bool_type<'a>(producer: & dyn LLVMIRProducer<'a>) -> IntType<'a> {
         producer.context().bool_type()
    }
pub fn void_type<'a>(producer: & dyn LLVMIRProducer<'a>) -> VoidType<'a> {
         producer.context().void_type()
    }
pub  fn i32_type<'a>(producer: & dyn LLVMIRProducer<'a>) -> IntType<'a> {
         producer.context().i32_type()
    }



