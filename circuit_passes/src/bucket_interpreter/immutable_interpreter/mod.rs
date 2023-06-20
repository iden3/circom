pub mod observer;

use compiler::intermediate_representation::{Instruction, InstructionPointer};
use compiler::intermediate_representation::ir_interface::{AddressType, AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, InputInformation, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, OperatorType, ReturnBucket, StatusInput, StoreBucket, BlockBucket, ValueBucket, ValueType};
use compiler::num_bigint::BigInt;
use observer::InterpreterObserver;
use program_structure::constants::UsefulConstants;
use crate::bucket_interpreter::env::immutable_env::FrozenEnv;
use crate::bucket_interpreter::value;
use crate::bucket_interpreter::value::{mod_value, resolve_operation, Value};
use crate::bucket_interpreter::value::Value::{KnownBigInt, KnownU32, Unknown};

pub struct BucketInterpreter<'a> {
    constants: UsefulConstants,
    prime: &'a String,
    pub constant_fields: &'a Vec<String>,
    observer: &'a dyn InterpreterObserver
}

pub type R = (Option<Value>, FrozenEnv);

macro_rules! abort_if_false {
    ($e: expr, $env: expr) => {
        if !$e {
            return (None, $env.clone())
        }
    }
}

impl<'a> BucketInterpreter<'a> {
    pub fn init(prime: &'a String, constant_fields: &'a Vec<String>, observer: &'a dyn InterpreterObserver) -> Self {
        BucketInterpreter {
            constants: UsefulConstants::new(prime),
            prime,
            constant_fields,
            observer
        }
    }

    pub fn prime(&self) -> &String {
        &self.prime
    }

    pub fn observer(&self) -> &dyn InterpreterObserver {
        self.observer
    }

    pub fn execute_value_bucket(&self, bucket: &ValueBucket, env: &FrozenEnv, _observe: bool) -> R {
        (Some(match bucket.parse_as {
            ValueType::BigInt => {
                let constant = &self.constant_fields[bucket.value];
                KnownBigInt(BigInt::parse_bytes(constant.as_bytes(), 10).expect(format!("Cannot parse constant {}", constant).as_str()))
            },
            ValueType::U32 => KnownU32(bucket.value)
        }), env.clone())
    }

    pub fn execute_load_bucket(&self, bucket: &LoadBucket, env: &FrozenEnv, observe: bool) -> R {
        let (index, env) = self.execute_location_rule(&bucket.src, env, observe);
        let index = index.expect("src instruction in LoadBucket must yield a value!").get_u32();

        match &bucket.address_type {
            AddressType::Variable => (Some(env.get_var(index)), env),
            AddressType::Signal => (Some(env.get_signal(index)), env),
            AddressType::SubcmpSignal { cmp_address, .. } => {
                let (cmp_index, env) = self.execute_instruction(cmp_address, &env, observe);
                let cmp_index = cmp_index.expect("cmp_index in LoadBucket SubcmpSignal address type must yield a value!").get_u32();
                (Some(env.get_subcmp_signal(cmp_index, index)), env)
            }
        }
    }

    pub fn execute_location_rule(&self, loc: &LocationRule, env: &FrozenEnv, observe: bool) -> R {
        let continue_observing = if observe { self.observer.on_location_rule(loc, env) } else { false };
        match loc {
            LocationRule::Indexed { location, .. } => {
                self.execute_instruction(location, env, continue_observing)
            }
            LocationRule::Mapped { .. } => todo!()
        }
    }

    pub fn execute_store_bucket(&self, bucket: &StoreBucket, env: &FrozenEnv, observe: bool) -> R {
        let (src, env) = self.execute_instruction(&bucket.src, env, observe);
        let (idx, env) = self.execute_location_rule(&bucket.dest, &env, observe);
        let idx = idx.expect("dest instruction in StoreBucket must produce a value!").get_u32();
        let src = src.expect("src instruction in StoreBucket must produce a value!");
        match &bucket.dest_address_type {
            AddressType::Variable => (None, env.set_var(idx, src)),
            AddressType::Signal => (None, env.set_signal(idx, src)),
            AddressType::SubcmpSignal { cmp_address, input_information, .. } => {
                let (addr, env) = self.execute_instruction(cmp_address, &env, observe);
                let addr = addr.expect("cmp_address instruction in StoreBucket SubcmpSignal must produce a value!").get_u32();
                let env = env.set_subcmp_signal(addr, idx, src)
                    .decrease_subcmp_counter(addr);

                let sub_cmp_name = match &bucket.dest {
                    LocationRule::Indexed { template_header, ..} => template_header.clone(),
                    _ => None
                };

                if let InputInformation::Input { status } = input_information {
                    match status {
                        StatusInput::Last => {
                            return (None, env.run_subcmp(addr, &sub_cmp_name.unwrap(), self, observe));
                        }
                        StatusInput::Unknown => {
                            if env.subcmp_counter_is_zero(addr) {
                                return (None, env.run_subcmp(addr, &sub_cmp_name.unwrap(), self, observe));
                            }
                        }
                        _ => {}
                    }
                }
                (None, env)
            }
        }
    }

