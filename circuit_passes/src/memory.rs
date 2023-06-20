use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};

pub struct PassMemory {
    pub templates_library: TemplatesLibrary,
    pub functions_library: FunctionsLibrary,
    pub interpreter: BucketInterpreter
}

impl PassMemory {
    pub fn add_template(&mut self, template: &TemplateCode) {
        self.templates_library.borrow_mut().insert(template.header.clone(), (*template).clone());
    }

    pub fn add_function(&mut self, function: &FunctionCode) {
        self.functions_library.borrow_mut().insert(function.header.clone(), (*function).clone());
    }
}
