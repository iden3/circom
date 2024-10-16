use super::executed_bus::BusConnexion;
use super::type_definitions::*;
use super::ExecutedBus;
use circom_algebra::algebra::ArithmeticExpression;
use compiler::hir::very_concrete_program::*;
use dag::DAG;
use num_bigint::BigInt;
use program_structure::ast::{SignalType, Statement};
use std::collections::{HashMap, HashSet};
use crate::execution_data::AExpressionSlice;


struct Connexion {
    full_name: String,
    inspect: SubComponentData,
    dag_offset: usize,
    dag_component_offset: usize,
    dag_jump: usize,
    dag_component_jump: usize,
}

#[derive(Clone)]
pub struct PreExecutedTemplate {
    pub template_name: String,
    pub parameter_instances: Vec<AExpressionSlice>,
    pub inputs: HashMap<String, TagNames>,
    pub outputs: HashMap<String, TagNames>,
} 

impl PreExecutedTemplate {
    pub fn new(
        name: String,
        instance: Vec<AExpressionSlice>,
        inputs: HashMap<String, TagNames>,
        outputs: HashMap<String, TagNames>,
    ) -> PreExecutedTemplate {
        PreExecutedTemplate {
            template_name: name,
            parameter_instances: instance,
            inputs,
            outputs,
        }
    }

    pub fn template_name(&self) -> &String {
        &self.template_name
    }

    pub fn parameter_instances(&self) -> &Vec<AExpressionSlice>{
        &self.parameter_instances
    }

    pub fn inputs(&self) -> &HashMap<String, TagNames>{
        &self.inputs
    }

    pub fn outputs(&self) -> &HashMap<String, TagNames> {
        &self.outputs
    }
}



pub struct ExecutedTemplate {
    pub code: Statement,
    pub template_name: String,
    pub report_name: String,
    pub inputs: WireCollector,
    pub outputs: WireCollector,
    pub intermediates: WireCollector,
    pub ordered_signals: WireCollector,
    pub constraints: Vec<Constraint>,
    pub components: ComponentCollector,
    pub number_of_components: usize,
    pub public_inputs: HashSet<String>,
    pub parameter_instances: ParameterContext,
    pub tag_instances: HashMap<String, TagWire>,
    pub signal_to_tags: HashMap<Vec<String>, BigInt>, 
    // only store the info of the tags with value
    // name of tag -> value
    pub is_parallel: bool,
    pub has_parallel_sub_cmp: bool,
    pub is_custom_gate: bool,
    pub underscored_signals: Vec<String>,
    connexions: Vec<Connexion>,
    pub bus_connexions: HashMap<String, BusConnexion>,
}

impl ExecutedTemplate {
    pub fn new(
        public: Vec<String>,
        name: String,
        report_name: String,
        instance: ParameterContext,
        tag_instances: HashMap<String, TagWire>,
        code: Statement,
        is_parallel: bool,
        is_custom_gate: bool
    ) -> ExecutedTemplate {
        let public_inputs: HashSet<_> = public.iter().cloned().collect();


        ExecutedTemplate {
            report_name,
            public_inputs,
            is_parallel,
            has_parallel_sub_cmp: false,
            is_custom_gate,
            code: code.clone(),
            template_name: name,
            parameter_instances: instance,
            signal_to_tags: HashMap::new(),
            tag_instances,
            inputs: WireCollector::new(),
            outputs: WireCollector::new(),
            intermediates: WireCollector::new(),
            ordered_signals: WireCollector::new(),
            constraints: Vec::new(),
            components: ComponentCollector::new(),
            number_of_components: 0,
            connexions: Vec::new(),
            bus_connexions: HashMap::new(),
            underscored_signals: Vec::new(),
        }
    }

    pub fn is_equal(&self, name: &str, context: &ParameterContext, tag_context: &HashMap<String, TagWire>) -> bool {
        self.template_name == name 
            && self.parameter_instances == *context
            && self.tag_instances == *tag_context
    }

