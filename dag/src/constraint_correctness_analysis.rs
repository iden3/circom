use super::Node;
use circom_algebra::algebra::Constraint;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use std::collections::{HashMap, HashSet};

type C = Constraint<usize>;
const UNCONSTRAINED_SIGNAL_CODE: ReportCode = ReportCode::UnconstrainedSignal;
const UNCONSTRAINED_IOSIGNAL_CODE: ReportCode = ReportCode::UnconstrainedIOSignal;



struct UnconstrainedSignal;
impl UnconstrainedSignal {
    pub fn new(signal: &str, template: &str, examples: &Vec<String>) -> Report {
        
        if examples.len() == 1{
            let msg = format!("In template \"{}\": Local signal {} does not appear in any constraint", template, examples[0]);
            
            Report::warning(msg, UNCONSTRAINED_SIGNAL_CODE)
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
            
            Report::warning(msg, UNCONSTRAINED_IOSIGNAL_CODE)
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
    let split_components:Vec<&str> = name.split('.').collect(); // split the name of components
    let mut signal_name = "".to_string();
    for i in 0..split_components.len()-1{
        signal_name = signal_name + split_components[i] + "."; // take the index of the components
    }
    // no take the index of the array position
    let aux_last_component = split_components[split_components.len()-1].to_string();
    let split_index_last_component = 
        aux_last_component.split('[').next().unwrap(); 
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

fn visit_node(node: &Node) -> Analysis {

    let mut constraint_counter = HashMap::new();
    let mut rev_correspondence = HashMap::new();
    for (name, id) in &node.signal_correspondence {
        if node.is_reachable_signal(*id){
            rev_correspondence.insert(*id, name.to_string());
            constraint_counter.insert(*id, 0);
        }
    }
    for constraint in &node.constraints {
        let signals = constraint.take_cloned_signals();
        for signal in signals {
            let prev = constraint_counter.remove(&signal).unwrap();
            constraint_counter.insert(signal, prev + 1);
        }
    }

    for signal in &node.underscored_signals{
        let prev = constraint_counter.remove(signal).unwrap();
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
pub fn clean_constraints(nodes: &mut [Node]){
    for node in nodes{
        let length_bound = Vec::len(&node.constraints);
        let work = std::mem::replace(&mut node.constraints, Vec::with_capacity(length_bound));
        for mut constraint in work {
            C::remove_zero_value_coefficients(&mut constraint);
            if !C::is_empty(&constraint) {
                Vec::push(&mut node.constraints, constraint);
            }
        }
    }
}

pub fn analyse(nodes: &[Node]) -> AnalysisResult {
    let mut result = AnalysisResult { errors: vec![], warnings: vec![] };
    let mut visited : HashSet<String> = HashSet::new();
    for node in nodes {
        if !node.is_custom_gate() && !visited.contains(&node.template_name.clone()){
            let analysis = visit_node(node);
            let mut result2 = AnalysisResult { errors: vec![], warnings: vec![] };
            analysis_interpretation(analysis, &mut result2);    
            result.errors.append(&mut result2.errors);
            result.warnings.append(&mut result2.warnings);
            visited.insert(node.template_name.clone());
        }
    }
    result
}
