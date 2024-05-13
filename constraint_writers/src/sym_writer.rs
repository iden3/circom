pub struct SymElem {
    pub original: i64,
    pub witness: i64,
    pub node_id: i64,
    pub symbol: String,
}
impl ToString for SymElem {
    fn to_string(&self) -> String {
        format!("{},{},{},{}", self.original, self.witness, self.node_id, self.symbol)
    }
}

pub struct SymFile {
    pub data: Vec<u8>,
}

impl SymFile {
    pub fn new() -> SymFile {
        SymFile { data: vec![] }
    }

    pub fn write_sym_elem(sym: &mut SymFile, elem: SymElem) {
        sym.data.extend_from_slice(elem.to_string().as_bytes());
        sym.data.extend_from_slice(b"\n");
    }
}
