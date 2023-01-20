pub mod llvm_code_generator;
use inkwell::values::AnyValue;
use inkwell::context::Context;
use inkwell::module::Module;
use crate::components::*;


type LLVMInstruction<'a> = dyn AnyValue<'a>; // TODO Change this to an llvm::Value

pub struct LLVMProducer {
    pub context: Box<Context>,
}

impl Default for LLVMProducer {
    fn default() -> Self {
        let context = Box::new(Context::create());
        LLVMProducer {
            context
        }
    }
}

impl LLVMProducer {
    pub fn create_module(&self, file_name: &str) -> Module {
        self.context.create_module(file_name)
    }
}