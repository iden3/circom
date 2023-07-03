use code_producers::llvm_elements::LLVMCircuitData;
use compiler::circuit_design::function::{FunctionCode, FunctionCodeInfo};
use compiler::circuit_design::template::{TemplateCode, TemplateCodeInfo};
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::{Instruction, InstructionList, InstructionPointer, new_id};


use std::cell::RefCell;

use compiler::intermediate_representation::ir_interface::{
    Allocate, AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket,
    CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, ReturnBucket,
    StoreBucket, BlockBucket, ValueBucket, AddressType, ReturnType, FinalData, LogBucketArg,
};

use crate::passes::loop_unroll::LoopUnrollPass;
use crate::passes::conditional_flattening::ConditionalFlattening;
use crate::passes::deterministic_subcomponent_invocation::DeterministicSubCmpInvokePass;
use crate::passes::simplification::SimplificationPass;
use crate::passes::mapped_to_indexed::MappedToIndexedPass;

mod conditional_flattening;
mod loop_unroll;
mod memory;
mod simplification;
mod deterministic_subcomponent_invocation;
mod mapped_to_indexed;

macro_rules! pre_hook {
    ($name: ident, $bucket_ty: ty) => {
        fn $name(&self, _bucket: &$bucket_ty) {}
    };
}

pub trait CircuitTransformationPass {
    fn transform_circuit(&self, circuit: &Circuit) -> Circuit {
        self.pre_hook_circuit(&circuit);
        let templates = circuit.templates.iter().map(|t| self.transform_template(t)).collect();
        let field_tracking = self.get_updated_field_constants();
        Circuit {
            wasm_producer: circuit.wasm_producer.clone(),
            c_producer: circuit.c_producer.clone(),
            llvm_data: circuit.llvm_data.clone_with_new_field_tracking(field_tracking),
            templates,
            functions: circuit.functions.iter().map(|f| self.transform_function(f)).collect(),
        }
    }

    fn get_updated_field_constants(&self) -> Vec<String>;

