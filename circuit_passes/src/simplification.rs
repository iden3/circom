use std::cell::RefCell;
use std::ops::Add;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::{Instruction, InstructionList, InstructionPointer};
use compiler::intermediate_representation::either::EitherExprOrStmt;
use compiler::intermediate_representation::ir_interface::{AddressType, Allocate, AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LogBucketArg, LoopBucket, NopBucket, ReturnBucket, StoreBucket, UnrolledLoopBucket, ValueBucket, ValueType};
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::env::{Env, FunctionsLibrary, TemplatesLibrary};
use crate::CircuitTransformationPass;
use crate::loop_unroll::PassMemory;

/// Implement this pass as follows
/// For each bucket check if there is a compute bucket inside
/// If there is one we take the current environment and evaluate the compute bucket
/// If the result is a known value we replace that compute bucket with a literal value
/// Then we run the original bucket to get the new environment
/// We reconstruct the bucket with the replaced new value
/// If there is no compute bucket in the expression we just clone the bucket, run
/// it to update the environment and return.


fn location_rule_has_compute_bucket(loc: &LocationRule) -> bool {
    match loc {
        LocationRule::Indexed { location, .. } => has_compute_bucket(location),
        LocationRule::Mapped { indexes, .. } => indexes.iter().any(|i| has_compute_bucket(i))
    }
}

fn has_compute_bucket(i: &Instruction) -> bool {
    match i {
        Instruction::Value(_) => false,
        Instruction::Load(b) => location_rule_has_compute_bucket(&b.src),
        Instruction::Store(b) => location_rule_has_compute_bucket(&b.dest) || has_compute_bucket(&b.src),
        Instruction::Compute(_) => true,
        Instruction::Call(b) => b.arguments.iter().any(|i| has_compute_bucket(i)),
        Instruction::Branch(b) => b.if_branch.iter().any(|i| has_compute_bucket(i)) || b.else_branch.iter().any(|i| has_compute_bucket(i)),
        Instruction::Return(b) => has_compute_bucket(&b.value),
        Instruction::Assert(b) => has_compute_bucket(&b.evaluate),
        Instruction::Log(b) => b.argsprint.iter().any(|a| {
            if let LogBucketArg::LogExp(e) = a {
                has_compute_bucket(e)
            } else {
                false
            }
        }),
        Instruction::Loop(b) => b.body.iter().any(|i| has_compute_bucket(i)),
        Instruction::CreateCmp(b) => has_compute_bucket(&b.sub_cmp_id),
        Instruction::Constraint(b) => has_compute_bucket(match b {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }),
        Instruction::UnrolledLoop(b) => b.body.iter().any(|i| i.iter().any(|i| has_compute_bucket(i))),
        Instruction::Nop(_) => false
    }
}

pub struct ComputeSimplificationPass {
    memory: RefCell<PassMemory>
}

impl ComputeSimplificationPass {
    pub fn new(prime: &String) -> Self {
        let cl: TemplatesLibrary = Default::default();
        let fl: FunctionsLibrary = Default::default();
        ComputeSimplificationPass {
            memory: RefCell::new(PassMemory {
                templates_library: cl.clone(),
                functions_library: fl.clone(),
                interpreter: BucketInterpreter::init(Env::new(cl, fl), prime, vec![])
            })
        }
    }

