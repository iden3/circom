use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use crate::bucket_interpreter::env::mutable_env::Env;
use crate::bucket_interpreter::value;
use crate::bucket_interpreter::value::{mod_value, resolve_operation, Value};
use crate::bucket_interpreter::value::Value::{KnownBigInt, KnownU32, Unknown};
use compiler::intermediate_representation::{Instruction, InstructionPointer};
use compiler::intermediate_representation::ir_interface::{AddressType, AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, InputInformation, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, OperatorType, ReturnBucket, StatusInput, StoreBucket, BlockBucket, ValueBucket, ValueType};

use compiler::num_bigint::BigInt;
use program_structure::utils::constants::UsefulConstants;

pub struct MutableBucketInterpreter {
    env: Vec<Rc<RefCell<Env>>>,
    constants: UsefulConstants,
    pub prime: String,
    pub constant_fields: Vec<String>
}

impl MutableBucketInterpreter {
    pub fn init(env: Env, prime: &String, constant_fields: Vec<String>) -> Self {
        MutableBucketInterpreter {
            env: vec![Rc::new(RefCell::new(env))],
            constants: UsefulConstants::new(prime),
            prime: prime.clone(),
            constant_fields
        }
    }

    pub fn get_env(&self) -> Ref<Env> {
        self.env.last().unwrap().borrow()
    }

