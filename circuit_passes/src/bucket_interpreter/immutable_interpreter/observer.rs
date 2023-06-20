use compiler::intermediate_representation::{Instruction, InstructionPointer};
use compiler::intermediate_representation::ir_interface::{
    AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket,
    LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, ReturnBucket, StoreBucket,
    BlockBucket, ValueBucket,
};
use crate::bucket_interpreter::env::immutable_env::FrozenEnv;

/// Will get called everytime we are about to execute a bucket, with access to the environment
/// prior to the execution of the bucket
pub trait InterpreterObserver {
    fn on_value_bucket(&self, bucket: &ValueBucket, env: &FrozenEnv) -> bool;
    fn on_load_bucket(&self, bucket: &LoadBucket, env: &FrozenEnv) -> bool;
    fn on_store_bucket(&self, bucket: &StoreBucket, env: &FrozenEnv) -> bool;
    fn on_compute_bucket(&self, bucket: &ComputeBucket, env: &FrozenEnv) -> bool;
    fn on_assert_bucket(&self, bucket: &AssertBucket, env: &FrozenEnv) -> bool;
    fn on_loop_bucket(&self, bucket: &LoopBucket, env: &FrozenEnv) -> bool;
    fn on_create_cmp_bucket(&self, bucket: &CreateCmpBucket, env: &FrozenEnv) -> bool;
    fn on_constraint_bucket(&self, bucket: &ConstraintBucket, env: &FrozenEnv) -> bool;
    fn on_unrolled_loop_bucket(&self, bucket: &BlockBucket, env: &FrozenEnv) -> bool;
    fn on_nop_bucket(&self, bucket: &NopBucket, env: &FrozenEnv) -> bool;
    fn on_location_rule(&self, location_rule: &LocationRule, env: &FrozenEnv) -> bool;
    fn on_call_bucket(&self, bucket: &CallBucket, env: &FrozenEnv) -> bool;
    fn on_branch_bucket(&self, bucket: &BranchBucket, env: &FrozenEnv) -> bool;
    fn on_return_bucket(&self, bucket: &ReturnBucket, env: &FrozenEnv) -> bool;
    fn on_log_bucket(&self, bucket: &LogBucket, env: &FrozenEnv) -> bool;

    fn on_instruction(&self, inst: &InstructionPointer, env: &FrozenEnv) -> bool {
        match inst.as_ref() {
            Instruction::Value(bucket) => self.on_value_bucket(bucket, env),
            Instruction::Load(bucket) => self.on_load_bucket(bucket, env),
            Instruction::Store(bucket) => self.on_store_bucket(bucket, env),
            Instruction::Compute(bucket) => self.on_compute_bucket(bucket, env),
            Instruction::Call(bucket) => self.on_call_bucket(bucket, env),
            Instruction::Branch(bucket) => self.on_branch_bucket(bucket, env),
            Instruction::Return(bucket) => self.on_return_bucket(bucket, env),
            Instruction::Assert(bucket) => self.on_assert_bucket(bucket, env),
            Instruction::Log(bucket) => self.on_log_bucket(bucket, env),
            Instruction::Loop(bucket) => self.on_loop_bucket(bucket, env),
            Instruction::CreateCmp(bucket) => self.on_create_cmp_bucket(bucket, env),
            Instruction::Constraint(bucket) => self.on_constraint_bucket(bucket, env),
            Instruction::Block(bucket) => self.on_unrolled_loop_bucket(bucket, env),
            Instruction::Nop(bucket) => self.on_nop_bucket(bucket, env),
        }
    }
}
