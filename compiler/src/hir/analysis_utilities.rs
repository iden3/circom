use crate::hir::very_concrete_program::*;
use program_structure::ast::Statement;
use program_structure::environment::VarEnvironment;
use program_structure::program_archive::ProgramArchive;
use std::collections::HashMap;

pub type E = VarEnvironment<VCT>;

pub struct GenericFunction {
    pub name: String,
    pub params_names: Vec<String>,
    pub body: Statement,
    pub concrete_instances: Vec<usize>,
}

pub struct State {
    pub external_signals: HashMap<String, HashMap<String, VCT>>,
    pub generic_functions: HashMap<String, GenericFunction>,
    pub vcf_collector: Vec<VCF>,
    pub quick_knowledge: HashMap<String, VCT>,
}

pub fn build_function_knowledge(program: ProgramArchive) -> State {
    let function_info = program.get_functions();
    let mut generic_functions = HashMap::new();
    for f in function_info.values() {
        let name = f.get_name().to_string();
        let gen_function = GenericFunction {
            name: name.clone(),
            params_names: f.get_name_of_params().clone(),
            body: f.get_body().clone(),
            concrete_instances: Vec::new(),
        };
        generic_functions.insert(name, gen_function);
    }

    State {
        external_signals: HashMap::with_capacity(0),
        vcf_collector: Vec::new(),
        generic_functions,
        quick_knowledge: HashMap::new(),
    }
}

pub fn build_component_info(triggers: &Vec<Trigger>) -> HashMap<String, HashMap<String, VCT>> {
    let mut external_signals = HashMap::new();
    for trigger in triggers {
        let mut signals = HashMap::new();
        for s in &trigger.external_signals {
            signals.insert(s.name.clone(), s.lengths.clone());
        }
        let signals = match external_signals.remove(&trigger.component_name) {
            None => signals,
            Some(old) => max_vct(signals, old),
        };
        external_signals.insert(trigger.component_name.clone(), signals);
    }
    external_signals
}
fn max_vct(l: HashMap<String, VCT>, mut r: HashMap<String, VCT>) -> HashMap<String, VCT> {
    let mut result = HashMap::new();
    for (s, tl) in l {
        let tr = r.remove(&s).unwrap();
        let max = std::cmp::max(tl, tr);
        result.insert(s, max);
    }
    result
}
pub fn build_environment(constants: &[Argument], params: &[Param]) -> E {
    let mut environment = E::new();
    for constant in constants {
        environment.add_variable(&constant.name, constant.lengths.clone());
    }
    for p in params {
        environment.add_variable(&p.name, p.length.clone());
    }
    environment
}
