use std::cell::RefCell;
use code_producers::components::TemplateInstanceIOMap;
use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};
use crate::bucket_interpreter::env::Env;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::observer::InterpreterObserver;

pub struct PassMemory {
    pub templates_library: TemplatesLibrary,
    pub functions_library: FunctionsLibrary,
    pub prime: String,
    pub constant_fields: Vec<String>,
    pub current_scope: String,
    pub io_map: TemplateInstanceIOMap
}

impl PassMemory {
    pub fn new_cell(prime: &String, current_scope: String, io_map: TemplateInstanceIOMap) -> RefCell<Self> {
        RefCell::new(PassMemory {
            templates_library: Default::default(),
            functions_library: Default::default(),
            prime: prime.to_string(),
            constant_fields: vec![],
            current_scope,
            io_map
        })
    }

    pub fn run_template(&self, observer: &dyn InterpreterObserver, template: &TemplateCode) {
        eprintln!("Starting analysis of {}", template.header);
        let interpreter = BucketInterpreter::init(template.name.clone(), &self.prime, &self.constant_fields, observer, self.io_map.clone());
        let env = Env::new(self.templates_library.clone(), self.functions_library.clone());
        interpreter.execute_instructions(&template.body, &env, true);
    }

    pub fn add_template(&mut self, template: &TemplateCode) {
        self.templates_library.borrow_mut().insert(template.header.clone(), (*template).clone());
    }

    pub fn add_function(&mut self, function: &FunctionCode) {
        self.functions_library.borrow_mut().insert(function.header.clone(), (*function).clone());
    }

    pub fn fill_from_circuit(&mut self, circuit: &Circuit) {
        for template in &circuit.templates {
            self.add_template(template);
        }
        for function in &circuit.functions {
            self.add_function(function);
        }
        self.constant_fields = circuit.llvm_data.field_tracking.clone();
        self.io_map = circuit.llvm_data.io_map.clone();
    }
}
