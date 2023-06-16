mod address_type;
mod assert_bucket;
mod branch_bucket;
mod call_bucket;
mod compute_bucket;
mod create_component_bucket;
mod load_bucket;
mod location_rule;
mod log_bucket;
mod loop_bucket;
mod return_bucket;
mod store_bucket;
mod types;
mod value_bucket;
mod constraint_bucket;
mod unrolled_loop_bucket;
mod nop_bucket;

pub mod ir_interface;
pub mod translate;
pub mod either;

pub use ir_interface::{Instruction, InstructionList, InstructionPointer};
