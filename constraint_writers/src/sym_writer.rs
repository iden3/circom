use std::io::{BufWriter, Write};

type VfsBufWriter = std::io::BufWriter<Box<(dyn vfs::SeekAndWrite + Send + 'static)>>;

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
    writer: VfsBufWriter,
}

impl SymFile {
    pub fn new(fs: &dyn vfs::FileSystem, file: &str) -> Result<SymFile, ()> {
        let file = fs.create_file(file).map_err(|_err| {})?;
        let writer = BufWriter::new(file);
        Result::Ok(SymFile { writer })
    }

    pub fn write_sym_elem(sym: &mut SymFile, elem: SymElem) -> Result<(), ()> {
        sym.writer.write_all(elem.to_string().as_bytes()).map_err(|_err| {})?;
        sym.writer.write_all(b"\n").map_err(|_err| {}) //?;
        //sym.writer.flush().map_err(|_err| {})
    }
    
    pub fn finish_writing(mut sym: SymFile) -> Result<(), ()> {
	sym.writer.flush().map_err(|_err| {})
    }

    // pub fn close(_sym: SymFile) {}
}
