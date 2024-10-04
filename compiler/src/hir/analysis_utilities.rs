use crate::hir::very_concrete_program::*;
use program_structure::ast::Statement;
use program_structure::environment::VarEnvironment;
use program_structure::program_archive::ProgramArchive;
use std::collections::HashMap;

pub type E = VarEnvironment<VCT>;

pub struct InfoBus {
    pub size: usize,
    pub signals: HashMap<String, VCT>,
    pub buses: HashMap<String, (VCT, InfoBus)>
}



pub struct GenericFunction {
    pub name: String,
    pub params_names: Vec<String>,
    pub body: Statement,
    pub concrete_instances: Vec<usize>,
}

pub struct State {
    pub external_signals: HashMap<String, (HashMap<String, VCT>, HashMap<String, (VCT, InfoBus)>)>,
    pub generic_functions: HashMap<String, GenericFunction>,
    pub vcf_collector: Vec<VCF>,
    pub quick_knowledge: HashMap<String, VCT>,
    pub buses_info: HashMap<String, InfoBus>
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
        buses_info: HashMap::new()
    }
}

pub fn build_component_info(triggers: &Vec<Trigger>, buses_table: &Vec<BusInstance>) -> 
        HashMap<String, (HashMap<String, VCT>, HashMap<String, (VCT, InfoBus)>)> {
    let mut external_signals = HashMap::new();
    for trigger in triggers {
        let mut signals = HashMap::new();
        let mut buses = HashMap::new();
        for s in &trigger.external_wires {
            match s{
                Wire::TSignal(s) =>{
                    signals.insert(s.name.clone(), s.lengths.clone());
                },
                Wire::TBus(s) =>{
                    buses.insert(s.name.clone(), (s.lengths.clone(), build_single_bus_info(s.bus_id, buses_table)));
                }
            }
        }

        let (signals, buses) = match external_signals.remove(&trigger.component_name){
            None => (signals, buses),
            Some((old_signals, old_buses)) =>{
                (max_vct(signals, old_signals), max_vct_bus(buses, old_buses))
            }
        };
        external_signals.insert(trigger.component_name.clone(), (signals, buses));

    }
    external_signals
}

pub fn build_buses_info(wires: &Vec<Wire>, buses_table: &Vec<BusInstance>) -> HashMap<String, InfoBus>{
    
    let mut info_buses = HashMap::new();
    for s in wires{
        if let Wire::TBus(bus) = s{
            info_buses.insert(bus.name.clone(), build_single_bus_info(bus.bus_id, buses_table));
        }
    }
    info_buses
}

fn build_single_bus_info(bus_id: usize, buses_table: &Vec<BusInstance>) -> InfoBus{
    let bus_instance = buses_table.get(bus_id).unwrap();
    let mut signals = HashMap::new();
    let mut buses = HashMap::new();

    for (name,s) in &bus_instance.fields{
        if s.bus_id.is_none(){ // case signal
            signals.insert(
                name.clone(), 
                s.dimensions.clone()
            );
        } else{
            let bus_id = s.bus_id.unwrap();
            buses.insert(
                name.clone(), 
                (s.dimensions.clone(), build_single_bus_info(bus_id, buses_table))
            );
        }
    }

    InfoBus{size: bus_instance.size, signals, buses}
}

fn max_vct(l: HashMap<String, VCT>, mut r: HashMap<String, VCT>) -> HashMap<String, VCT> {
    let mut result = HashMap::new();

    for (s, tl) in l {
        if r.contains_key(&s) {
            let tr = r.remove(&s).unwrap();
            let max = std::cmp::max(tl, tr);
            result.insert(s, max);
        } else{
            result.insert(s, tl);
        }
    }
    for (s, tr) in r{
        result.insert(s, tr);
    }
    result
}
fn max_vct_bus(l: HashMap<String, (VCT, InfoBus)>, mut r: HashMap<String, (VCT, InfoBus)>) -> HashMap<String, (VCT, InfoBus)> {
    fn compute_max_bus(l: InfoBus, r: InfoBus) ->InfoBus{
        let size = std::cmp::max(l.size, r.size);
        let signals = max_vct(l.signals, r.signals);
        let buses = max_vct_bus(l.buses, r.buses);
        InfoBus{size, signals, buses}
    }
    let mut result = HashMap::new();

    for (s, (tl, bl)) in l {
        if r.contains_key(&s) {
            let (tr, br) = r.remove(&s).unwrap();
            let tmax = std::cmp::max(tl, tr);
            let bmax = compute_max_bus(bl, br);
            result.insert(s, (tmax, bmax));
        } else{
            result.insert(s, (tl, bl));
        }
    }
    for (s, (tr, br)) in r{
        result.insert(s, (tr, br));
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
