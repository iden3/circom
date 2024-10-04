use std::collections::{HashSet, HashMap};


pub type TagInfo = HashSet<String>;
pub type WireInfo = HashMap<String, WireData>;
pub type WireDeclarationOrder = Vec<(String, usize)>;


#[derive(Clone, PartialEq, Eq)]
pub enum WireType {
    Signal,
    Bus(String),
}


#[derive(Clone)]
pub struct WireData {
    wire_type: WireType,
    dimension: usize,
    tag_info: TagInfo,
}

impl WireData {
    pub fn new(
        wire_type: WireType,
        dimension: usize,
        tag_info: TagInfo,
    ) -> WireData {
        WireData {
            wire_type,
            dimension,
            tag_info
        }
    }
    pub fn get_type(&self) -> WireType {
        self.wire_type.clone()
    }
    pub fn get_dimension(&self) -> usize {
        self.dimension
    }
    pub fn contains_tag(&self, name: &str) -> bool {
        self.tag_info.contains(name)
    }
    pub fn get_tags(&self) -> &TagInfo {
        &self.tag_info
    }
}