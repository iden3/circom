use super::{ConstraintList, C, EncodingIterator, SignalMap};
use constraint_writers::r1cs_writer::{ConstraintSection, CustomGatesAppliedData, HeaderData, R1CSWriter, SignalSection};

pub fn port_r1cs(list: &ConstraintList, output: &str) -> Result<(), ()> {
    use constraint_writers::log_writer::Log;
    let field_size = (list.field.bits() / 64 + 1) * 8;
    let mut log = Log::new();
    log.no_labels = ConstraintList::no_labels(list);
    log.no_wires = ConstraintList::no_wires(list);
    log.no_private_inputs = list.no_private_inputs;
    log.no_public_inputs = list.no_public_inputs;
    log.no_public_outputs = list.no_public_outputs;

    let r1cs = R1CSWriter::new(output.to_string(), field_size)?;
    let mut constraint_section = R1CSWriter::start_constraints_section(r1cs)?;
    let mut written = 0;

    for c_id in list.constraints.get_ids() {
        let c = list.constraints.read_constraint(c_id).unwrap();
        let c = C::apply_correspondence(&c, &list.signal_map);
        ConstraintSection::write_constraint_usize(&mut constraint_section, c.a(), c.b(), c.c())?;
        if C::is_linear(&c) {
            log.no_linear += 1;
        } else {
            log.no_non_linear += 1;
        }
        written += 1;
    }

    let r1cs = constraint_section.end_section()?;
    let mut header_section = R1CSWriter::start_header_section(r1cs)?;
    let header_data = HeaderData {
        field: list.field.clone(),
        public_outputs: list.no_public_outputs,
        public_inputs: list.no_public_inputs,
        private_inputs: list.no_private_inputs,
        total_wires: ConstraintList::no_wires(list),
        number_of_labels: ConstraintList::no_labels(list),
        number_of_constraints: written,
    };
    header_section.write_section(header_data)?;
    let r1cs = header_section.end_section()?;
    let mut signal_section = R1CSWriter::start_signal_section(r1cs)?;

    for id in list.get_witness_as_vec() {
        SignalSection::write_signal_usize(&mut signal_section, id)?;
    }
    let r1cs = signal_section.end_section()?;

    let mut custom_gates_used_section = R1CSWriter::start_custom_gates_used_section(r1cs)?;
    let (usage_data, occurring_order) = {
        let mut usage_data = vec![];
        let mut occurring_order = vec![];
        for node in &list.dag_encoding.nodes {
            if node.is_custom_gate {
                let mut name = node.name.clone();
                occurring_order.push(name.clone());
                while name.pop() != Some('(') {};
                usage_data.push((name, node.parameters.clone()));
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
            let mut application_data_mut = vec![];
            for (custom_gate_name, constraints) in application_data {
                let mut index = 0;
                while occurring_order[index] != custom_gate_name {
                    index += 1;
                }
                application_data_mut.push((index, constraints));
            }
            application_data_mut
        }

        fn iterate(
            iterator: EncodingIterator,
            map: &SignalMap,
            application_data: &mut Vec<(String, Vec<usize>)>
        ) {
            let dag = iterator.encoding;
            let mut index = 0;
            for edge in EncodingIterator::edges(&iterator) {
                let node = &dag.nodes[edge.goes_to];
                if node.is_custom_gate {
                    let custom_gate_constraints = iterator.custom_gates_constraints[index].clone();
                    if let Some(constraints) = custom_gate_constraints {
                        let mut constraints_mut = vec![];
                        for constraint in constraints {
                            if let Some(constraint_mapped) = map.get(&constraint) {
                                constraints_mut.push(*constraint_mapped);
                            } else {
                                unreachable!();
                            }
                        }
                        application_data.push((node.name.clone(), constraints_mut));
                    } else {
                        unreachable!();
                    }
                } else {
                    let next = EncodingIterator::next(&iterator, edge);
                    iterate(next, map, application_data);
                }
                index += 1;
            }
        }

        let mut application_data = vec![];
        let iterator = EncodingIterator::new(&list.dag_encoding);
        iterate(iterator, &list.signal_map, &mut application_data);
        find_indexes(occurring_order, application_data)
    };
    custom_gates_applied_section.write_custom_gates_applications(application_data)?;
    let _r1cs = custom_gates_applied_section.end_section()?;

    Log::print(&log);
    Ok(())
}
