use super::AExpressionSlice;
use super::Constraint as ConstraintGen;
use std::collections::BTreeMap;
use num_bigint_dig::BigInt;
use std::collections::HashSet;
use std::collections::HashMap;


pub type NodePointer = usize;
pub type Constraint = ConstraintGen<String>;
pub type ParameterContext = BTreeMap<String, AExpressionSlice>;
pub type TagInfo = BTreeMap<String, Option<BigInt>>;

#[derive(Clone)]
pub struct TagNames{
    pub tag_names: HashSet<String>,
    pub fields: Option<HashMap<String, TagNames>>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct TagWire{
    pub tags: TagInfo,
    pub fields: Option<HashMap<String, TagWire>>,
}


// From name to dimensions and if it is bus or not
#[derive(Clone)]
pub struct WireData{
    pub name: String,
    pub length: Vec<usize>,
    pub is_bus: bool
}
pub type WireCollector = Vec<WireData>;
pub type ComponentCollector = Vec<(String, Vec<usize>)>;
pub struct SubComponentData {
    pub name: String,
    pub is_parallel: bool,
    pub indexed_with: Vec<usize>,
    pub goes_to: NodePointer,
}

pub struct BusData {
    pub name: String,
    pub goes_to: NodePointer,
    pub size: usize,
}

/*
    Usable representation of a series of accesses performed over a symbol representing a bus.
    AccessingInformationBus {
        pub undefined: bool ===> true if one of the index values could not be transformed into a SliceCapacity during the process,
        pub array_access: Vec<SliceCapacity> 
        pub field_access: Option<String> // may not appear
        pub remaining_access: Option<AccessingInformation>, // may not appear
    }
*/
#[derive(Clone, Debug)]
pub struct AccessingInformationBus {
    pub undefined: bool,
    pub array_access: Vec<usize>,
    pub field_access: Option<String>,
    pub remaining_access: Option<Box<AccessingInformationBus>>,
}


/*
    Usable representation of a series of accesses performed over a symbol.
    AccessingInformation {
        pub undefined: bool ===> true if one of the index values could not be transformed into a SliceCapacity during the process,
        pub before_signal: Vec<SliceCapacity>,
        pub signal_access: Option<String> ==> may not appear,
        pub after_signal: Vec<SliceCapacity>
        pub tag_access: Option<String> ==> may not appear,
    }
*/
pub struct AccessingInformation {
    pub undefined: bool,
    pub before_signal: Vec<usize>,
    pub signal_access: Option<String>,
    pub after_signal: Vec<usize>,
    pub tag_access: Option<String>
}