    pub fn add_arrow(&mut self, component_name: String, data: SubComponentData) {
        let cnn =
            Connexion { full_name: component_name, inspect: data, dag_offset: 0, dag_component_offset: 0, dag_jump: 0, dag_component_jump: 0};
            self.connexions.push(cnn);
    }

    pub fn add_bus_arrow(&mut self, bus_name: String, data: BusData){
        let cnn =
            BusConnexion { full_name:bus_name.clone(), inspect: data, dag_offset: 0, dag_jump: 0};
            self.bus_connexions.insert(bus_name, cnn);
    }

    pub fn add_input(
        &mut self, 
        input_name: &str, 
        dimensions: &[usize], 
        is_bus: bool
    ) {
        let wire_info = WireData{
            name: input_name.to_string(),
            length: dimensions.to_vec(),
            is_bus
        };
        self.inputs.push(wire_info.clone());
        self.ordered_signals.push(wire_info);
    }

    pub fn add_output(
        &mut self, 
        output_name: &str, 
        dimensions: &[usize], 
        is_bus: bool
    ) {
        let wire_info = WireData{
            name: output_name.to_string(),
            length: dimensions.to_vec(),
            is_bus
        };
        self.outputs.push(wire_info.clone());
        self.ordered_signals.push(wire_info);
    }

    pub fn add_intermediate(
        &mut self, 
        intermediate_name: &str, 
        dimensions: &[usize], 
        is_bus: bool
    ) {
        let wire_info = WireData{
            name: intermediate_name.to_string(),
            length: dimensions.to_vec(),
            is_bus
        };
        self.intermediates.push(wire_info.clone());    
        self.ordered_signals.push(wire_info);
    }

    // Used to update the values of the signals 
    // We call to this function to store the signals with values
    // when we finish the execution of a template
    pub fn add_tag_signal(
        &mut self, 
        signal_name: Vec<String>, 
        value: BigInt
    ){
        self.signal_to_tags.insert(signal_name, value);
    }

