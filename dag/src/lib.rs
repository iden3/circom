mod constraint_correctness_analysis;
mod json_porting;
mod map_to_constraint_list;
mod r1cs_porting;
mod sym_porting;
mod witness_producer;
use circom_algebra::num_bigint::BigInt;
use constraint_list::ConstraintList;
use constraint_writers::debug_writer::DebugWriter;
use constraint_writers::ConstraintExporter;
use program_structure::constants::UsefulConstants;
use program_structure::error_definition::ReportCollection;
use std::collections::{HashMap, HashSet};
type Signal = usize;
type Constraint = circom_algebra::algebra::Constraint<usize>;
type Substitution = circom_algebra::algebra::Substitution<usize>;
type Range = std::ops::Range<usize>;

pub type FastSubAccess = HashMap<usize, Substitution>;

pub struct Tree<'a> {
    dag: &'a DAG,
    pub field: BigInt,
    pub path: String,
    pub offset: usize,
    pub node_id: usize,
    pub signals: Vec<usize>,
    pub forbidden: HashSet<usize>,
    pub id_to_name: HashMap<usize, String>,
    pub constraints: Vec<Constraint>,
}

impl<'a> Tree<'a> {
    pub fn new(dag: &DAG) -> Tree {
        let constants = UsefulConstants::new(&dag.prime);
        let field = constants.get_p().clone();
        let root = dag.get_main().unwrap();
        let node_id = dag.main_id();
        let offset = dag.get_entry().unwrap().in_number;
        let path = dag.get_entry().unwrap().label.clone();
        let constraints = root.constraints.clone();
        let mut id_to_name = HashMap::new();
        let mut signals: Vec<_> = Vec::new();
        let forbidden: HashSet<_> =
            root.forbidden_if_main.iter().cloned().map(|s| s + offset).collect();
        for (name, id) in root.correspondence() {
            if root.is_local_signal(*id) {
                Vec::push(&mut signals, *id + offset);
                HashMap::insert(&mut id_to_name, *id, name.clone());
            }
        }
        signals.sort();
        Tree { field, dag, path, offset, node_id, signals, forbidden, id_to_name, constraints }
    }

