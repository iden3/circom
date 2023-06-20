use std::cell::RefCell;
use std::collections::BTreeMap;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::InstructionPointer;
use compiler::intermediate_representation::ir_interface::{Allocate, AssertBucket, BlockBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, ReturnBucket, StoreBucket, ValueBucket};
use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};
use crate::bucket_interpreter::env::Env;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::observer::InterpreterObserver;
use crate::bucket_interpreter::value::Value;
use crate::passes::CircuitTransformationPass;
use crate::passes::memory::PassMemory;

pub struct SimplificationPass {
    // Wrapped in a RefCell because the reference to the static analysis is immutable but we need mutability
    memory: RefCell<PassMemory>,
    replacements: RefCell<BTreeMap<ComputeBucket, Value>>,
}

impl SimplificationPass {
    pub fn new(prime: &String) -> Self {
        SimplificationPass {
            memory: PassMemory::new_cell(prime),
            replacements: Default::default(),
        }
    }
}

impl InterpreterObserver for SimplificationPass {
    fn on_value_bucket(&self, bucket: &ValueBucket, env: &Env) -> bool {
        true
    }

    fn on_load_bucket(&self, bucket: &LoadBucket, env: &Env) -> bool {
        true
    }

    fn on_store_bucket(&self, bucket: &StoreBucket, env: &Env) -> bool {
        true
    }

    fn on_compute_bucket(&self, bucket: &ComputeBucket, env: &Env) -> bool {
        let mem = self.memory.borrow();
        let interpreter = BucketInterpreter::init(&mem.prime, &mem.constant_fields, self);
        let (eval, _) = interpreter.execute_compute_bucket(bucket, env, false);
        let eval = eval.expect("Compute bucket must produce a value!");
        if !eval.is_unknown() {
            self.replacements.borrow_mut().insert(bucket.clone(), eval);
            return false;
        }
        true
    }

    fn on_assert_bucket(&self, bucket: &AssertBucket, env: &Env) -> bool {
        true
    }

    fn on_loop_bucket(&self, bucket: &LoopBucket, env: &Env) -> bool {
        true
    }

    fn on_create_cmp_bucket(&self, bucket: &CreateCmpBucket, env: &Env) -> bool {
        true
    }

    fn on_constraint_bucket(&self, bucket: &ConstraintBucket, env: &Env) -> bool {
        true
    }

    fn on_block_bucket(&self, bucket: &BlockBucket, env: &Env) -> bool {
        true
    }

    fn on_nop_bucket(&self, bucket: &NopBucket, env: &Env) -> bool {
        true
    }

    fn on_location_rule(&self, location_rule: &LocationRule, env: &Env) -> bool {
        true
    }

    fn on_call_bucket(&self, bucket: &CallBucket, env: &Env) -> bool {
        true
    }

    fn on_branch_bucket(&self, bucket: &BranchBucket, env: &Env) -> bool {
        true
    }

    fn on_return_bucket(&self, bucket: &ReturnBucket, env: &Env) -> bool {
        true
    }

    fn on_log_bucket(&self, bucket: &LogBucket, env: &Env) -> bool {
        true
    }
}

impl CircuitTransformationPass for SimplificationPass {
    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().constant_fields.clone()
    }

    fn run_on_compute_bucket(&self, bucket: &ComputeBucket) -> InstructionPointer {
        if let Some(value) = self.replacements.borrow().get(&bucket) {
            let mut constant_fields = &mut self.memory.borrow_mut().constant_fields;
            return value.to_value_bucket(constant_fields).allocate();
        }
        bucket.clone().allocate()
    }

    fn pre_hook_circuit(&self, circuit: &Circuit) {
        self.memory.borrow_mut().fill_from_circuit(circuit);
    }

    fn pre_hook_template(&self, template: &TemplateCode) {
        self.memory.borrow().run_template(self, template);
    }
}