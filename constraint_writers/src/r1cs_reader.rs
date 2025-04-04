use circom_algebra::num_bigint::{BigInt, Sign};
use std::collections::HashMap;
use circom_algebra::num_traits::ToPrimitive;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fmt;

const SECTIONS: u8 = 5;
const MAGIC: &[u8] = b"r1cs";
const VERSION: &[u8] = &[1, 0, 0, 0];
const HEADER_TYPE: &[u8] = &[1, 0, 0, 0];
const CONSTRAINT_TYPE: &[u8] = &[2, 0, 0, 0];
const WIRE2LABEL_TYPE: &[u8] = &[3, 0, 0, 0];
const CUSTOM_GATES_USED_TYPE: &[u8] = &[4, 0, 0, 0];
const CUSTOM_GATES_APPLIED_TYPE: &[u8] = &[5, 0, 0, 0];
//This is used only to skip the section size.
const PLACE_HOLDER: &[u8] = &[3, 3, 3, 3, 3, 3, 3, 3];


pub enum R1CSParsingError {
    InvalidMagicNumber,
    InvalidVersion,
    InvalidSectionHeader,
    InvalidSectionType,
    SectionNotPresent(String),
}

impl fmt::Display for R1CSParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            R1CSParsingError::InvalidMagicNumber => write!(f, "Invalid magic number"),
            R1CSParsingError::InvalidVersion => write!(f, "Invalid version"),
            R1CSParsingError::InvalidSectionHeader => write!(f, "Invalid section header"),
            R1CSParsingError::InvalidSectionType => write!(f, "Invalid section type"),
            R1CSParsingError::SectionNotPresent(section) => write!(f, "Section \"{}\" not present", section),
        }
    }
}

fn from_format(bytes: &[u8]) -> BigInt {
    BigInt::from_bytes_le(Sign::Plus, bytes)
}

fn read_bigint(reader: &mut BufReader<File>, size: usize) -> Result<BigInt, std::io::Error> {
    let mut buffer = vec![0; size];
    reader.read_exact(&mut buffer)?;
    Ok(from_format(&buffer))
}

fn read_section(reader: &mut BufReader<File>, header: &[u8]) -> Result<u64, std::io::Error> {
    let mut buffer = vec![0; header.len()];
    reader.read_exact(&mut buffer)?;
    if buffer != header {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, R1CSParsingError::InvalidSectionHeader.to_string()));
    }
    let go_back = reader.seek(SeekFrom::Current(0))?;
    reader.seek(SeekFrom::Current(PLACE_HOLDER.len() as i64))?;
    Ok(go_back)
}

fn end_section(reader: &mut BufReader<File>, go_back: u64) -> Result<usize, std::io::Error> {
    let go_back_1 = reader.seek(SeekFrom::Current(0))?;
    reader.seek(SeekFrom::Start(go_back))?;
    let size = read_bigint(reader, 8)?.to_usize().unwrap();
    reader.seek(SeekFrom::Start(go_back_1))?;
    Ok(size)
}

fn read_linear_combination_block<T>(
    reader: &mut BufReader<File>,
    field_size: usize,
) -> Result<HashMap<T, BigInt>, std::io::Error>
where
    T: AsRef<[u8]> + std::cmp::Ord + std::hash::Hash + From<Vec<u8>>,
{
    let mut linear_combination = HashMap::new();
    let non_zero_factors = read_bigint(reader, 4)?.to_usize().unwrap();
    for _ in 0..non_zero_factors {
        let mut id = vec![0; 4];
        reader.read_exact(&mut id)?;
        let factor = read_bigint(reader, field_size)?;
        linear_combination.insert(T::from(id), factor);
    }
    Ok(linear_combination)
}

fn read_constraint<T>(
    reader: &mut BufReader<File>,
    field_size: usize,
) -> Result<(HashMap<T, BigInt>, HashMap<T, BigInt>, HashMap<T, BigInt>), std::io::Error>
where
    T: AsRef<[u8]> + std::cmp::Ord + std::hash::Hash + From<Vec<u8>>,
{
    let a = read_linear_combination_block(reader, field_size)?;
    let b = read_linear_combination_block(reader, field_size)?;
    let c = read_linear_combination_block(reader, field_size)?;
    Ok((a, b, c))
}


pub struct R1CSReader {
    field_size: usize,
    reader: BufReader<File>,
    sections: [bool; SECTIONS as usize],
    
}

pub struct HeaderSection {
    reader: BufReader<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize],
}

