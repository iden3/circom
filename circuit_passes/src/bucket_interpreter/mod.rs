pub mod value;
pub mod env;
pub mod mutable_interpreter;
pub mod immutable_interpreter;

use std::cell::{Ref, RefCell, RefMut};



use std::rc::Rc;


use compiler::intermediate_representation::{Instruction, InstructionPointer};
use compiler::intermediate_representation::ir_interface::{AddressType, AssertBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, InputInformation, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, OperatorType, ReturnBucket, StatusInput, StoreBucket, BlockBucket, ValueBucket, ValueType};
use compiler::num_bigint::BigInt;

use env::mutable_env::Env;
use program_structure::constants::UsefulConstants;
use value::Value;
use value::Value::{KnownBigInt, KnownU32, Unknown};
use crate::bucket_interpreter::value::{mod_value, resolve_operation};
