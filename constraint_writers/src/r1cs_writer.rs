use circom_algebra::num_bigint::BigInt;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};

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

fn initialize_section(writer: &mut BufWriter<File>, header: &[u8]) -> Result<u64, ()> {
    writer.write_all(header).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    let go_back = writer.seek(SeekFrom::Current(0)).map_err(|_err| {})?;
    writer.write_all(PLACE_HOLDER).map_err(|_| {})?;
    //writer.flush().map_err(|_err| {})?;
    Result::Ok(go_back)
}

fn end_section(writer: &mut BufWriter<File>, go_back: u64, size: usize) -> Result<(), ()> {
    let go_back_1 = writer.seek(SeekFrom::Current(0)).map_err(|_err| {})?;
    writer.seek(SeekFrom::Start(go_back)).map_err(|_err| {})?;
    let (stream, _) = bigint_as_bytes(&BigInt::from(size), 8);
    writer.write_all(&stream).map_err(|_err| {})?;
    writer.seek(SeekFrom::Start(go_back_1)).map_err(|_err| {})?;
    //writer.flush().map_err(|_| {})
    Result::Ok(())
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
    file: &mut BufWriter<File>,
    a: &HashMap<T, BigInt>,
    b: &HashMap<T, BigInt>,
    c: &HashMap<T, BigInt>,
    field_size: usize,
) -> Result<usize, ()> where T: AsRef<[u8]> + std::cmp::Ord + std::hash::Hash {
    let (block_a, size_a) = obtain_linear_combination_block(a, field_size);
    let (block_b, size_b) = obtain_linear_combination_block(b, field_size);
    let (block_c, size_c) = obtain_linear_combination_block(c, field_size);
    file.write_all(&block_a).map_err(|_err| {})?;
    //file.flush().map_err(|_err| {})?;
    file.write_all(&block_b).map_err(|_err| {})?;
    //file.flush().map_err(|_err| {})?;
    file.write_all(&block_c).map_err(|_err| {})?;
    //file.flush().map_err(|_err| {})?;
    Result::Ok(size_a + size_b + size_c)
}

fn initialize_file(writer: &mut BufWriter<File>, num_sections: u8) -> Result<(), ()> {
    writer.write_all(MAGIC).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    writer.write_all(VERSION).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    writer.write_all(&[num_sections, 0, 0, 0]).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    Result::Ok(())
}

pub struct R1CSWriter {
    field_size: usize,
    writer: BufWriter<File>,
    sections: [bool; SECTIONS as usize]
}