    pub fn go_to_subtree(current: &'a Tree, edge: &Edge) -> Tree<'a> {
        let field = current.field.clone();
        let dag = current.dag;
        let node_id = edge.goes_to;
        let node = &current.dag.nodes[node_id];
        let path = format!("{}.{}", current.path, edge.label);
        let offset = current.offset + edge.in_number;
        let mut id_to_name = HashMap::new();
        let forbidden = HashSet::with_capacity(0);
        let mut signals: Vec<_> = Vec::new();
        for (name, id) in node.correspondence() {
            if node.is_local_signal(*id) {
                Vec::push(&mut signals, *id + offset);
                HashMap::insert(&mut id_to_name, *id + offset, name.clone());
            }
        }
        signals.sort();
        let constraints: Vec<_> = node
            .constraints
            .iter()
            .filter(|c| !c.is_empty())
            .map(|c| Constraint::apply_offset(c, offset))
            .collect();
        Tree { field, dag, path, offset, node_id, signals, forbidden, id_to_name, constraints }
    }

    pub fn get_edges(tree: &'a Tree) -> &'a Vec<Edge> {
        &tree.dag.adjacency[tree.node_id]
    }
}

#[derive(Default)]
pub struct Edge {
    label: String,
    goes_to: usize,
    in_number: usize,
    out_number: usize,
    in_component_number: usize,
    out_component_number: usize
}
impl Edge {
    fn new_entry(id: usize) -> Edge {
        Edge { label: "main".to_string(), goes_to: id, in_number: 0, out_number: 0, in_component_number: 0, out_component_number: 0  }
    }

    pub fn get_goes_to(&self) -> usize {
        self.goes_to
    }

    pub fn get_signal_range(&self) -> Range {
        (self.in_number + 1)..(self.out_number + 1)
    }

    pub fn get_in(&self) -> usize {
        self.in_number
    }

    pub fn get_out(&self) -> usize {
        self.out_number
    }

    pub fn get_in_component(&self) -> usize {
        self.in_component_number
    }

    pub fn get_out_component(&self) -> usize {
        self.out_component_number
    }

    pub fn reach(&self, with_offset: usize) -> usize {
        with_offset + self.in_number
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }
}

#[derive(Default)]
pub struct Node {
    entry: Edge,
    template_name: String,
    parameters: Vec<BigInt>,
    number_of_signals: usize,
    number_of_components: usize,
    intermediates_length: usize,
    public_inputs_length: usize,
    inputs_length: usize,
    outputs_length: usize,
    signal_correspondence: HashMap<String, Signal>,
    ordered_signals: Vec<String>,
    locals: HashSet<usize>,
    reachables: HashSet<usize>, // locals and io of subcomponents
    forbidden_if_main: HashSet<usize>,
    io_signals: Vec<usize>,
    constraints: Vec<Constraint>,
    underscored_signals: Vec<usize>,
    is_parallel: bool,
    has_parallel_sub_cmp: bool,
    is_custom_gate: bool,
    number_of_subcomponents_indexes: usize,
}

impl Node {
    fn new(
        id: usize,
        template_name: String,
        parameters: Vec<BigInt>,
        ordered_signals: Vec<String>,
        is_parallel: bool,
        is_custom_gate: bool
    ) -> Node {
        Node {
            template_name, entry: Edge::new_entry(id),
            parameters,
            number_of_components: 1,
            ordered_signals,
            is_parallel,
            has_parallel_sub_cmp: false,
            is_custom_gate,
            forbidden_if_main: vec![0].into_iter().collect(),
            ..Node::default()
        }
    }

    fn add_input(&mut self, name: String, is_public: bool) {
        let id = self.number_of_signals + 1;
        self.io_signals.push(id);
        self.public_inputs_length += if is_public { 1 } else { 0 };
        self.signal_correspondence.insert(name, id);
        self.locals.insert(id);
        self.reachables.insert(id);
        self.number_of_signals += 1;
        self.entry.out_number += 1;
        self.inputs_length += 1;
        if is_public {
            self.forbidden_if_main.insert(id);
        }
    }

    fn add_output(&mut self, name: String) {
        let id = self.number_of_signals + 1;
        self.io_signals.push(id);
        self.signal_correspondence.insert(name, id);
        self.forbidden_if_main.insert(id);
        self.locals.insert(id);
        self.reachables.insert(id);
        self.number_of_signals += 1;
        self.entry.out_number += 1;
        self.outputs_length += 1;
    }

    fn add_intermediate(&mut self, name: String) {
        let id = self.number_of_signals + 1;
        self.signal_correspondence.insert(name, id);
        self.locals.insert(id);
        self.reachables.insert(id);
        self.number_of_signals += 1;
        self.entry.out_number += 1;
        self.intermediates_length += 1;
    }

    fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint)
    }

    fn add_underscored_signal(&mut self, signal: usize) {
        self.underscored_signals.push(signal)
    }

    fn set_number_of_subcomponents_indexes(&mut self, number_scmp: usize) {
        self.number_of_subcomponents_indexes = number_scmp
    }

    pub fn parameters(&self) -> &Vec<BigInt> {
        &self.parameters
    }

    fn is_local_signal(&self, s: usize) -> bool {
        self.locals.contains(&s)
    }

    fn is_reachable_signal(&self, s: usize) -> bool {
        self.reachables.contains(&s)
    }

    pub fn number_of_signals(&self) -> usize {
        self.number_of_signals
    }