    pub fn add_component(&mut self, component_name: &str, dimensions: &[usize]) {
        self.components.push((component_name.to_string(), dimensions.to_vec()));
        self.number_of_components += dimensions.iter().fold(1, |p, c| p * (*c));
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn add_underscored_signal(&mut self, signal: &str) {
        self.underscored_signals.push(signal.to_string());
    }

    pub fn template_name(&self) -> &String {
        &self.template_name
    }

    pub fn parameter_instances(&self) -> &ParameterContext {
        &self.parameter_instances
    }

    pub fn tag_instances(&self) -> &HashMap<String, TagWire> {
        &self.tag_instances
    }

    pub fn inputs(&self) -> &WireCollector {
        &self.inputs
    }

    pub fn outputs(&self) -> &WireCollector {
        &self.outputs
    }

    pub fn intermediates(&self) -> &WireCollector {
        &self.intermediates
    }

    pub fn insert_in_dag(&mut self, dag: &mut DAG, buses_info : &Vec<ExecutedBus>) {
        let parameters = {
            let mut parameters = vec![];
            for (_, data) in self.parameter_instances.clone() {
                let (_, values) = data.destruct();
                for value in as_big_int(values) {
                    parameters.push(value);
                }
            }
            parameters
        }; // repeated code from function build_arguments in export_to_circuit

        dag.add_node(
            self.report_name.clone(),
            parameters,
            self.is_parallel,
            self.is_custom_gate
        );
        self.build_wires(dag, buses_info);
        self.build_ordered_signals(dag, buses_info);
        self.build_connexions(dag);
        self.build_constraints(dag);
    }

    fn build_wires(&self, dag: &mut DAG, buses_info : &Vec<ExecutedBus>) {
        for wire_data in self.outputs() {
            let state = State { basic_name: wire_data.name.clone(), name: wire_data.name.clone(), dim: 0 };
            let config = SignalConfig { signal_type: 1, dimensions: &wire_data.length, is_public: false };
            if wire_data.is_bus{
                generate_bus_symbols(dag, state, &config, &self.bus_connexions, buses_info );
            } else{
                generate_symbols(dag, state, &config);
            }
        }
        for wire_data in self.inputs() {
            if self.public_inputs.contains(&wire_data.name) {
                let state = State { basic_name: wire_data.name.clone(),  name: wire_data.name.clone(), dim: 0 };
                let config = SignalConfig { signal_type: 0, dimensions: &wire_data.length, is_public: true };
                if wire_data.is_bus{
                    generate_bus_symbols(dag, state, &config, &self.bus_connexions, buses_info );
                } else{
                    generate_symbols(dag, state, &config);
                }
            }
        }
        for wire_data in self.inputs() {
            if !self.public_inputs.contains(&wire_data.name) {
                let state = State { basic_name: wire_data.name.clone(), name: wire_data.name.clone(), dim: 0 };
                let config = SignalConfig { signal_type: 0, dimensions: &wire_data.length, is_public: false };
                if wire_data.is_bus{
                    generate_bus_symbols(dag, state, &config, &self.bus_connexions, buses_info );
                } else{
                    generate_symbols(dag, state, &config);
                }
            }
        }
        for wire_data in self.intermediates() {
            let state = State { basic_name: wire_data.name.clone(), name: wire_data.name.clone(), dim: 0 };
            let config = SignalConfig { signal_type: 2, dimensions: &wire_data.length, is_public: false };
            if wire_data.is_bus{
                generate_bus_symbols(dag, state, &config, &self.bus_connexions, buses_info );
            } else{
                generate_symbols(dag, state, &config);
            }
        }
    }

    fn build_ordered_signals(&self, dag: &mut DAG, buses_info : &Vec<ExecutedBus>) {
        for wire_data in &self.ordered_signals {
            let state = State { basic_name: wire_data.name.clone(), name: wire_data.name.clone(), dim: 0 };
            let config = OrderedSignalConfig { dimensions: &wire_data.length };
            if wire_data.is_bus{
                generate_ordered_bus_symbols(dag, state, &config, &self.bus_connexions, buses_info );
            } else{
                generate_ordered_symbols(dag, state, &config);
            }
        }
    }

    fn build_connexions(&mut self, dag: &mut DAG) {
        self.connexions.sort_by(|l, r| {
            use std::cmp::Ordering;
            let l_data = &l.inspect;
            let r_data = &r.inspect;
            let cmp_0 = l_data.name.cmp(&r_data.name);
            match cmp_0 {
                Ordering::Equal => l_data.indexed_with.cmp(&r_data.indexed_with),
                v => v,
            }
        });
        let filtered_components = filter_used_components(self);
        self.components = filtered_components.0;
        self.number_of_components = filtered_components.1;
        for cnn in &mut self.connexions {
            cnn.dag_offset = dag.get_entry().unwrap().get_out();
            cnn.dag_component_offset = dag.get_entry().unwrap().get_out_component();
            dag.add_edge(cnn.inspect.goes_to, &cnn.full_name, cnn.inspect.is_parallel);
            cnn.dag_jump = dag.get_entry().unwrap().get_out() - cnn.dag_offset;
            cnn.dag_component_jump = dag.get_entry().unwrap().get_out_component() - cnn.dag_component_offset;
        }
        self.has_parallel_sub_cmp = dag.nodes[dag.main_id()].has_parallel_sub_cmp();
        dag.set_number_of_subcomponents_indexes(self.number_of_components);
    }
    fn build_constraints(&self, dag: &mut DAG) {
        
        for c in &self.constraints {
            let correspondence = dag.get_main().unwrap().correspondence();
            let cc = Constraint::apply_correspondence(c, correspondence);
            dag.add_constraint(cc);
        }
        for s in &self.underscored_signals{
            let correspondence = dag.get_main().unwrap().correspondence();
            let new_s = correspondence.get(s).unwrap().clone();
            dag.add_underscored_signal(new_s);
        }
    }

    pub fn export_to_circuit(self, instances: &mut [TemplateInstance], buses_info : &Vec<BusInstance>) -> TemplateInstance {
        use SignalType::*;
        fn build_triggers(
            instances: &mut [TemplateInstance],
            connexions: Vec<Connexion>,
        ) -> Vec<Trigger> {
            let mut triggers = vec![];
            for cnn in connexions {
                let data = cnn.inspect;
                instances[data.goes_to].is_parallel_component |= data.is_parallel;
                instances[data.goes_to].is_not_parallel_component |= !(data.is_parallel);
                
                let mut external_wires = Vec::new();
                for wire in &instances[data.goes_to].wires{
                    if wire.xtype() != SignalType::Intermediate{
                        external_wires.push(wire.clone());
                    }
                }

                let trigger = Trigger {
                    offset: cnn.dag_offset,
                    component_offset: cnn.dag_component_offset,
                    component_name: data.name,
                    indexed_with: data.indexed_with,
                    is_parallel: data.is_parallel || instances[data.goes_to].is_parallel,
                    runs: instances[data.goes_to].template_header.clone(),
                    template_id: data.goes_to,
                    external_wires,
                    has_inputs: instances[data.goes_to].number_of_inputs > 0,
                };
                triggers.push(trigger);
            }
            triggers
        }

        fn build_components(components: ComponentCollector) -> Vec<Component> {
            let mut cmp = vec![];
            for (name, lengths) in components {
                cmp.push(Component { name, lengths })
            }
            cmp
        }

        fn build_arguments(parameter_instances: ParameterContext) -> Vec<Argument> {
            let mut arguments = vec![];
            for (name, data) in parameter_instances {
                let (dim, value) = data.destruct();
                let argument = Argument { name, lengths: dim, values: as_big_int(value) };
                arguments.push(argument);
            }
            arguments
        }

        let header = format!("{}_{}", self.template_name, instances.len());
        let clusters = build_clusters(&self, instances);
        let triggers = build_triggers(instances, self.connexions);
        let components = build_components(self.components);
        let arguments = build_arguments(self.parameter_instances);
        let config = TemplateConfig {
            header,
            clusters,
            triggers,
            arguments,
            components,
            id: instances.len(),
            is_parallel: self.is_parallel,
            has_parallel_sub_cmp: self.has_parallel_sub_cmp,
            code: self.code,
            name: self.template_name,
            number_of_components : self.number_of_components,
            signals_to_tags: self.signal_to_tags,
        };

        let mut instance = TemplateInstance::new(config);

        let mut public = vec![];
        let mut not_public = vec![];
        for s in self.inputs {
            if self.public_inputs.contains(&s.name) {
                public.push(s);
            } else {
                not_public.push(s);
            }
        }
        let mut local_id = 0;
        let mut dag_local_id = 1;
        for s in self.outputs {
            if s.is_bus{
                let bus_node = self.bus_connexions.get(&s.name).unwrap().inspect.goes_to;
                let info_bus = buses_info.get(bus_node).unwrap();
                let size = s.length.iter().fold(info_bus.size, |p, c| p * (*c));
                let bus = Bus{
                    name: s.name,
                    lengths: s.length,
                    local_id,
                    dag_local_id,
                    bus_id: bus_node,
                    size,
                    xtype: Output,
                };
                local_id += bus.size;
                dag_local_id += bus.size;
                instance.add_signal(Wire::TBus(bus));
            } else{
                let size = s.length.iter().fold(1, |p, c| p * (*c));
                let signal = Signal { name: s.name, lengths: s.length, local_id, dag_local_id, xtype: Output, size};
                local_id += signal.size;
                dag_local_id += signal.size;
                instance.add_signal(Wire::TSignal(signal));
            }
        }

        for s in public {
            if s.is_bus{
                let bus_node = self.bus_connexions.get(&s.name).unwrap().inspect.goes_to;
                let info_bus = buses_info.get(bus_node).unwrap();
                let size = s.length.iter().fold(info_bus.size, |p, c| p * (*c));
                let bus = Bus{
                    name: s.name,
                    lengths: s.length,
                    local_id,
                    dag_local_id,
                    bus_id: bus_node,
                    size,
                    xtype: Input,
                };
                local_id += bus.size;
                dag_local_id += bus.size;
                instance.add_signal(Wire::TBus(bus));
            } else{
                let size = s.length.iter().fold(1, |p, c| p * (*c));
                let signal = Signal { name: s.name, lengths: s.length, local_id, dag_local_id, xtype: Input, size};
                local_id += signal.size;
                dag_local_id += signal.size;
                instance.add_signal(Wire::TSignal(signal));
            }
        }
        for s in not_public {
            if s.is_bus{
                let bus_node = self.bus_connexions.get(&s.name).unwrap().inspect.goes_to;
                let info_bus = buses_info.get(bus_node).unwrap();
                let size = s.length.iter().fold(info_bus.size, |p, c| p * (*c));
                let bus = Bus{
                    name: s.name,
                    lengths: s.length,
                    local_id,
                    dag_local_id,
                    bus_id: bus_node,
                    size,
                    xtype: Input,
                };
                local_id += bus.size;
                dag_local_id += bus.size;
                instance.add_signal(Wire::TBus(bus));
            } else{
                let size = s.length.iter().fold(1, |p, c| p * (*c));
                let signal = Signal { name: s.name, lengths: s.length, local_id, dag_local_id, xtype: Input, size};
                local_id += signal.size;
                dag_local_id += signal.size;
                instance.add_signal(Wire::TSignal(signal));
            }
        }
        for s in self.intermediates {
            if s.is_bus{
                let bus_node = self.bus_connexions.get(&s.name).unwrap().inspect.goes_to;
                let info_bus = buses_info.get(bus_node).unwrap();
                let size = s.length.iter().fold(info_bus.size, |p, c| p * (*c));
                let bus = Bus{
                    name: s.name,
                    lengths: s.length,
                    local_id,
                    dag_local_id,
                    bus_id: bus_node,
                    size,
                    xtype: Intermediate,
                };
                local_id += bus.size;
                dag_local_id += bus.size;
                instance.add_signal(Wire::TBus(bus));
            } else{
                let size = s.length.iter().fold(1, |p, c| p * (*c));
                let signal = Signal { name: s.name, lengths: s.length, local_id, dag_local_id, xtype: Intermediate, size};
                local_id += signal.size;
                dag_local_id += signal.size;
                instance.add_signal(Wire::TSignal(signal));
            }
        }

        instance
    }
    
}

struct SignalConfig<'a> {
    is_public: bool,
    signal_type: usize,
    dimensions: &'a [usize],
}
struct State {
    basic_name: String, //Only name without array accesses [].
    name: String, //Full name with array accesses.
    dim: usize,
}
fn generate_symbols(dag: &mut DAG, state: State, config: &SignalConfig) {
    if state.dim == config.dimensions.len() {
        if config.signal_type == 0 {
            dag.add_input(state.name, config.is_public);
        } else if config.signal_type == 1 {
            dag.add_output(state.name);
        } else if config.signal_type == 2 {
            dag.add_intermediate(state.name);
        }
    } else {
        let mut index = 0;
        while index < config.dimensions[state.dim] {
            let new_state =
                State { basic_name: state.basic_name.clone(), name: format!("{}[{}]", state.name, index), dim: state.dim + 1 };
            generate_symbols(dag, new_state, config);
            index += 1;
        }
    }
}

