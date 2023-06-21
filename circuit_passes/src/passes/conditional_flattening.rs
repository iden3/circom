use std::cell::RefCell;
use std::collections::BTreeMap;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::{InstructionPointer, new_id};
use compiler::intermediate_representation::ir_interface::{
    Allocate, AssertBucket, BlockBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket,
    CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, ReturnBucket,
    StoreBucket, ValueBucket,
};
use crate::bucket_interpreter::env::Env;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::observer::InterpreterObserver;
use crate::passes::CircuitTransformationPass;
use crate::passes::memory::PassMemory;

pub struct ConditionalFlattening {
    // Wrapped in a RefCell because the reference to the static analysis is immutable but we need mutability
    memory: RefCell<PassMemory>,
    replacements: RefCell<BTreeMap<BranchBucket, bool>>,
}

impl ConditionalFlattening {
    pub fn new(prime: &String) -> Self {
        ConditionalFlattening {
            memory: PassMemory::new_cell(prime),
            replacements: Default::default(),
        }
    }
}

impl InterpreterObserver for ConditionalFlattening {
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
        let mem = self.memory.borrow();
        let interpreter = BucketInterpreter::init(&mem.prime, &mem.constant_fields, self);
        let (_, cond_result, _) = interpreter.execute_conditional_bucket(
            &bucket.cond,
            &bucket.if_branch,
            &bucket.else_branch,
            env,
            false,
        );
        if cond_result.is_some() {
            self.replacements.borrow_mut().insert(bucket.clone(), cond_result.unwrap());
        }
        true
    }

    fn on_return_bucket(&self, bucket: &ReturnBucket, env: &Env) -> bool {
        true
    }

    fn on_log_bucket(&self, bucket: &LogBucket, env: &Env) -> bool {
        true
    }
}

impl CircuitTransformationPass for ConditionalFlattening {
    fn pre_hook_circuit(&self, circuit: &Circuit) {
        self.memory.borrow_mut().fill_from_circuit(circuit);
    }

    fn pre_hook_template(&self, template: &TemplateCode) {
        self.memory.borrow().run_template(self, template);
    }

    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().constant_fields.clone()
    }

    fn transform_branch_bucket(&self, bucket: &BranchBucket) -> InstructionPointer {
        if let Some(side) = self.replacements.borrow().get(&bucket) {
            let code = if *side { &bucket.if_branch } else { &bucket.else_branch };
            let block = BlockBucket {
                id: new_id(),
                line: bucket.line,
                message_id: bucket.message_id,
                body: code.clone(),
            };
            return self.transform_block_bucket(&block);
        }
        BranchBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            cond: self.transform_instruction(&bucket.cond),
            if_branch: self.transform_instructions(&bucket.if_branch),
            else_branch: self.transform_instructions(&bucket.else_branch),
        }
        .allocate()
    }
}