    pub fn correspondence(&self) -> &HashMap<String, usize> {
        &self.signal_correspondence
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn io_signals(&self) -> &Vec<usize> {
        &self.io_signals
    }

    pub fn get_entry(&self) -> &Edge {
        &self.entry
    }

    pub fn number_of_public_inputs(&self) -> usize {
        self.public_inputs_length
    }

    pub fn number_of_private_inputs(&self) -> usize {
        self.inputs_length - self.public_inputs_length
    }

    pub fn number_of_inputs(&self) -> usize {
        self.inputs_length
    }

    pub fn number_of_outputs(&self) -> usize {
        self.outputs_length
    }

    pub fn number_of_intermediates(&self) -> usize {
        self.intermediates_length
    }

    pub fn has_parallel_sub_cmp(&self) -> bool {
        self.has_parallel_sub_cmp
    }

    pub fn is_custom_gate(&self) -> bool {
        self.is_custom_gate
    }

    pub fn number_of_subcomponents_indexes(&self) -> usize {
        self.number_of_subcomponents_indexes
    }
}

pub struct DAG {
    pub one_signal: usize,
    pub nodes: Vec<Node>,
    pub adjacency: Vec<Vec<Edge>>,
    pub prime: String,
}

impl ConstraintExporter for DAG {
    fn r1cs(&self, out: &str, custom_gates: bool) -> Result<(), ()> {
        DAG::generate_r1cs_output(self, out, custom_gates)
    }

    fn json_constraints(&self, writer: &DebugWriter) -> Result<(), ()> {
        DAG::generate_json_constraints(self, writer)
    }

