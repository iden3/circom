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
mod block_bucket;
mod nop_bucket;

pub mod ir_interface;
pub mod translate;

use rand::Rng;
pub use ir_interface::{Instruction, InstructionList, InstructionPointer};

pub type BucketId = u128;

pub fn new_id() -> BucketId {
    let mut rng = rand::thread_rng();
    rng.gen()
}
