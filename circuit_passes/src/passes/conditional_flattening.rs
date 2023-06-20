use std::cell::RefCell;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::ir_interface::{
    AssertBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket,
    LogBucket, LoopBucket, ReturnBucket, StoreBucket, ValueBucket,
};
use crate::CircuitTransformationPass;
use crate::passes::memory::PassMemory;

pub struct ConditionalFlattening {
    memory: RefCell<PassMemory>,
}

impl CircuitTransformationPass for ConditionalFlattening {
    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().interpreter.constant_fields.clone()
    }

    fn pre_hook_circuit(&self, circuit: &Circuit) {
        for template in &circuit.templates {
            self.memory.borrow_mut().add_template(template);
        }
        for function in &circuit.functions {
            self.memory.borrow_mut().add_function(function);
        }
        self.memory.borrow_mut().interpreter.constant_fields =
            circuit.llvm_data.field_tracking.clone();
    }

    /// Reset the interpreter when we are about to enter a new template
    fn pre_hook_template(&self, template: &TemplateCode) {
        eprintln!("Starting analysis of {}", template.header);
        self.memory.borrow_mut().interpreter.reset();
    }

    fn pre_hook_store_bucket(&self, bucket: &StoreBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_store_bucket(bucket);
    }

    fn pre_hook_value_bucket(&self, bucket: &ValueBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_value_bucket(bucket);
    }

    fn pre_hook_load_bucket(&self, bucket: &LoadBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_load_bucket(bucket);
    }

    fn pre_hook_compute_bucket(&self, bucket: &ComputeBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_compute_bucket(bucket);
    }

    fn pre_hook_call_bucket(&self, bucket: &CallBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_call_bucket(bucket);
    }

    fn pre_hook_loop_bucket(&self, bucket: &LoopBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_loop_bucket(bucket);
    }

    fn pre_hook_return_bucket(&self, bucket: &ReturnBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_return_bucket(bucket);
    }

    fn pre_hook_assert_bucket(&self, bucket: &AssertBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_assert_bucket(bucket);
    }

    fn pre_hook_log_bucket(&self, bucket: &LogBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_log_bucket(bucket);
    }

    fn pre_hook_create_cmp_bucket(&self, bucket: &CreateCmpBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_create_cmp_bucket(bucket);
    }

    fn pre_hook_constraint_bucket(&self, bucket: &ConstraintBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_constraint_bucket(bucket);
    }
}
