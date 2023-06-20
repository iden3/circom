use std::fmt::{Display, Formatter};
use compiler::intermediate_representation::ir_interface::{ValueBucket, ValueType};
use compiler::num_bigint::BigInt;
use compiler::num_traits::{One, ToPrimitive, Zero};
use std::ops::{Add, Div, Mul, Sub};
use crate::bucket_interpreter::value::Value::{KnownBigInt, KnownU32, Unknown};

#[derive(Clone, Debug)]
pub enum Value {
    Unknown,
    KnownU32(usize),
    KnownBigInt(BigInt),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unknown => write!(f, "Unknown"),
            KnownU32(n) => write!(f, "{}", n),
            KnownBigInt(n) => write!(f, "{}", n),
        }
    }
}

impl Value {
    pub fn get_u32(&self) -> usize {
        match self {
            KnownU32(i) => *i,
            _ => panic!("Can't unwrap a u32 from a non KnownU32 value! {:?}", self),
        }
    }

    pub fn get_bigint_as_string(&self) -> String {
        match self {
            KnownBigInt(b) => b.to_string(),
            _ => panic!("Can't extract a string representation of a non big int"),
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            Unknown => true,
            _ => false,
        }
    }

    pub fn is_bigint(&self) -> bool {
        match self {
            KnownBigInt(_) => true,
            _ => false,
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            KnownU32(0) => false,
            KnownU32(1) => true,
            KnownBigInt(n) => {
                if n.is_zero() {
                    return false;
                }
                if n.is_one() {
                    return true;
                }
                panic!("Attempted to convert a bigint that does not have the value either 0 or 1!")
            }
            _ => panic!(
                "Attempted to convert a value that cannot be converted to boolean! {:?}",
                self
            ),
        }
    }

    pub fn to_value_bucket(&self, constant_fields: &mut Vec<String>) -> ValueBucket {
        match self {
            Unknown => panic!("Can't create a value bucket from an unknown value!"),
            KnownU32(n) => ValueBucket {
                line: 0,
                message_id: 0,
                parse_as: ValueType::U32,
                op_aux_no: 0,
                value: *n,
            },
            KnownBigInt(n) => {
                let str_repr = n.to_string();
                let idx = constant_fields.len();
                constant_fields.push(str_repr);
                ValueBucket {
                    line: 0,
                    message_id: 0,
                    parse_as: ValueType::BigInt,
                    op_aux_no: 0,
                    value: idx,
                }
            }
        }
    }
}

pub fn add_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs + rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs).add(rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs.add(rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs.add(BigInt::from(*rhs))),
    }
}

pub fn sub_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs - rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs).sub(rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs.sub(rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs.sub(BigInt::from(*rhs))),
    }
}

pub fn mul_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs * rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs).mul(rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs.mul(rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs.mul(BigInt::from(*rhs))),
    }
}

fn fr_div(lhs: &BigInt, rhs: &BigInt) -> BigInt {
    let inv = BigInt::from(1).div(rhs);
    lhs.mul(inv)
}

pub fn div_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs / rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(fr_div(&BigInt::from(*lhs), rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(fr_div(lhs, rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(fr_div(lhs, &BigInt::from(*rhs))),
    }
}

fn fr_pow(lhs: &BigInt, rhs: &BigInt) -> BigInt {
    let abv: BigInt = if rhs < &BigInt::from(0) { -rhs.clone() } else { rhs.clone() };
    let mut res = BigInt::from(1);
    let mut i = BigInt::from(0);
    while i < abv {
        res *= lhs;
        i += 1
    }
    if rhs < &BigInt::from(0) {
        res = 1 / res;
    }
    res
}

pub fn pow_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs.pow(*rhs as u32)),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(fr_pow(&BigInt::from(*lhs), rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(fr_pow(lhs, rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(fr_pow(lhs, &BigInt::from(*rhs))),
    }
}

pub fn int_div_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs / rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs).div(rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs.div(rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs.div(BigInt::from(*rhs))),
    }
}

pub fn mod_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs % rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs) % (rhs)),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs % (rhs)),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs % (BigInt::from(*rhs))),
    }
}

pub fn shift_l_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs << rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => {
            KnownBigInt(BigInt::from(*lhs) << rhs.to_u64().unwrap() as usize)
        }
        (KnownBigInt(lhs), KnownBigInt(rhs)) => {
            KnownBigInt(lhs << (rhs.to_u64().unwrap() as usize))
        }
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs << *rhs),
    }
}

