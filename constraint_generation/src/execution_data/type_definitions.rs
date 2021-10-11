use super::AExpressionSlice;
use super::Constraint as ConstraintGen;
use std::collections::HashMap;

pub type NodePointer = usize;
pub type Constraint = ConstraintGen<String>;
pub type ParameterContext = HashMap<String, AExpressionSlice>;
pub type SignalCollector = Vec<(String, Vec<usize>)>;
pub type ComponentCollector = Vec<(String, Vec<usize>)>;
pub struct SubComponentData {
    pub name: String,
    pub indexed_with: Vec<usize>,
    pub goes_to: NodePointer,
}
