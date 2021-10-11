use super::Node;
use circom_algebra::algebra::Constraint;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use std::collections::{HashMap, HashSet};

type C = Constraint<usize>;
const UNCONSTRAINED_SIGNAL: &'static str = "Unconstrained signal.";
const UNCONSTRAINED_SIGNAL_CODE: ReportCode = ReportCode::UnconstrainedSignal;

const ONE_CONSTRAINT_INTERMEDIATE: &'static str = "One constraint intermediate:";
const ONE_CONSTRAINT_INTERMEDIATE_CODE: ReportCode = ReportCode::OneConstraintIntermediate;

const NO_OUTPUT: &'static str = "There is no output signal";
const NO_OUTPUT_CODE: ReportCode = ReportCode::NoOutputInInstance;

struct UnconstrainedSignal;
impl UnconstrainedSignal {
    pub fn new(signal: &str, template: &str) -> Report {
        let msg = format!("In template \"{}\". {} \"{}\"", template, UNCONSTRAINED_SIGNAL, signal);
        let hint = format!("Maybe use: {}*0 === 0", signal);
        let mut report = Report::warning(msg, UNCONSTRAINED_SIGNAL_CODE);
        report.add_note(hint);
        report
    }
}

struct OneConstraintIntermediate;
impl OneConstraintIntermediate {
    pub fn new(signal: &str, template: &str) -> Report {
        let msg =
            format!("In template \"{}\". {} \"{}\"", template, ONE_CONSTRAINT_INTERMEDIATE, signal);
        let hint = format!("Maybe use: {}*0 === 0", signal);
        let mut report = Report::warning(msg, ONE_CONSTRAINT_INTERMEDIATE_CODE);
        report.add_note(hint);
        report
    }
}

struct NoOutputInNode;
impl NoOutputInNode {
    pub fn new(template: &str) -> Report {
        let msg = format!("In template \"{}\". {}.", template, NO_OUTPUT);
        Report::warning(msg, NO_OUTPUT_CODE)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SignalType {
    IO,
    Intermediate,
}
struct Analysis {
    template_name: String,
    no_outputs: usize,
    // signal name, type and number of appearances
    signal_stats: Vec<(String, SignalType, usize)>,
}

fn analysis_interpretation(analysis: Analysis, result: &mut AnalysisResult) {
    let tmp_name = analysis.template_name;
    let stats = analysis.signal_stats;
    if analysis.no_outputs == 0 {
        result.warnings.push(NoOutputInNode::new(&tmp_name));
    }
    for (name, xtype, no_appearances) in stats {
        if no_appearances == 0 {
            result.warnings.push(UnconstrainedSignal::new(&name, &tmp_name));
        } else if SignalType::Intermediate == xtype && no_appearances < 2 {
            result.warnings.push(OneConstraintIntermediate::new(&name, &tmp_name));
        }
    }
}

fn visit_node(node: &mut Node) -> Analysis {
    let mut io = HashSet::new();
    for io_signal in &node.io_signals {
        io.insert(*io_signal);
    }

    let mut constraint_counter = HashMap::new();
    let mut rev_correspondence = HashMap::new();
    for (name, id) in &node.signal_correspondence {
        rev_correspondence.insert(*id, name.to_string());
        constraint_counter.insert(*id, 0);
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

    let mut signal_stats = vec![];
    for (id, appearances) in constraint_counter {
        let name = rev_correspondence.remove(&id).unwrap();
        let signal_type = if io.contains(&id) || !node.is_local_signal(id) {
            SignalType::IO
        } else {
            SignalType::Intermediate
        };
        signal_stats.push((name, signal_type, appearances));
    }
    signal_stats.sort_by(|a, b| a.0.cmp(&b.0));
    Analysis {
        template_name: node.template_name.clone(),
        no_outputs: node.outputs_length,
        signal_stats,
    }
}

pub struct AnalysisResult {
    pub errors: ReportCollection,
    pub warnings: ReportCollection,
}
pub fn analyse(nodes: &mut [Node]) -> AnalysisResult {
    let mut result = AnalysisResult { errors: vec![], warnings: vec![] };
    for node in nodes {
        let analysis = visit_node(node);
        analysis_interpretation(analysis, &mut result);
    }
    result
}
