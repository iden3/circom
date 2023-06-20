use code_producers::llvm_elements::LLVMCircuitData;
use compiler::circuit_design::function::{FunctionCode, FunctionCodeInfo};
use compiler::circuit_design::template::{TemplateCode, TemplateCodeInfo};
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::{Instruction, InstructionList, InstructionPointer};
use constraint_generation::execute::RuntimeInformation;
use program_structure::program_archive::ProgramArchive;
use std::cell::RefCell;
use compiler::intermediate_representation::Instruction::Branch;
use crate::passes::loop_unroll::LoopUnrollPass;
use compiler::intermediate_representation::ir_interface::{Allocate, AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, ReturnBucket, StoreBucket, BlockBucket, ValueBucket, AddressType, ReturnType, FinalData, LogBucketArg};
use program_structure::ast::Expression::Call;
use crate::passes::simplification::SimplificationPass;

mod conditional_flattening;
mod loop_unroll;
mod memory;
mod simplification;

macro_rules! pre_hook {
    ($name: ident, $bucket_ty: ty) => {
        fn $name(&self, bucket: &$bucket_ty) {}
    };
}

pub trait CircuitTransformationPass {
    fn run_on_circuit(&self, circuit: &Circuit) -> Circuit {
        self.pre_hook_circuit(&circuit);
        let templates = circuit.templates.iter().map(|t| self.run_on_template(t)).collect();

        Circuit {
            wasm_producer: circuit.wasm_producer.clone(),
            c_producer: circuit.c_producer.clone(),
            llvm_data: LLVMCircuitData { field_tracking: self.get_updated_field_constants() },
            templates,
            functions: circuit.functions.iter().map(|f| self.run_on_function(f)).collect(),
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
        i.iter().map(|i| self.run_on_instruction(i)).collect()
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

    fn run_on_instruction(&self, i: &Instruction) -> InstructionPointer {
        self.pre_hook_instruction(i);
        use compiler::intermediate_representation::Instruction::*;
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
            Block(b) => self.run_on_block_bucket(b),
            Nop(b) => self.run_on_nop_bucket(b),
        }
    }

    // This macros both define the interface of each bucket method and
    // the default behaviour which is to just copy the bucket without modifying it
    fn run_on_value_bucket(&self, bucket: &ValueBucket) -> InstructionPointer {
        bucket.clone().allocate()
    }

    fn run_on_address_type(&self, address: &AddressType) -> AddressType {
        match address {
            AddressType::SubcmpSignal {
                cmp_address,
                uniform_parallel_value,
                is_output,
                input_information
            } => AddressType::SubcmpSignal {
                cmp_address: self.run_on_instruction(cmp_address),
                uniform_parallel_value: uniform_parallel_value.clone(),
                is_output: *is_output,
                input_information: input_information.clone(),
            },
            x => x.clone()
        }
    }

    fn run_on_location_rule(&self, location_rule: &LocationRule) -> LocationRule {
        match location_rule {
            LocationRule::Indexed { location, template_header } => LocationRule::Indexed {
                location: self.run_on_instruction(location),
                template_header: template_header.clone()
            },
            LocationRule::Mapped { signal_code, indexes } => LocationRule::Mapped {
                signal_code: *signal_code,
                indexes: self.run_on_instructions(indexes)
            }
        }
    }

    fn run_on_load_bucket(&self, bucket: &LoadBucket) -> InstructionPointer {
        LoadBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            address_type: self.run_on_address_type(&bucket.address_type),
            src: self.run_on_location_rule(&bucket.src),
        }.allocate()
    }

