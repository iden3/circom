use std::cell::RefCell;
use std::collections::BTreeMap;
use code_producers::c_elements::TemplateInstanceIOMap;

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
        SimplificationPass { memory: PassMemory::new_cell(prime, "".to_string(), Default::default()), replacements: Default::default() }
    }
}

impl InterpreterObserver for SimplificationPass {
    fn on_value_bucket(&self, _bucket: &ValueBucket, _env: &Env) -> bool {
        true
    }

    fn on_load_bucket(&self, _bucket: &LoadBucket, _env: &Env) -> bool {
        true
    }

    fn on_store_bucket(&self, _bucket: &StoreBucket, _env: &Env) -> bool {
        true
    }

    fn on_compute_bucket(&self, bucket: &ComputeBucket, env: &Env) -> bool {
        let mem = self.memory.borrow();
        let interpreter = BucketInterpreter::init(mem.current_scope.clone(), &mem.prime, &mem.constant_fields, self, mem.io_map.clone());
        let (eval, _) = interpreter.execute_compute_bucket(bucket, env, false);
        let eval = eval.expect("Compute bucket must produce a value!");
        if !eval.is_unknown() {
            println!("{} ==> {}", bucket.to_string(), eval);
            self.replacements.borrow_mut().insert(bucket.clone(), eval);
            return false;
        }
        true
    }

    fn on_assert_bucket(&self, _bucket: &AssertBucket, _env: &Env) -> bool {
        true
    }

    fn on_loop_bucket(&self, _bucket: &LoopBucket, _env: &Env) -> bool {
        true
    }

    fn on_create_cmp_bucket(&self, _bucket: &CreateCmpBucket, _env: &Env) -> bool {
        true
    }

    fn on_constraint_bucket(&self, _bucket: &ConstraintBucket, _env: &Env) -> bool {
        true
    }

    fn on_block_bucket(&self, _bucket: &BlockBucket, _env: &Env) -> bool { true }

    fn on_nop_bucket(&self, _bucket: &NopBucket, _env: &Env) -> bool {
        true
    }

    fn on_location_rule(&self, _location_rule: &LocationRule, _env: &Env) -> bool {
        true
    }

    fn on_call_bucket(&self, _bucket: &CallBucket, _env: &Env) -> bool {
        true
    }

    fn on_branch_bucket(&self, _bucket: &BranchBucket, _env: &Env) -> bool {
        true
    }

    fn on_return_bucket(&self, _bucket: &ReturnBucket, _env: &Env) -> bool {
        true
    }

    fn on_log_bucket(&self, _bucket: &LogBucket, _env: &Env) -> bool {
        true
    }

    fn ignore_function_calls(&self) -> bool {
        true
    }

    fn ignore_subcmp_calls(&self) -> bool {
        true
    }
}

impl CircuitTransformationPass for SimplificationPass {
    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().constant_fields.clone()
    }

    fn transform_compute_bucket(&self, bucket: &ComputeBucket) -> InstructionPointer {
        if let Some(value) = self.replacements.borrow().get(&bucket) {
            println!("{} --> {}", bucket.to_string(), value);
            let constant_fields = &mut self.memory.borrow_mut().constant_fields;
            return value.to_value_bucket(constant_fields).allocate();
        }
        ComputeBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            op: bucket.op,
            op_aux_no: bucket.op_aux_no,
            stack: self.transform_instructions(&bucket.stack),
        }.allocate()
    }

    fn pre_hook_circuit(&self, circuit: &Circuit) {
        self.memory.borrow_mut().fill_from_circuit(circuit);
    }

    fn pre_hook_template(&self, template: &TemplateCode) {
        self.memory.borrow().run_template(self, template);
    }
}
