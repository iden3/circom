use super::{ConstraintList, C};
use constraint_writers::r1cs_writer::{ConstraintSection, HeaderData, R1CSWriter, SignalSection};

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
    let _r1cs = signal_section.end_section()?;
    Log::print(&log);
    Ok(())
}
