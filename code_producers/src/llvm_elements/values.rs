use inkwell::types::StringRadix;
use inkwell::values::{AnyValue, AnyValueEnum, IntValue};

use crate::llvm_elements::{LLVMIRProducer};
use crate::llvm_elements::types::bigint_type;

pub fn create_literal_u32<'a>(producer: &dyn LLVMIRProducer<'a>, val: u64) -> IntValue<'a> {
    producer.context().i32_type().const_int(val, false)
}

pub fn zero<'a>(producer: &dyn LLVMIRProducer<'a>) -> IntValue<'a> {
    producer.context().i32_type().const_zero()
}

pub fn get_const<'a>(producer: &dyn LLVMIRProducer<'a>, value: usize) -> AnyValueEnum<'a> {
    let f = &producer.constant_fields()[value];
    bigint_type(producer)
        .const_int_from_string(f, StringRadix::Decimal)
        .unwrap()
        .as_any_value_enum()
}
