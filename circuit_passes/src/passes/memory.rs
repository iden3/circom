use std::cell::RefCell;
use std::collections::HashMap;
use code_producers::components::TemplateInstanceIOMap;
use code_producers::llvm_elements::IndexMapping;
use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};
use crate::bucket_interpreter::env::Env;
use crate::bucket_interpreter::observer::InterpreterObserver;

pub struct PassMemory {
    pub templates_library: TemplatesLibrary,
    pub functions_library: FunctionsLibrary,
    pub prime: String,
    pub constant_fields: Vec<String>,
    pub current_scope: String,
    pub io_map: TemplateInstanceIOMap,
    pub signal_index_mapping: HashMap<String, IndexMapping>,
    pub variables_index_mapping: HashMap<String, IndexMapping>
}

impl PassMemory {
    pub fn new_cell(prime: &String, current_scope: String, io_map: TemplateInstanceIOMap, signal_index_mapping: IndexMapping, variables_index_mapping: IndexMapping) -> RefCell<Self> {
        RefCell::new(PassMemory {
            templates_library: Default::default(),
            functions_library: Default::default(),
            prime: prime.to_string(),
            constant_fields: vec![],
            current_scope,
            io_map,
            signal_index_mapping: Default::default(),
            variables_index_mapping: Default::default()
        })
    }

    pub fn run_template(&mut self, observer: &dyn InterpreterObserver, template: &TemplateCode) {
        self.current_scope = template.name.clone();
        let interpreter = self.build_interpreter(observer);
        let env = Env::new(&self.templates_library, &self.functions_library);
        interpreter.execute_instructions(&template.body, env, true);
    }

    pub fn build_interpreter<'a>(&'a self, observer: &'a dyn InterpreterObserver) -> BucketInterpreter {
        BucketInterpreter::init(&self.current_scope, &self.prime, &self.constant_fields, observer, &self.io_map, &self.signal_index_mapping[&self.current_scope], &self.variables_index_mapping[&self.current_scope])
    }

    pub fn add_template(&mut self, template: &TemplateCode) {
        self.templates_library.insert(template.header.clone(), (*template).clone());
    }

    pub fn add_function(&mut self, function: &FunctionCode) {
        self.functions_library.insert(function.header.clone(), (*function).clone());
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