    pub fn execute_compute_bucket(&self, bucket: &ComputeBucket, env: &FrozenEnv, observe: bool) -> R {

        let mut stack = vec![];
        let mut env = env.clone();
        for i in &bucket.stack {
            let (value, new_env) = self.execute_instruction(i, &env, observe);
            env = new_env;
            stack.push(value.expect("Stack value in ComputeBucket must yield a value!"));
        }
        // If any value of the stack is unknown we just return unknown
        if stack.iter().any(|v| v.is_unknown()) {
            return (Some(Unknown), env.clone())
        }
        let p = self.constants.get_p();
        let computed_value = Some(match bucket.op {
            OperatorType::Mul => resolve_operation(value::mul_value, p, &stack),
            OperatorType::Div => resolve_operation(value::div_value, p, &stack),
            OperatorType::Add => resolve_operation(value::add_value, p, &stack),
            OperatorType::Sub => resolve_operation(value::sub_value, p, &stack),
            OperatorType::Pow => resolve_operation(value::pow_value, p, &stack),
            OperatorType::IntDiv => resolve_operation(value::int_div_value, p, &stack),
            OperatorType::Mod => resolve_operation(value::mod_value, p, &stack),
            OperatorType::ShiftL => resolve_operation(value::shift_l_value, p, &stack),
            OperatorType::ShiftR => resolve_operation(value::shift_r_value, p, &stack),
            OperatorType::LesserEq => value::lesser_eq(&stack[0], &stack[1]),
            OperatorType::GreaterEq => value::greater_eq(&stack[0], &stack[1]),
            OperatorType::Lesser => value::lesser(&stack[0], &stack[1]),
            OperatorType::Greater => value::greater(&stack[0], &stack[1]),
            OperatorType::Eq(1) => value::eq1(&stack[0], &stack[1]),
            OperatorType::Eq(_) => todo!(),
            OperatorType::NotEq => value::not_eq(&stack[0], &stack[1]),
            OperatorType::BoolOr => stack.iter().fold(KnownU32(0), value::bool_or_value),
            OperatorType::BoolAnd => stack.iter().fold(KnownU32(1), value::bool_and_value),
            OperatorType::BitOr => resolve_operation(value::bit_or_value, p, &stack),
            OperatorType::BitAnd => resolve_operation(value::bit_and_value, p, &stack),
            OperatorType::BitXor => resolve_operation(value::bit_xor_value, p, &stack),
            OperatorType::PrefixSub => mod_value(&value::prefix_sub(&stack[0]), &KnownBigInt(p.clone())),
            OperatorType::BoolNot => KnownU32((!stack[0].to_bool()).into()),
            OperatorType::Complement => mod_value(&value::complement(&stack[0]), &KnownBigInt(p.clone())),
            OperatorType::ToAddress => value::to_address(&stack[0]),
            OperatorType::MulAddress => stack.iter().fold(KnownU32(1), value::mul_address),
            OperatorType::AddAddress => stack.iter().fold(KnownU32(0), value::add_address),
        });
        (computed_value, env.clone())
    }

    pub fn execute_call_bucket(&self, bucket: &CallBucket, env: &FrozenEnv, observe: bool) -> R {
        let mut args = vec![];
        let mut env = env.clone();
        for i in &bucket.arguments {
            let (value, new_env) = self.execute_instruction(i, &env, observe);
            env = new_env;
            args.push(value);
        }

        todo!()
    }

    pub fn execute_branch_bucket(&self, bucket: &BranchBucket, env: &FrozenEnv, observe: bool) -> R {
        let (value, _, env) = self.execute_conditional_bucket(&bucket.cond, &bucket.if_branch, &bucket.else_branch, &env, observe);
        (value, env)
    }

    pub fn execute_return_bucket(&self, bucket: &ReturnBucket, observe: bool) -> R {
        todo!()
    }

    pub fn execute_assert_bucket(&self, bucket: &AssertBucket, env: &FrozenEnv, observe: bool) -> R {
        self.observer.on_assert_bucket(bucket, &env);

        let (cond, env) = self.execute_instruction(&bucket.evaluate, env, observe);
        let cond = cond.expect("cond in AssertBucket must produce a value!");
        if !cond.is_unknown() {
            assert!(cond.to_bool());
        }
        (None, env)
    }

    pub fn execute_log_bucket(&self, bucket: &LogBucket, observe: bool) -> R {
        todo!()
    }

    // TODO: Needs more work!
    pub fn execute_conditional_bucket(&self, cond: &InstructionPointer,
                                      true_branch: &[InstructionPointer],
                                      false_branch: &[InstructionPointer],
                                      env: &FrozenEnv, observe: bool) -> (Option<Value>, Option<bool>, FrozenEnv) {
        let (executed_cond, env) = self.execute_instruction(cond, env, observe);
        let executed_cond = executed_cond.expect("executed_cond must produce a value!");
        println!("executed_cond == {:?}", executed_cond);
        let cond_bool_result = match executed_cond {
            Unknown => None,
            KnownU32(1) => Some(true),
            KnownU32(0) => Some(false),
            KnownU32(_) => todo!(),
            KnownBigInt(_) => todo!()
        };

        return match cond_bool_result {
            None => {
                let (mut ret, mut new_env) = self.execute_instructions(true_branch, &env, observe);
                if ret.is_none() {
                    (ret, new_env) = self.execute_instructions(false_branch, &env, observe);
                }
                (ret.clone(), None, new_env.clone())
            }
            Some(true) => {
                let (ret, env) = self.execute_instructions(&true_branch, &env, observe);
                (ret, Some(true), env)
            },
            Some(false) => {
                let (ret, env) = self.execute_instructions(&false_branch, &env, observe);
                (ret, Some(false), env)
            }
        }
    }

