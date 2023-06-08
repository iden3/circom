use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::value::Value;

#[derive(Clone)]
pub struct SubcmpEnv {
    pub signals: HashMap<usize, Value>,
    counter: usize
}

impl SubcmpEnv {
    pub fn new(inputs: usize) -> Self {
        SubcmpEnv {
            signals: Default::default(),
            counter: inputs
        }
    }

    pub fn decrease_counter(&mut self) {
        self.counter -= 1;
    }

    pub fn get_signal(&self, index: usize) -> Value {
        self.signals.get(&index).unwrap_or_default().clone()
    }

    pub fn set_signal(&mut self, index: usize, value: Value) {
        self.signals.insert(index, value);
    }

    pub fn counter_is_zero(&self) -> bool {
        self.counter == 0
    }

    pub fn copy_signals(&mut self, signals: &HashMap<usize, Value>) {
        self.signals = signals.clone();
    }
}

impl std::fmt::Display for SubcmpEnv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Subcmp {{")?;
        writeln!(f, "  counter: {}", self.counter)?;
        writeln!(f, "  signals: {{")?;
        for (idx, value) in self.signals.iter() {
            writeln!(f, "    {} => {}", idx, value)?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}")
    }
}

#[derive(Clone)]
pub struct Env {
    vars: HashMap<usize, Value>,
    signals: HashMap<usize, Value>,
    subcmps: HashMap<usize, SubcmpEnv>,
    templates_library: TemplatesLibrary,
    functions_library: FunctionsLibrary
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "vars {{")?;
        for (idx, value) in self.vars.iter() {
            writeln!(f, "  {} => {}", idx, value)?;
        }
        writeln!(f, "}}")?;

        writeln!(f, "signals {{")?;
        for (idx, value) in self.signals.iter() {
            writeln!(f, "  {} => {}", idx, value)?;
        }
        writeln!(f, "}}")?;

        writeln!(f, "subcomponents:")?;
        for (idx, cmp) in self.subcmps.iter() {
            writeln!(f, "{} => {}", idx, cmp)?;
        }
        write!(f, "")
    }
}

impl Env {
    pub fn new(templates_library: TemplatesLibrary, functions_library: FunctionsLibrary) -> Self {
        Env {
            vars: Default::default(),
            signals: Default::default(),
            subcmps: Default::default(),
            templates_library,
            functions_library
        }
    }

    pub fn reset(&mut self) {
        self.vars = Default::default();
        self.signals = Default::default();
        self.subcmps = Default::default();
    }

    pub fn copy_signals(&mut self, signals: &HashMap<usize, Value>) {
        self.signals = signals.clone();
    }

    pub fn get_var(&self, idx: usize) -> Value {
        self.vars.get(&idx).unwrap_or_default().clone()
    }

    pub fn set_var(&mut self, idx: usize, value: Value) {
        self.vars.insert(idx, value);
    }

    pub fn get_signal(&self, idx: usize) -> Value {
        self.signals.get(&idx).unwrap_or_default().clone()
    }

    pub fn set_signal(&mut self, idx: usize, value: Value) {
        self.signals.insert(idx, value);
    }

    pub fn get_subcmp_signal(&self, subcmp_idx: usize, signal_idx: usize) -> Value {
        self.subcmps[&subcmp_idx].get_signal(signal_idx)
    }

    pub fn set_subcmp_signal(&mut self, subcmp_idx: usize, signal_idx: usize, value: Value)  {
        self.subcmps.get_mut(&subcmp_idx).unwrap().set_signal(signal_idx, value)
    }

    pub fn decrease_subcmp_counter(&mut self, subcmp_idx: usize) {
        self.subcmps.get_mut(&subcmp_idx).unwrap().decrease_counter()
    }

    pub fn subcmp_counter_is_zero(&self, subcmp_idx: usize) -> bool {
        self.subcmps.get(&subcmp_idx).unwrap().counter_is_zero()
    }

    pub fn run_subcmp(&mut self, subcmp_idx: usize, name: &String, interpreter: &BucketInterpreter) {
        let code =  &self.templates_library.borrow()[name].body;
        let mut subcmp_env = Env::new(self.templates_library.clone(), self.functions_library.clone());
        subcmp_env.copy_signals(&self.subcmps[&subcmp_idx].signals);
        let interpreter = BucketInterpreter::init(subcmp_env, &interpreter.prime, interpreter.constant_fields.clone());
        interpreter.execute_instructions(code);
        self.subcmps.get_mut(&subcmp_idx).unwrap().copy_signals(&interpreter.get_env().signals);
    }

    pub fn create_subcmp(&mut self, name: &String, base_index: usize, count: usize) {
        for i in base_index..(base_index+count) {
            self.subcmps.insert(i, SubcmpEnv::new(self.templates_library.borrow()[name].number_of_inputs));
        }
    }
}

pub type TemplatesLibrary = Rc<RefCell<HashMap<String, TemplateCode>>>;
pub type FunctionsLibrary = Rc<RefCell<HashMap<String, FunctionCode>>>;