    fn run_on_store_bucket(&self, bucket: &StoreBucket) -> InstructionPointer {
        StoreBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            context: bucket.context.clone(),
            dest_is_output: bucket.dest_is_output,
            dest_address_type: self.run_on_address_type(&bucket.dest_address_type),
            dest: self.run_on_location_rule(&bucket.dest),
            src: self.run_on_instruction(&bucket.src),
        }.allocate()
    }

    fn run_on_compute_bucket(&self, bucket: &ComputeBucket) -> InstructionPointer {
        ComputeBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            op: bucket.op,
            op_aux_no: bucket.op_aux_no,
            stack: self.run_on_instructions(&bucket.stack),
        }.allocate()
    }

    fn run_on_final_data(&self, final_data: &FinalData) -> FinalData {
        FinalData {
            context: final_data.context,
            dest_is_output: final_data.dest_is_output,
            dest_address_type: self.run_on_address_type(&final_data.dest_address_type),
            dest: self.run_on_location_rule(&final_data.dest),
        }
    }

    fn run_on_return_type(&self, return_type: &ReturnType) -> ReturnType {
        match return_type {
            ReturnType::Final(f) => ReturnType::Final(self.run_on_final_data(f)),
            x => x.clone()
        }
    }

    fn run_on_call_bucket(&self, bucket: &CallBucket) -> InstructionPointer {
        CallBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            symbol: bucket.symbol.to_string(),
            argument_types: bucket.argument_types.clone(),
            arguments: self.run_on_instructions(&bucket.arguments),
            arena_size: bucket.arena_size,
            return_info: self.run_on_return_type(&bucket.return_info),
        }.allocate()
    }

    fn run_on_branch_bucket(&self, bucket: &BranchBucket) -> InstructionPointer {
        BranchBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            cond: self.run_on_instruction(&bucket.cond),
            if_branch: self.run_on_instructions(&bucket.if_branch),
            else_branch: self.run_on_instructions(&bucket.else_branch),
        }.allocate()
    }

    fn run_on_return_bucket(&self, bucket: &ReturnBucket) -> InstructionPointer {
        ReturnBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            with_size: bucket.with_size,
            value: self.run_on_instruction(&bucket.value),
        }.allocate()
    }

    fn run_on_assert_bucket(&self, bucket: &AssertBucket) -> InstructionPointer {
        AssertBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            evaluate: self.run_on_instruction(&bucket.evaluate),
        }.allocate()
    }

    fn run_on_log_bucket_arg(&self, args: &Vec<LogBucketArg>) -> Vec<LogBucketArg> {
        args.iter().map(|arg| {
            match arg {
                LogBucketArg::LogExp(e) => LogBucketArg::LogExp(self.run_on_instruction(e)),
                x => x.clone()
            }
        }).collect()
    }

    fn run_on_log_bucket(&self, bucket: &LogBucket) -> InstructionPointer {
        LogBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            argsprint: self.run_on_log_bucket_arg(&bucket.argsprint),
        }.allocate()
    }

    fn run_on_loop_bucket(&self, bucket: &LoopBucket) -> InstructionPointer {
        LoopBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            continue_condition: self.run_on_instruction(&bucket.continue_condition),
            body: self.run_on_instructions(&bucket.body),
        }.allocate()
    }

    fn run_on_create_cmp_bucket(&self, bucket: &CreateCmpBucket) -> InstructionPointer {
        CreateCmpBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            template_id: bucket.template_id,
            cmp_unique_id: bucket.cmp_unique_id,
            symbol: bucket.symbol.clone(),
            sub_cmp_id: self.run_on_instruction(&bucket.sub_cmp_id),
            name_subcomponent: bucket.name_subcomponent.to_string(),
            defined_positions: bucket.defined_positions.clone(),
            is_part_mixed_array_not_uniform_parallel: bucket.is_part_mixed_array_not_uniform_parallel,
            uniform_parallel: bucket.uniform_parallel,
            dimensions: bucket.dimensions.clone(),
            signal_offset: bucket.signal_offset,
            signal_offset_jump: bucket.signal_offset_jump,
            component_offset: bucket.component_offset,
            component_offset_jump: bucket.component_offset_jump,
            number_of_cmp: bucket.number_of_cmp,
            has_inputs: bucket.has_inputs,
        }.allocate()
    }

    fn run_on_constraint_bucket(&self, bucket: &ConstraintBucket) -> InstructionPointer {
        self.run_on_instruction(bucket.unwrap())
    }

    fn run_on_block_bucket(&self, bucket: &BlockBucket) -> InstructionPointer {
        BlockBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            body: self.run_on_instructions(&bucket.body),
        }.allocate()
    }

    fn run_on_nop_bucket(&self, bucket: &NopBucket) -> InstructionPointer {
        bucket.clone().allocate()
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

    pub fn schedule_loop_unroll_pass(
        &self,
        program_archive: ProgramArchive,
        prime: &String,
    ) -> &Self {
        let main_file_id = program_archive.get_file_id_main();
        let _runtime = RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);
        self.passes.borrow_mut().push(Box::new(LoopUnrollPass::new(prime)));
        self
    }

    pub fn schedule_simplification_pass(&self, prime: &String) -> &Self {
        self.passes.borrow_mut().push(Box::new(SimplificationPass::new(prime)));
        self
    }

    pub fn run_on_circuit(&self, circuit: Circuit) -> Circuit {
        let mut transformed_circuit = circuit;
        for pass in self.passes.borrow().iter() {
            transformed_circuit = pass.run_on_circuit(&transformed_circuit);
        }
        transformed_circuit
    }
}
