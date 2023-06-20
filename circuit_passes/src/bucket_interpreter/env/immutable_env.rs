use std::collections::HashMap;

use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};
use crate::bucket_interpreter::immutable_interpreter::BucketInterpreter;
use crate::bucket_interpreter::value::Value;

#[derive(Clone, Default, Debug)]
pub struct FrozenSubcmpEnv {
    pub signals: HashMap<usize, Value>,
    counter: usize
}

impl FrozenSubcmpEnv {
    pub fn new(inputs: usize) -> Self {
        FrozenSubcmpEnv {
            signals: Default::default(),
            counter: inputs
        }
    }

    pub fn get_signal(&self, index: usize) -> Value {
        self.signals.get(&index).unwrap_or_default().clone()
    }

    pub fn set_signal(&self, idx: usize, value: Value) -> FrozenSubcmpEnv {
        FrozenSubcmpEnv {
            signals: self.signals.clone().into_iter().chain([(idx, value)]).collect(),
            counter: self.counter
        }
    }

    pub fn set_signals(&self, signals: HashMap<usize, Value>) -> FrozenSubcmpEnv {
        FrozenSubcmpEnv {
            signals,
            counter: self.counter
        }
    }

    pub fn counter_is_zero(&self) -> bool {
        self.counter == 0
    }

    pub fn decrease_counter(&self) -> FrozenSubcmpEnv {
        FrozenSubcmpEnv {
            signals: self.signals.clone(),
            counter: self.counter - 1
        }
    }
}

// An immutable env that returns a new copy when modified
#[derive(Clone, Default)]
pub struct FrozenEnv {
    vars: HashMap<usize, Value>,
    signals: HashMap<usize, Value>,
    subcmps: HashMap<usize, FrozenSubcmpEnv>,
    templates_library: TemplatesLibrary,
    functions_library: FunctionsLibrary
}

impl FrozenEnv {
    pub fn new(templates_library: TemplatesLibrary, functions_library: FunctionsLibrary) -> Self {
        FrozenEnv {
            vars: Default::default(),
            signals: Default::default(),
            subcmps: Default::default(),
            templates_library,
            functions_library
        }
    }

    // READ OPERATIONS
    pub fn get_var(&self, idx: usize) -> Value {
        self.vars.get(&idx).unwrap_or_default().clone()
    }

    pub fn get_signal(&self, idx: usize) -> Value {
        self.signals.get(&idx).unwrap_or_default().clone()
    }

    pub fn get_subcmp_signal(&self, subcmp_idx: usize, signal_idx: usize) -> Value {
        self.subcmps[&subcmp_idx].get_signal(signal_idx)
    }

    pub fn subcmp_counter_is_zero(&self, subcmp_idx: usize) -> bool {
        self.subcmps.get(&subcmp_idx).unwrap().counter_is_zero()
    }

    // WRITE OPERATIONS
    pub fn set_var(&self, idx: usize, value: Value) -> FrozenEnv {
        FrozenEnv {
            vars: self.vars.clone().into_iter().chain([(idx, value)]).collect(),
            signals: self.signals.clone(),
            subcmps: self.subcmps.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
        }
    }

    pub fn set_signals(&self, signals: HashMap<usize, Value>) -> FrozenEnv {
        FrozenEnv {
            vars: self.vars.clone(),
            signals,
            subcmps: self.subcmps.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
        }
    }

    pub fn set_signal(&self, idx: usize, value: Value) -> FrozenEnv {
        FrozenEnv {
            vars: self.vars.clone(),
            signals: self.signals.clone().into_iter().chain([(idx, value)]).collect(),
            subcmps: self.subcmps.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
        }
    }

    pub fn set_subcmp_signal(&self, subcmp_idx: usize, signal_idx: usize, value: Value) -> FrozenEnv {
        let subcmp = &self.subcmps[&subcmp_idx];
        FrozenEnv {
            vars: self.vars.clone(),
            signals: self.vars.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
            subcmps: self.subcmps.clone().into_iter().chain([
                (subcmp_idx, subcmp.set_signal(signal_idx, value))
            ]).collect()
        }
    }

    pub fn decrease_subcmp_counter(&self, subcmp_idx: usize) -> FrozenEnv {
        let subcmp = &self.subcmps[&subcmp_idx];
        FrozenEnv {
            vars: self.vars.clone(),
            signals: self.vars.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
            subcmps: self.subcmps.clone().into_iter().chain([
                (subcmp_idx, subcmp.decrease_counter())
            ]).collect()
        }
    }

    pub fn set_subcmp_signals(&self, subcmp_idx: usize, signals: HashMap<usize, Value>) -> FrozenEnv {
        println!("{subcmp_idx} {:?}", self.subcmps);
        let subcmp = &self.subcmps[&subcmp_idx];
        FrozenEnv {
            vars: self.vars.clone(),
            signals: self.vars.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
            subcmps: self.subcmps.clone().into_iter().chain([
                (subcmp_idx, subcmp.set_signals(signals))
            ]).collect()
        }
    }

    pub fn run_subcmp(&self, subcmp_idx: usize, name: &String, interpreter: &BucketInterpreter, observe: bool) -> FrozenEnv {
        let code =  &self.templates_library.borrow()[name].body;
        let subcmp_env = FrozenEnv::new(self.templates_library.clone(), self.functions_library.clone())
            .set_signals(self.subcmps[&subcmp_idx].signals.clone());

        let interpreter = BucketInterpreter::init(interpreter.prime(), interpreter.constant_fields, interpreter.observer());
        let (_, env) = interpreter.execute_instructions(code, &subcmp_env, observe);
        self.set_subcmp_signals(subcmp_idx, env.signals.clone())
    }

    pub fn create_subcmp(&self, name: &String, base_index: usize, count: usize) -> FrozenEnv {
        let mut subcmps = self.subcmps.clone();
        for i in base_index..(base_index+count) {
            subcmps.insert(i, FrozenSubcmpEnv::new(self.templates_library.borrow()[name].number_of_inputs));
        }
        println!("{:?}", subcmps);

        FrozenEnv {
            vars: self.vars.clone(),
            signals: self.vars.clone(),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone(),
            subcmps
        }
    }
}