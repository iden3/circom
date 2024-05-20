use circom_algebra::num_bigint::BigInt;
use std::collections::HashMap;

const SECTIONS: u8 = 5;
const MAGIC: &[u8] = b"r1cs";
const VERSION: &[u8] = &[1, 0, 0, 0];
const HEADER_TYPE: &[u8] = &[1, 0, 0, 0];
const CONSTRAINT_TYPE: &[u8] = &[2, 0, 0, 0];
const WIRE2LABEL_TYPE: &[u8] = &[3, 0, 0, 0];
const CUSTOM_GATES_USED_TYPE: &[u8] = &[4, 0, 0, 0];
const CUSTOM_GATES_APPLIED_TYPE: &[u8] = &[5, 0, 0, 0];
const PLACE_HOLDER: &[u8] = &[3, 3, 3, 3, 3, 3, 3, 3];

fn into_format(number: &[u8], with_bytes: usize) -> (Vec<u8>, usize) {
    let mut value = number.to_vec();
    while value.len() < with_bytes {
        value.push(0);
    }
    let size = value.len();
    (value, size)
}

fn bigint_as_bytes(number: &BigInt, with_bytes: usize) -> (Vec<u8>, usize) {
    let (_, value) = number.to_bytes_le();
    into_format(&value, with_bytes)
}

fn initialize_section(data: &mut Vec<u8>, header: &[u8]) -> usize {
    data.extend_from_slice(header);
    let go_back = data.len();
    data.extend_from_slice(PLACE_HOLDER);

    go_back
}

fn end_section(data: &mut Vec<u8>, go_back: usize, size: usize) {
    let (stream, _) = bigint_as_bytes(&BigInt::from(size), 8);

    for (i, b) in stream.iter().enumerate() {
        data[go_back + i] = *b;
    }
}

fn obtain_linear_combination_block<T>(
    linear_combination: &HashMap<T, BigInt>,
    field_size: usize,
) -> (Vec<u8>, usize) where T: AsRef<[u8]> + std::cmp::Ord + std::hash::Hash {
    let mut block = Vec::new();
    let non_zero_factors = BigInt::from(linear_combination.len());
    let mut size = 0;
    let (stream, bytes) = bigint_as_bytes(&non_zero_factors, 4);
    size += bytes;
    block.extend_from_slice(&stream);
    let mut order: Vec<&T> = linear_combination.keys().collect();
    order.sort();
    for i in order {
        let (id, factor) = linear_combination.get_key_value(i).unwrap();
        let (stream, bytes) = into_format(id.as_ref(), 4);
        size += bytes;
        block.extend_from_slice(&stream);

        let (stream, bytes) = bigint_as_bytes(factor, field_size);
        size += bytes;
        block.extend_from_slice(&stream);
    }
    (block, size)
}

fn write_constraint<T>(
    data: &mut Vec<u8>,
    a: &HashMap<T, BigInt>,
    b: &HashMap<T, BigInt>,
    c: &HashMap<T, BigInt>,
    field_size: usize,
) -> usize where T: AsRef<[u8]> + std::cmp::Ord + std::hash::Hash {
    let (block_a, size_a) = obtain_linear_combination_block(a, field_size);
    let (block_b, size_b) = obtain_linear_combination_block(b, field_size);
    let (block_c, size_c) = obtain_linear_combination_block(c, field_size);

    data.extend_from_slice(&block_a);
    data.extend_from_slice(&block_b);
    data.extend_from_slice(&block_c);
    
    size_a + size_b + size_c
}

fn initialize_file(data: &mut Vec<u8>, num_sections: u8) {
    data.extend_from_slice(MAGIC);
    data.extend_from_slice(VERSION);
    data.extend_from_slice(&[num_sections, 0, 0, 0]);
}

pub struct R1CSWriter {
    field_size: usize,
    pub data: Vec<u8>,
    sections: [bool; SECTIONS as usize]
}

