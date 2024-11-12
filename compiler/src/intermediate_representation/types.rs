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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SizeOption{
    Single(usize),
    Multiple(Vec<(usize, usize)>) // The first value indicates the cmp_id, the second the size
}

#[derive(Clone, PartialEq, Eq)]
pub struct InstrContext {
    pub size: SizeOption,
}