pub struct HeaderSection {
    writer: BufWriter<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct ConstraintSection {
    writer: BufWriter<File>,
    number_of_constraints: usize,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct SignalSection {
    writer: BufWriter<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct CustomGatesUsedSection {
    writer: BufWriter<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct CustomGatesAppliedSection {
    writer: BufWriter<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

impl R1CSWriter {
    pub fn new(
        output_file: String,
        field_size: usize,
        custom_gates: bool
    ) -> Result<R1CSWriter, ()> {
        let sections = [false; SECTIONS as usize];
        let num_sections: u8 = if custom_gates { 5 } else { 3 };
        let mut writer =
            File::create(&output_file).map_err(|_err| {}).map(|f| BufWriter::new(f))?;
        initialize_file(&mut writer, num_sections)?;
        Result::Ok(R1CSWriter { writer, sections, field_size })
    }

    pub fn start_header_section(mut r1cs: R1CSWriter) -> Result<HeaderSection, ()> {
        let start = initialize_section(&mut r1cs.writer, HEADER_TYPE)?;
        Result::Ok(HeaderSection {
            writer: r1cs.writer,
            go_back: start,
            size: 0,
            index: 0,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_constraints_section(mut r1cs: R1CSWriter) -> Result<ConstraintSection, ()> {
        let start = initialize_section(&mut r1cs.writer, CONSTRAINT_TYPE)?;
        Result::Ok(ConstraintSection {
            number_of_constraints: 0,
            writer: r1cs.writer,
            go_back: start,
            size: 0,
            index: 1,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_signal_section(mut r1cs: R1CSWriter) -> Result<SignalSection, ()> {
        let start = initialize_section(&mut r1cs.writer, WIRE2LABEL_TYPE)?;
        Result::Ok(SignalSection {
            writer: r1cs.writer,
            go_back: start,
            size: 0,
            index: 2,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_custom_gates_used_section(mut r1cs: R1CSWriter) -> Result<CustomGatesUsedSection, ()> {
        let start = initialize_section(&mut r1cs.writer, CUSTOM_GATES_USED_TYPE)?;
        Result::Ok(CustomGatesUsedSection {
            writer: r1cs.writer,
            go_back: start,
            size: 0,
            index: 3,
            field_size: r1cs.field_size,
            sections: r1cs.sections
        })
    }

    pub fn start_custom_gates_applied_section(mut r1cs: R1CSWriter) -> Result<CustomGatesAppliedSection, ()> {
        let start = initialize_section(&mut r1cs.writer, CUSTOM_GATES_APPLIED_TYPE)?;
        Result::Ok(CustomGatesAppliedSection {
            writer: r1cs.writer,
            go_back: start,
            size: 0,
            index: 4,
            field_size: r1cs.field_size,
            sections: r1cs.sections
        })
    }

    pub fn finish_writing(mut r1cs: R1CSWriter) -> Result<(), ()> {
	r1cs.writer.flush().map_err(|_err| {})
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
    pub fn write_section(&mut self, data: HeaderData) -> Result<(), ()> {
        let (field_stream, bytes_field) = bigint_as_bytes(&data.field, self.field_size);
        let (length_stream, bytes_size) = bigint_as_bytes(&BigInt::from(self.field_size), 4);
        self.writer.write_all(&length_stream).map_err(|_err| {})?;
        self.writer.write_all(&field_stream).map_err(|_err| {})?;
        //self.writer.flush().map_err(|_err| {})?;
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
            self.writer.write_all(&stream).map_err(|_err| {})?;
            //self.writer.flush().map_err(|_err| {})?;
        }
        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<R1CSWriter, ()> {
        end_section(&mut self.writer, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok(R1CSWriter { writer: self.writer, field_size: self.field_size, sections })
    }
}

type Constraint = HashMap<usize, BigInt>;
impl ConstraintSection {
    pub fn write_constraint_usize(
        &mut self,
        a: &Constraint,
        b: &Constraint,
        c: &Constraint,
    ) -> Result<(), ()> {
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
        let size = write_constraint(&mut self.writer, &r1cs_a, &r1cs_b, &r1cs_c, field_size)?;
        self.size += size;
        self.number_of_constraints += 1;
        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<R1CSWriter, ()> {
        end_section(&mut self.writer, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok(R1CSWriter {
            writer: self.writer,
            field_size: self.field_size,
            sections
        })
    }

    pub fn constraints_written(&self) -> usize {
        self.number_of_constraints
    }
}

impl SignalSection {
    pub fn write_signal<T>(
        &mut self,
        bytes: &T
    ) -> Result<(), ()> where T: AsRef<[u8]> {
        let (bytes, size) = into_format(bytes.as_ref(), 8);
        self.size += size;
        self.writer.write_all(&bytes).map_err(|_err| {})//?;
        //self.writer.flush().map_err(|_err| {})
    }

    pub fn write_signal_usize(&mut self, signal: usize) -> Result<(), ()> {
        let (_, as_bytes) = BigInt::from(signal).to_bytes_le();
        SignalSection::write_signal(self, &as_bytes)
    }

    pub fn end_section(mut self) -> Result<R1CSWriter, ()> {
        end_section(&mut self.writer, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok(R1CSWriter {
            writer: self.writer,
            field_size: self.field_size,
            sections
        })
    }
}

pub type CustomGatesUsedData = Vec<(String, Vec<BigInt>)>;
impl CustomGatesUsedSection {
    pub fn write_custom_gates_usages(&mut self, data: CustomGatesUsedData) -> Result<(), ()> {
        let no_custom_gates = data.len();
        let (no_custom_gates_stream, no_custom_gates_size) =
            bigint_as_bytes(&BigInt::from(no_custom_gates), 4);
        self.size += no_custom_gates_size;
        self.writer.write_all(&no_custom_gates_stream).map_err(|_err| {})?;
        //self.writer.flush().map_err(|_err| {})?;

        for custom_gate in data {
            let custom_gate_name = custom_gate.0;
            let custom_gate_name_stream = custom_gate_name.as_bytes();
            self.size += custom_gate_name_stream.len() + 1;
            self.writer.write_all(custom_gate_name_stream).map_err(|_err| {})?;
            self.writer.write_all(&[0]).map_err(|_err| {})?;
            //self.writer.flush().map_err(|_err| {})?;

            let custom_gate_parameters = custom_gate.1;
            let no_custom_gate_parameters = custom_gate_parameters.len();
            let (no_custom_gate_parameters_stream, no_custom_gate_parameters_size) =
                bigint_as_bytes(&BigInt::from(no_custom_gate_parameters), 4);
            self.size += no_custom_gate_parameters_size;
            self.writer.write_all(&no_custom_gate_parameters_stream).map_err(|_err| {})?;
            //self.writer.flush().map_err(|_err| {})?;

            for parameter in custom_gate_parameters {
                let (parameter_stream, parameter_size) = bigint_as_bytes(&parameter, self.field_size);
                self.size += parameter_size;
                self.writer.write(&parameter_stream).map_err(|_err| {})?;
                //self.writer.flush().map_err(|_err| {})?;
            }
        }

        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<R1CSWriter, ()> {
        end_section(&mut self.writer, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok(R1CSWriter {
            writer: self.writer,
            field_size: self.field_size,
            sections
        })
    }
}

pub type CustomGatesAppliedData = Vec<(usize, Vec<usize>)>;
impl CustomGatesAppliedSection {
    pub fn write_custom_gates_applications(&mut self, data: CustomGatesAppliedData) -> Result<(), ()> {
        let no_custom_gate_applications = data.len();
        let (no_custom_gate_applications_stream, no_custom_gate_applications_size) =
            bigint_as_bytes(&BigInt::from(no_custom_gate_applications), 4);
        self.size += no_custom_gate_applications_size;
        self.writer.write_all(&no_custom_gate_applications_stream).map_err(|_err| {})?;
        //self.writer.flush().map_err(|_err| {})?;

        for custom_gate_application in data {
            let custom_gate_index = custom_gate_application.0;
            let (custom_gate_index_stream, custom_gate_index_size) =
                bigint_as_bytes(&BigInt::from(custom_gate_index), 4);
            self.size += custom_gate_index_size;
            self.writer.write_all(&custom_gate_index_stream).map_err(|_err| {})?;
            //self.writer.flush().map_err(|_err| {})?;

            let custom_gate_signals = custom_gate_application.1;
            let no_custom_gate_signals = custom_gate_signals.len();
            let (no_custom_gate_signals_stream, no_custom_gate_signals_size) =
                bigint_as_bytes(&BigInt::from(no_custom_gate_signals), 4);
            self.size += no_custom_gate_signals_size;
            self.writer.write_all(&no_custom_gate_signals_stream).map_err(|_err| {})?;
            //self.writer.flush().map_err(|_err| {})?;

            for signal in custom_gate_signals {
                let (signal_stream, signal_size) = bigint_as_bytes(&BigInt::from(signal), 8);
                self.size += signal_size;
                self.writer.write(&signal_stream).map_err(|_err| {})?;
                //self.writer.flush().map_err(|_err| {})?;
            }
        }
	//self.writer.flush().map_err(|_err| {})?;
        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<R1CSWriter, ()> {
        end_section(&mut self.writer, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok(R1CSWriter {
            writer: self.writer,
            field_size: self.field_size,
            sections
        })
    }
}
