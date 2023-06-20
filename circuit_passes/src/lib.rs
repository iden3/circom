extern crate core;

mod loop_unroll;
mod bucket_interpreter;
mod simplification;
mod conditional_flattening;
mod memory;

use std::cell::RefCell;
use code_producers::llvm_elements::LLVMCircuitData;
use crate::loop_unroll::LoopUnrollPass;
use compiler::circuit_design::function::{FunctionCode, FunctionCodeInfo};
use compiler::circuit_design::template::{TemplateCode, TemplateCodeInfo};
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::{Instruction, InstructionList};
use compiler::intermediate_representation::ir_interface::{AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LogBucket, LoopBucket, NopBucket, ReturnBucket, StoreBucket, UnrolledLoopBucket, ValueBucket};
use compiler::intermediate_representation::InstructionPointer;
use compiler::intermediate_representation::ir_interface::Allocate;
use constraint_generation::execute::RuntimeInformation;
use program_structure::program_archive::ProgramArchive;
use crate::simplification::ComputeSimplificationPass;

macro_rules! run_on_bucket {
    ($name: ident, $bucket_ty: ty) => {
        fn $name(&self, bucket: &$bucket_ty) -> InstructionPointer {
            bucket.clone().allocate()
        }
    }
}

macro_rules! pre_hook {
    ($name: ident, $bucket_ty: ty) => {
        fn $name(&self, _bucket: &$bucket_ty)  {}
    }
}

pub trait CircuitTransformationPass {
    fn run_on_circuit(&self, circuit: Circuit) -> Circuit {
        self.pre_hook_circuit(&circuit);
        let templates = circuit.templates.iter().map(|t| {
            self.run_on_template(t)
        }).collect();

        Circuit {
            wasm_producer: circuit.wasm_producer,
            c_producer: circuit.c_producer,
            llvm_data: LLVMCircuitData {
                field_tracking: self.get_updated_field_constants(),
            },
            templates,
            functions: circuit.functions.iter().map(|f| {
                self.run_on_function(f)
            }).collect(),
        }
    }

    fn get_updated_field_constants(&self) -> Vec<String>;

    fn run_on_template(&self, template: &TemplateCode) -> TemplateCode {
        self.pre_hook_template(template);
        Box::new(TemplateCodeInfo {
            id: template.id,
            header: template.header.clone(),
            name: template.name.clone(),
            is_parallel: template.is_parallel,
            is_parallel_component: template.is_parallel_component,
            is_not_parallel_component: template.is_not_parallel_component,
            has_parallel_sub_cmp: template.has_parallel_sub_cmp,
            number_of_inputs: template.number_of_inputs,
            number_of_outputs: template.number_of_outputs,
            number_of_intermediates: template.number_of_intermediates,
            body: self.run_on_instructions(&template.body),
            var_stack_depth: template.var_stack_depth,
            expression_stack_depth: template.expression_stack_depth,
            signal_stack_depth: template.signal_stack_depth,
            number_of_components: template.number_of_components,
        })
    }

    fn run_on_function(&self, function: &FunctionCode) -> FunctionCode {
        self.pre_hook_function(function);
        Box::new(FunctionCodeInfo {
            header: function.header.clone(),
            name: function.name.clone(),
            params: function.params.clone(),
            returns: function.returns.clone(),
            body: self.run_on_instructions(&function.body),
            max_number_of_vars: function.max_number_of_vars,
            max_number_of_ops_in_expression: function.max_number_of_ops_in_expression,
        })
    }

    fn run_on_instructions(&self, i: &InstructionList) -> InstructionList {
        i.iter().map(|i| {
            self.run_on_instruction(i)
        }).collect()
    }

    fn pre_hook_instruction(&self, i: &Instruction) {
        use Instruction::*;
        match i {
            Value(b) => self.pre_hook_value_bucket(b),
            Load(b) => self.pre_hook_load_bucket(b),
            Store(b) => self.pre_hook_store_bucket(b),
            Compute(b) => self.pre_hook_compute_bucket(b),
            Call(b) => self.pre_hook_call_bucket(b),
            Branch(b) => self.pre_hook_branch_bucket(b),
            Return(b) => self.pre_hook_return_bucket(b),
            Assert(b) => self.pre_hook_assert_bucket(b),
            Log(b) => self.pre_hook_log_bucket(b),
            Loop(b) => self.pre_hook_loop_bucket(b),
            CreateCmp(b) => self.pre_hook_create_cmp_bucket(b),
            Constraint(b) => self.pre_hook_constraint_bucket(b),
            UnrolledLoop(b) => self.pre_hook_unrolled_loop_bucket(b),
            Nop(b) => self.pre_hook_nop_bucket(b)
        }
    }
    