// TODO: move to bus?
fn generate_bus_symbols(dag: &mut DAG, state: State, config: &SignalConfig, bus_connexions: &HashMap<String, BusConnexion>, buses: &Vec<ExecutedBus>) {
    let bus_connection = bus_connexions.get(&state.basic_name).unwrap();
    let ex_bus2 = buses.get(bus_connection.inspect.goes_to).unwrap();
    if state.dim == config.dimensions.len() {
        for info_field in ex_bus2.fields(){
            let signal_name = format!("{}.{}",state.name, info_field.name);
            let state = State { basic_name: info_field.name.clone(), name: signal_name, dim: 0 };
            let config = SignalConfig { signal_type: config.signal_type, dimensions: &info_field.length, is_public: config.is_public };
            if info_field.is_bus{
                generate_bus_symbols(dag, state, &config, ex_bus2.bus_connexions(), buses);
            } else{
                generate_symbols(dag, state, &config);
            }
        }

    } else {
        let mut index = 0;
        while index < config.dimensions[state.dim] {
            let new_state =
                State { basic_name: state.basic_name.clone(), name: format!("{}[{}]", state.name, index), dim: state.dim + 1 };
            generate_bus_symbols(dag, new_state, config, bus_connexions, buses);
            index += 1;
        }
    }
}


