use super::{Constraint, Edge, Node, SimplificationFlags, Tree, DAG};
use constraint_list::{ConstraintList, DAGEncoding, EncodingEdge, EncodingNode, SignalInfo, Simplifier};
use program_structure::utils::constants::UsefulConstants;
use std::collections::{HashSet, LinkedList};
#[derive(Default)]
struct CHolder {
    linear: LinkedList<Constraint>,
    equalities: LinkedList<Constraint>,
    constant_equalities: LinkedList<Constraint>,
}
#[derive(Default)]
pub struct TreeConstraints {
    constraints: LinkedList<Constraint>,
    number_inputs: usize,
    number_outputs: usize,
    signals: LinkedList<usize>,
    subcomponents: LinkedList<TreeConstraints>,
}

fn map_tree(tree: &Tree, witness: &mut Vec<usize>, c_holder: &mut CHolder, tree_constraints: &mut TreeConstraints) -> usize {
    let mut no_constraints = 0;

    for signal in &tree.signals {
        tree_constraints.signals.push_back(signal.clone());
        Vec::push(witness, *signal);
    }

    tree_constraints.number_inputs = tree.inputs_length;
    tree_constraints.number_outputs = tree.outputs_length;

    for constraint in &tree.constraints {
        if Constraint::is_constant_equality(constraint) {
            LinkedList::push_back(&mut c_holder.constant_equalities, constraint.clone());
        } else if Constraint::is_equality(constraint, &tree.field) {
            LinkedList::push_back(&mut c_holder.equalities, constraint.clone());
        } else if Constraint::is_linear(constraint) {
            LinkedList::push_back(&mut c_holder.linear, constraint.clone());
        } else {
            no_constraints += 1;
        }
        tree_constraints.constraints.push_back(constraint.clone());
    }

    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        let mut subtree_constraints = TreeConstraints::default();
        no_constraints += map_tree(&subtree, witness, c_holder, &mut subtree_constraints);
        tree_constraints.subcomponents.push_back(subtree_constraints);
    }
    no_constraints
}

fn produce_encoding(
    no_constraints: usize,
    init: usize,
    dag_nodes: Vec<Node>,
    dag_edges: Vec<Vec<Edge>>,
) -> DAGEncoding {
    let mut adjacency = Vec::new();
    let mut nodes = Vec::new();
    let mut id = 0;
    for node in dag_nodes {
        let encoded = map_node_to_encoding(id, node);
        Vec::push(&mut nodes, encoded);
        id += 1;
    }
    for edges in dag_edges {
        let mut encoded = Vec::new();
        for edge in edges {
            let new = map_edge_to_encoding(edge);
            Vec::push(&mut encoded, new);
        }
        Vec::push(&mut adjacency, encoded);
    }
    DAGEncoding { init, no_constraints, nodes, adjacency }
}

fn map_node_to_encoding(id: usize, node: Node) -> EncodingNode {
    let mut signals = Vec::new();
    let locals = node.locals;
    let mut non_linear = LinkedList::new();
    for c in node.constraints {
        if !Constraint::is_linear(&c) {
            LinkedList::push_back(&mut non_linear, c);
        }
    }
    for (name, id) in node.signal_correspondence {
        if HashSet::contains(&locals, &id) {
            let new_signal = SignalInfo { name, id };
            Vec::push(&mut signals, new_signal);
        }
    }
    signals.sort_by(|a, b| a.id.cmp(&b.id));

    EncodingNode { id, signals, non_linear }
}

fn map_edge_to_encoding(edge: Edge) -> EncodingEdge {
    EncodingEdge { goes_to: edge.goes_to, path: edge.label, offset: edge.in_number }
}

pub fn map(dag: DAG, flags: SimplificationFlags,) -> ConstraintList {
    use std::time::SystemTime;
    // println!("Start of dag to list mapping");
    let now = SystemTime::now();
    let constants = UsefulConstants::new();
    let field = constants.get_p().clone();
    let init_id = dag.main_id();
    let no_public_inputs = dag.public_inputs();
    let no_public_outputs = dag.public_outputs();
    let no_private_inputs = dag.private_inputs();
    let forbidden = dag.get_main().unwrap().forbidden_if_main.clone();
    let mut c_holder = CHolder::default();
    let mut tree_constraints = TreeConstraints::default();
    let mut signal_map = vec![0];
    let no_constraints = map_tree(&Tree::new(&dag), &mut signal_map, &mut c_holder, &mut tree_constraints);
    let max_signal = Vec::len(&signal_map);
    let name_encoding = produce_encoding(no_constraints, init_id, dag.nodes, dag.adjacency);
    let _dur = now.elapsed().unwrap().as_millis();
    // println!("End of dag to list mapping: {} ms", dur);
    print_tree_constraints(&tree_constraints);

    Simplifier {
        field,
        no_public_inputs,
        no_public_outputs,
        no_private_inputs,
        forbidden,
        max_signal,
        dag_encoding: name_encoding,
        linear: c_holder.linear,
        equalities: c_holder.equalities,
        cons_equalities: c_holder.constant_equalities,
        no_rounds: flags.no_rounds,
        flag_s: flags.flag_s,
        parallel_flag: flags.parallel_flag,
        port_substitution: flags.port_substitution,
    }
    .simplify_constraints()
}

pub fn map_tree_constraints(dag: &DAG) -> TreeConstraints {

    let mut c_holder = CHolder::default();
    let mut tree_constraints = TreeConstraints::default();
    let mut signal_map = vec![0];
    let _no_constraints = map_tree(&Tree::new(&dag), &mut signal_map, &mut c_holder, &mut tree_constraints);
    
    print_tree_constraints(&tree_constraints);
    tree_constraints
}



fn print_tree_constraints(tree: &TreeConstraints){
    println!("Mostrando nuevo nodo, sus constraints son:");
    for constraint in &tree.constraints{
        println!("  Mostrando constraint:");
        println!("    Mostrando A:");
        for (signal, value) in constraint.a(){
            println!("      Signal: {}, value: {}", signal, value.to_string());
        }
        println!("    Mostrando B:");
        for (signal, value) in constraint.b(){
            println!("      Signal: {}, value: {}", signal, value.to_string());
        }
        println!("    Mostrando C:");
        for (signal, value) in constraint.c(){
            println!("      Signal: {}, value: {}", signal, value.to_string());
        }
    }
    if tree.signals.is_empty() {
        println!("El nodo no tiene señales");
    } else{
        println!("Tiene un total de {} señales comenzando en {}", 
            tree.signals.len(), 
            tree.signals.front().unwrap()
        );
    }
    println!("Numero de inputs: {}", tree.number_inputs);
    println!("Numero de outputs: {}", tree.number_outputs);
    println!("");
    for node in &tree.subcomponents{
        print_tree_constraints(node);
    }

}