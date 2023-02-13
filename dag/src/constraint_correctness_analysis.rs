use super::Node;
use circom_algebra::algebra::Constraint;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use std::collections::{HashMap, HashSet, LinkedList};

type C = Constraint<usize>;
const UNCONSTRAINED_SIGNAL_CODE: ReportCode = ReportCode::UnconstrainedSignal;
const UNUSED_OUTPUT_CODE: ReportCode = ReportCode::UnusedOutput;
const UNCONSTRAINED_IOSIGNAL_CODE: ReportCode = ReportCode::UnconstrainedIOSignal;

struct UnusedOutput;
impl UnusedOutput {
    pub fn new(signal: &str, template: &str) -> Report {
            let msg = format!("In template \"{}\": Output signal {} does not depend on any input via constraints.", template, signal);
            let report = Report::warning(msg, UNUSED_OUTPUT_CODE);
            report
    }
}


struct UnconstrainedSignal;
impl UnconstrainedSignal {
    pub fn new(signal: &str, template: &str, examples: &Vec<String>) -> Report {
        
        if examples.len() == 1{
            let msg = format!("In template \"{}\": Local signal {} does not appear in any constraint", template, examples[0]);
            let report = Report::warning(msg, UNCONSTRAINED_SIGNAL_CODE);
            report
        } else{
            let msg = format!("In template \"{}\": Array of local signals {} contains a total of {} signals that do not appear in any constraint", template, signal, examples.len());
            let mut report = Report::warning(msg, UNCONSTRAINED_SIGNAL_CODE);
            let ex = format!("For example: {}, {}.", examples[0], examples[1]);
            report.add_note(ex);
            report
        }
    }
}

