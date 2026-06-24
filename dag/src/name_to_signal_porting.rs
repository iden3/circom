use super::{Tree, DAG};
use circom_algebra::num_traits::AsPrimitive;
use constraint_writers::name_to_signal_writer::*;
use std::collections::HashMap;

pub fn write(dag: &DAG, file_name: &str) -> Result<(), ()> {
    let tree = Tree::new(dag);
    let mut dot_sym: Name2SignalFile = Name2SignalFile::new(file_name)?;
    visit_tree(&tree, &mut dot_sym)?;
    Name2SignalFile::finish_writing(dot_sym)?;
    //SymFile::close(dot_sym);
    Ok(())
}

fn visit_tree(tree: &Tree, dot_sym: &mut Name2SignalFile) -> Result<(), ()> {
    for signal in &tree.signals {
        let name = HashMap::get(&tree.id_to_name, signal).unwrap();
        let symbol = format!("{}.{}", tree.path, name);
        let witness = signal.as_();
        let sym_elem = Name2Signal { witness, symbol };
        Name2SignalFile::write_sym_elem(dot_sym, sym_elem)?;
    }
    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        visit_tree(&subtree, dot_sym)?;
    }
    Ok(())
}
