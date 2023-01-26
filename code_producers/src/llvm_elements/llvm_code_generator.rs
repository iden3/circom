use std::cell::RefCell;
use std::rc::Rc;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use crate::llvm_elements::{LLVMInstruction, LLVMProducer, ModuleWrapperStruct, ModuleWrapper};

pub fn create_module<'a>(producer: &'a LLVMProducer, name: &str) -> ModuleWrapper<'a> {
    Rc::new(RefCell::new(ModuleWrapperStruct::from_context(&producer.context, name)))
}

pub fn load_fr<'a>(producer: &'a LLVMProducer, module: ModuleWrapper<'a>) {
    let fr_ir = include_bytes!(concat!(env!("OUT_DIR"), "/fr.bc"));
    let fr_mem = MemoryBuffer::create_from_memory_range(fr_ir, "fr");
    let fr_mod = producer.context.create_module_from_ir(fr_mem).expect("Cannot load fr into memory!");
    module.borrow().merge_module(fr_mod);
}