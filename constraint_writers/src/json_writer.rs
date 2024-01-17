use std::fs::File;
use std::io::{BufWriter, Write};

pub struct ConstraintJSON {
    writer_constraints: BufWriter<File>,
    constraints_flag: bool,
}

impl ConstraintJSON {
    pub fn new(file: &str) -> Result<ConstraintJSON, ()> {
        let file_constraints = File::create(file).map_err(|_err| {})?;
        let mut writer_constraints = BufWriter::new(file_constraints);

        writer_constraints.write_all(b"{").map_err(|_err| {})?;
        writer_constraints.flush().map_err(|_err| {})?;
        writer_constraints.write_all(b"\n\"constraints\": [").map_err(|_err| {})?;
        writer_constraints.flush().map_err(|_err| {})?;

        Result::Ok(ConstraintJSON { writer_constraints, constraints_flag: false })
    }
    pub fn write_constraint(&mut self, constraint: &str) -> Result<(), ()> {
        if !self.constraints_flag {
            self.constraints_flag = true;
            self.writer_constraints.write_all(b"\n").map_err(|_err| {})?;
            self.writer_constraints.flush().map_err(|_err| {})?;
        } else {
            self.writer_constraints.write_all(b",\n").map_err(|_err| {})?;
            self.writer_constraints.flush().map_err(|_err| {})?;
        }
        self.writer_constraints.write_all(constraint.as_bytes()).map_err(|_err| {})?;
        self.writer_constraints.flush().map_err(|_err| {})?;
        Result::Ok(())
    }
    pub fn end(mut self) -> Result<(), ()> {
        self.writer_constraints.write_all(b"\n]\n}").map_err(|_err| {})?;
        self.writer_constraints.flush().map_err(|_err| {})?;
        Result::Ok(())
    }
}

pub struct SignalsJSON {
    writer_signals: BufWriter<File>,
}
impl SignalsJSON {
    pub fn new(file: &str) -> Result<SignalsJSON, ()> {
        let file_signals = File::create(file).map_err(|_err| {})?;
        let mut writer_signals = BufWriter::new(file_signals);
        writer_signals.write_all(b"{").map_err(|_err| {})?;
        writer_signals.flush().map_err(|_err| {})?;
        writer_signals.write_all(b"\n\"signalName2Idx\": {").map_err(|_err| {})?;
        writer_signals.flush().map_err(|_err| {})?;
        writer_signals.write_all(b"\n\"one\" : \"0\"").map_err(|_err| {})?;
        writer_signals.flush().map_err(|_err| {})?;
        Result::Ok(SignalsJSON { writer_signals })
    }
    pub fn write_correspondence(&mut self, signal: String, data: String) -> Result<(), ()> {
        self.writer_signals
            .write_all(format!(",\n\"{}\" : {}", signal, data).as_bytes())
            .map_err(|_err| {})?;
        self.writer_signals.flush().map_err(|_err| {})
    }
    pub fn end(mut self) -> Result<(), ()> {
        self.writer_signals.write_all(b"\n}\n}").map_err(|_err| {})?;
        self.writer_signals.flush().map_err(|_err| {})
    }
}

pub struct SubstitutionJSON {
    writer_substitutions: BufWriter<File>,
    first: bool,
}
impl SubstitutionJSON {
    pub fn new(file: &str) -> Result<SubstitutionJSON, ()> {
        let first = true;
        let file_substitutions = File::create(file).map_err(|_err| {})?;
        let mut writer_substitutions = BufWriter::new(file_substitutions);
        writer_substitutions.write_all(b"{").map_err(|_err| {})?;
        writer_substitutions.flush().map_err(|_err| {})?;
        Result::Ok(SubstitutionJSON { writer_substitutions, first })
    }
    pub fn write_substitution(&mut self, signal: &str, substitution: &str) -> Result<(), ()> {
        if self.first {
            self.first = false;
            self.writer_substitutions.write_all(b"\n").map_err(|_err| {})?;
        } else {
            self.writer_substitutions.write_all(b",\n").map_err(|_err| {})?;
        }
        let substitution = format!("\"{}\" : {}", signal, substitution);
        self.writer_substitutions.flush().map_err(|_err| {})?;
        self.writer_substitutions.write_all(substitution.as_bytes()).map_err(|_err| {})?;
        self.writer_substitutions.flush().map_err(|_err| {})?;
        Result::Ok(())
    }
    pub fn end(mut self) -> Result<(), ()> {
        self.writer_substitutions.write_all(b"\n}").map_err(|_err| {})?;
        self.writer_substitutions.flush().map_err(|_err| {})
    }
}
