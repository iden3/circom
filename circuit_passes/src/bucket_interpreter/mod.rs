pub mod value;
pub mod env;
pub mod observer;

use std::collections::BTreeSet;
use std::ops::Add;
use std::os::unix::raw::ino_t;
use code_producers::components::TemplateInstanceIOMap;
use compiler::intermediate_representation::{Instruction, InstructionList, InstructionPointer};
use compiler::intermediate_representation::ir_interface::{AddressType, AssertBucket, BlockBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, InputInformation, LoadBucket, LocationRule, LogBucket, LogBucketArg, LoopBucket, NopBucket, ObtainMeta, OperatorType, ReturnBucket, ReturnType, StatusInput, StoreBucket, ValueBucket, ValueType};
use compiler::num_bigint::BigInt;
use observer::InterpreterObserver;
use program_structure::constants::UsefulConstants;
use crate::bucket_interpreter::env::{Env, EnvSet};
use crate::bucket_interpreter::value::{JoinSemiLattice, mod_value, resolve_operation, Value};
use crate::bucket_interpreter::value::Value::{KnownBigInt, KnownU32, Unknown};

pub struct BucketInterpreter<'a> {
    scope: String,
    constants: UsefulConstants,
    prime: &'a String,
    pub constant_fields: &'a Vec<String>,
    observer: &'a dyn InterpreterObserver,
    io_map: TemplateInstanceIOMap
}

pub type R = (Option<Value>, Env);

impl JoinSemiLattice for Option<Value> {
    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (x, None) => x.clone(),
            (None, x) => x.clone(),
            (Some(x), Some(y)) => Some(x.join(y))
        }
    }
}

impl JoinSemiLattice for R {
    fn join(&self, other: &Self) -> Self {
        (self.0.join(&other.0), self.1.join(&other.1))
    }
}

impl<'a> BucketInterpreter<'a> {
    pub fn init(
        scope: String,
        prime: &'a String,
        constant_fields: &'a Vec<String>,
        observer: &'a dyn InterpreterObserver,
        io_map: TemplateInstanceIOMap
    ) -> Self {
        BucketInterpreter {
            scope,
            constants: UsefulConstants::new(prime),
            prime,
            constant_fields,
            observer,
            io_map
        }
    }

