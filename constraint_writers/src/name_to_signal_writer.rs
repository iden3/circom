use std::fs::File;
use std::io::{BufWriter, Write};

pub struct Name2Signal {
    pub witness: i64,
    pub symbol: String,
}
impl ToString for Name2Signal {
    fn to_string(&self) -> String {
        format!(" \"{}\" : {}", self.symbol, self.witness)
    }
}

pub struct Name2SignalFile {
    writer: BufWriter<File>,
}

impl Name2SignalFile {
    pub fn new(file: &str) -> Result<Name2SignalFile, ()> {
        let file = File::create(file).map_err(|_err| {})?;
        let mut writer = BufWriter::new(file);
        writer.write_all(b"{").map_err(|_err| {})?; //?;

        Result::Ok(Name2SignalFile { writer })
    }

    pub fn write_sym_elem(sym: &mut Name2SignalFile, elem: Name2Signal) -> Result<(), ()> {
        sym.writer.write_all(elem.to_string().as_bytes()).map_err(|_err| {})?;
        sym.writer.write_all(b",\n").map_err(|_err| {}) //?;
        //sym.writer.flush().map_err(|_err| {})
    }
    
    pub fn finish_writing(mut sym: Name2SignalFile) -> Result<(), ()> {
        use std::io::SeekFrom;
        use std::io::Seek;
        sym.writer.seek(SeekFrom::Current(-2)).map_err(|_err| {})?; // to remove the last comma
        sym.writer.write_all(b"}").map_err(|_err| {})?; //?;
	    sym.writer.flush().map_err(|_err| {})
    }

    // pub fn close(_sym: SymFile) {}
}