struct UnconstrainedIOSignal;
impl UnconstrainedIOSignal {
    pub fn new(signal: &str, template: &str, examples: &Vec<String>) -> Report {
        
        if examples.len() == 1{
            let msg = format!("In template \"{}\": Subcomponent input/output signal {} does not appear in any constraint of the father component", template, examples[0]);
            let report = Report::warning(msg, UNCONSTRAINED_IOSIGNAL_CODE);
            report
        } else{
            let msg = format!("In template \"{}\": Array of subcomponent input/output signals {} contains a total of {} signals that do not appear in any constraint of the father component", template, signal, examples.len());
            let mut report = Report::warning(msg, UNCONSTRAINED_IOSIGNAL_CODE);
            let ex = format!("For example: {}, {}.", examples[0], examples[1]);
            report.add_note(ex);
            report
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SignalType {
    Local,
    IOSubcomponent,
}
struct Analysis {
    template_name: String,
    // signal name, type and number of appearances
    signal_stats: Vec<(String, SignalType, usize)>,
}

fn split_signal_name_index(name: &String)-> String{
    let split_components:Vec<&str> = name.split(".").collect(); // split the name of components
    let mut signal_name = "".to_string();
    for i in 0..split_components.len()-1{
        signal_name = signal_name + split_components[i] + "."; // take the index of the components
    }
    // no take the index of the array position
    let aux_last_component = split_components[split_components.len()-1].to_string();
    let split_index_last_component = 
        aux_last_component.split("[").next().unwrap(); 
    signal_name + split_index_last_component
}

fn analysis_interpretation(analysis: Analysis, result: &mut AnalysisResult) {
    let tmp_name = analysis.template_name;
    let stats = analysis.signal_stats;

    let mut signal2unconstrainedex: HashMap<String, (SignalType, Vec<String>)> = HashMap::new();

    for (name, xtype, no_appearances) in stats {
        if no_appearances == 0 {
            let signal_name = split_signal_name_index(&name);
                
            match signal2unconstrainedex.get_mut(&signal_name){
                Some((_, examples)) =>{
                    examples.push(name.clone());
                },
                None =>{
                    signal2unconstrainedex.insert(signal_name.to_string(), (xtype, vec![name.clone()]));
                }
            }
        }
    }
    for (name, (xtype, examples)) in signal2unconstrainedex{
        if xtype == SignalType::Local{
            result.warnings.push(UnconstrainedSignal::new(&name, &tmp_name, &examples));
        } else{
            result.warnings.push(UnconstrainedIOSignal::new(&name, &tmp_name, &examples));
        }
    }
}

fn visit_node(node: &mut Node) -> Analysis {

    let mut constraint_counter = HashMap::new();
    let mut rev_correspondence = HashMap::new();
    for (name, id) in &node.signal_correspondence {
        if node.is_reachable_signal(*id){
            rev_correspondence.insert(*id, name.to_string());
            constraint_counter.insert(*id, 0);
        }
    }
    let length_bound = Vec::len(&node.constraints);
    let work = std::mem::replace(&mut node.constraints, Vec::with_capacity(length_bound));
    for mut constraint in work {
        let signals = constraint.take_cloned_signals();
        for signal in signals {
            let prev = constraint_counter.remove(&signal).unwrap();
            constraint_counter.insert(signal, prev + 1);
        }
        C::remove_zero_value_coefficients(&mut constraint);
        if !C::is_empty(&constraint) {
            Vec::push(&mut node.constraints, constraint);
        }
    }

    for signal in &node.underscored_signals{
        let prev = constraint_counter.remove(&signal).unwrap();
        constraint_counter.insert(*signal, prev + 1);
    }

    let mut signal_stats = vec![];
    for (id, appearances) in constraint_counter {
        let name = rev_correspondence.remove(&id).unwrap();
        let signal_type = if node.is_local_signal(id) {
            SignalType::Local
        } else {
            SignalType::IOSubcomponent
        };
        signal_stats.push((name, signal_type, appearances));
    }
    signal_stats.sort_by(|a, b| a.0.cmp(&b.0));
    Analysis {
        template_name: node.template_name.clone(),
        signal_stats,
    }
}

pub struct AnalysisResult {
    pub errors: ReportCollection,
    pub warnings: ReportCollection,
}
pub fn analyse(nodes: &mut [Node]) -> AnalysisResult {
    let mut result = AnalysisResult { errors: vec![], warnings: vec![] };
    let mut visited : HashSet<String> = HashSet::new();
    for node in nodes {
        let consts = node.constraints.clone();
        let template_name = node.template_name.clone();
        let mut usize_to_signal_corresponde = HashMap::new();
        let mut component_to_cluster: HashMap<String,Vec<usize>> = HashMap::new();
        for (a,b) in node.signal_correspondence.clone(){
            usize_to_signal_corresponde.insert(b, a.clone());
            if a.contains("."){
                let component_name = &a[..a.find(".").unwrap()].to_string();
                component_to_cluster.entry(component_name.clone()).and_modify(|vec| vec.push(b)).or_insert(vec![b]);
            }
        }
        let analysis = visit_node(node);
        if !node.is_custom_gate() && !visited.contains(&node.template_name.clone()){
            let mut result2 = AnalysisResult { errors: vec![], warnings: vec![] };
            analysis_interpretation(analysis, &mut result2);    
            result.errors.append(&mut result2.errors);
            result.warnings.append(&mut result2.warnings);
            visited.insert(node.template_name.clone());
            // mover esto a una función para que quede más claro?
            let mut linked_constraints = LinkedList::new();
            for c in consts {
                linked_constraints.push_back(c);
            }
            let constant_signals = get_constant_signals(&node);
            let clusters = build_clusters(linked_constraints, node.number_of_signals+1,component_to_cluster);
            for cluster in clusters {
                let (inputs, outputs) = get_inputs_outputs_from_cluster(& node, cluster);
                if inputs.is_empty() && !outputs.is_empty() && node.number_of_inputs() > 0{
                    for o in outputs{
                        if !constant_signals.contains(&o){
                            result.warnings.push(UnusedOutput::new(&usize_to_signal_corresponde[&o],&template_name));
                        }
                    }
                // } else if !inputs.is_empty() && outputs.is_empty() && node.number_of_outputs() > 0{
                //      for o in inputs{
                //         result.warnings.push(UnusedInput::new(&usize_to_signal_corresponde[&o],&template_name));
                //   }
                }
            }
        }
    }
    result
}

fn get_constant_signals(node: &Node)->HashSet<usize>{
    let mut constants =  HashSet::new();
    for c in &node.constraints{
        if c.is_constant_equality(){
            let signals_aux = c.take_cloned_signals();
            constants.insert(*signals_aux.iter().next().unwrap());
        }
    }
    constants
}

fn get_inputs_outputs_from_cluster(node: &Node, cluster: Cluster) -> (Vec<usize>, Vec<usize>) {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for c in cluster.constraints{
        let signals = c.take_cloned_signals();
        for s in signals {
            if node.inputs.contains(&s){
                inputs.push(s);
            } else if node.outputs.contains(&s){
                outputs.push(s);
            }
        }
    }
    (inputs, outputs)
}


#[derive(Default, Clone)]
struct Cluster {
    constraints: LinkedList<C>,
    num_signals: usize
}
impl Cluster {
    pub fn new(constraint: C, num_signals: usize) -> Cluster {
        let mut new = Cluster::default();
        LinkedList::push_back(&mut new.constraints, constraint);
        new.num_signals = num_signals;
        new
    }

    pub fn merge(mut c0: Cluster, mut c1: Cluster) -> Cluster {
        let mut result = Cluster::default();
        LinkedList::append(&mut result.constraints, &mut c0.constraints);
        LinkedList::append(&mut result.constraints, &mut c1.constraints);
        result.num_signals = c0.num_signals + c1.num_signals - 1;
        result
    }

    pub fn size(&self) -> usize {
        LinkedList::len(&self.constraints)
    }
}

fn build_clusters(constraints: LinkedList<C>, no_vars: usize, subcomponents : HashMap<String,Vec<usize>>) -> Vec<Cluster> {
    type ClusterArena = Vec<Option<Cluster>>;
    type ClusterPath = Vec<usize>;
    fn shrink_jumps_and_find(c_to_c: &mut ClusterPath, org: usize) -> usize {
        let mut current = org;
        let mut jumps = Vec::new();
        while current != c_to_c[current] {
            Vec::push(&mut jumps, current);
            current = c_to_c[current];
        }
        while let Some(redirect) = Vec::pop(&mut jumps) {
            c_to_c[redirect] = current;
        }
        current
    }

    fn arena_merge(arena: &mut ClusterArena, c_to_c: &mut ClusterPath, src: usize, dest: usize) {
        let current_dest = shrink_jumps_and_find(c_to_c, dest);
        let current_src = shrink_jumps_and_find(c_to_c, src);
        let c0 = std::mem::replace(&mut arena[current_dest], None).unwrap_or_default();
        let c1 = std::mem::replace(&mut arena[current_src], None).unwrap_or_default();
        let merged = Cluster::merge(c0, c1);
        arena[current_dest] = Some(merged);
        c_to_c[current_src] = current_dest;
    }

    let no_linear = LinkedList::len(&constraints);
    let mut arena = ClusterArena::with_capacity(no_linear);
    let mut cluster_to_current = ClusterPath::with_capacity(no_linear);
    let mut signal_to_cluster = vec![no_linear; no_vars];
    for constraint in constraints {
        if !constraint.is_empty(){
            let signals = C::take_cloned_signals(&constraint);
            let dest = ClusterArena::len(&arena);
            ClusterArena::push(&mut arena, Some(Cluster::new(constraint, signals.len())));
            Vec::push(&mut cluster_to_current, dest);
            for signal in signals {
                let prev = signal_to_cluster[signal];
                signal_to_cluster[signal] = dest;
                if prev < no_linear {
                    arena_merge(&mut arena, &mut cluster_to_current, prev, dest);
                }
            }
        }
    }
    for (_,subcomp) in subcomponents {
        let mut i = 1;
        while i < subcomp.len() {
            let one = signal_to_cluster[subcomp[0]];
            let two = signal_to_cluster[subcomp[i]];
            if one != two {
                if one < no_linear && two < no_linear{
                    arena_merge(&mut arena, &mut cluster_to_current, 
                    one,two);
                }
                else if one == no_linear{
                    signal_to_cluster[subcomp[0]] = two;
                }
            }
            i += 1;
        }
    }

    let mut clusters = Vec::new();
    for cluster in arena {
        if let Some(cluster) = cluster {
            if Cluster::size(&cluster) != 0 {
                Vec::push(&mut clusters, cluster);
            }
        }
    }
    clusters
}