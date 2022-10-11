use super::AExpressionSlice;
use super::Constraint as ConstraintGen;
use std::collections::BTreeMap;
use num_bigint_dig::BigInt;


pub type NodePointer = usize;
pub type Constraint = ConstraintGen<String>;
pub type ParameterContext = BTreeMap<String, AExpressionSlice>;
pub type TagContext = BTreeMap<String, TagInfo>;
pub type TagInfo = BTreeMap<String, Option<BigInt>>;
// From name to dimensions
pub type SignalCollector = Vec<(String, Vec<usize>)>;
pub type ComponentCollector = Vec<(String, Vec<usize>)>;
pub struct SubComponentData {
    pub name: String,
    pub is_parallel: bool,
    pub indexed_with: Vec<usize>,
    pub goes_to: NodePointer,
}
