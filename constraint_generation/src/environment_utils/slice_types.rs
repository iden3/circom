pub use super::component_representation::ComponentRepresentation;
pub use super::bus_representation::BusRepresentation;
pub use super::memory_slice::MemorySlice;
pub use super::memory_slice::{MemoryError, TypeInvalidAccess, TypeAssignmentError, SliceCapacity};
pub use circom_algebra::algebra::ArithmeticExpression;
pub use num_bigint::BigInt;
use std::collections::BTreeMap;
use program_structure::ast::Meta;

#[derive(Debug, Copy, Clone)]
pub struct TagState{
    pub defined: bool, // if it appears in the definition of the signal
    pub value_defined: bool, // if the value is given by the user
}
pub type TagInfo = BTreeMap<String, Option<BigInt>>;
pub type TagDefinitions = BTreeMap<String, TagState>; // the tags defined for each signal and if the info about their state

#[derive(Clone)]
pub struct SignalTagInfo{
    pub tags: TagInfo,
    pub definitions: TagDefinitions,
    pub remaining_inserts: usize,
    pub is_init: bool,
}

#[derive(Clone)]
pub struct BusTagInfo{
    pub tags: TagInfo,
    pub definitions: TagDefinitions,
    pub remaining_inserts: usize, // indicates the number of remaining inserts to be complete
    pub size: usize, // the size of the array generating the bus
    pub is_init: bool, // to check if the bus has been initialized or not (no valid tag declarations if init)
    pub fields: BTreeMap<String, BusTagInfo>,
}

#[derive(Clone)]
pub enum AssignmentState {
    Assigned(Option<Meta>), // location of the assignment
    MightAssigned(
        Vec<(usize, bool)>,
        Option<Meta> // location of the assignment
    ), // the number of the conditional and if it is true/false
    NoAssigned
}


pub type AExpressionSlice = MemorySlice<ArithmeticExpression<String>>;
// The boolean is true if the signal contains a value
pub type SignalSlice = MemorySlice<AssignmentState>;
pub type ComponentSlice = MemorySlice<ComponentRepresentation>;

// To store the buses, similar to the components
pub type BusSlice = MemorySlice<BusRepresentation>;

// To store the fields of a bus
#[derive(Clone)]
pub enum FieldTypes { // For each field, we store the info depending on if it is a signal o a bus
                    // Depending on the case we store a different slice
    Signal(SignalSlice),
    Bus(BusSlice),
}

#[derive(Clone)]
pub enum FoldedResult { // For each possible returning value, we store the info depending on if it is a signal o a bus
    // Depending on the case we store a different slice
    Signal(SignalSlice),
    Bus(BusSlice),
}


pub enum FoldedArgument<'a> { // For each possible argument, we store the info depending on if it is a signal o a bus
    // Depending on the case we store a different slice
    Signal(&'a Vec<usize>),
    Bus(&'a BusSlice),
}