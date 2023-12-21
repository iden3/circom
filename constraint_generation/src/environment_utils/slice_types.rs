pub use super::component_representation::ComponentRepresentation;
pub use super::memory_slice::MemorySlice;
pub use super::memory_slice::{MemoryError, TypeInvalidAccess, TypeAssignmentError, SliceCapacity};
pub use circom_algebra::algebra::ArithmeticExpression;
pub use num_bigint::BigInt;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone)]
pub struct TagState{
    pub defined: bool, // if it appears in the definition of the signal
    pub value_defined: bool, // if the value is given by the user
    pub complete: bool, // if the signal is completely initialized
}
pub type TagInfo = BTreeMap<String, Option<BigInt>>;
pub type TagDefinitions = BTreeMap<String, TagState>; // the tags defined for each signal and if the info about their state
pub type AExpressionSlice = MemorySlice<ArithmeticExpression<String>>;
// The boolean is true if the signal contains a value
pub type SignalSlice = MemorySlice<bool>;
pub type ComponentSlice = MemorySlice<ComponentRepresentation>;
