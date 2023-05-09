use inkwell::basic_block::BasicBlock;
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

use crate::llvm_elements::LLVMIRProducer;

pub fn create_function<'a>(producer: &dyn LLVMIRProducer<'a>, name: &str, ty: FunctionType<'a>) -> FunctionValue<'a> {
    producer.llvm().module.add_function(name, ty, None)
}

pub fn create_bb<'a>(producer: &dyn LLVMIRProducer<'a>, func: FunctionValue<'a>, name: &str) -> BasicBlock<'a> {
    producer.context().append_basic_block(func, name)
}
