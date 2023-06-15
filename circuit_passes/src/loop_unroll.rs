use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::Instruction::UnrolledLoop;
use program_structure::ast::Statement::While;
use crate::CircuitTransformationPass;
use compiler::intermediate_representation::ir_interface::{AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LogBucket, LoopBucket, ReturnBucket, StoreBucket, UnrolledLoopBucket, ValueBucket};
use compiler::intermediate_representation::InstructionPointer;
use constraint_generation::execute::{execute_conditional_statement, RuntimeInformation};
use compiler::intermediate_representation::ir_interface::Allocate;
use program_structure::program_archive::ProgramArchive;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::env::{Env, FunctionsLibrary, TemplatesLibrary};

pub struct PassMemory {
    pub templates_library: TemplatesLibrary,
    pub functions_library: FunctionsLibrary,
    pub interpreter: BucketInterpreter
}

impl PassMemory {
    pub fn add_template(&mut self, template: &TemplateCode) {
        self.templates_library.borrow_mut().insert(template.header.clone(), (*template).clone());
    }

    pub fn add_function(&mut self, function: &FunctionCode) {
        self.functions_library.borrow_mut().insert(function.header.clone(), (*function).clone());
    }
}

pub struct LoopUnrollPass {
    // Wrapped in a RefCell because the reference to the static analysis is immutable but we need mutability
    memory: RefCell<PassMemory>
}

impl LoopUnrollPass {
    pub fn new(prime: &String) -> Self {
        let cl: TemplatesLibrary = Default::default();
        let fl: FunctionsLibrary = Default::default();
        LoopUnrollPass {
            memory: RefCell::new(PassMemory {
                templates_library: cl.clone(),
                functions_library: fl.clone(),
                interpreter: BucketInterpreter::init(Env::new(cl, fl), prime, vec![])
            })
        }
    }
}

impl CircuitTransformationPass for LoopUnrollPass {
    fn pre_hook_circuit(&self, circuit: &Circuit) {
        for template in &circuit.templates {
            self.memory.borrow_mut().add_template(template);
        }
        for function in &circuit.functions {
            self.memory.borrow_mut().add_function(function);
        }
        self.memory.borrow_mut().interpreter.constant_fields = circuit.llvm_data.field_tracking.clone();
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

    fn pre_hook_branch_bucket(&self, bucket: &BranchBucket) {
        eprintln!("[PRE HOOK] Executing {}", bucket.to_string());
        self.memory.borrow().interpreter.execute_branch_bucket(bucket);
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

    fn run_on_loop_bucket(&self, bucket: &LoopBucket) -> InstructionPointer {
        println!("[RUN ON] Executing {}", bucket.to_string());
        let loop_iterations = {
            let mut interpreter = &mut self.memory.borrow_mut().interpreter;

            interpreter.push_env(); // Save the environment
            // First we run the loop once. If the result is None that means that the condition is unknown
            let (_, cond_result) = interpreter.execute_loop_bucket_once(bucket);
            interpreter.pop_env(); // Restore back to before running the loop

            interpreter.push_env(); // Save the environment for running the loop for extracting the number of iterations
            if cond_result.is_none() {
                // We just run the loop and return the bucket as is.
                interpreter.execute_loop_bucket(bucket);

                return bucket.clone().allocate();
            }

            // If we got something, either true or false we run the loop many times and construct the
            // a UnrolledLoopBucket that contains the unrolled loop
            let mut loop_iterations = vec![];
            let mut cond_result = Some(true);
            while cond_result.unwrap() {
                let (_, new_cond) = interpreter.execute_loop_bucket_once(bucket);
                cond_result = new_cond;
                if let Some(true) = new_cond {
                    loop_iterations.push(bucket.body.clone());
                }
            }
            interpreter.pop_env(); // Restore back
            loop_iterations
        };
        UnrolledLoopBucket {
            original_loop: bucket.clone().allocate(),
            line: bucket.line,
            message_id: bucket.message_id,
            // We run the analysis recursively on each iteration of the loop in order
            // This replicates running the loop but without checking the condition
            body: loop_iterations.iter().map(|body| self.run_on_instructions(body)).collect(),
        }.allocate()
    }

    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().interpreter.constant_fields.clone()
    }
}