    fn transform_template(&self, template: &TemplateCode) -> TemplateCode {
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
            body: self.transform_instructions(&template.body),
            var_stack_depth: template.var_stack_depth,
            expression_stack_depth: template.expression_stack_depth,
            signal_stack_depth: template.signal_stack_depth,
            number_of_components: template.number_of_components,
        })
    }

    fn transform_function(&self, function: &FunctionCode) -> FunctionCode {
        self.pre_hook_function(function);
        Box::new(FunctionCodeInfo {
            header: function.header.clone(),
            name: function.name.clone(),
            params: function.params.clone(),
            returns: function.returns.clone(),
            body: self.transform_instructions(&function.body),
            max_number_of_vars: function.max_number_of_vars,
            max_number_of_ops_in_expression: function.max_number_of_ops_in_expression,
        })
    }

    fn transform_instructions(&self, i: &InstructionList) -> InstructionList {
        i.iter().map(|i| self.transform_instruction(i)).collect()
    }

    fn pre_hook_instruction(&self, i: &Instruction) {
        use compiler::intermediate_representation::Instruction::*;
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
            Block(b) => self.pre_hook_unrolled_loop_bucket(b),
            Nop(b) => self.pre_hook_nop_bucket(b),
        }
    }

    fn transform_instruction(&self, i: &Instruction) -> InstructionPointer {
        self.pre_hook_instruction(i);
        use compiler::intermediate_representation::Instruction::*;
        match i {
            Value(b) => self.transform_value_bucket(b),
            Load(b) => self.transform_load_bucket(b),
            Store(b) => self.transform_store_bucket(b),
            Compute(b) => self.transform_compute_bucket(b),
            Call(b) => self.transform_call_bucket(b),
            Branch(b) => self.transform_branch_bucket(b),
            Return(b) => self.transform_return_bucket(b),
            Assert(b) => self.transform_assert_bucket(b),
            Log(b) => self.transform_log_bucket(b),
            Loop(b) => self.transform_loop_bucket(b),
            CreateCmp(b) => self.transform_create_cmp_bucket(b),
            Constraint(b) => self.transform_constraint_bucket(b),
            Block(b) => self.transform_block_bucket(b),
            Nop(b) => self.transform_nop_bucket(b),
        }
    }

    // This macros both define the interface of each bucket method and
    // the default behaviour which is to just copy the bucket without modifying it
    fn transform_value_bucket(&self, bucket: &ValueBucket) -> InstructionPointer {
        ValueBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            parse_as: bucket.parse_as,
            op_aux_no: bucket.op_aux_no,
            value: bucket.value,
        }.allocate()
    }

    fn transform_address_type(&self, address: &AddressType) -> AddressType {
        match address {
            AddressType::SubcmpSignal {
                cmp_address,
                uniform_parallel_value,
                is_output,
                input_information,
            } => AddressType::SubcmpSignal {
                cmp_address: self.transform_instruction(cmp_address),
                uniform_parallel_value: uniform_parallel_value.clone(),
                is_output: *is_output,
                input_information: input_information.clone(),
            },
            x => x.clone(),
        }
    }

    fn transform_location_rule(&self, location_rule: &LocationRule) -> LocationRule {
        match location_rule {
            LocationRule::Indexed { location, template_header } => LocationRule::Indexed {
                location: self.transform_instruction(location),
                template_header: template_header.clone(),
            },
            LocationRule::Mapped { signal_code, indexes } => LocationRule::Mapped {
                signal_code: *signal_code,
                indexes: self.transform_instructions(indexes),
            },
        }
    }

    fn transform_load_bucket(&self, bucket: &LoadBucket) -> InstructionPointer {
        LoadBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            address_type: self.transform_address_type(&bucket.address_type),
            src: self.transform_location_rule(&bucket.src),
        }
        .allocate()
    }

    fn transform_store_bucket(&self, bucket: &StoreBucket) -> InstructionPointer {
        StoreBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            context: bucket.context.clone(),
            dest_is_output: bucket.dest_is_output,
            dest_address_type: self.transform_address_type(&bucket.dest_address_type),
            dest: self.transform_location_rule(&bucket.dest),
            src: self.transform_instruction(&bucket.src),
        }
        .allocate()
    }

    fn transform_compute_bucket(&self, bucket: &ComputeBucket) -> InstructionPointer {
        ComputeBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            op: bucket.op,
            op_aux_no: bucket.op_aux_no,
            stack: self.transform_instructions(&bucket.stack),
        }
        .allocate()
    }

    fn transform_final_data(&self, final_data: &FinalData) -> FinalData {
        FinalData {
            context: final_data.context,
            dest_is_output: final_data.dest_is_output,
            dest_address_type: self.transform_address_type(&final_data.dest_address_type),
            dest: self.transform_location_rule(&final_data.dest),
        }
    }

    fn transform_return_type(&self, return_type: &ReturnType) -> ReturnType {
        match return_type {
            ReturnType::Final(f) => ReturnType::Final(self.transform_final_data(f)),
            x => x.clone(),
        }
    }

    fn transform_call_bucket(&self, bucket: &CallBucket) -> InstructionPointer {
        CallBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            symbol: bucket.symbol.to_string(),
            argument_types: bucket.argument_types.clone(),
            arguments: self.transform_instructions(&bucket.arguments),
            arena_size: bucket.arena_size,
            return_info: self.transform_return_type(&bucket.return_info),
        }
        .allocate()
    }

    fn transform_branch_bucket(&self, bucket: &BranchBucket) -> InstructionPointer {
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

    fn transform_return_bucket(&self, bucket: &ReturnBucket) -> InstructionPointer {
        ReturnBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            with_size: bucket.with_size,
            value: self.transform_instruction(&bucket.value),
        }
        .allocate()
    }

    fn transform_assert_bucket(&self, bucket: &AssertBucket) -> InstructionPointer {
        AssertBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            evaluate: self.transform_instruction(&bucket.evaluate),
        }
        .allocate()
    }

    fn transform_log_bucket_arg(&self, args: &Vec<LogBucketArg>) -> Vec<LogBucketArg> {
        args.iter()
            .map(|arg| match arg {
                LogBucketArg::LogExp(e) => LogBucketArg::LogExp(self.transform_instruction(e)),
                x => x.clone(),
            })
            .collect()
    }

    fn transform_log_bucket(&self, bucket: &LogBucket) -> InstructionPointer {
        LogBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            argsprint: self.transform_log_bucket_arg(&bucket.argsprint),
        }
        .allocate()
    }

    fn transform_loop_bucket(&self, bucket: &LoopBucket) -> InstructionPointer {
        LoopBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            continue_condition: self.transform_instruction(&bucket.continue_condition),
            body: self.transform_instructions(&bucket.body),
        }
        .allocate()
    }

    fn transform_create_cmp_bucket(&self, bucket: &CreateCmpBucket) -> InstructionPointer {
        CreateCmpBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            template_id: bucket.template_id,
            cmp_unique_id: bucket.cmp_unique_id,
            symbol: bucket.symbol.clone(),
            sub_cmp_id: self.transform_instruction(&bucket.sub_cmp_id),
            name_subcomponent: bucket.name_subcomponent.to_string(),
            defined_positions: bucket.defined_positions.clone(),
            is_part_mixed_array_not_uniform_parallel: bucket
                .is_part_mixed_array_not_uniform_parallel,
            uniform_parallel: bucket.uniform_parallel,
            dimensions: bucket.dimensions.clone(),
            signal_offset: bucket.signal_offset,
            signal_offset_jump: bucket.signal_offset_jump,
            component_offset: bucket.component_offset,
            component_offset_jump: bucket.component_offset_jump,
            number_of_cmp: bucket.number_of_cmp,
            has_inputs: bucket.has_inputs,
        }
        .allocate()
    }

    fn transform_constraint_bucket(&self, bucket: &ConstraintBucket) -> InstructionPointer {
        match bucket {
            ConstraintBucket::Substitution(i) => {
                ConstraintBucket::Substitution(self.transform_instruction(i))
            }
            ConstraintBucket::Equality(i) => {
                ConstraintBucket::Equality(self.transform_instruction(i))
            }
        }
        .allocate()
    }

    fn transform_block_bucket(&self, bucket: &BlockBucket) -> InstructionPointer {
        BlockBucket {
            id: new_id(),
            line: bucket.line,
            message_id: bucket.message_id,
            body: self.transform_instructions(&bucket.body),
        }
        .allocate()
    }

    fn transform_nop_bucket(&self, _bucket: &NopBucket) -> InstructionPointer {
        NopBucket {
            id: new_id()
        }.allocate()
    }

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
    pre_hook!(pre_hook_unrolled_loop_bucket, BlockBucket);
    pre_hook!(pre_hook_nop_bucket, NopBucket);
}

pub type Passes = RefCell<Vec<Box<dyn CircuitTransformationPass>>>;

pub struct PassManager {
    passes: Passes,
}

impl PassManager {
    pub fn new() -> Self {
        PassManager { passes: Default::default() }
    }

    pub fn schedule_loop_unroll_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(LoopUnrollPass::new(prime)));
        self
    }

    pub fn schedule_simplification_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(SimplificationPass::new(prime)));
        self
    }

    pub fn schedule_conditional_flattening_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(ConditionalFlattening::new(prime)));
        self
    }

    pub fn schedule_deterministic_subcmp_invoke_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(DeterministicSubCmpInvokePass::new(prime)));
        self
    }

    pub fn schedule_mapped_to_indexed_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(MappedToIndexedPass::new(prime)));
        self
    }

    pub fn transform_circuit(&self, circuit: Circuit) -> Circuit {
        let mut transformed_circuit = circuit;
        for pass in self.passes.borrow().iter() {
            transformed_circuit = pass.transform_circuit(&transformed_circuit);
        }
        transformed_circuit
    }
}
