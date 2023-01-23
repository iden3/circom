use std::cell::RefCell;
use std::rc::Rc;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use crate::llvm_elements::{LLVMInstruction, LLVMProducer, ModuleWrapperStruct, ModuleWrapper};

pub fn create_module<'a>(producer: &'a LLVMProducer, name: &str) -> ModuleWrapper<'a> {
    Rc::new(RefCell::new(ModuleWrapperStruct::from_context(&producer.context, name)))
}