struct OrderedSignalConfig<'a> {
    dimensions: &'a [usize],
}
fn generate_ordered_symbols(dag: &mut DAG, state: State, config: &OrderedSignalConfig) {
    if state.dim == config.dimensions.len() {
        dag.add_ordered_signal(state.name);
    } else {
        let mut index = 0;
        while index < config.dimensions[state.dim] {
            let new_state =
                State { basic_name: state.basic_name.clone(), name: format!("{}[{}]", state.name, index), dim: state.dim + 1 };
            generate_ordered_symbols(dag, new_state, config);
            index += 1;
        }
    }
}

// TODO: move to bus?
fn generate_ordered_bus_symbols(dag: &mut DAG, state: State, config: &OrderedSignalConfig, bus_connexions: &HashMap<String, BusConnexion>, buses: &Vec<ExecutedBus>) {
    let bus_connection = bus_connexions.get(&state.basic_name).unwrap();
    let ex_bus2 = buses.get(bus_connection.inspect.goes_to).unwrap();
    if state.dim == config.dimensions.len() {
        for info_field in ex_bus2.fields(){
            let signal_name = format!("{}.{}",state.name, info_field.name);
            let state = State { basic_name: info_field.name.clone(), name: signal_name, dim: 0 };
            let config = OrderedSignalConfig {dimensions: &info_field.length };
            if info_field.is_bus{
                generate_ordered_bus_symbols(dag, state, &config, ex_bus2.bus_connexions(), buses);
            } else{
                generate_ordered_symbols(dag, state, &config);
            }
        }

    } else {
        let mut index = 0;
        while index < config.dimensions[state.dim] {
            let new_state =
                State { basic_name: state.basic_name.clone(), name: format!("{}[{}]", state.name, index), dim: state.dim + 1 };
            generate_ordered_bus_symbols(dag, new_state, config, bus_connexions, buses);
            index += 1;
        }
    }
}

