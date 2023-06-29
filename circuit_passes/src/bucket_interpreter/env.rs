use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;

pub type TemplatesLibrary = Rc<RefCell<HashMap<String, TemplateCode>>>;
pub type FunctionsLibrary = Rc<RefCell<HashMap<String, FunctionCode>>>;

use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::value::{JoinSemiLattice, Value};

impl<L: JoinSemiLattice + Clone> JoinSemiLattice for HashMap<usize, L> {
    fn join(&self, other: &Self) -> Self {
        let mut new: HashMap<usize, L> = Default::default();
        for (k, v) in self {
            new.insert(*k, v.clone());
        }

        for (k, v) in other {
            if new.contains_key(&k) {
                new.get_mut(&k).unwrap().join(v);
            } else {
                new.insert(*k, v.clone());
            }
        }
        new
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubcmpEnv<'a> {
    pub signals: HashMap<usize, Value>,
    counter: usize,
    name: &'a String,
    template_id: usize
}

impl JoinSemiLattice for SubcmpEnv<'_> {
    fn join(&self, other: &Self) -> Self {
        assert_eq!(self.name, other.name);
        assert_eq!(self.template_id, other.template_id);
        SubcmpEnv {
            signals: self.signals.join(&other.signals),
            counter: std::cmp::min(self.counter, other.counter),
            name: self.name,
            template_id: self.template_id
        }
    }
}

impl<'a> SubcmpEnv<'a> {
    pub fn new(inputs: usize, name: &'a String, template_id: usize) -> Self {
        SubcmpEnv { signals: Default::default(), counter: inputs, name, template_id }
    }

    pub fn get_signal(&self, index: usize) -> Value {
        self.signals.get(&index).unwrap_or_default().clone()
    }

    pub fn set_signal(self, idx: usize, value: Value) -> SubcmpEnv<'a> {
        let mut copy = self;
        copy.signals.insert(idx, value);
        copy
        // SubcmpEnv {
        //     signals: self.signals.clone().into_iter().chain([(idx, value)]).collect(),
        //     counter: self.counter,
        //     name: self.name.clone(),
        //     template_id: self.template_id
        // }
    }

    pub fn set_signals(self, signals: HashMap<usize, Value>) -> SubcmpEnv<'a> {
        let mut copy = self;
        copy.signals = signals;
        copy
        // SubcmpEnv { signals, counter: self.counter, name: self.name.clone(),
        //     template_id: self.template_id }
    }

    pub fn counter_is_zero(&self) -> bool {
        self.counter == 0
    }

    pub fn decrease_counter(self) -> SubcmpEnv<'a> {
        let mut copy = self;
        copy.counter -= 1;
        copy
        // SubcmpEnv { signals: self.signals.clone(), counter: self.counter - 1, name: self.name.clone(),
        //     template_id: self.template_id }
    }

    pub fn counter_equal_to(&self, value: usize) -> bool {
        self.counter == value
    }
}

// /// Very inefficient
// #[derive(Default)]
// pub struct EnvSet<'a> {
//     envs: Vec<Env<'a>>
// }
//
// impl EnvSet<'_> {
//     pub fn new() -> Self {
//         EnvSet {
//             envs: Default::default()
//         }
//     }
//
//     pub fn contains(&self, env: &Env) -> bool {
//         for e in &self.envs {
//             if e == env {
//                 return true;
//             }
//         }
//         false
//     }
//
//     pub fn add(&self, env: &Env) -> EnvSet {
//         let mut new = vec![env.clone()];
//         for e in &self.envs {
//             new.push(e.clone())
//         }
//         EnvSet { envs: new }
//     }
// }

// An immutable env that returns a new copy when modified
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Env<'a> {
    vars: HashMap<usize, Value>,
    signals: HashMap<usize, Value>,
    subcmps: HashMap<usize, SubcmpEnv<'a>>,
    templates_library: TemplatesLibrary,
    functions_library: FunctionsLibrary,
}

impl Display for Env<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n  vars = {:?}\n  signals = {:?}\n  subcmps = {:?}", self.vars, self.signals, self.subcmps)
    }
}

