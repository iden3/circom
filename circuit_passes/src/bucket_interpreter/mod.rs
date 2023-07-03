pub mod value;
pub mod env;
pub mod observer;

use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};
use code_producers::components::TemplateInstanceIOMap;
use code_producers::llvm_elements::IndexMapping;
use compiler::intermediate_representation::{Instruction, InstructionList, InstructionPointer};
use compiler::intermediate_representation::ir_interface::{AddressType, AssertBucket, BlockBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, InputInformation, LoadBucket, LocationRule, LogBucket, LogBucketArg, LoopBucket, NopBucket, OperatorType, ReturnBucket, ReturnType, StatusInput, StoreBucket, ValueBucket, ValueType};
use compiler::num_bigint::BigInt;
use observer::InterpreterObserver;
use program_structure::constants::UsefulConstants;
use crate::bucket_interpreter::env::{Env};
use crate::bucket_interpreter::value::{JoinSemiLattice, mod_value, resolve_operation, Value};
use crate::bucket_interpreter::value::Value::{KnownBigInt, KnownU32, Unknown};

pub struct BucketInterpreter<'a> {
    scope: &'a String,
    prime: &'a String,
    pub constant_fields: &'a Vec<String>,
    observer: &'a dyn InterpreterObserver,
    io_map: &'a TemplateInstanceIOMap,
    p: Value,
    signal_index_mapping: &'a IndexMapping,
    variables_index_mapping: &'a IndexMapping
}

pub type R<'a> = (Option<Value>, Env<'a>);

impl JoinSemiLattice for Option<Value> {
    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (x, None) => x.clone(),
            (None, x) => x.clone(),
            (Some(x), Some(y)) => Some(x.join(y))
        }
    }
}

impl JoinSemiLattice for R<'_> {
    fn join(&self, other: &Self) -> Self {
        (self.0.join(&other.0), self.1.join(&other.1))
    }
}


impl<'a> BucketInterpreter<'a> {
    pub fn init(
        scope: &'a String,
        prime: &'a String,
        constant_fields: &'a Vec<String>,
        observer: &'a dyn InterpreterObserver,
        io_map: &'a TemplateInstanceIOMap,
        signal_index_mapping: &'a IndexMapping,
        variables_index_mapping: &'a IndexMapping
    ) -> Self {
        BucketInterpreter {
            scope,
            prime,
            constant_fields,
            observer,
            io_map,
            p: KnownBigInt(UsefulConstants::new(prime).get_p().clone()),
            signal_index_mapping,
            variables_index_mapping
        }
    }

    fn get_id_from_indexed_location(&self, location: &LocationRule, env: &Env) -> usize {
        match location {
            LocationRule::Indexed { location, .. } => {
                let (idx, _) = self.execute_instruction(location, env.clone(), false);
                idx.expect("LocationRule must produce a value!").get_u32()
            }
            LocationRule::Mapped { .. } => unreachable!()
        }
    }

    fn get_write_operations_in_store_bucket(&self, bucket: &StoreBucket,
                                            vars: &mut Vec<usize>,
                                            signals: &mut Vec<usize>,
                                            subcmps: &mut Vec<(usize, usize)>,
                                            env: &Env) {
        match bucket.dest_address_type {
            AddressType::Variable => {
                let idx = self.get_id_from_indexed_location(&bucket.dest, env);
                let indices = self.variables_index_mapping[&idx].clone();
                for index in indices {
                    vars.push(index);
                }
            }
            AddressType::Signal => {
                let idx = self.get_id_from_indexed_location(&bucket.dest, env);
                let indices = self.signal_index_mapping[&idx].clone();
                for index in indices {
                    signals.push(index);
                }
            }
            AddressType::SubcmpSignal { .. } => {
                todo!()
            }
        };
        // 1. Evaluate what indexing are we doing here:
        //    What index are we trying to write into?
        //    What kind? Var, signal or subcmp signal
        // 2. Get the name of the thing we are trying to write into
        // 3. Get all the possible indices for that thing
        // 4. For each index that correspond to a possible offset of the thing write it in the list
    }

