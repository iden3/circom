use super::{Tree, DAG};

pub fn produce_witness(dag: &DAG) -> Vec<usize> {
    let mut witness = vec![0];
    let tree = Tree::new(dag);
    produce_tree_witness(&tree, &mut witness);
    Vec::shrink_to_fit(&mut witness);
    witness
}

fn produce_tree_witness(tree: &Tree, witness: &mut Vec<usize>) {
    for signal in &tree.signals {
        Vec::push(witness, *signal);
    }
    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        produce_tree_witness(&subtree, witness);
    }
}