fn as_big_int(exprs: Vec<ArithmeticExpression<String>>) -> Vec<BigInt> {
    let mut numbers = Vec::with_capacity(exprs.len());
    for e in exprs {
        if let ArithmeticExpression::Number { value } = e {
            numbers.push(value);
        }
    }
    numbers
}

fn filter_used_components(tmp: &ExecutedTemplate) -> (ComponentCollector, usize) {
    fn compute_number_cmp(lengths: &Vec<usize>) -> usize {
        lengths.iter().fold(1, |p, c| p * (*c))
    }
    let mut used = HashSet::with_capacity(tmp.components.len());
    for cnn in &tmp.connexions {
        used.insert(cnn.inspect.name.clone());
    }
    let mut filtered = Vec::with_capacity(used.len());
    let mut number_of_components = 0;
    for cmp in &tmp.components {
        if used.contains(&cmp.0) {
            filtered.push(cmp.clone());
            number_of_components = number_of_components + compute_number_cmp(&cmp.1);
        }
    }
    (filtered, number_of_components)
}

#[derive(Copy, Clone)]
enum POS {
    T,
    K(usize),
    B,
}
impl POS {
    pub fn least_upper_bound(l: POS, r: POS) -> POS {
        use POS::*;
        match (l, r) {
            (K(v0), K(v1)) if v0 == v1 => K(v0),
            (B, p) | (p, B) => p,
            _ => T,
        }
    }
}
fn apply_pos_to_connexions(connexions: &[Connexion]) -> HashMap<String, POS> {
    use POS::*;
    let mut solution = HashMap::with_capacity(connexions.len());
    for cnn in connexions {
        let name = &cnn.inspect.name;
        solution.insert(name.clone(), B);
    }
    for cnn in connexions {
        let data = &cnn.inspect;
        let prev = solution.remove(&data.name).unwrap();
        let new = K(data.goes_to);
        let val = POS::least_upper_bound(prev, new);
        solution.insert(data.name.clone(), val);
    }
    solution
}