    fn get_write_operations_in_inst_rec(
        &self,
        inst: &Instruction,
        vars: &mut Vec<usize>,
        signals: &mut Vec<usize>,
        subcmps: &mut Vec<(usize, usize)>,
        env: &Env
    ) {
        match inst {
            Instruction::Value(_) => {} // Cannot write
            Instruction::Load(_) => {} // Should not have a StoreBucket inside
            Instruction::Store(bucket) => {
                self.get_write_operations_in_store_bucket(bucket, vars, signals, subcmps, env)
            }
            Instruction::Compute(_) => {} // Should not have a StoreBucket inside
            Instruction::Call(_) => {} // Should not have a StoreBucket as argument
            Instruction::Branch(bucket) => {
                self.get_write_operations_in_body_rec(&bucket.if_branch, vars, signals, subcmps, env);
                self.get_write_operations_in_body_rec(&bucket.else_branch, vars, signals, subcmps, env);
            }
            Instruction::Return(_) => {} // Should not have a StoreBucket in the return expression
            Instruction::Assert(_) => {} // Should not have a StoreBucket inside
            Instruction::Log(_) => {} // Should not have a StoreBucket inside
            Instruction::Loop(bucket) => {
                self.get_write_operations_in_body_rec(&bucket.body, vars, signals, subcmps, env)
            }
            Instruction::CreateCmp(_) => {} // Should not have a StoreBucket inside
            Instruction::Constraint(bucket) => {
                self.get_write_operations_in_inst_rec(match bucket {
                    ConstraintBucket::Substitution(i) => i,
                    ConstraintBucket::Equality(i) => i
                }, vars, signals, subcmps, env)
            }
            Instruction::Block(bucket) => {
                self.get_write_operations_in_body_rec(&bucket.body, vars, signals, subcmps, env)
            }
            Instruction::Nop(_) => {} // Should not have a StoreBucket inside
        }
    }

    fn get_write_operations_in_body_rec(
        &self,
        body: &InstructionList,
        vars: &mut Vec<usize>,
        signals: &mut Vec<usize>,
        subcmps: &mut Vec<(usize, usize)>,
        env: &Env
    ) {
        for inst in body {
            self.get_write_operations_in_inst_rec(inst, vars, signals, subcmps, env);
        }
    }

    /// Returns a triple with a list of indices for each
    /// 0: Indices of variables
    /// 1: Indices of signals
    /// 2: Indices of subcmps
    fn get_write_operations_in_body(&self, body: &InstructionList, env: &Env) -> (Vec<usize>, Vec<usize>, Vec<(usize, usize)>) {
        let mut vars = vec![];
        let mut signals = vec![];
        let mut subcmps = vec![];

        self.get_write_operations_in_body_rec(body, &mut vars, &mut signals, &mut subcmps, env);

        return (vars, signals, subcmps);
    }

