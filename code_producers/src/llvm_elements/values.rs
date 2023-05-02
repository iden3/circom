use inkwell::values::{AggregateValue, AnyValue, AnyValueEnum, IntValue};
use crate::llvm_elements::{LLVMContext, LLVMIRProducer};

pub trait ValueProducer<'a>: LLVMContext<'a> {
     fn create_literal_u32(&self, val: u64) -> IntValue<'a> {
        self.context().i32_type().const_int(val, false)
    }

     fn zero(&self) -> IntValue<'a> {
        self.context().i32_type().const_zero()
    }

     fn get_const(&self, value: usize) -> AnyValueEnum<'a> {
        let arr = self.constant_fields().expect("Access to constant before initialization!").get_initializer().unwrap().into_array_value();
        let mut idx = vec![value as u32];
        let gep = arr.const_extract_value(&mut idx);
        gep.as_any_value_enum()
    }
}