    fn sym(&self, out: &str) -> Result<(), ()> {
        DAG::generate_sym_output(self, out)
    }
}

impl DAG {
    pub fn new(prime: &String) -> DAG {
        DAG{
            prime : prime.clone(),
            one_signal: 0,
            nodes: Vec::new(),
            adjacency: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, to: usize, label: &str, is_parallel: bool) -> Option<&Edge> {
        if to < self.main_id() {
            // create arrow
            let from = self.main_id();
            let in_num = self.nodes[from].number_of_signals;
            let in_component_num = self.nodes[from].number_of_components;
            let out_num = in_num + self.nodes[to].number_of_signals;
            let out_component_num = in_component_num + self.nodes[to].number_of_components;
            self.nodes[from].number_of_signals += self.nodes[to].number_of_signals;
            self.nodes[from].entry.out_number += self.nodes[to].number_of_signals;
            self.nodes[from].number_of_components += self.nodes[to].number_of_components;
            self.nodes[from].entry.out_component_number += self.nodes[to].number_of_components;
            self.nodes[from].has_parallel_sub_cmp |= self.nodes[to].is_parallel || is_parallel;
            let with = Edge {
                label: label.to_string(),
                goes_to: to,
                in_number: in_num,
                out_number: out_num,
                in_component_number: in_component_num,
                out_component_number: out_component_num,
            };
            // add correspondence to current node
            let mut correspondence = std::mem::take(&mut self.nodes[from].signal_correspondence);
            let mut reachables = std::mem::take(&mut self.nodes[from].reachables);
            for (signal, id) in self.nodes[to].correspondence() {
                if self.nodes[to].is_local_signal(*id) {
                    let concrete_name = format!("{}.{}", label, signal);
                    let concrete_value = with.in_number + *id;
                    correspondence.insert(concrete_name, concrete_value);
                    if *id <= self.nodes[to].inputs_length + self.nodes[to].outputs_length{
                        // in case it is an input/output signal
                        reachables.insert(concrete_value);
                    }
                }
            }
            self.nodes[from].signal_correspondence = correspondence;
            self.nodes[from].reachables = reachables;
            self.nodes[from].has_parallel_sub_cmp |= self.nodes[to].is_parallel;
            self.adjacency[from].push(with);
            self.adjacency[from].last()
        } else {
            Option::None
        }
    }

    pub fn add_node(
        &mut self,
        template_name: String,
        parameters: Vec<BigInt>,
        ordered_signals: Vec<String>,
        is_parallel: bool,
        is_custom_gate: bool
    ) -> usize {
        let id = self.nodes.len();
        self.nodes.push(
            Node::new(id, template_name, parameters, ordered_signals, is_parallel, is_custom_gate)
        );
        self.adjacency.push(vec![]);
        id
    }

    pub fn add_input(&mut self, name: String, is_public: bool) {
        if let Option::Some(node) = self.get_mut_main() {
            node.add_input(name, is_public);
        }
    }

    pub fn add_output(&mut self, name: String) {
        if let Option::Some(node) = self.get_mut_main() {
            node.add_output(name);
        }
    }

    pub fn add_intermediate(&mut self, name: String) {
        if let Option::Some(node) = self.get_mut_main() {
            node.add_intermediate(name);
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        if let Option::Some(node) = self.get_mut_main() {
            node.add_constraint(constraint);
        }
    }

    pub fn add_underscored_signal(&mut self, signal: usize) {
        if let Option::Some(node) = self.get_mut_main() {
            node.add_underscored_signal(signal);
        }
    }

    pub fn set_number_of_subcomponents_indexes(&mut self, number_scmp: usize){
        if let Option::Some(node) = self.get_mut_main() {
            node.set_number_of_subcomponents_indexes(number_scmp);
        }
    }

    pub fn get_node(&self, id: usize) -> Option<&Node> {
        if id < self.nodes.len() {
            Some(&self.nodes[id])
        } else {
            None
        }
    }

    fn raw_find_id_connexion(&self, source: usize, target: usize) -> Option<usize> {
        let cc = &self.adjacency[source];
        let mut index = 0;
        while index < cc.len() && cc[index].goes_to != target {
            index += 1;
        }
        if index == cc.len() {
            Option::None
        } else {
            Option::Some(index)
        }
    }

    pub fn get_connexion(&self, from: usize, to: usize) -> Option<&Edge> {
        let index = self.raw_find_id_connexion(from, to);
        if let Option::Some(i) = index {
            Some(&self.adjacency[from][i])
        } else {
            None
        }
    }

    pub fn get_edges(&self, node: usize) -> Option<&Vec<Edge>> {
        if node < self.nodes.len() {
            Some(&self.adjacency[node])
        } else {
            None
        }
    }

    pub fn constraint_analysis(&mut self) -> Result<ReportCollection, ReportCollection> {
        let reports = constraint_correctness_analysis::analyse(&self.nodes);
        if reports.errors.is_empty() {
            Ok(reports.warnings)
        } else {
            Err(reports.errors)
        }
    }

    pub fn clean_constraints(&mut self) {
        constraint_correctness_analysis::clean_constraints(&mut self.nodes);
    }

    pub fn generate_r1cs_output(&self, output_file: &str, custom_gates: bool) -> Result<(), ()> {
        r1cs_porting::write(self, output_file, custom_gates)
    }

    pub fn generate_sym_output(&self, output_file: &str) -> Result<(), ()> {
        sym_porting::write(self, output_file)
    }

    pub fn generate_json_constraints(&self, debug: &DebugWriter) -> Result<(), ()> {
        json_porting::port_constraints(self, debug)
    }

    pub fn produce_witness(&self) -> Vec<usize> {
        witness_producer::produce_witness(self)
    }

    fn get_mut_main(&mut self) -> Option<&mut Node> {
        self.nodes.last_mut()
    }

    pub fn get_main(&self) -> Option<&Node> {
        self.nodes.last()
    }

    pub fn main_id(&self) -> usize {
        self.nodes.len() - 1
    }

    pub fn number_of_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_entry(&self) -> Option<&Edge> {
        self.get_main().map(|v| v.get_entry())
    }

    pub fn public_inputs(&self) -> usize {
        if let Option::Some(main) = self.get_main() {
            main.number_of_public_inputs()
        } else {
            0
        }
    }

    pub fn private_inputs(&self) -> usize {
        if let Option::Some(main) = self.get_main() {
            main.number_of_private_inputs()
        } else {
            0
        }
    }

    pub fn public_outputs(&self) -> usize {
        if let Option::Some(main) = self.get_main() {
            main.number_of_outputs()
        } else {
            0
        }
    }

    pub fn map_to_list(self, flags: SimplificationFlags) -> ConstraintList {
        map_to_constraint_list::map(self, flags)
    }
}

pub struct SimplificationFlags {
    pub no_rounds: usize,
    pub flag_s: bool,
    pub parallel_flag: bool,
    pub port_substitution: bool,
    pub json_substitutions: String,
    pub flag_old_heuristics: bool,
    pub prime : String,
}