    pub fn clone_in_new_scope(interpreter: &Self, new_scope: &'a String) -> BucketInterpreter<'a> {
        Self::init(new_scope, interpreter.prime, interpreter.constant_fields, interpreter.observer, interpreter.io_map, &interpreter.signal_index_mapping, &interpreter.variables_index_mapping)
    }

    pub fn execute_value_bucket<'env>(&self, bucket: &ValueBucket, env: Env<'env>, _observe: bool) -> R<'env> {
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
            env
        )
    }

    pub fn execute_load_bucket<'env>(&self, bucket: &'env LoadBucket, env: Env<'env>, observe: bool) -> R<'env> {
        match &bucket.address_type {
            AddressType::Variable => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(&bucket.src, &env) } else { false };
                let (idx, env) = match &bucket.src {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                (Some(env.get_var(idx)), env)
            },
            AddressType::Signal => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(&bucket.src, &env) } else { false };
                let (idx, env) = match &bucket.src {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                (Some(env.get_signal(idx)), env)
            },
            AddressType::SubcmpSignal { cmp_address, .. } => {
                let (addr, env) = self.execute_instruction(cmp_address, env, observe);
                let addr = addr
                    .expect(
                        "cmp_address instruction in StoreBucket SubcmpSignal must produce a value!",
                    )
                    .get_u32();
                let continue_observing =
                    if observe { self.observer.on_location_rule(&bucket.src, &env) } else { false };
                let (idx, env) = match &bucket.src {
                    LocationRule::Indexed { location, .. } => {
                        let (idx, env) = self.execute_instruction(location, env, continue_observing);
                        (idx.expect("Indexed location must produce a value!").get_u32(), env)
                    },
                    LocationRule::Mapped { signal_code, indexes } => {
                        let mut indexes_values = vec![];
                        let mut acc_env = env;
                        for i in indexes {
                            let (val, new_env) = self.execute_instruction(i, acc_env, continue_observing);
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

    pub fn store_value_in_address<'env>(&self, address: &'env AddressType, location: &'env LocationRule, value: Value, env: Env<'env>, observe: bool) -> Env<'env> {
        match address {
            AddressType::Variable => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(location, &env) } else { false };
                let (idx, env) = match location {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                env.set_var(idx, value)
            },
            AddressType::Signal => {
                let continue_observing =
                    if observe { self.observer.on_location_rule(location, &env) } else { false };
                let (idx, env) = match location {
                    LocationRule::Indexed { location, .. } => self.execute_instruction(location, env, continue_observing),
                    LocationRule::Mapped { .. } => unreachable!()
                };
                let idx = idx.expect("Indexed location must produce a value!").get_u32();
                env.set_signal(idx, value)
            },
            AddressType::SubcmpSignal { cmp_address, input_information, .. } => {
                let (addr, env) = self.execute_instruction(cmp_address, env, observe);
                let addr = addr
                    .expect(
                        "cmp_address instruction in StoreBucket SubcmpSignal must produce a value!",
                    )
                    .get_u32();
                let continue_observing =
                    if observe { self.observer.on_location_rule(location, &env) } else { false };
                let (idx, env, sub_cmp_name) = match location {
                    LocationRule::Indexed { location, template_header } => {
                        let (idx, env) = self.execute_instruction(location, env, continue_observing);
                        (idx.expect("Indexed location must produce a value!").get_u32(), env, template_header.clone())
                    },
                    LocationRule::Mapped { signal_code, indexes } => {
                        let mut indexes_values = vec![];
                        let mut acc_env = env;
                        for i in indexes {
                            let (val, new_env) = self.execute_instruction(i, acc_env, continue_observing);
                            indexes_values.push(val.expect("Mapped location must produce a value!").get_u32());
                            acc_env = new_env;
                        }
                        let name = Some(acc_env.get_subcmp_name(addr).clone());
                        if indexes.len() > 0 {
                            //eprintln!("IO MAP crashes ({addr}): {:?}", self.io_map.contains_key(&1));
                            let map_access = &self.io_map[&acc_env.get_subcmp_template_id(addr)][*signal_code].offset;
                            if indexes.len() == 1 {
                                (map_access + indexes_values[0], acc_env, name)
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

    pub fn execute_store_bucket<'env>(&self, bucket: &'env StoreBucket, env: Env<'env>, observe: bool) -> R<'env> {
        let (src, env) = self.execute_instruction(&bucket.src, env, observe);
        let src = src.expect("src instruction in StoreBucket must produce a value!");
        let env = self.store_value_in_address(&bucket.dest_address_type, &bucket.dest, src, env, observe);
        (None, env)
    }

    pub fn execute_compute_bucket<'env>(&self, bucket: &'env ComputeBucket, env: Env<'env>, observe: bool) -> R<'env> {
        let mut stack = vec![];
        let mut env = env;
        for i in &bucket.stack {
            let (value, new_env) = self.execute_instruction(i, env, observe);
            env = new_env;
            stack.push(value.expect("Stack value in ComputeBucket must yield a value!"));
        }
        // If any value of the stack is unknown we just return unknown
        if stack.iter().any(|v| v.is_unknown()) {
            return (Some(Unknown), env);
        }
        let p = &self.p;
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
                mod_value(&value::prefix_sub(&stack[0]), p)
            }
            OperatorType::BoolNot => KnownU32((!stack[0].to_bool()).into()),
            OperatorType::Complement => {
                mod_value(&value::complement(&stack[0]), p)
            }
            OperatorType::ToAddress => value::to_address(&stack[0]),
            OperatorType::MulAddress => stack.iter().fold(KnownU32(1), value::mul_address),
            OperatorType::AddAddress => stack.iter().fold(KnownU32(0), value::add_address),
        });
        (computed_value, env)
    }

    pub fn execute_call_bucket<'env>(&self, bucket: &'env CallBucket, env: Env<'env>, observe: bool) -> R<'env> {
        let mut args = vec![];
        let mut env = env;
        for i in &bucket.arguments {
            let (value, new_env) = self.execute_instruction(i, env, observe);
            env = new_env;
            args.push(value.expect("Function argument must produce a value!"));
        }

        let result = env.run_function(&bucket.symbol, self, args, observe);

        // Write the result in the destination according to the address type
        match &bucket.return_info {
            ReturnType::Intermediate { .. } => (Some(result), env),
            ReturnType::Final(final_data) => {
                (None, self.store_value_in_address(&final_data.dest_address_type, &final_data.dest, result, env, observe))
            }
        }
    }

    pub fn execute_branch_bucket<'env>(&self, bucket: &'env BranchBucket, env: Env<'env>, observe: bool) -> R<'env> {
        let (value, _, env) = self.execute_conditional_bucket(
            &bucket.cond,
            &bucket.if_branch,
            &bucket.else_branch,
            env,
            observe,
        );
        (value, env)
    }

    pub fn execute_return_bucket<'env>(&self, bucket: &'env ReturnBucket, env: Env<'env>, observe: bool) -> R<'env> {
        self.execute_instruction(&bucket.value, env, observe)
    }

    pub fn execute_assert_bucket<'env>(&self, bucket: &'env AssertBucket, env: Env<'env>, observe: bool) -> R<'env> {
        self.observer.on_assert_bucket(bucket, &env);

        let (cond, env) = self.execute_instruction(&bucket.evaluate, env, observe);
        let cond = cond.expect("cond in AssertBucket must produce a value!");
        if !cond.is_unknown() {
            assert!(cond.to_bool());
        }
        (None, env)
    }

    pub fn execute_log_bucket<'env>(&self, bucket: &'env LogBucket, env: Env<'env>, observe: bool) -> R<'env> {
        let mut env = env;
        for arg in &bucket.argsprint {
            if let LogBucketArg::LogExp(i) = arg {
                let (_, new_env) = self.execute_instruction(i, env, observe);
                env = new_env
            }
        }
        (None, env)
    }

    // TODO: Needs more work!
    pub fn execute_conditional_bucket<'env>(
        &self,
        cond: &'env InstructionPointer,
        true_branch: &'env [InstructionPointer],
        false_branch: &'env [InstructionPointer],
        env: Env<'env>,
        observe: bool,
    ) -> (Option<Value>, Option<bool>, Env<'env>) {
        let (executed_cond, env) = self.execute_instruction(cond, env, observe);
        let executed_cond = executed_cond.expect("executed_cond must produce a value!");
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
                let (ret, new_env) = self.execute_instructions(true_branch, env.clone(), observe);
                if ret.is_none() {
                    let (ret, new_env) = self.execute_instructions(false_branch, env, observe);
                    (ret, None, new_env)
                } else {
                    (ret, None, new_env)
                }
            }
            Some(true) => {
                let (ret, env) = self.execute_instructions(&true_branch, env, observe);
                (ret, Some(true), env)
            }
            Some(false) => {
                let (ret, env) = self.execute_instructions(&false_branch, env, observe);
                (ret, Some(false), env)
            }
        };
    }

    pub fn execute_loop_bucket_once<'env>(
        &self,
        bucket: &'env LoopBucket,
        env: Env<'env>,
        observe: bool,
    ) -> (Option<Value>, Option<bool>, Env<'env>) {
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

    // fn compute_while(&self, predicate: &InstructionPointer, body: &InstructionList, env: &Env, visited_states: EnvSet, observe: bool) -> R {
    //     if visited_states.contains(env) {
    //         return (None, env.clone());
    //     }
    //     let (cond, _) = self.execute_instruction(predicate, env, observe);
    //     let cond = cond.expect("executed_cond must produce a value!");
    //     let cond_bool_result = self.value_to_bool(&cond);
    //     match cond_bool_result {
    //         None => {
    //             let r = self.execute_instructions(body, env, observe);
    //             let rec = self.compute_while(predicate, body, &r.1, visited_states.add(env), observe);
    //             (None, env.clone()).join(&rec)
    //         }
    //         Some(true) => {
    //             let r = self.execute_instructions(body, env, observe);
    //             self.compute_while(predicate, body, &r.1, visited_states.add(env), observe)
    //         }
    //         Some(false) => {
    //             self.execute_instructions(body, env, observe)
    //         }
    //     }
    // }



    /// Executes the loop many times. If the result of the loop condition is unknown
    /// the interpreter assumes that the result of the loop is unknown.
    /// In this case of unknown condition the loop is executed twice.
    /// This logic is inspired by the logic for the AST interpreter.
    pub fn execute_loop_bucket<'env>(&self, bucket: &'env LoopBucket, env: Env<'env>, observe: bool) -> R<'env> {
        self.observer.on_loop_bucket(bucket, &env);
        let mut last_value = Some(Unknown);
        let mut loop_env = env;
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
                loop_env,
                observe,
            );
            loop_env = new_env;
            match cond {
                None => {
                    let (vars, signals, subcmps) = self.get_write_operations_in_body(&bucket.body, &loop_env);

                    for var in vars {
                        loop_env = loop_env.set_var(var, Unknown);
                    }
                    for signal in signals {
                        loop_env = loop_env.set_signal(signal, Unknown);
                    }
                    for (subcmp_id, subcmp_signal) in subcmps {
                        loop_env = loop_env.set_subcmp_signal(subcmp_id, subcmp_signal, Unknown);
                    }
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

    pub fn execute_create_cmp_bucket<'env>(
        &self,
        bucket: &'env CreateCmpBucket,
        env: Env<'env>,
        observe: bool,
    ) -> R<'env> {
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

    pub fn execute_constraint_bucket<'env>(
        &self,
        bucket: &'env ConstraintBucket,
        env: Env<'env>,
        observe: bool,
    ) -> R<'env> {
        self.observer.on_constraint_bucket(bucket, &env);

        self.execute_instruction(
            match bucket {
                ConstraintBucket::Substitution(i) => i,
                ConstraintBucket::Equality(i) => i,
            },
            env,
            observe,
        )
    }

    pub fn execute_instructions<'env>(
        &self,
        instructions: &'env [InstructionPointer],
        env: Env<'env>,
        observe: bool,
    ) -> R<'env> {
        let mut last = (None, env);
        for inst in instructions {
            last = self.execute_instruction(inst, last.1, observe);
        }
        last
    }

    pub fn execute_unrolled_loop_bucket<'env>(
        &self,
        bucket: &'env BlockBucket,
        env: Env<'env>,
        observe: bool,
    ) -> R<'env> {
        self.execute_instructions(&bucket.body, env, observe)
    }

    pub fn execute_nop_bucket<'env>(&self, _bucket: &NopBucket, env: Env<'env>, _observe: bool) -> R<'env> {
        (None, env)
    }

    pub fn execute_instruction<'env>(&self, inst: &'env InstructionPointer, env: Env<'env>, observe: bool) -> R<'env> {
        let continue_observing =
            if observe { self.observer.on_instruction(inst, &env) } else { observe };
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
