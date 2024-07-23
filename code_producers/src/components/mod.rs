use lz_fnv::Fnv1a;
use std::collections::BTreeMap;

pub struct IODef {
    pub code: usize,
    pub offset: usize,
    pub lengths: Vec<usize>,
    pub size: usize,
    pub bus_id: Option<usize>
}

// Previously an array that contains, now struct (name, start position, size, bus_id (if any))
#[derive(Clone)]
pub struct InputInfo{
    pub name: String,
    pub dimensions: Vec<usize>,
    pub start: usize,
    pub size: usize, //full size (not only the content if array)
    pub bus_id: Option<usize>
}

#[derive(Default, Clone)]
pub struct FieldData{
    pub dimensions: Vec<usize>,
    pub size: usize, // it is only the size of the content if array
    pub offset: usize,
    pub bus_id: Option<usize>,
    pub name: String
}

pub type FieldMap = Vec<Vec<FieldData>>;

pub type InputList = Vec<InputInfo>;
pub type TemplateList = Vec<String>;
pub struct InfoParallel{
    pub name: String,
    pub is_parallel: bool,
    pub is_not_parallel: bool,
}
pub type TemplateListParallel = Vec<InfoParallel>;
pub type SignalList = Vec<usize>;
pub type InputOutputList = Vec<IODef>;
pub type TemplateInstanceIOMap = BTreeMap<usize, InputOutputList>;
pub type MessageList = Vec<String>;

pub fn hasher(value: &str) -> u64 {
    use lz_fnv::FnvHasher;
    let mut fnv_hasher: Fnv1a<u64> = Fnv1a::with_key(14695981039346656037);
    fnv_hasher.write(value.as_bytes());
    fnv_hasher.finish()
}
