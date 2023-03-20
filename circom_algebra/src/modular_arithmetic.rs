use num_bigint::{BigInt, ModInverse, Sign};
use num_traits::ToPrimitive;

pub enum ArithmeticError {
    DivisionByZero,
    BitOverFlowInShift,
}

fn modulus(a: &BigInt, b: &BigInt) -> BigInt {
    ((a % b) + b) % b
}
// The maximum number of bits a BigInt can have is 18_446_744_073_709_551_615
// Returns the LITTLE ENDIAN representation of the bigint
fn bit_representation(elem: &BigInt) -> (Sign, Vec<u8>) {
    elem.to_radix_le(2)
}
// Computes 2**b -1 where b is the number of bits of field
fn mask(field: &BigInt) -> BigInt {
    let two = BigInt::from(2);
    let b = bit_representation(field).1.len();
    let mask = num_traits::pow::pow(two, b);
    mask - 1
}

// Arithmetic operations
pub fn add(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    //let left = modulus(left,field);
    //let right = modulus(right,field);
    modulus(&(left + right), field)
}
pub fn mul(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    //let left = modulus(left,field);
    //let right = modulus(right,field);
    modulus(&(left * right), field)
}
pub fn sub(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    //let left = modulus(left,field);
    //let right = modulus(right,field);
    modulus(&(left - right), field)
}
pub fn div(left: &BigInt, right: &BigInt, field: &BigInt) -> Result<BigInt, ArithmeticError> {
    let right_inverse = right
        .mod_inverse(field)
        .map_or(Result::Err(ArithmeticError::DivisionByZero), |a| Result::Ok(a))?;
    let res = mul(left, &right_inverse, field);
    Result::Ok(res)
}
pub fn idiv(left: &BigInt, right: &BigInt, field: &BigInt) -> Result<BigInt, ArithmeticError> {
    let zero = BigInt::from(0);
    let left = modulus(left, field);
    let right = modulus(right, field);
    if right == zero {
        Result::Err(ArithmeticError::DivisionByZero)
    } else {
        Result::Ok(left / right)
    }
}
pub fn mod_op(left: &BigInt, right: &BigInt, field: &BigInt) -> Result<BigInt, ArithmeticError> {
    let left = modulus(left, field);
    let right = modulus(right, field);
    Result::Ok(modulus(&left, &right))
}
pub fn pow(base: &BigInt, exp: &BigInt, field: &BigInt) -> BigInt {
    base.modpow(exp, field)
}
pub fn prefix_sub(elem: &BigInt, field: &BigInt) -> BigInt {
    let minus_one = BigInt::from(-1);
    mul(elem, &minus_one, field)
}

pub fn multi_inv(values: &Vec<BigInt>, field: &BigInt) -> Vec<BigInt>{
    let one : BigInt = BigInt::from(1);
    let mut partials : Vec<BigInt> = Vec::new();
    partials.push(one.clone());
    for i in 0..values.len(){
        partials.push(mul(partials.get(partials.len()-1).unwrap(),
                                          values.get(i).unwrap(),
                                          &field));
    }
    let mut inverse = div(&one,
                   partials.get(partials.len()-1).unwrap(),
                   &field).ok().unwrap();
    let mut outputs : Vec<BigInt> = vec![BigInt::from(0); partials.len()];
    let mut i = values.len();
    while i > 0{
        outputs[i-1] = mul(partials.get(i-1).unwrap(), &inverse, &field);
        inverse = mul(&inverse, values.get(i-1).unwrap(), &field);
        i = i - 1;
    }
    return outputs;
}

//Bit operations

// 256 bit complement
pub fn complement_256(elem: &BigInt, field: &BigInt) -> BigInt {
    let (sign, mut bit_repr) = bit_representation(elem);
    while bit_repr.len() > 256 {
        bit_repr.pop();
    }
    for _i in bit_repr.len()..256 {
        bit_repr.push(0);
    }
    for bit in &mut bit_repr {
        *bit = if *bit == 0 { 1 } else { 0 };
    }
    let cp = BigInt::from_radix_le(sign, &bit_repr, 2).unwrap();
    modulus(&cp, field)
}

