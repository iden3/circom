pub struct ConstraintJSON {
    pub data: Vec<u8>, // TODO: Make sure this always get written to FS
    constraints_flag: bool,
}

impl ConstraintJSON {
    pub fn new() -> ConstraintJSON {
        let mut data = Vec::<u8>::new();

        data.extend_from_slice(b"{");
        data.extend_from_slice(b"\n\"constraints\": [");

        ConstraintJSON { data, constraints_flag: false }
    }
    pub fn write_constraint(&mut self, constraint: &str) {
        if !self.constraints_flag {
            self.constraints_flag = true;
            self.data.extend_from_slice(b"\n");
        } else {
            self.data.extend_from_slice(b",\n");
        }
        self.data.extend_from_slice(constraint.as_bytes());
    }
    pub fn end(&mut self) {
        self.data.extend_from_slice(b"\n]\n}");
    }
}

pub struct SignalsJSON {
    data: Vec<u8>, // TODO: Make sure this always get written to FS
}
impl SignalsJSON {
    pub fn new() -> Self {
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(b"{");
        data.extend_from_slice(b"\n\"signalName2Idx\": {");
        data.extend_from_slice(b"\n\"one\" : \"0\"");
        Self { data }
    }
    pub fn write_correspondence(&mut self, signal: String, data: String) {
        self.data
            .extend_from_slice(format!(",\n\"{}\" : {}", signal, data).as_bytes());
    }
    pub fn end(mut self) {
        self.data.extend_from_slice(b"\n}\n}");
    }
}

pub struct SubstitutionJSON {
    pub data: Vec<u8>, // TODO: Make sure this always get written to FS
    first: bool,
}
impl SubstitutionJSON {
    pub fn new() -> Self {
        let first = true;
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(b"{");
        Self { data, first }
    }
    pub fn write_substitution(&mut self, signal: &str, substitution: &str) {
        if self.first {
            self.first = false;
            self.data.extend_from_slice(b"\n");
        } else {
            self.data.extend_from_slice(b",\n");
        }
        let substitution = format!("\"{}\" : {}", signal, substitution);
        self.data.extend_from_slice(substitution.as_bytes());
    }
    pub fn end(&mut self) {
        self.data.extend_from_slice(b"\n}");
    }
}
