use crate::circuit_design::function::{FunctionCode, FunctionCodeInfo};
use crate::circuit_design::template::{TemplateCode, TemplateCodeInfo};
use crate::compiler_interface::Circuit;
use crate::intermediate_representation::Instruction;
use crate::intermediate_representation::ir_interface::{AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LogBucket, LoopBucket, ReturnBucket, StoreBucket, ValueBucket};

macro_rules! do_nothing_by_default {
    ($name: ident, $bucket_ty: ty) => {
        fn $name(&self, bucket: &$bucket_ty) -> $bucket_ty {
            bucket.clone()
        }
    }
}

pub trait BucketTransformationPass {
    fn run_on_circuit(&self, circuit: Circuit) -> Circuit {
        Circuit {
            wasm_producer: circuit.wasm_producer,
            c_producer:circuit.c_producer,
            llvm_data: circuit.llvm_data,
            templates: circuit.templates.iter().map(|t| {
                self.run_on_template(t)
            }).collect(),
            functions: circuit.functions.iter().map(|f| {
                self.run_on_function(f)
            }).collect(),
        }
    }

    fn run_on_template(&self, template: &TemplateCode) -> TemplateCode {
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
                body: template.body.iter().map(|i| {
                    Box::new(self.run_on_instruction(i))
                }).collect(),
                var_stack_depth: template.var_stack_depth,
                expression_stack_depth: template.expression_stack_depth,
                signal_stack_depth: template.signal_stack_depth,
                number_of_components: template.number_of_components,
            })
    }

    fn run_on_function(&self, function: &FunctionCode) -> FunctionCode {
        Box::new(FunctionCodeInfo {
            header: function.header.clone(),
            name: function.name.clone(),
            params: function.params.clone(),
            returns: function.returns.clone(),
            body: function.body.iter().map(|i| {
                Box::new(self.run_on_instruction(i))
            }).collect(),
            max_number_of_vars: function.max_number_of_vars,
            max_number_of_ops_in_expression: function.max_number_of_ops_in_expression,
        })
    }

    fn run_on_instruction(&self, i: &Instruction) -> Instruction {
        use Instruction::*;
        match i {
            Value(b) => Value(self.run_on_value_bucket(b)),
            Load(b) => Load(self.run_on_load_bucket(b)),
            Store(b) => Store(self.run_on_store_bucket(b)),
            Compute(b) => Compute(self.run_on_compute_bucket(b)),
            Call(b) => Call(self.run_on_call_bucket(b)),
            Branch(b) => Branch(self.run_on_branch_bucket(b)),
            Return(b) => Return(self.run_on_return_bucket(b)),
            Assert(b) => Assert(self.run_on_assert_bucket(b)),
            Log(b) => Log(self.run_on_log_bucket(b)),
            Loop(b) => Loop(self.run_on_loop_bucket(b)),
            CreateCmp(b) => CreateCmp(self.run_on_create_cmp_bucket(b)),
            Constraint(b) => Constraint(self.run_on_constraint_bucket(b))
        }
    }

    do_nothing_by_default!(run_on_value_bucket, ValueBucket);
    do_nothing_by_default!(run_on_load_bucket, LoadBucket);
    do_nothing_by_default!(run_on_store_bucket, StoreBucket);
    do_nothing_by_default!(run_on_compute_bucket, ComputeBucket);
    do_nothing_by_default!(run_on_call_bucket, CallBucket);
    do_nothing_by_default!(run_on_branch_bucket, BranchBucket);
    do_nothing_by_default!(run_on_return_bucket, ReturnBucket);
    do_nothing_by_default!(run_on_assert_bucket, AssertBucket);
    do_nothing_by_default!(run_on_log_bucket, LogBucket);
    do_nothing_by_default!(run_on_loop_bucket, LoopBucket);
    do_nothing_by_default!(run_on_create_cmp_bucket, CreateCmpBucket);
    do_nothing_by_default!(run_on_constraint_bucket, ConstraintBucket);
}

pub type Passes = Vec<Box<dyn BucketTransformationPass>>;

pub struct PassManager {
    passes: Passes
}

impl PassManager {
    pub fn new() -> Self {
        PassManager { passes: Vec::new() }
    }

    pub fn schedule_identity_pass(&mut self) -> &Self {
        self.passes.push(Box::new(IdentityPass));
        self
    }

    pub fn run_on_circuit(&self, circuit: Circuit) -> Circuit {
        let mut transformed_circuit = circuit;
        for pass in &self.passes {
            transformed_circuit = pass.run_on_circuit(transformed_circuit);
        }
        transformed_circuit
    }
}

/// A pass that does nothing
pub struct IdentityPass;

impl BucketTransformationPass for IdentityPass {}