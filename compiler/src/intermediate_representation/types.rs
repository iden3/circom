#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    BigInt,
    U32,
}

impl ToString for ValueType {
    fn to_string(&self) -> String {
        match self {
            ValueType::U32 => "U32",
            ValueType::BigInt => "BigInt",
        }
        .to_string()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct InstrContext {
    pub size: usize,
}