pub fn shift_l(left: &BigInt, right: &BigInt, field: &BigInt) -> Result<BigInt, ArithmeticError> {
    let two = BigInt::from(2);
    let top = field / &two;
    if right <= &top {
        let usize_repr = right
            .to_usize()
            .map_or(Result::Err(ArithmeticError::DivisionByZero), |a| Result::Ok(a))?;
        let value = modulus(&((left * &num_traits::pow(two, usize_repr)) & &mask(field)), field);
        Result::Ok(value)
    } else {
        shift_r(left, &(field - right), field)
    }
}
pub fn shift_r(left: &BigInt, right: &BigInt, field: &BigInt) -> Result<BigInt, ArithmeticError> {
    let two = BigInt::from(2);
    let top = field / &two;
    if right <= &top {
        let usize_repr = right
            .to_usize()
            .map_or(Result::Err(ArithmeticError::DivisionByZero), |a| Result::Ok(a))?;
        let value = left / &num_traits::pow(two, usize_repr);
        Result::Ok(value)
    } else {
        shift_l(left, &(field - right), field)
    }
}
pub fn bit_or(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    modulus(&(left | right), field)
}
pub fn bit_and(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    modulus(&(left & right), field)
}
pub fn bit_xor(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    modulus(&(left ^ right), field)
}

// Boolean operations
fn constant_true() -> BigInt {
    BigInt::from(1)
}
fn constant_false() -> BigInt {
    BigInt::from(0)
}
fn val(elem: &BigInt, field: &BigInt) -> BigInt {
    let c = (field / &BigInt::from(2)) + 1;
    if &c <= elem && elem < field {
        elem - field
    } else {
        elem.clone()
    }
}
fn comparable_element(elem: &BigInt, field: &BigInt) -> BigInt {
    val(&modulus(elem, field), field)
}
fn normalize(elem: &BigInt, field: &BigInt) -> BigInt {
    let f = constant_false();
    let t = constant_true();
    if comparable_element(elem, field) == f {
        f
    } else {
        t
    }
}
pub fn as_bool(elem: &BigInt, field: &BigInt) -> bool {
    normalize(elem, field) != constant_false()
}
pub fn not(elem: &BigInt, field: &BigInt) -> BigInt {
    (normalize(elem, field) + 1) % 2
}
pub fn bool_or(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    (normalize(left, field) + normalize(right, field) + bool_and(left, right, field)) % 2
}
pub fn bool_and(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    normalize(left, field) * normalize(right, field)
}
pub fn eq(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    let left = modulus(left, field);
    let right = modulus(right, field);
    if left == right {
        constant_true()
    } else {
        constant_false()
    }
}
pub fn lesser(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    let left = comparable_element(left, field);
    let right = comparable_element(right, field);
    if left < right {
        constant_true()
    } else {
        constant_false()
    }
}
pub fn not_eq(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    not(&eq(left, right, field), field)
}
pub fn lesser_eq(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    bool_or(&lesser(left, right, field), &eq(left, right, field), field)
}
pub fn greater(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    not(&lesser_eq(left, right, field), field)
}
pub fn greater_eq(left: &BigInt, right: &BigInt, field: &BigInt) -> BigInt {
    bool_or(&greater(left, right, field), &eq(left, right, field), field)
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIELD: &str = "257";
    #[test]
    fn mod_check() {
        let a = BigInt::from(-8);
        let b = BigInt::from(5);
        let res = super::modulus(&a, &b);
        assert_eq!(res, BigInt::from(2));
    }
    #[test]
    fn comparison_check() {
        let field = BigInt::parse_bytes(FIELD.as_bytes(), 10)
            .expect("generating the big int was not possible");
        let a = sub(&BigInt::from(2), &BigInt::from(1), &field);
        let b = BigInt::from(-1);
        let res = not_eq(&a, &b, &field);
        assert!(as_bool(&res, &field));
    }
    #[test]
    fn mod_operation_check() {
        let field = BigInt::parse_bytes(FIELD.as_bytes(), 10)
            .expect("generating the big int was not possible");
        let a = BigInt::from(17);
        let b = BigInt::from(32);
        if let Result::Ok(res) = mod_op(&a, &b, &field) {
            assert_eq!(a, res)
        } else {
            assert!(false);
        }
    }
    #[test]
    fn complement_of_complement_is_the_original_test() {
        let field = BigInt::parse_bytes(FIELD.as_bytes(), 10)
            .expect("generating the big int was not possible");
        let big_num = BigInt::parse_bytes("1234".as_bytes(), 10)
            .expect("generating the big int was not possible");
        let big_num_complement = complement_256(&big_num, &field);
        let big_num_complement_complement = complement_256(&big_num_complement, &field);
        let big_num_modulus = modulus(&big_num, &field);
        assert_eq!(big_num_complement_complement, big_num_modulus);
    }
    #[test]
    fn lesser_eq_test() {
        let field = BigInt::parse_bytes(FIELD.as_bytes(), 10)
            .expect("generating the big int was not possible");
        let zero = BigInt::from(0);
        let two = BigInt::from(2);
        assert!(zero < two);
        assert!(as_bool(&lesser_eq(&zero, &two, &field), &field));
    }
}