    fn get_env_mut(&self) -> RefMut<'_, Env> {
        self.env.last().unwrap().borrow_mut()
    }

    pub fn print_env(&self) {
        println!("{}", self.get_env());
    }

    pub fn reset(&mut self) {
        self.get_env_mut().reset();
    }

    /// Pops the top of the envs stack
    pub fn pop_env(&mut self) {
        self.env.pop();
    }

    /// Pushes a copy of the current environment to the top
    pub fn push_env(&mut self) {
        let current_env = self.get_env().clone();
        self.env.push(Rc::new(RefCell::new(current_env)))
    }

    pub fn execute_value_bucket(&self, bucket: &ValueBucket) -> Option<Value> {
        Some(match bucket.parse_as {
            ValueType::BigInt => {
                let constant = &self.constant_fields[bucket.value];
                KnownBigInt(BigInt::parse_bytes(constant.as_bytes(), 10).expect(format!("Cannot parse constant {}", constant).as_str()))
            },
            ValueType::U32 => KnownU32(bucket.value)
        })
    }

    pub fn execute_load_bucket(&self, bucket: &LoadBucket) -> Option<Value> {
        println!("{}", bucket.to_string());
        let index = self.execute_location_rule(&bucket.src).unwrap().get_u32();
        let env = self.get_env();
        Some(match &bucket.address_type {
            AddressType::Variable => env.get_var(index),
            AddressType::Signal => env.get_signal(index),
            AddressType::SubcmpSignal { cmp_address, .. } => {
                let cmp_index = self.execute_instruction_ptr(cmp_address).unwrap().get_u32();
                env.get_subcmp_signal(cmp_index, index)
            }
        })
    }

    pub fn execute_location_rule(&self,loc: &LocationRule) -> Option<Value> {
        match loc {
            LocationRule::Indexed { location, .. } => {
                self.execute_instruction_ptr(location)
            }
            LocationRule::Mapped { .. } => todo!()
        }
    }

    pub fn execute_store_bucket(&self, bucket: &StoreBucket) -> Option<Value> {
        println!("{}", bucket.to_string());
        let src = self.execute_instruction_ptr(&bucket.src).unwrap();
        let idx = self.execute_location_rule(&bucket.dest).unwrap().get_u32();
        match &bucket.dest_address_type {
            AddressType::Variable => self.get_env_mut().set_var(idx, src),
            AddressType::Signal => self.get_env_mut().set_signal(idx, src),
            AddressType::SubcmpSignal { cmp_address, input_information, .. } => {
                let addr = self.execute_instruction_ptr(cmp_address).unwrap().get_u32();
                let mut env = self.get_env_mut();
                env.set_subcmp_signal(addr, idx, src);
                env.decrease_subcmp_counter(addr);

                let sub_cmp_name = match &bucket.dest {
                    LocationRule::Indexed { template_header, ..} => template_header.clone(),
                    _ => None
                };

                if let InputInformation::Input { status } = input_information {
                    match status {
                        StatusInput::Last => {
                            env.run_subcmp(addr, &sub_cmp_name.unwrap(), self);
                        }
                        StatusInput::Unknown => {
                            if env.subcmp_counter_is_zero(addr) {
                                env.run_subcmp(addr, &sub_cmp_name.unwrap(), self);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        self.print_env();
        None
    }

    pub fn execute_compute_bucket(&self, bucket: &ComputeBucket) -> Option<Value> {
        let stack: Vec<Value> = bucket.stack.iter().map(|i| self.execute_instruction_ptr(i).unwrap()).collect();
        // If any value of the stack is unknown we just return unknown
        if stack.iter().any(|v| v.is_unknown()) {
            return Some(Unknown)
        }
        let p = self.constants.get_p();
        Some(match bucket.op {
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
        })
    }

    pub fn execute_call_bucket(&self, _bucket: &CallBucket) -> Option<Value> {
        todo!()
    }

    pub fn execute_branch_bucket(&self, bucket: &BranchBucket) -> Option<Value> {
        self.execute_conditional_bucket(&bucket.cond, &bucket.if_branch, &bucket.else_branch).0
    }

    pub fn execute_return_bucket(&self, _bucket: &ReturnBucket) -> Option<Value> {
        todo!()
    }

    pub fn execute_assert_bucket(&self, bucket: &AssertBucket) -> Option<Value> {
        let cond = self.execute_instruction_ptr(&bucket.evaluate).unwrap();
        if !cond.is_unknown() {
            assert!(cond.to_bool());
        }
        None
    }

    pub fn execute_log_bucket(&self, _bucket: &LogBucket) -> Option<Value> {
        todo!()
    }

    // TODO: Needs more work!
    pub fn execute_conditional_bucket(&self, cond: &InstructionPointer,
                                      true_branch: &[InstructionPointer],
                                      false_branch: &[InstructionPointer]) -> (Option<Value>, Option<bool>) {
        let executed_cond = self.execute_instruction_ptr(cond).unwrap();
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
                let mut ret = self.execute_instructions(true_branch);
                if ret.is_none() {
                    ret = self.execute_instructions(false_branch);
                }
                (ret, None)
            }
            Some(true) => (self.execute_instructions(&true_branch), Some(true)),
            Some(false) => {
                (self.execute_instructions(&false_branch), Some(false))
            }
        }
    }

    pub fn execute_loop_bucket_once(&self, bucket: &LoopBucket) -> (Option<Value>, Option<bool>) {
        self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[])
    }

    /// Executes the loop many times. If the result of the loop condition is unknown
    /// the interpreter assumes that the result of the loop is unknown.
    /// In this case of unknown condition the loop is executed twice.
    /// This logic is inspired by the logic for the AST interpreter.
    pub fn execute_loop_bucket(&self, bucket: &LoopBucket) -> Option<Value> {
        let mut last_value = Some(Unknown);
        loop {
            let (value, cond) = self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[]);
            match cond {
                None => {
                    let (value, _) = self.execute_conditional_bucket(&bucket.continue_condition, &bucket.body, &[]);
                    break value;
                }
                Some(false) => {
                    break last_value;
                }
                Some(true) => {
                    last_value = value;
                }
            }

        }
    }

    pub fn execute_create_cmp_bucket(&self, bucket: &CreateCmpBucket) -> Option<Value> {
        let cmp_id = self.execute_instruction_ptr(&bucket.sub_cmp_id).unwrap().get_u32();
        let mut env = self.get_env_mut();
        env.create_subcmp(&bucket.symbol, cmp_id,bucket.number_of_cmp);
        // Run the subcomponents with 0 inputs directly
        for i in cmp_id..(cmp_id + bucket.number_of_cmp) {
            if env.subcmp_counter_is_zero(i) {
                env.run_subcmp(i, &bucket.symbol, self);
            }
        }
        None
    }

    pub fn execute_constraint_bucket(&self, bucket: &ConstraintBucket) -> Option<Value> {
        self.execute_instruction_ptr(match bucket {
            ConstraintBucket::Substitution(i) => i,
            ConstraintBucket::Equality(i) => i
        })
    }

    pub fn execute_instructions(&self, instructions: &[InstructionPointer]) -> Option<Value> {
        let mut last = None;
        for inst in instructions {
            last = self.execute_instruction_ptr(inst);
        }
        last
    }

    pub fn execute_unrolled_loop_bucket(&self, bucket: &BlockBucket) -> Option<Value> {
        let mut last = None;
        for iteration in &bucket.body {
            last = self.execute_instruction(iteration);
        }
        return last;
    }

    pub fn execute_nop_bucket(&self, _bucket: &NopBucket) -> Option<Value> {
        None
    }

    pub fn execute_instruction_ptr(&self, inst: &InstructionPointer) -> Option<Value> {
        self.execute_instruction(&inst)
    }

    pub fn execute_instruction(&self, inst: &Instruction) -> Option<Value> {
        match inst {
            Instruction::Value(b) => self.execute_value_bucket(b),
            Instruction::Load(b) => self.execute_load_bucket(b),
            Instruction::Store(b) => self.execute_store_bucket(b),
            Instruction::Compute(b) => self.execute_compute_bucket(b),
            Instruction::Call(b) => self.execute_call_bucket(b),
            Instruction::Branch(b) => self.execute_branch_bucket(b),
            Instruction::Return(b) => self.execute_return_bucket(b),
            Instruction::Assert(b) => self.execute_assert_bucket(b),
            Instruction::Log(b) => self.execute_log_bucket(b),
            Instruction::Loop(b) => self.execute_loop_bucket(b),
            Instruction::CreateCmp(b) => self.execute_create_cmp_bucket(b),
            Instruction::Constraint(b) => self.execute_constraint_bucket(b),
            Instruction::UnrolledLoop(b) => self.execute_unrolled_loop_bucket(b),
            Instruction::Nop(b) => self.execute_nop_bucket(b)
        }
    }

}