    fn run_on_instruction(&self, i: &Instruction) -> InstructionPointer {
        self.pre_hook_instruction(i);
        use Instruction::*;
        match i {
            Value(b) => self.run_on_value_bucket(b),
            Load(b) => self.run_on_load_bucket(b),
            Store(b) => self.run_on_store_bucket(b),
            Compute(b) => self.run_on_compute_bucket(b),
            Call(b) => self.run_on_call_bucket(b),
            Branch(b) => self.run_on_branch_bucket(b),
            Return(b) => self.run_on_return_bucket(b),
            Assert(b) => self.run_on_assert_bucket(b),
            Log(b) => self.run_on_log_bucket(b),
            Loop(b) => self.run_on_loop_bucket(b),
            CreateCmp(b) => self.run_on_create_cmp_bucket(b),
            Constraint(b) => self.run_on_constraint_bucket(b),
            UnrolledLoop(b) => self.run_on_unrolled_loop_bucket(b),
            Nop(b) => self.run_on_nop_bucket(b)
        }
    }

    // This macros both define the interface of each bucket method and
    // the default behaviour which is to just copy the bucket without modifying it
    run_on_bucket!(run_on_value_bucket, ValueBucket);
    run_on_bucket!(run_on_load_bucket, LoadBucket);
    run_on_bucket!(run_on_store_bucket, StoreBucket);
    run_on_bucket!(run_on_compute_bucket, ComputeBucket);
    run_on_bucket!(run_on_call_bucket, CallBucket);
    run_on_bucket!(run_on_branch_bucket, BranchBucket);
    run_on_bucket!(run_on_return_bucket, ReturnBucket);
    run_on_bucket!(run_on_assert_bucket, AssertBucket);
    run_on_bucket!(run_on_log_bucket, LogBucket);
    run_on_bucket!(run_on_loop_bucket, LoopBucket);
    run_on_bucket!(run_on_create_cmp_bucket, CreateCmpBucket);
    run_on_bucket!(run_on_constraint_bucket, ConstraintBucket);
    run_on_bucket!(run_on_unrolled_loop_bucket, UnrolledLoopBucket);
    run_on_bucket!(run_on_nop_bucket, NopBucket);

    pre_hook!(pre_hook_circuit, Circuit);
    pre_hook!(pre_hook_template, TemplateCode);
    pre_hook!(pre_hook_function, FunctionCode);

    pre_hook!(pre_hook_value_bucket, ValueBucket);
    pre_hook!(pre_hook_load_bucket, LoadBucket);
    pre_hook!(pre_hook_store_bucket, StoreBucket);
    pre_hook!(pre_hook_compute_bucket, ComputeBucket);
    pre_hook!(pre_hook_call_bucket, CallBucket);
    pre_hook!(pre_hook_branch_bucket, BranchBucket);
    pre_hook!(pre_hook_return_bucket, ReturnBucket);
    pre_hook!(pre_hook_assert_bucket, AssertBucket);
    pre_hook!(pre_hook_log_bucket, LogBucket);
    pre_hook!(pre_hook_loop_bucket, LoopBucket);
    pre_hook!(pre_hook_create_cmp_bucket, CreateCmpBucket);
    pre_hook!(pre_hook_constraint_bucket, ConstraintBucket);
    pre_hook!(pre_hook_unrolled_loop_bucket, UnrolledLoopBucket);
    pre_hook!(pre_hook_nop_bucket, NopBucket);
    
}

pub type Passes = RefCell<Vec<Box<dyn CircuitTransformationPass>>>;

pub struct PassManager {
    passes: Passes
}

impl PassManager {
    pub fn new() -> Self {
        PassManager { passes: Default::default() }
    }

    pub fn schedule_loop_unroll_pass(&self, program_archive: ProgramArchive, prime: &String) -> &Self {
        let main_file_id = program_archive.get_file_id_main();
        let _runtime = RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);
        self.passes.borrow_mut().push(Box::new(LoopUnrollPass::new(prime)));
        self
    }

    pub fn schedule_simplification_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(ComputeSimplificationPass::new(prime)));
        self
    }

    pub fn run_on_circuit(&self, circuit: Circuit) -> Circuit {
        let mut transformed_circuit = circuit;
        for pass in self.passes.borrow().iter() {
            transformed_circuit = pass.run_on_circuit(transformed_circuit);
        }
        transformed_circuit
    }
}