pub struct HeaderData {
    pub field: BigInt,
    pub field_size: usize,
    pub total_wires: usize,
    pub public_outputs: usize,
    pub public_inputs: usize,
    pub private_inputs: usize,
    pub number_of_labels: usize,
    pub number_of_constraints: usize,
}


type Constraint = HashMap<usize, BigInt>;
type ConstraintList = Vec<(Constraint, Constraint, Constraint)>;
type SignalList = Vec<usize>;
pub struct ConstraintSection {
    reader: BufReader<File>,
    number_of_constraints: usize,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize],
}



pub struct SignalSection {
    reader: BufReader<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize],
}

pub struct CustomGatesUsedSection {
    reader: BufReader<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize],
}

pub struct CustomGatesAppliedSection {
    reader: BufReader<File>,
    go_back: u64,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize],
}

impl R1CSReader {
    pub fn new(input_file: String) -> Result<R1CSReader, std::io::Error> {
        let sections = [false; SECTIONS as usize];
        let reader = File::open(&input_file).map(BufReader::new)?;
        Ok(R1CSReader { reader, sections, field_size:0 })
    }

    pub fn start_header_section(mut r1cs: R1CSReader) -> Result<HeaderSection, std::io::Error> {
        let start = read_section(&mut r1cs.reader, HEADER_TYPE)?;
        Ok(HeaderSection {
            reader: r1cs.reader,
            go_back: start,
            size: 0,
            index: 0,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_constraints_section(mut r1cs: R1CSReader) -> Result<ConstraintSection, std::io::Error> {
        let start = read_section(&mut r1cs.reader, CONSTRAINT_TYPE)?;
        Ok(ConstraintSection {
            number_of_constraints: 0,
            reader: r1cs.reader,
            go_back: start,
            size: 0,
            index: 1,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_signal_section(mut r1cs: R1CSReader) -> Result<SignalSection, std::io::Error> {
        let start = read_section(&mut r1cs.reader, WIRE2LABEL_TYPE)?;
        Ok(SignalSection {
            reader: r1cs.reader,
            go_back: start,
            size: 0,
            index: 2,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_custom_gates_used_section(mut r1cs: R1CSReader) -> Result<CustomGatesUsedSection, std::io::Error> {
        let start = read_section(&mut r1cs.reader, CUSTOM_GATES_USED_TYPE)?;
        Ok(CustomGatesUsedSection {
            reader: r1cs.reader,
            go_back: start,
            size: 0,
            index: 3,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_custom_gates_applied_section(mut r1cs: R1CSReader) -> Result<CustomGatesAppliedSection, std::io::Error> {
        let start = read_section(&mut r1cs.reader, CUSTOM_GATES_APPLIED_TYPE)?;
        Ok(CustomGatesAppliedSection {
            reader: r1cs.reader,
            go_back: start,
            size: 0,
            index: 4,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }
}

impl HeaderSection {
    pub fn read_section(&mut self) -> Result<HeaderData, std::io::Error> {
        let field_size = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let field = read_bigint(&mut self.reader, field_size)?;
        let total_wires = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let public_outputs = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let public_inputs = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let private_inputs = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let number_of_labels = read_bigint(&mut self.reader, 8)?.to_usize().unwrap();
        let number_of_constraints = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();

        Ok(HeaderData {
            field,
            field_size,
            total_wires,
            public_outputs,
            public_inputs,
            private_inputs,
            number_of_labels,
            number_of_constraints,
        })
    }

    pub fn end_section(mut self) -> Result<R1CSReader, std::io::Error> {
        let size = end_section(&mut self.reader, self.go_back)?;
        self.size = size;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Ok(R1CSReader { reader: self.reader, field_size: self.field_size, sections })
    }
}

impl ConstraintSection {
    pub fn read_constraint_usize(&mut self) -> Result<(Constraint, Constraint, Constraint), std::io::Error> {
        let field_size = self.field_size;
        let (a, b, c) = read_constraint::<Vec<u8>>(&mut self.reader, field_size)?;
        let mut constraint_a = HashMap::new();
        for (k, v) in a {
            let id = BigInt::from_bytes_le(Sign::Plus, &k.as_ref()).to_usize().unwrap();
            constraint_a.insert(id, v);
        }
        let mut constraint_b = HashMap::new();
        for (k, v) in b {
            let id = BigInt::from_bytes_le(Sign::Plus, &k.as_ref()).to_usize().unwrap();
            constraint_b.insert(id, v);
        }
        let mut constraint_c = HashMap::new();
        for (k, v) in c {
            let id = BigInt::from_bytes_le(Sign::Plus, &k.as_ref()).to_usize().unwrap();
            constraint_c.insert(id, v);
        }
        self.number_of_constraints += 1;
        Ok((constraint_a, constraint_b, constraint_c))
    }

    pub fn end_section(mut self) -> Result<R1CSReader, std::io::Error> {
        let size = end_section(&mut self.reader, self.go_back)?;
        self.size = size;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Ok(R1CSReader {
            reader: self.reader,
            field_size: self.field_size,
            sections,
        })
    }

    pub fn constraints_read(&self) -> usize {
        self.number_of_constraints
    }
}

impl SignalSection {
    pub fn read_signal(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = vec![0; 8];
        self.reader.read_exact(&mut buffer)?;
        self.size += buffer.len();
        Ok(buffer)
    }

    pub fn read_signal_usize(&mut self) -> Result<usize, std::io::Error> {
        let signal = self.read_signal()?;
        Ok(BigInt::from_bytes_le(Sign::Plus,&signal).to_usize().unwrap())
    }

    pub fn end_section(mut self) -> Result<R1CSReader, std::io::Error> {
        let size = end_section(&mut self.reader, self.go_back)?;
        self.size = size;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Ok(R1CSReader {
            reader: self.reader,
            field_size: self.field_size,
            sections,
        })
    }
}

pub type CustomGatesUsedData = Vec<(String, Vec<BigInt>)>;
impl CustomGatesUsedSection {
    pub fn read_custom_gates_usages(&mut self) -> Result<CustomGatesUsedData, std::io::Error> {
        let no_custom_gates = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let mut data = Vec::with_capacity(no_custom_gates);

        for _ in 0..no_custom_gates {
            let mut custom_gate_name = Vec::new();
            loop {
                let mut buffer = [0; 1];
                self.reader.read_exact(&mut buffer)?;
                if buffer[0] == 0 {
                    break;
                }
                custom_gate_name.push(buffer[0]);
            }
            let custom_gate_name = String::from_utf8(custom_gate_name).unwrap();

            let no_custom_gate_parameters = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
            let mut custom_gate_parameters = Vec::with_capacity(no_custom_gate_parameters);
            for _ in 0..no_custom_gate_parameters {
                let parameter = read_bigint(&mut self.reader, self.field_size)?;
                custom_gate_parameters.push(parameter);
            }
            data.push((custom_gate_name, custom_gate_parameters));
        }

        Ok(data)
    }

    pub fn end_section(mut self) -> Result<R1CSReader, std::io::Error> {
        let size = end_section(&mut self.reader, self.go_back)?;
        self.size = size;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Ok(R1CSReader {
            reader: self.reader,
            field_size: self.field_size,
            sections,
        })
    }
}

pub type CustomGatesAppliedData = Vec<(usize, Vec<usize>)>;
impl CustomGatesAppliedSection {
    pub fn read_custom_gates_applications(&mut self) -> Result<CustomGatesAppliedData, std::io::Error> {
        let no_custom_gate_applications = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
        let mut data = Vec::with_capacity(no_custom_gate_applications);

        for _ in 0..no_custom_gate_applications {
            let custom_gate_index = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();

            let no_custom_gate_signals = read_bigint(&mut self.reader, 4)?.to_usize().unwrap();
            let mut custom_gate_signals = Vec::with_capacity(no_custom_gate_signals);
            for _ in 0..no_custom_gate_signals {
                let signal = read_bigint(&mut self.reader, 8)?.to_usize().unwrap();
                custom_gate_signals.push(signal);
            }
            data.push((custom_gate_index, custom_gate_signals));
        }

        Ok(data)
    }

    pub fn end_section(mut self) -> Result<R1CSReader, std::io::Error> {
        let size = end_section(&mut self.reader, self.go_back)?;
        self.size = size;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Ok(R1CSReader {
            reader: self.reader,
            field_size: self.field_size,
            sections,
        })
    }
}

//This struct contained all the sections
pub struct R1CSData {
    header_data: HeaderData,
    constraints: ConstraintList,
    signals: SignalList,
    custom_gates: bool,
    custom_gates_used_data: Option<CustomGatesUsedData>,
    custom_gates_applied_data: Option<CustomGatesAppliedData>,
}

impl R1CSData {
    pub fn new() -> Self {
        R1CSData {
            header_data: HeaderData {
                field: BigInt::from(0),
                field_size: 0,
                total_wires: 0,
                public_outputs: 0,
                public_inputs: 0,
                private_inputs: 0,
                number_of_labels: 0,
                number_of_constraints: 0,
            },
            custom_gates: false,
            constraints: ConstraintList::new(),
            signals: SignalList::new(),
            custom_gates_used_data: None,
            custom_gates_applied_data: None,
        }
    }
}


pub fn read_r1cs(input: &str) -> Result<R1CSData, std::io::Error> {
    let mut info = R1CSData::new();
    let mut r1cs = R1CSReader::new(input.to_string())?;

    let buffer = read_initialization(&mut r1cs)?;

    //compute the beginning of each section
    let n_sections = buffer[0] as usize;
    info.custom_gates = n_sections == 5;
    let mut current_offset = 0;
    let mut section_starts = HashMap::new();
    for _ in 0..n_sections{
        let mut buffer = vec![0; 4];
        let new_offset = r1cs.reader.seek(SeekFrom::Current(current_offset))?;
        r1cs.reader.read_exact(&mut buffer)?;
        let section_type = buffer[0];
        let section_size = read_bigint(&mut r1cs.reader, 8)?.to_usize().unwrap();
        if section_type >= 1 && section_type <= n_sections as u8 {
            section_starts.insert(section_type, new_offset);
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, R1CSParsingError::InvalidSectionType.to_string()));
         }
        current_offset = (section_size.to_u64().unwrap()) as i64;
    }

    read_sections(&mut info, r1cs, section_starts)?;
    Ok(info)
}

fn read_initialization(r1cs: &mut R1CSReader) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; MAGIC.len()];
    r1cs.reader.read_exact(&mut buffer)?;
    if buffer != MAGIC {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, R1CSParsingError::InvalidMagicNumber.to_string()));
    }
    let mut buffer = vec![0; VERSION.len()];
    r1cs.reader.read_exact(&mut buffer)?;
    if buffer != VERSION {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, R1CSParsingError::InvalidVersion.to_string()));
    }
    let mut buffer = vec![0; 4];
    r1cs.reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn read_sections(info: &mut R1CSData, mut r1cs: R1CSReader, starts: HashMap<u8, u64>) -> Result<(), std::io::Error> {
    //reading the header section
    if let Some(&start) = starts.get(&1) {
        r1cs.reader.seek(SeekFrom::Start(start))?;
        let mut header_section = R1CSReader::start_header_section(r1cs)?;
        info.header_data = header_section.read_section()?;
        r1cs = header_section.end_section()?;
        //Important: After r eading the header we know the size of the field
        r1cs.field_size = info.header_data.field_size;
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other,
             R1CSParsingError::SectionNotPresent("Header".to_string()).to_string()));
    }

    //reading the constraints section
    if let Some(&start) = starts.get(&2) {
        r1cs.reader.seek(SeekFrom::Start(start))?;
        let mut constraint_section = R1CSReader::start_constraints_section(r1cs)?;
        info.constraints.reserve(info.header_data.number_of_constraints);
        while constraint_section.constraints_read() < info.header_data.number_of_constraints {
            info.constraints.push(constraint_section.read_constraint_usize()?);
        }
        r1cs = constraint_section.end_section()?;
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other,
            R1CSParsingError::SectionNotPresent("Constraints".to_string()).to_string()));
    }
    //reading the signals section
    if let Some(&start) = starts.get(&3) {
        r1cs.reader.seek(SeekFrom::Start(start))?;
        let mut signal_section = R1CSReader::start_signal_section(r1cs)?;
        info.signals.reserve(info.header_data.total_wires);
        for _ in 0..info.header_data.total_wires {
            info.signals.push(signal_section.read_signal_usize()?);
        }
        r1cs = signal_section.end_section()?;
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other,
            R1CSParsingError::SectionNotPresent("Signals".to_string()).to_string()));
    }

    //reading the custom gates sections
    if info.custom_gates {
        //reading the Custom Gates Used Section
        if let Some(&start) = starts.get(&4) {
            r1cs.reader.seek(SeekFrom::Start(start))?;
            let mut custom_gates_used_section = R1CSReader::start_custom_gates_used_section(r1cs)?;
            info.custom_gates_used_data = Some(custom_gates_used_section.read_custom_gates_usages()?);
            r1cs = custom_gates_used_section.end_section()?;
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other,
                R1CSParsingError::SectionNotPresent("Custom Gates Used".to_string()).to_string()));
        }

        //reading the Custom Gates Applied Section
        if let Some(&start) = starts.get(&5) {
            r1cs.reader.seek(SeekFrom::Start(start))?;
            let mut custom_gates_applied_section = R1CSReader::start_custom_gates_applied_section(r1cs)?;
            info.custom_gates_applied_data = Some(custom_gates_applied_section.read_custom_gates_applications()?);
            custom_gates_applied_section.end_section()?;
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other,
                R1CSParsingError::SectionNotPresent("Custom Gates Applied".to_string()).to_string()));
        }
    }
    Ok(())
}