fn mixed_components(exec_tmp: &ExecutedTemplate) -> Vec<bool> {
    use POS::*;
    let solution = apply_pos_to_connexions(&exec_tmp.connexions);
    let mut mixed = vec![false; exec_tmp.components.len()];
    for (index, value) in exec_tmp.components.iter().enumerate() {
        let pos_value = solution.get(&value.0).unwrap();
        mixed[index] = mixed[index] || matches!(pos_value, T);
    }
    mixed
}

fn build_clusters(tmp: &ExecutedTemplate, instances: &[TemplateInstance]) -> Vec<TriggerCluster> {
    let components = &tmp.components;
    let connexions = &tmp.connexions;
    let mixed = mixed_components(tmp);
    let mut result = Vec::with_capacity(components.len());

    // Cluster initialization
    let mut cmp_data = HashMap::with_capacity(components.len());
    let mut index = 0;
    while index < connexions.len() {
        let cnn_data = &connexions[index].inspect;
        let offset_jump = connexions[index].dag_jump;
        let component_offset_jump = connexions[index].dag_component_jump;
        let instance_id = connexions[index].inspect.goes_to;
        let sub_cmp_header = instances[instance_id].template_header.clone();
        let start = index;
        let mut end = index;
        let mut defined_positions: Vec<Vec<usize>> = vec![];
        loop {
            if end == connexions.len() {
                break;
            } else if connexions[end].inspect.name != cnn_data.name {
                break;
            } else {
                defined_positions.push(connexions[end].inspect.indexed_with.clone());
                end += 1;
            }
        }
        
        let cluster = TriggerCluster {
            slice: start..end,
            length: end - start,
            defined_positions: defined_positions,
            cmp_name: cnn_data.name.clone(),
            xtype: ClusterType::Uniform { offset_jump, component_offset_jump, instance_id, header: sub_cmp_header },
        };
        cmp_data.insert(cnn_data.name.clone(), cluster);
        index = end;
    }

    // cmp_data and result binding
    let mut index = 0;
    while index < components.len() {
        let cmp_name = &components[index].0;
        let mut cluster = cmp_data.remove(cmp_name).unwrap();
        let start = cluster.slice.start;
        let tmp_id = connexions[start].inspect.goes_to;
        let tmp_name = instances[tmp_id].template_name.clone();
        if mixed[index] {
            cluster.xtype = ClusterType::Mixed { tmp_name };
        }
        result.push(cluster);
        index += 1;
    }
    result
}

pub fn templates_in_mixed_arrays(exec_tmp: &ExecutedTemplate, no_templates: usize) -> Vec<bool> {
    use POS::*;
    let solution = apply_pos_to_connexions(&exec_tmp.connexions);
    let mut mixed = vec![false; no_templates];
    for cnn in &exec_tmp.connexions {
        let data = &cnn.inspect;
        let pos_value = solution.get(&data.name).unwrap();
        mixed[data.goes_to] = mixed[data.goes_to] || matches!(pos_value, T);
    }
    mixed
}