pub fn shift_r_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs >> rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => {
            KnownBigInt(BigInt::from(*lhs) >> (rhs.to_u64().unwrap() as usize))
        }
        (KnownBigInt(lhs), KnownBigInt(rhs)) => {
            KnownBigInt(lhs >> (rhs.to_u64().unwrap() as usize))
        }
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs >> *rhs),
    }
}

pub fn lesser_eq(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs <= rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownU32((BigInt::from(*lhs) <= *rhs).into()),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownU32((lhs <= rhs).into()),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownU32((lhs <= &BigInt::from(*rhs)).into()),
    }
}

pub fn greater_eq(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs >= rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownU32((BigInt::from(*lhs) >= *rhs).into()),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownU32((lhs >= rhs).into()),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownU32((lhs >= &BigInt::from(*rhs)).into()),
    }
}

pub fn lesser(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs < rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownU32((BigInt::from(*lhs) < *rhs).into()),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownU32((lhs < rhs).into()),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownU32((lhs < &BigInt::from(*rhs)).into()),
    }
}

pub fn greater(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs > rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownU32((BigInt::from(*lhs) > *rhs).into()),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownU32((lhs > rhs).into()),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownU32((lhs > &BigInt::from(*rhs)).into()),
    }
}

pub fn eq1(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs == rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownU32((BigInt::from(*lhs) == *rhs).into()),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownU32((lhs == rhs).into()),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownU32((lhs == &BigInt::from(*rhs)).into()),
    }
}

pub fn not_eq(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs != rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownU32((BigInt::from(*lhs) != *rhs).into()),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownU32((lhs != rhs).into()),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownU32((lhs != &BigInt::from(*rhs)).into()),
    }
}

pub fn bool_or_value(lhs: Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (lhs, rhs) => KnownU32((lhs.to_bool() || rhs.to_bool()).into()),
    }
}

pub fn bool_and_value(lhs: Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (lhs, rhs) => KnownU32((lhs.to_bool() && rhs.to_bool()).into()),
    }
}

pub fn bit_or_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs | rhs),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs) | rhs),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs | rhs),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs | &BigInt::from(*rhs)),
    }
}

pub fn bit_and_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs & rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs) & rhs),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs & rhs),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs & &BigInt::from(*rhs)),
    }
}

pub fn bit_xor_value(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Unknown, _) => Unknown,
        (_, Unknown) => Unknown,
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32((lhs ^ rhs).into()),
        (KnownU32(lhs), KnownBigInt(rhs)) => KnownBigInt(BigInt::from(*lhs) ^ rhs),
        (KnownBigInt(lhs), KnownBigInt(rhs)) => KnownBigInt(lhs ^ rhs),
        (KnownBigInt(lhs), KnownU32(rhs)) => KnownBigInt(lhs ^ &BigInt::from(*rhs)),
    }
}

pub fn prefix_sub(v: &Value) -> Value {
    match v {
        Unknown => Unknown,
        KnownU32(_n) => panic!("We cannot get the negative of an unsigned integer!"),
        KnownBigInt(n) => KnownBigInt(-n.clone()),
    }
}

pub fn complement(v: &Value) -> Value {
    match v {
        Unknown => Unknown,
        KnownU32(n) => KnownU32(!(*n)),
        KnownBigInt(n) => KnownBigInt(!n.clone()),
    }
}

pub fn to_address(v: &Value) -> Value {
    match v {
        Unknown => panic!("Cant convert into an address an unknown value!"),
        KnownBigInt(b) => KnownU32(b.to_u64().unwrap() as usize),
        x => x.clone(),
    }
}

pub fn mul_address(lhs: Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs * rhs),
        _ => panic!("Can't do address multiplication over unknown values or big integers!"),
    }
}

pub fn add_address(lhs: Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (KnownU32(lhs), KnownU32(rhs)) => KnownU32(lhs + rhs),
        _ => panic!("Can't do address addition over unknown values or big integers!"),
    }
}

impl Default for Value {
    fn default() -> Self {
        Unknown
    }
}

impl Default for &Value {
    fn default() -> Self {
        &Unknown
    }
}

pub fn resolve_operation(op: fn(&Value, &Value) -> Value, p: &BigInt, stack: &[Value]) -> Value {
    assert!(stack.len() > 0);
    let p = KnownBigInt(p.clone());
    let mut acc = stack[0].clone();
    for i in &stack[1..] {
        let result = mod_value(&op(&acc, i), &p);
        acc = result.clone();
    }
    acc.clone()
}