    pub fn clone_in_new_scope(interpreter: &Self, new_scope: String) -> BucketInterpreter<'a> {
        Self::init(new_scope, interpreter.prime, interpreter.constant_fields, interpreter.observer, interpreter.io_map.clone())
    }

    pub fn prime(&self) -> &String {
        &self.prime
    }

    pub fn observer(&self) -> &dyn InterpreterObserver {
        self.observer
    }

    pub fn execute_value_bucket(&self, bucket: &ValueBucket, env: &Env, _observe: bool) -> R {
        (
            Some(match bucket.parse_as {
                ValueType::BigInt => {
                    let constant = &self.constant_fields[bucket.value];
                    KnownBigInt(
                        BigInt::parse_bytes(constant.as_bytes(), 10)
                            .expect(format!("Cannot parse constant {}", constant).as_str()),
                    )
                }
                ValueType::U32 => KnownU32(bucket.value),
            }),
            env.clone(),
        )
    }

    pub fn execute_load_bucket(&self, bucket: &LoadBucket, env: &Env, observe: bool) -> R {
        match &bucket.address_type {
            AddressType::Variable => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(&bucket.src, env) } else { false };
                let (idx, env) = match &bucket.src {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                (Some(env.get_var(idx)), env)
            },
            AddressType::Signal => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(&bucket.src, env) } else { false };
                let (idx, env) = match &bucket.src {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                (Some(env.get_signal(idx)), env)
            },
            AddressType::SubcmpSignal { cmp_address, .. } => {
                let (addr, env) = self.execute_instruction(cmp_address, &env, observe);
                let addr = addr
                    .expect(
                        "cmp_address instruction in StoreBucket SubcmpSignal must produce a value!",
                    )
                    .get_u32();
                let continue_observing =
                    if observe { self.observer.on_location_rule(&bucket.src, &env) } else { false };
                let (idx, env) = match &bucket.src {
                    LocationRule::Indexed { location, .. } => {
                        let (idx, env) = self.execute_instruction(location, &env, continue_observing);
                        (idx.expect("Indexed location must produce a value!").get_u32(), env)
                    },
                    LocationRule::Mapped { signal_code, indexes } => {
                        let mut indexes_values = vec![];
                        let mut acc_env = env.clone();
                        for i in indexes {
                            let (val, new_env) = self.execute_instruction(i, &acc_env, continue_observing);
                            indexes_values.push(val.expect("Mapped location must produce a value!").get_u32());
                            acc_env = new_env;
                        }
                        if indexes.len() > 0 {
                            let map_access = &self.io_map[&acc_env.get_subcmp_template_id(addr)][*signal_code].offset;
                            if indexes.len() == 1 {
                                (map_access + indexes_values[0], acc_env)
                            } else {
                                todo!()
                            }
                        } else {
                            unreachable!()
                        }
                    }
                };
                (Some(env.get_subcmp_signal(addr, idx)), env)
            }
        }
    }

    pub fn store_value_in_address(&self, address: &AddressType, location: &LocationRule, value: Value, env: &Env, observe: bool) -> Env {
        match address {
            AddressType::Variable => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(location, env) } else { false };
                let (idx, env) = match location {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                env.set_var(idx, value)
            },
            AddressType::Signal => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(location, env) } else { false };
                let (idx, env) = match location {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                env.set_signal(idx, value)
            },
            AddressType::SubcmpSignal { cmp_address, input_information, .. } => {
                let (addr, env) = self.execute_instruction(cmp_address, &env, observe);
                let addr = addr
                    .expect(
                        "cmp_address instruction in StoreBucket SubcmpSignal must produce a value!",
                    )
                    .get_u32();
                let continue_observing =
                    if observe { self.observer.on_location_rule(location, &env) } else { false };
                let (idx, env, sub_cmp_name) = match location {
                    LocationRule::Indexed { location, template_header } => {
                        let (idx, env) = self.execute_instruction(location, &env, continue_observing);
                        (idx.expect("Indexed location must produce a value!").get_u32(), env, template_header.clone())
                    },
                    LocationRule::Mapped { signal_code, indexes } => {
                        let mut indexes_values = vec![];
                        let mut acc_env = env.clone();
                        for i in indexes {
                            let (val, new_env) = self.execute_instruction(i, &acc_env, continue_observing);
                            indexes_values.push(val.expect("Mapped location must produce a value!").get_u32());
                            acc_env = new_env;
                        }
                        let name = acc_env.get_subcmp_name(addr);
                        if indexes.len() > 0 {
                            //eprintln!("IO MAP crashes ({addr}): {:?}", self.io_map.contains_key(&1));
                            let map_access = &self.io_map[&acc_env.get_subcmp_template_id(addr)][*signal_code].offset;
                            if indexes.len() == 1 {
                                (map_access + indexes_values[0], acc_env, Some(name))
                            } else {
                                todo!()
                            }
                        } else {
                            unreachable!()
                        }
                    }
                };

                let env = env
                    .set_subcmp_signal(addr, idx, value)
                    .decrease_subcmp_counter(addr);

                if let InputInformation::Input { status } = input_information {
                    match status {
                        StatusInput::Last => {
                            return
                                env.run_subcmp(addr, &sub_cmp_name.unwrap(), self, observe)
                            ;
                        }
                        StatusInput::Unknown => {
                            if env.subcmp_counter_is_zero(addr) {
                                return
                                    env.run_subcmp(addr, &sub_cmp_name.unwrap(), self, observe)
                                ;
                            }
                        }
                        _ => {}
                    }
                }
                env
            }
        }
    }

    pub fn execute_store_bucket(&self, bucket: &StoreBucket, env: &Env, observe: bool) -> R {
        let (src, env) = self.execute_instruction(&bucket.src, env, observe);
        let src = src.expect("src instruction in StoreBucket must produce a value!");
        let env = self.store_value_in_address(&bucket.dest_address_type, &bucket.dest, src, &env, observe);
        (None, env)
    }

    pub fn execute_compute_bucket(&self, bucket: &ComputeBucket, env: &Env, observe: bool) -> R {
        let mut stack = vec![];
        let mut env = env.clone();
        for i in &bucket.stack {
            let (value, new_env) = self.execute_instruction(i, &env, observe);
            env = new_env;
            stack.push(value.expect("Stack value in ComputeBucket must yield a value!"));
        }
        // If any value of the stack is unknown we just return unknown
        if stack.iter().any(|v| v.is_unknown()) {
            return (Some(Unknown), env.clone());
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
            OperatorType::PrefixSub => {
                mod_value(&value::prefix_sub(&stack[0]), &KnownBigInt(p.clone()))
            }
            OperatorType::BoolNot => KnownU32((!stack[0].to_bool()).into()),
            OperatorType::Complement => {
                mod_value(&value::complement(&stack[0]), &KnownBigInt(p.clone()))
            }
            OperatorType::ToAddress => value::to_address(&stack[0]),
            OperatorType::MulAddress => stack.iter().fold(KnownU32(1), value::mul_address),
            OperatorType::AddAddress => stack.iter().fold(KnownU32(0), value::add_address),
        });
        (computed_value, env.clone())
    }

    pub fn execute_call_bucket(&self, bucket: &CallBucket, env: &Env, observe: bool) -> R {
        let mut args = vec![];
        let mut env = env.clone();
        for i in &bucket.arguments {
            let (value, new_env) = self.execute_instruction(i, &env, observe);
            env = new_env;
            args.push(value.expect("Function argument must produce a value!"));
        }

        let result = env.run_function(&bucket.symbol, self, args, observe);

        // Write the result in the destination according to the address type
        match &bucket.return_info {
            ReturnType::Intermediate { .. } => (Some(result), env),
            ReturnType::Final(final_data) => {
                (None, self.store_value_in_address(&final_data.dest_address_type, &final_data.dest, result, &env, observe))
            }
        }
    }

    pub fn execute_branch_bucket(&self, bucket: &BranchBucket, env: &Env, observe: bool) -> R {
        let (value, _, env) = self.execute_conditional_bucket(
            &bucket.cond,
            &bucket.if_branch,
            &bucket.else_branch,
            &env,
            observe,
        );
        (value, env)
    }

    pub fn execute_return_bucket(&self, bucket: &ReturnBucket, env: &Env, observe: bool) -> R {
        self.execute_instruction(&bucket.value, env, observe)
    }

    pub fn execute_assert_bucket(&self, bucket: &AssertBucket, env: &Env, observe: bool) -> R {
        self.observer.on_assert_bucket(bucket, &env);

        let (cond, env) = self.execute_instruction(&bucket.evaluate, env, observe);
        let cond = cond.expect("cond in AssertBucket must produce a value!");
        if !cond.is_unknown() {
            assert!(cond.to_bool());
        }
        (None, env)
    }

    pub fn execute_log_bucket(&self, bucket: &LogBucket, env: &Env, observe: bool) -> R {
        let mut env = env.clone();
        for arg in &bucket.argsprint {
            if let LogBucketArg::LogExp(i) = arg {
                let (_, new_env) = self.execute_instruction(i, &env, observe);
                env = new_env
            }
        }
        (None, env)
    }

    // TODO: Needs more work!
    pub fn execute_conditional_bucket(
        &self,
        cond: &InstructionPointer,
        true_branch: &[InstructionPointer],
        false_branch: &[InstructionPointer],
        env: &Env,
        observe: bool,
    ) -> (Option<Value>, Option<bool>, Env) {
        let (executed_cond, env) = self.execute_instruction(cond, env, observe);
        let executed_cond = executed_cond.expect("executed_cond must produce a value!");
        println!("executed_cond == {:?}", executed_cond);
        let cond_bool_result = self.value_to_bool(&executed_cond);

        return match cond_bool_result {
            None => {
                // let (then_side, then_env) = self.execute_instructions(true_branch, &env, observe);
                // let (else_side, else_env) = self.execute_instructions(false_branch, &env, observe);
                //
                // let value = match (then_side, else_side) {
                //     (None, None) => None,
                //     (Some(x), None) => Some(x),
                //     (None, Some(x)) => Some(x),
                //     (Some(x), Some(y)) => Some(x.join(&y))
                // };
                //
                // return (value, None, then_env.join(&else_env));
                let (mut ret, mut new_env) = self.execute_instructions(true_branch, &env, observe);
                if ret.is_none() {
                    (ret, new_env) = self.execute_instructions(false_branch, &env, observe);
                }
                (ret.clone(), None, new_env.clone())
            }
            Some(true) => {
                let (ret, env) = self.execute_instructions(&true_branch, &env, observe);
                (ret, Some(true), env)
            }
            Some(false) => {
                let (ret, env) = self.execute_instructions(&false_branch, &env, observe);
                (ret, Some(false), env)
            }
        };
    }

    pub fn execute_loop_bucket_once(
        &self,
        bucket: &LoopBucket,
        env: &Env,
        observe: bool,
    ) -> (Option<Value>, Option<bool>, Env) {
        println!("[execute_loop_bucket_once] Executing line {} in {}", bucket.line, self.scope);
        self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[], env, observe)
    }

    fn value_to_bool(&self, value: &Value) -> Option<bool> {
        match value {
            Unknown => None,
            KnownU32(1) => Some(true),
            KnownU32(0) => Some(false),
            KnownU32(_) => todo!(),
            KnownBigInt(_) => todo!(),
        }
    }

    fn compute_while(&self, predicate: &InstructionPointer, body: &InstructionList, env: &Env, visited_states: EnvSet, observe: bool) -> R {
        if visited_states.contains(env) {
            return (None, env.clone());
        }
        let (cond, _) = self.execute_instruction(predicate, env, observe);
        let cond = cond.expect("executed_cond must produce a value!");
        let cond_bool_result = self.value_to_bool(&cond);
        match cond_bool_result {
            None => {
                let r = self.execute_instructions(body, env, observe);
                let rec = self.compute_while(predicate, body, &r.1, visited_states.add(env), observe);
                (None, env.clone()).join(&rec)
            }
            Some(true) => {
                let r = self.execute_instructions(body, env, observe);
                self.compute_while(predicate, body, &r.1, visited_states.add(env), observe)
            }
            Some(false) => {
                self.execute_instructions(body, env, observe)
            }
        }
    }

    /// Executes the loop many times. If the result of the loop condition is unknown
    /// the interpreter assumes that the result of the loop is unknown.
    /// In this case of unknown condition the loop is executed twice.
    /// This logic is inspired by the logic for the AST interpreter.
    pub fn execute_loop_bucket(&self, bucket: &LoopBucket, env: &Env, observe: bool) -> R {
        self.observer.on_loop_bucket(bucket, env);
        let mut last_value = Some(Unknown);
        let mut loop_env = env.clone();
        let mut n_iters = 0;
        let limit = 1_000_000;
        loop {
            n_iters += 1;
            if n_iters >= limit {
                panic!("We have been running the same loop for {limit} iterations!! Is there an infinite loop?");
            }

            let (value, cond, new_env) = self.execute_conditional_bucket(
                &bucket.continue_condition,
                &bucket.body,
                &[],
                &loop_env,
                observe,
            );
            loop_env = new_env;
            match cond {
                None => {
                    let (value, _, loop_env) = self.execute_conditional_bucket(
                        &bucket.continue_condition,
                        &bucket.body,
                        &[],
                        &loop_env,
                        observe,
                    );
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
        // self.compute_while(
        //     &bucket.continue_condition,
        //     &bucket.body,
        //     env,
        //     Default::default(),
        //     observe
        // )
    }

    pub fn execute_create_cmp_bucket(
        &self,
        bucket: &CreateCmpBucket,
        env: &Env,
        observe: bool,
    ) -> R {
        self.observer.on_create_cmp_bucket(bucket, &env);

        let (cmp_id, env) = self.execute_instruction(&bucket.sub_cmp_id, env, observe);
        let cmp_id = cmp_id.expect("sub_cmp_id subexpression must yield a value!").get_u32();
        let mut env = env.create_subcmp(&bucket.symbol, cmp_id, bucket.number_of_cmp, bucket.template_id);
        // Run the subcomponents with 0 inputs directly
        for i in cmp_id..(cmp_id + bucket.number_of_cmp) {
            if env.subcmp_counter_is_zero(i) {
                env = env.run_subcmp(i, &bucket.symbol, self, observe);
            }
        }
        (None, env)
    }

    pub fn execute_constraint_bucket(
        &self,
        bucket: &ConstraintBucket,
        env: &Env,
        observe: bool,
    ) -> R {
        self.observer.on_constraint_bucket(bucket, env);

        self.execute_instruction(
            match bucket {
                ConstraintBucket::Substitution(i) => i,
                ConstraintBucket::Equality(i) => i,
            },
            env,
            observe,
        )
    }

    pub fn execute_instructions(
        &self,
        instructions: &[InstructionPointer],
        env: &Env,
        observe: bool,
    ) -> R {
        let mut last = (None, env.clone());
        for inst in instructions {
            last = self.execute_instruction(inst, &last.1, observe);
        }
        last
    }

    pub fn execute_unrolled_loop_bucket(
        &self,
        bucket: &BlockBucket,
        env: &Env,
        observe: bool,
    ) -> R {
        self.execute_instructions(&bucket.body, env, observe)
    }

    pub fn execute_nop_bucket(&self, _bucket: &NopBucket, env: &Env, _observe: bool) -> R {
        (None, env.clone())
    }

    pub fn execute_instruction(&self, inst: &InstructionPointer, env: &Env, observe: bool) -> R {
        let continue_observing =
            if observe { self.observer.on_instruction(inst, env) } else { observe };
        match inst.as_ref() {
            Instruction::Value(b) => self.execute_value_bucket(b, env, continue_observing),
            Instruction::Load(b) => self.execute_load_bucket(b, env, continue_observing),
            Instruction::Store(b) => self.execute_store_bucket(b, env, continue_observing),
            Instruction::Compute(b) => self.execute_compute_bucket(b, env, continue_observing),
            Instruction::Call(b) => self.execute_call_bucket(b, env, continue_observing),
            Instruction::Branch(b) => self.execute_branch_bucket(b, env, continue_observing),
            Instruction::Return(b) => self.execute_return_bucket(b, env, continue_observing),
            Instruction::Assert(b) => self.execute_assert_bucket(b, env, continue_observing),
            Instruction::Log(b) => self.execute_log_bucket(b, env, continue_observing),
            Instruction::Loop(b) => self.execute_loop_bucket(b, env, continue_observing),
            Instruction::CreateCmp(b) => self.execute_create_cmp_bucket(b, env, continue_observing),
            Instruction::Constraint(b) => {
                self.execute_constraint_bucket(b, env, continue_observing)
            }
            Instruction::Block(b) => self.execute_unrolled_loop_bucket(b, env, continue_observing),
            Instruction::Nop(b) => self.execute_nop_bucket(b, env, continue_observing),
        }
    }
}
