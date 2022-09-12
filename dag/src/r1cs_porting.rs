use super::{Constraint, Tree, DAG};
use constraint_writers::log_writer::Log;
use constraint_writers::r1cs_writer::{ConstraintSection, CustomGatesAppliedData, HeaderData, R1CSWriter};

pub fn write(dag: &DAG, output: &str, custom_gates: bool) -> Result<(), ()> {
    let tree = Tree::new(dag);
    let field_size = (tree.field.bits() / 64 + 1) * 8;
    let mut log = Log::new();
    let r1cs = R1CSWriter::new(output.to_string(), field_size, custom_gates)?;

    let mut constraint_section = R1CSWriter::start_constraints_section(r1cs)?;
    let wires = write_constraint_section(&mut constraint_section, &mut log, &tree)? + 1; // adding 1 to include the signal used to represent value 1 in the field (signal one)
    let labels = wires;
    let constraint_counter = constraint_section.constraints_written();
    let r1cs = constraint_section.end_section()?;

    let header_data = HeaderData {
        field: tree.field.clone(),
        total_wires: wires,
        number_of_labels: labels,
        public_outputs: dag.public_outputs(),
        public_inputs: dag.public_inputs(),
        private_inputs: dag.private_inputs(),
        number_of_constraints: constraint_counter,
    };

    log.no_public_inputs = dag.public_inputs();
    log.no_public_outputs = dag.public_outputs();
    log.no_private_inputs = dag.private_inputs();
    log.no_labels = labels;
    log.no_wires = wires;

    let mut header_section = R1CSWriter::start_header_section(r1cs)?;
    header_section.write_section(header_data)?;
    let r1cs = header_section.end_section()?;

    let mut signal_section = R1CSWriter::start_signal_section(r1cs)?;
    for signal in 0..labels {
        signal_section.write_signal_usize(signal)?;
    }
    let r1cs = signal_section.end_section()?;

    if custom_gates {
        let mut custom_gates_used_section = R1CSWriter::start_custom_gates_used_section(r1cs)?;
        let (usage_data, occurring_order) = {
            let mut usage_data = vec![];
            let mut occurring_order = vec![];
            for node in &dag.nodes {
                if node.is_custom_gate() {
                    let mut name = node.template_name.clone();
                    occurring_order.push(name.clone());
                    while name.pop() != Some('(') {};
                    usage_data.push((name, node.parameters().clone()));
                }
            }
            (usage_data, occurring_order)
        };
        custom_gates_used_section.write_custom_gates_usages(usage_data)?;
        let r1cs = custom_gates_used_section.end_section()?;

        let mut custom_gates_applied_section = R1CSWriter::start_custom_gates_applied_section(r1cs)?;
        let application_data = {
            fn find_indexes(
                occurring_order: Vec<String>,
                application_data: Vec<(String, Vec<usize>)>
            ) -> CustomGatesAppliedData {
                let mut new_application_data = vec![];
                for (custom_gate_name, signals) in application_data {
                    let mut index = 0;
                    while occurring_order[index] != custom_gate_name {
                        index += 1;
                    }
                    new_application_data.push((index, signals));
                }
                new_application_data
            }

            fn traverse_tree(tree: &Tree, application_data: &mut Vec<(String, Vec<usize>)>) {
                let node = &tree.dag.nodes[tree.node_id];
                if node.is_custom_gate() {
                    let mut signals = vec![];
                    for signal in &node.ordered_signals {
                        let signal_numbering = node.signal_correspondence.get(signal).unwrap();
                        signals.push(*signal_numbering + tree.offset);
                    }
                    application_data.push((node.template_name.clone(), signals));
                } else {
                    for edge in Tree::get_edges(tree) {
                        let subtree = Tree::go_to_subtree(tree, edge);
                        traverse_tree(&subtree, application_data);
                    }
                }
            }

            let mut application_data = vec![];
            traverse_tree(&tree, &mut application_data);
            find_indexes(occurring_order, application_data)
        };
        custom_gates_applied_section.write_custom_gates_applications(application_data)?;
        let _r1cs = custom_gates_applied_section.end_section()?;
    }

    Log::print(&log);
    Result::Ok(())
}

fn write_constraint_section(
    constraint_section: &mut ConstraintSection,
    log: &mut Log,
    tree: &Tree,
) -> Result<usize, ()> {
    let mut no_signals = tree.signals.len();
    for c in &tree.constraints {
        if Constraint::is_linear(c) {
            log.no_linear += 1;
        } else {
            log.no_non_linear += 1;
        }
        ConstraintSection::write_constraint_usize(constraint_section, c.a(), c.b(), c.c())?;
    }
    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        let subtree_signals = write_constraint_section(constraint_section, log, &subtree)?;
        no_signals += subtree_signals;
    }
    Result::Ok(no_signals)
}