pub struct HeaderSection {
    data: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct ConstraintSection {
    data: Vec<u8>,
    number_of_constraints: usize,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct SignalSection {
    data: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct CustomGatesUsedSection {
    data: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct CustomGatesAppliedSection {
    data: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

impl R1CSWriter {
    pub fn new(
        field_size: usize,
        custom_gates: bool
    ) -> R1CSWriter {
        let sections = [false; SECTIONS as usize];
        let num_sections: u8 = if custom_gates { 5 } else { 3 };
        let mut data = Vec::<u8>::new();
        initialize_file(&mut data, num_sections);
        R1CSWriter { data, sections, field_size }
    }

    pub fn start_header_section(r1cs: R1CSWriter) -> HeaderSection {
        let R1CSWriter { field_size, mut data, sections } = r1cs;

        let start = initialize_section(&mut data, HEADER_TYPE);
        
        HeaderSection {
            data,
            go_back: start,
            size: 0,
            index: 0,
            field_size,
            sections,
        }
    }

    pub fn start_constraints_section(r1cs: R1CSWriter) -> ConstraintSection {
        let R1CSWriter { field_size, mut data, sections } = r1cs;
        let start = initialize_section(&mut data, CONSTRAINT_TYPE);
        
        ConstraintSection {
            number_of_constraints: 0,
            data,
            go_back: start,
            size: 0,
            index: 1,
            field_size,
            sections,
        }
    }

    pub fn start_signal_section(r1cs: R1CSWriter) -> SignalSection {
        let R1CSWriter { field_size, mut data, sections } = r1cs;
        let start = initialize_section(&mut data, WIRE2LABEL_TYPE);
        
        SignalSection {
            data,
            go_back: start,
            size: 0,
            index: 2,
            field_size,
            sections,
        }
    }

    pub fn start_custom_gates_used_section(r1cs: R1CSWriter) -> CustomGatesUsedSection {
        let R1CSWriter { field_size, mut data, sections } = r1cs;
        let start = initialize_section(&mut data, CUSTOM_GATES_USED_TYPE);

        CustomGatesUsedSection {
            data,
            go_back: start,
            size: 0,
            index: 3,
            field_size,
            sections
        }
    }

    pub fn start_custom_gates_applied_section(r1cs: R1CSWriter) -> CustomGatesAppliedSection {
        let R1CSWriter { field_size, mut data, sections } = r1cs;
        let start = initialize_section(&mut data, CUSTOM_GATES_APPLIED_TYPE);

        CustomGatesAppliedSection {
            data,
            go_back: start,
            size: 0,
            index: 4,
            field_size,
            sections
        }
    }
}

pub struct HeaderData {
    pub field: BigInt,
    pub total_wires: usize,
    pub public_outputs: usize,
    pub public_inputs: usize,
    pub private_inputs: usize,
    pub number_of_labels: usize,
    pub number_of_constraints: usize,
}

impl HeaderSection {
    pub fn write_section(&mut self, data: HeaderData) {
        let (field_stream, bytes_field) = bigint_as_bytes(&data.field, self.field_size);
        let (length_stream, bytes_size) = bigint_as_bytes(&BigInt::from(self.field_size), 4);
        self.data.extend_from_slice(&length_stream);
        self.data.extend_from_slice(&field_stream);
        self.size += bytes_field + bytes_size;

        let data_stream = [
            [data.total_wires, 4],
            [data.public_outputs, 4],
            [data.public_inputs, 4],
            [data.private_inputs, 4],
            [data.number_of_labels, 8],
            [data.number_of_constraints, 4],
        ];
        for data in &data_stream {
            let (stream, size) = bigint_as_bytes(&BigInt::from(data[0]), data[1]);
            self.size += size;
            self.data.extend_from_slice(&stream);
        }
    }

    pub fn end_section(self) -> R1CSWriter {
        let HeaderSection {
            mut data,
            go_back,
            size,
            index,
            field_size,
            mut sections,
        } = self;

        end_section(&mut data, go_back, size);
        sections[index] = true;

        R1CSWriter { data, field_size, sections }
    }
}

type Constraint = HashMap<usize, BigInt>;
impl ConstraintSection {
    pub fn write_constraint_usize(
        &mut self,
        a: &Constraint,
        b: &Constraint,
        c: &Constraint,
    ) {
        let field_size = self.field_size;
        let mut r1cs_a = HashMap::new();
        for (k, v) in a {
            let (_, bytes) = BigInt::from(*k).to_bytes_le();
            r1cs_a.insert(bytes, v.clone());
        }
        let mut r1cs_b = HashMap::new();
        for (k, v) in b {
            let (_, bytes) = BigInt::from(*k).to_bytes_le();
            r1cs_b.insert(bytes, v.clone());
        }
        let mut r1cs_c = HashMap::new();
        for (k, v) in c {
            let (_, bytes) = BigInt::from(*k).to_bytes_le();
            r1cs_c.insert(bytes, v.clone());
        }
        let size = write_constraint(&mut self.data, &r1cs_a, &r1cs_b, &r1cs_c, field_size);
        self.size += size;
        self.number_of_constraints += 1;
    }

    pub fn end_section(self) -> R1CSWriter {
        let ConstraintSection {
            mut data,
            number_of_constraints: _,
            go_back,
            size,
            index,
            field_size,
            mut sections,
        } = self;

        end_section(&mut data, go_back, size);
        sections[index] = true;

        R1CSWriter {
            data,
            field_size,
            sections
        }
    }

    pub fn constraints_written(&self) -> usize {
        self.number_of_constraints
    }
}

impl SignalSection {
    pub fn write_signal<T>(
        &mut self,
        bytes: &T
    ) -> () where T: AsRef<[u8]> {
        let (bytes, size) = into_format(bytes.as_ref(), 8);
        self.size += size;
        self.data.extend_from_slice(&bytes);
    }

    pub fn write_signal_usize(&mut self, signal: usize) {
        let (_, as_bytes) = BigInt::from(signal).to_bytes_le();
        SignalSection::write_signal(self, &as_bytes)
    }

    pub fn end_section(self) -> R1CSWriter {
        let SignalSection {
            mut data,
            go_back,
            size,
            index,
            field_size,
            mut sections,
        } = self;

        end_section(&mut data, go_back, size);
        sections[index] = true;

        R1CSWriter {
            data,
            field_size,
            sections,
        }
    }
}

pub type CustomGatesUsedData = Vec<(String, Vec<BigInt>)>;
impl CustomGatesUsedSection {
    pub fn write_custom_gates_usages(&mut self, data: CustomGatesUsedData) {
        let no_custom_gates = data.len();
        let (no_custom_gates_stream, no_custom_gates_size) =
            bigint_as_bytes(&BigInt::from(no_custom_gates), 4);
        self.size += no_custom_gates_size;
        self.data.extend_from_slice(&no_custom_gates_stream);

        for custom_gate in data {
            let custom_gate_name = custom_gate.0;
            let custom_gate_name_stream = custom_gate_name.as_bytes();
            self.size += custom_gate_name_stream.len() + 1;
            self.data.extend_from_slice(custom_gate_name_stream);
            self.data.extend_from_slice(&[0]);

            let custom_gate_parameters = custom_gate.1;
            let no_custom_gate_parameters = custom_gate_parameters.len();
            let (no_custom_gate_parameters_stream, no_custom_gate_parameters_size) =
                bigint_as_bytes(&BigInt::from(no_custom_gate_parameters), 4);
            self.size += no_custom_gate_parameters_size;
            self.data.extend_from_slice(&no_custom_gate_parameters_stream);

            for parameter in custom_gate_parameters {
                let (parameter_stream, parameter_size) = bigint_as_bytes(&parameter, self.field_size);
                self.size += parameter_size;
                self.data.extend_from_slice(&parameter_stream);
            }
        }
    }

    pub fn end_section(self) -> R1CSWriter {
        let CustomGatesUsedSection {
            mut data,
            go_back,
            size,
            index,
            field_size,
            mut sections,
        } = self;

        end_section(&mut data, go_back, size);
        sections[index] = true;

        R1CSWriter {
            data,
            field_size,
            sections,
        }
    }
}

pub type CustomGatesAppliedData = Vec<(usize, Vec<usize>)>;
impl CustomGatesAppliedSection {
    pub fn write_custom_gates_applications(&mut self, data: CustomGatesAppliedData) {
        let no_custom_gate_applications = data.len();
        let (no_custom_gate_applications_stream, no_custom_gate_applications_size) =
            bigint_as_bytes(&BigInt::from(no_custom_gate_applications), 4);
        self.size += no_custom_gate_applications_size;
        self.data.extend_from_slice(&no_custom_gate_applications_stream);

        for custom_gate_application in data {
            let custom_gate_index = custom_gate_application.0;
            let (custom_gate_index_stream, custom_gate_index_size) =
                bigint_as_bytes(&BigInt::from(custom_gate_index), 4);
            self.size += custom_gate_index_size;
            self.data.extend_from_slice(&custom_gate_index_stream);

            let custom_gate_signals = custom_gate_application.1;
            let no_custom_gate_signals = custom_gate_signals.len();
            let (no_custom_gate_signals_stream, no_custom_gate_signals_size) =
                bigint_as_bytes(&BigInt::from(no_custom_gate_signals), 4);
            self.size += no_custom_gate_signals_size;
            self.data.extend_from_slice(&no_custom_gate_signals_stream);

            for signal in custom_gate_signals {
                let (signal_stream, signal_size) = bigint_as_bytes(&BigInt::from(signal), 8);
                self.size += signal_size;
                self.data.extend_from_slice(&signal_stream);
            }
        }
    }

    pub fn end_section(self) -> R1CSWriter {
        let CustomGatesAppliedSection {
            mut data,
            go_back,
            size,
            index,
            field_size,
            mut sections,
        } = self;

        end_section(&mut data, go_back, size);
        sections[index] = true;

        R1CSWriter {
            data,
            field_size,
            sections,
        }
    }
}