    /// Evaluated the compute bucket and returns either a copy of the bucket
    /// or a ValueBucket with the result of the expression represented in the ComputeBucket.
    fn eval_compute_bucket(&self, bucket: &ComputeBucket) -> InstructionPointer {
        let mut interpreter = &mut self.memory.borrow_mut().interpreter;
        interpreter.push_env();
        let result = interpreter.execute_compute_bucket(bucket).unwrap();
        if result.is_unknown() {
            interpreter.pop_env();
            println!("Value unknown. Not replacing");
            return bucket.clone().allocate();
        }
        let (value, parse_as) = if result.is_bigint() {
            let idx = interpreter.constant_fields.len();
            interpreter.constant_fields.push(result.get_bigint_as_string());
            (idx, ValueType::BigInt)
        } else {
            (result.get_u32(), ValueType::U32)
        };
        interpreter.pop_env();
        let b = ValueBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            parse_as,
            op_aux_no: 0,
            value,
        };
        println!("Replacing\n\n {}\n\n with\n\n {}\n\n", bucket.to_string(), b.to_string());
        b.allocate()
    }

    fn eval_call_bucket(&self, bucket: &CallBucket) -> InstructionPointer {
        let mut interpreter = &mut self.memory.borrow_mut().interpreter;
        interpreter.push_env();
        let result = interpreter.execute_call_bucket(bucket).unwrap();
        if result.is_unknown() {
            return bucket.clone().allocate();
        }
        let (value, parse_as) = if result.is_bigint() {
            let idx = interpreter.constant_fields.len();
            interpreter.constant_fields.push(result.get_bigint_as_string());
            (idx, ValueType::BigInt)
        } else {
            (result.get_u32(), ValueType::U32)
        };
        interpreter.pop_env();
        return ValueBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            parse_as,
            op_aux_no: 0,
            value,
        }.allocate();
    }

    fn reconstruct_location_rule(&self, loc: &LocationRule) -> LocationRule {
        match loc {
            LocationRule::Indexed { location, template_header, } => LocationRule::Indexed {
                location: self.reconstruct_bucket(&location),
                template_header: template_header.clone()
            },
            LocationRule::Mapped { signal_code, indexes } => LocationRule::Mapped {
                signal_code: *signal_code,
                indexes: indexes.iter().map(|i| self.reconstruct_bucket(i)).collect()
            }
        }
    }

    fn reconstruct_buckets(&self, l: &InstructionList) -> InstructionList {
        l.iter().map(|i| self.reconstruct_bucket(i)).collect()
    }

    fn reconstruct_address_type(&self, addr: &AddressType) -> AddressType {
        match addr {
            AddressType::SubcmpSignal { cmp_address, uniform_parallel_value, is_output, input_information } => AddressType::SubcmpSignal {
                uniform_parallel_value: uniform_parallel_value.clone(),
                is_output: *is_output,
                input_information: input_information.clone(),
                cmp_address: self.reconstruct_bucket(&cmp_address)
            },
            x => x.clone()
        }
    }

    fn reconstruct_bucket(&self, i: &Instruction) -> InstructionPointer {
        match i {
            Instruction::Value(b) => b.clone().allocate(),
            Instruction::Load(b) => LoadBucket {
                line: b.line,
                message_id: b.message_id,
                address_type: self.reconstruct_address_type(&b.address_type),
                src: self.reconstruct_location_rule(&b.src),
            }.allocate(),
            Instruction::Store(b) => StoreBucket {
                line: b.line,
                message_id: b.message_id,
                context: b.context,
                dest_is_output: b.dest_is_output,
                dest_address_type: self.reconstruct_address_type(&b.dest_address_type),
                dest: self.reconstruct_location_rule(&b.dest),
                src: self.reconstruct_bucket(&b.src),
            }.allocate(),
            Instruction::Compute(b) => self.eval_compute_bucket(b),
            Instruction::Call(b) => self.eval_call_bucket(b),
            Instruction::Branch(b) => BranchBucket {
                line: b.line,
                message_id: b.message_id,
                cond: b.cond.clone(),
                if_branch: self.reconstruct_buckets(&b.if_branch),
                else_branch: self.reconstruct_buckets(&b.else_branch),
            }.allocate(),
            Instruction::Return(b) => ReturnBucket {
                line: b.line,
                message_id: b.message_id,
                with_size: b.with_size,
                value: self.reconstruct_bucket(&b.value),
            }.allocate(),
            Instruction::Assert(b) => AssertBucket {
                line: b.line,
                message_id: b.message_id,
                evaluate: self.reconstruct_bucket(&b.evaluate),
            }.allocate(),
            Instruction::Log(b) => b.clone().allocate(),
            Instruction::Loop(b) => LoopBucket {
                line: b.line,
                message_id: b.message_id,
                continue_condition: b.continue_condition.clone(),
                body: self.reconstruct_buckets(&b.body),
            }.allocate(),
            Instruction::CreateCmp(b) => CreateCmpBucket {
                line: b.line,
                message_id: b.message_id,
                template_id: b.template_id,
                cmp_unique_id: b.cmp_unique_id,
                symbol: b.symbol.to_string(),
                sub_cmp_id: self.reconstruct_bucket(&b.sub_cmp_id),
                name_subcomponent: b.name_subcomponent.to_string(),
                defined_positions: b.defined_positions.clone(),
                is_part_mixed_array_not_uniform_parallel: b.is_part_mixed_array_not_uniform_parallel,
                uniform_parallel: b.uniform_parallel,
                dimensions: b.dimensions.clone(),
                signal_offset: b.signal_offset,
                signal_offset_jump: b.signal_offset_jump,
                component_offset: b.component_offset,
                component_offset_jump: b.component_offset_jump,
                number_of_cmp: b.number_of_cmp,
                has_inputs: b.has_inputs,
            }.allocate(),
            Instruction::Constraint(b) => match b {
                ConstraintBucket::Substitution(i) => ConstraintBucket::Substitution(self.reconstruct_bucket(i)),
                ConstraintBucket::Equality(i) => ConstraintBucket::Equality(self.reconstruct_bucket(i))
            }.allocate(),
            Instruction::UnrolledLoop(b) => UnrolledLoopBucket {
                original_loop: b.original_loop.clone().allocate(),
                line: b.line,
                message_id: b.message_id,
                body: b.body.iter().map(|iter| self.reconstruct_buckets(iter)).collect(),
            }.allocate(),
            Instruction::Nop(_) => NopBucket.allocate()
        }
    }
}



impl CircuitTransformationPass for ComputeSimplificationPass {
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

    fn run_on_instruction(&self, i: &Instruction) -> InstructionPointer {
        self.pre_hook_instruction(i);
        println!("\n[RUN ON INST] {}\n", i.to_string());
        let new_bucket = if has_compute_bucket(i) {

            let x = self.reconstruct_bucket(i);
            println!("[run_on_instruction] Replacing\n\n {}\n\nwith\n\n {}\n\n", i.to_string(), x.to_string());
            x
        } else {
            i.clone().allocate()
        };

        // Run the interpreter
        let mut interpreter = &mut self.memory.borrow_mut().interpreter;
        // We run the original instruction, not the new one.
        interpreter.execute_instruction(i);
        // Return the transformed bucket
        new_bucket
    }

    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().interpreter.constant_fields.clone()
    }


    // Any call to the run_on_*_bucket functions we know is a bucket that contains a compute bucket
}