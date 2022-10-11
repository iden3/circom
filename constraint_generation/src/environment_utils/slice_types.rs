pub use super::component_representation::ComponentRepresentation;
pub use super::memory_slice::MemorySlice;
pub use super::memory_slice::{MemoryError, SliceCapacity};
pub use circom_algebra::algebra::ArithmeticExpression;
pub use num_bigint::BigInt;
use std::collections::BTreeMap;

pub type TagInfo = BTreeMap<String, Option<BigInt>>;
pub type AExpressionSlice = MemorySlice<ArithmeticExpression<String>>;
// The boolean is true if the signal contains a value
pub type SignalSlice = MemorySlice<bool>;
pub type ComponentSlice = MemorySlice<ComponentRepresentation>;
