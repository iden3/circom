use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};
use crate::bucket_interpreter::mutable_interpreter::MutableBucketInterpreter;

pub struct PassMemory {
    pub templates_library: TemplatesLibrary,
    pub functions_library: FunctionsLibrary,
    pub interpreter: MutableBucketInterpreter
}

impl PassMemory {
    pub fn add_template(&mut self, template: &TemplateCode) {
        self.templates_library.borrow_mut().insert(template.header.clone(), (*template).clone());
    }

    pub fn add_function(&mut self, function: &FunctionCode) {
        self.functions_library.borrow_mut().insert(function.header.clone(), (*function).clone());
    }
}
