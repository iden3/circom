#[allow(dead_code)]
mod circuit_design;
mod intermediate_representation;
mod ir_processing;
pub extern crate num_bigint_dig as num_bigint;
pub extern crate num_traits;

pub mod compiler_interface;
pub mod hir;
mod translating_traits;