    pub fn execute_loop_bucket_once(&self, bucket: &LoopBucket, env: &FrozenEnv, observe: bool) -> (Option<Value>, Option<bool>, FrozenEnv) {
        self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[], env, observe)
    }

    /// Executes the loop many times. If the result of the loop condition is unknown
    /// the interpreter assumes that the result of the loop is unknown.
    /// In this case of unknown condition the loop is executed twice.
    /// This logic is inspired by the logic for the AST interpreter.
    pub fn execute_loop_bucket(&self, bucket: &LoopBucket, env: &FrozenEnv, observe: bool) -> R {
        self.observer.on_loop_bucket(bucket, env);
        let mut last_value = Some(Unknown);
        let mut loop_env = env.clone();
        loop {
            let (value, cond, new_env) = self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[], &loop_env, observe);
            loop_env = new_env;
            match cond {
                None => {
                    let (value, _, loop_env) = self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[], &loop_env, observe);
                    break (value, loop_env);
                }
                Some(false) => {
                    break (last_value, loop_env);
                }
                Some(true) => {
                    last_value = value;
                }
            }

        }
    }

    pub fn execute_create_cmp_bucket(&self, bucket: &CreateCmpBucket, env: &FrozenEnv, observe: bool) -> R {
        self.observer.on_create_cmp_bucket(bucket, &env);

        let (cmp_id, env) = self.execute_instruction(&bucket.sub_cmp_id, env, observe);
        let cmp_id = cmp_id.expect("sub_cmp_id subexpression must yield a value!").get_u32();
        let mut env = env.create_subcmp(&bucket.symbol, cmp_id,bucket.number_of_cmp);
        // Run the subcomponents with 0 inputs directly
        for i in cmp_id..(cmp_id + bucket.number_of_cmp) {
            if env.subcmp_counter_is_zero(i) {
                env = env.run_subcmp(i, &bucket.symbol, self, observe);
            }
        }
        (None, env)
    }

    pub fn execute_constraint_bucket(&self, bucket: &ConstraintBucket, env: &FrozenEnv, observe: bool) -> R {
        self.observer.on_constraint_bucket(bucket, env);

        self.execute_instruction(match bucket {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        }, env, observe)
    }

    pub fn execute_instructions(&self, instructions: &[InstructionPointer], env: &FrozenEnv, observe: bool) -> R {
        let mut last = (None, env.clone());
        for inst in instructions {
            last = self.execute_instruction(inst, &last.1, observe);
        }
        last
    }

    pub fn execute_unrolled_loop_bucket(&self, bucket: &BlockBucket, env: &FrozenEnv, observe: bool) -> R {
        let mut last = (None, env.clone());
        for iteration in &bucket.body {
            last = self.execute_instruction(iteration, &last.1, observe);
        }
        return last;
    }

    pub fn execute_nop_bucket(&self, _bucket: &NopBucket, env: &FrozenEnv, observe: bool) -> R {
        (None, env.clone())
    }

    pub fn execute_instruction(&self, inst: &InstructionPointer, env: &FrozenEnv, observe: bool) -> R {
        let continue_observing = if observe {
            self.observer.on_instruction(inst, env)
        } else { observe };
        match inst.as_ref() {
            Instruction::Value(b) => self.execute_value_bucket(b,env, continue_observing),
            Instruction::Load(b) => self.execute_load_bucket(b, env, continue_observing),
            Instruction::Store(b) => self.execute_store_bucket(b, env, continue_observing),
            Instruction::Compute(b) => self.execute_compute_bucket(b, env, continue_observing),
            Instruction::Call(b) => self.execute_call_bucket(b, env, continue_observing),
            Instruction::Branch(b) => self.execute_branch_bucket(b, env, continue_observing),
            Instruction::Return(b) => self.execute_return_bucket(b, continue_observing),
            Instruction::Assert(b) => self.execute_assert_bucket(b, env, continue_observing),
            Instruction::Log(b) => self.execute_log_bucket(b, continue_observing),
            Instruction::Loop(b) => self.execute_loop_bucket(b, env, continue_observing),
            Instruction::CreateCmp(b) => self.execute_create_cmp_bucket(b, env, continue_observing),
            Instruction::Constraint(b) => self.execute_constraint_bucket(b, env, continue_observing),
            Instruction::UnrolledLoop(b) => self.execute_unrolled_loop_bucket(b, env, continue_observing),
            Instruction::Nop(b) => self.execute_nop_bucket(b, env, continue_observing)
        }
    }

}