impl<'a> Env<'a> {
    pub fn new(templates_library: TemplatesLibrary, functions_library: FunctionsLibrary) -> Self {
        Env {
            vars: Default::default(),
            signals: Default::default(),
            subcmps: Default::default(),
            templates_library,
            functions_library,
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

    pub fn get_subcmp_name(&self, subcmp_idx: usize) -> String {
        self.subcmps[&subcmp_idx].name.clone()
    }

    pub fn get_subcmp_template_id(&self, subcmp_idx: usize) -> usize {
        self.subcmps[&subcmp_idx].template_id
    }

    pub fn subcmp_counter_is_zero(&self, subcmp_idx: usize) -> bool {
        self.subcmps.get(&subcmp_idx).unwrap().counter_is_zero()
    }

    pub fn subcmp_counter_equal_to(&self, subcmp_idx: usize, value: usize) -> bool {
        self.subcmps.get(&subcmp_idx).unwrap().counter_equal_to(value)
    }

    // WRITE OPERATIONS
    pub fn set_var(self, idx: usize, value: Value) -> Self {
        let mut copy = self;
        copy.vars.insert(idx, value);
        copy
        // Env {
        //     vars: self.vars.clone().into_iter().chain([(idx, value)]).collect(),
        //     signals: self.signals.clone(),
        //     subcmps: self.subcmps.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        // }
    }

    pub fn set_signals(self, signals: HashMap<usize, Value>) -> Self {
        let mut copy = self;
        copy.signals = signals;
        copy
        // Env {
        //     vars: self.vars.clone(),
        //     signals,
        //     subcmps: self.subcmps.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        // }
    }

    pub fn set_signal(self, idx: usize, value: Value) -> Self {
        let mut copy = self;
        copy.signals.insert(idx, value);
        copy
        // Env {
        //     vars: self.vars.clone(),
        //     signals: self.signals.clone().into_iter().chain([(idx, value)]).collect(),
        //     subcmps: self.subcmps.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        // }
    }

    pub fn set_subcmp_signal(self, subcmp_idx: usize, signal_idx: usize, value: Value) -> Self {
        //let subcmp = &self.subcmps[&subcmp_idx];
        let mut copy = self;
        let subcmp_env = copy.subcmps.remove(&subcmp_idx).expect(format!("Can't set a signal of subcomponent {}", subcmp_idx).as_str());
        copy.subcmps.insert(subcmp_idx, subcmp_env.set_signal(signal_idx, value));
        copy
        // Env {
        //     vars: self.vars.clone(),
        //     signals: self.vars.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        //     subcmps: self
        //         .subcmps
        //         .clone()
        //         .into_iter()
        //         .chain([(subcmp_idx, subcmp.set_signal(signal_idx, value))])
        //         .collect(),
        // }
    }

    pub fn decrease_subcmp_counter(self, subcmp_idx: usize) -> Self {
        let mut copy = self;
        let subcmp_env = copy.subcmps.remove(&subcmp_idx).expect(format!("Can't decrease counter of subcomponent {}", subcmp_idx).as_str());
        copy.subcmps.insert(subcmp_idx, subcmp_env.decrease_counter());
        copy
        // let subcmp = &self.subcmps[&subcmp_idx];
        // Env {
        //     vars: self.vars.clone(),
        //     signals: self.vars.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        //     subcmps: self
        //         .subcmps
        //         .clone()
        //         .into_iter()
        //         .chain([(subcmp_idx, subcmp.decrease_counter())])
        //         .collect(),
        // }
    }

    pub fn set_subcmp_signals(self, subcmp_idx: usize, signals: HashMap<usize, Value>) -> Self {
        let mut copy = self;
        let subcmp_env = copy.subcmps.remove(&subcmp_idx).expect(format!("Can't decrease counter of subcomponent {}", subcmp_idx).as_str());
        copy.subcmps.insert(subcmp_idx, subcmp_env.set_signals(signals));
        copy
        // let subcmp = &self.subcmps[&subcmp_idx];
        // Env {
        //     vars: self.vars.clone(),
        //     signals: self.vars.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        //     subcmps: self
        //         .subcmps
        //         .clone()
        //         .into_iter()
        //         .chain([(subcmp_idx, subcmp.set_signals(signals))])
        //         .collect(),
        // }
    }

    pub fn run_subcmp(
        self,
        subcmp_idx: usize,
        name: &String,
        interpreter: &BucketInterpreter,
        observe: bool,
    ) -> Self {
        let env = {
            let code = &self.templates_library.borrow()[name].body;
            let subcmp_env = Env::new(self.templates_library.clone(), self.functions_library.clone())
                .set_signals(self.subcmps[&subcmp_idx].signals.clone());

            let (_, env) = interpreter.execute_instructions(
                code,
                subcmp_env,
                !interpreter.observer.ignore_subcmp_calls() && observe);
            env
        };
        self.set_subcmp_signals(subcmp_idx, env.signals)
    }

    pub fn create_subcmp(self, name: &'a String, base_index: usize, count: usize, template_id: usize) -> Self {
        let number_of_inputs = {
            self.templates_library.borrow()[name].number_of_inputs
        };
        let mut copy = self;
        for i in base_index..(base_index + count) {
            copy.subcmps
                .insert(i, SubcmpEnv::new(number_of_inputs, name, template_id));
        }
        copy
        //
        // Env {
        //     vars: self.vars.clone(),
        //     signals: self.vars.clone(),
        //     templates_library: self.templates_library.clone(),
        //     functions_library: self.functions_library.clone(),
        //     subcmps,
        // }
    }

    pub fn run_function(&self, name: &String,
                        interpreter: &BucketInterpreter,
                        args: Vec<Value>,
                        observe: bool) -> Value {
        let code = &self.functions_library.borrow()[name].body;
        let mut function_env = Env::new(self.templates_library.clone(), self.functions_library.clone());
        for (id, arg) in args.iter().enumerate() {
            function_env = function_env.set_var(id, arg.clone());
        }
        let interpreter = BucketInterpreter::clone_in_new_scope(interpreter, name.to_string());
        let r = interpreter.execute_instructions(
            &code,
            function_env,
            !interpreter.observer.ignore_function_calls() && observe);
        r.0.expect("Function must return a value!")
    }

    pub fn join(&self, other: &Self) -> Self {
        Env {
            vars: self.vars.join(&other.vars),
            signals: self.signals.join(&other.signals),
            subcmps: self.subcmps.join(&other.subcmps),
            templates_library: self.templates_library.clone(),
            functions_library: self.functions_library.clone()
        }
    }
}
