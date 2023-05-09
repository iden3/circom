use crate::llvm_elements::LLVMIRProducer;
use crate::llvm_elements::functions::create_bb;
use crate::llvm_elements::functions::create_function;
use crate::llvm_elements::instructions::{
    create_add, create_sub, create_mul, create_div, create_eq, create_neq, create_lt, create_gt, create_le, create_ge, create_neg, create_shl,
    create_shr, create_bit_and, create_bit_or, create_bit_xor, create_logic_and, create_logic_or, create_logic_not, create_return,
};
use crate::llvm_elements::types::{bigint_type, bool_type};

pub const FR_ADD_FN_NAME: &str = "fr_add";
pub const FR_SUB_FN_NAME: &str = "fr_sub";
pub const FR_MUL_FN_NAME: &str = "fr_mul";
pub const FR_DIV_FN_NAME: &str = "fr_div";
pub const FR_EQ_FN_NAME: &str = "fr_eq";
pub const FR_NEQ_FN_NAME: &str = "fr_neq";
pub const FR_LT_FN_NAME: &str = "fr_lt";
pub const FR_GT_FN_NAME: &str = "fr_gt";
pub const FR_LE_FN_NAME: &str = "fr_le";
pub const FR_GE_FN_NAME: &str = "fr_ge";
pub const FR_NEG_FN_NAME: &str = "fr_neg";
pub const FR_SHL_FN_NAME: &str = "fr_shl";
pub const FR_SHR_FN_NAME: &str = "fr_shr";
pub const FR_BITAND_FN_NAME: &str = "fr_bit_and";
pub const FR_BITOR_FN_NAME: &str = "fr_bit_or";
pub const FR_BITXOR_FN_NAME: &str = "fr_bit_xor";
pub const FR_LAND_FN_NAME: &str = "fr_logic_and";
pub const FR_LOR_FN_NAME: &str = "fr_logic_or";
pub const FR_LNOT_FN_NAME: &str = "fr_logic_not";

macro_rules! fr_unary_op {
    ($name: expr, $producer: expr, $valTy: expr) => {{
        let args = &[$valTy.into()];
        let func = create_function($producer, $name, $valTy.fn_type(args, false));
        let main = create_bb($producer, func, $name);
        $producer.set_current_bb(main);

        let lhs = func.get_nth_param(0).unwrap();
        lhs
    }};
}

macro_rules! fr_binary_op {
    ($name: expr, $producer: expr, $argTy: expr, $retTy: expr) => {{
        let args = &[$argTy.into(), $argTy.into()];
        let func = create_function($producer, $name, $retTy.fn_type(args, false));
        let main = create_bb($producer, func, $name);
        $producer.set_current_bb(main);

        let lhs = func.get_nth_param(0).unwrap();
        let rhs = func.get_nth_param(1).unwrap();
        (lhs, rhs)
    }};
}

macro_rules! fr_binary_op_bigint {
    ($name: expr, $producer: expr) => {{
        let ty = bigint_type($producer);
        fr_binary_op!($name, $producer, ty, ty)
    }};
}

macro_rules! fr_binary_op_bool {
    ($name: expr, $producer: expr) => {{
        let ty = bool_type($producer);
        fr_binary_op!($name, $producer, ty, ty)
    }};
}

macro_rules! fr_binary_op_bigint_to_bool {
    ($name: expr, $producer: expr) => {{
        fr_binary_op!($name, $producer, bigint_type($producer), bool_type($producer))
    }};
}

pub fn add_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_ADD_FN_NAME, producer);
    let add = create_add(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, add.into_int_value());
}

pub fn sub_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_SUB_FN_NAME, producer);
    let add = create_sub(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, add.into_int_value());
}

pub fn mul_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_MUL_FN_NAME, producer);
    let add = create_mul(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, add.into_int_value());
}

pub fn div_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_DIV_FN_NAME, producer);
    let div = create_div(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, div.into_int_value());
}

pub fn eq_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint_to_bool!(FR_EQ_FN_NAME, producer);
    let eq = create_eq(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, eq.into_int_value());
}

pub fn neq_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint_to_bool!(FR_NEQ_FN_NAME, producer);
    let neq = create_neq(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, neq.into_int_value());
}

pub fn lt_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint_to_bool!(FR_LT_FN_NAME, producer);
    let res = create_lt(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn gt_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint_to_bool!(FR_GT_FN_NAME, producer);
    let res = create_gt(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn le_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint_to_bool!(FR_LE_FN_NAME, producer);
    let res = create_le(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn ge_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint_to_bool!(FR_GE_FN_NAME, producer);
    let res = create_ge(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn neg_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let arg = fr_unary_op!(FR_NEG_FN_NAME, producer, bigint_type(producer));
    let neg = create_neg(producer, arg.into_int_value());
    create_return(producer, neg.into_int_value());
}

pub fn shl_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_SHL_FN_NAME, producer);
    let res = create_shl(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn shr_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_SHR_FN_NAME, producer);
    let res = create_shr(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn bit_and_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_BITAND_FN_NAME, producer);
    let res = create_bit_and(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn bit_or_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_BITOR_FN_NAME, producer);
    let res = create_bit_or(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn bit_xor_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bigint!(FR_BITXOR_FN_NAME, producer);
    let res = create_bit_xor(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn logic_and_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bool!(FR_LAND_FN_NAME, producer);
    let res = create_logic_and(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn logic_or_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op_bool!(FR_LOR_FN_NAME, producer);
    let res = create_logic_or(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn logic_not_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let arg = fr_unary_op!(FR_LNOT_FN_NAME, producer, bool_type(producer));
    let res = create_logic_not(producer, arg.into_int_value());
    create_return(producer, res.into_int_value());
}

pub fn load_fr<'a>(producer: &dyn LLVMIRProducer<'a>) {
    add_fn(producer);
    sub_fn(producer);
    mul_fn(producer);
    div_fn(producer);
    eq_fn(producer);
    neq_fn(producer);
    lt_fn(producer);
    gt_fn(producer);
    le_fn(producer);
    ge_fn(producer);
    neg_fn(producer);
    shl_fn(producer);
    shr_fn(producer);
    bit_and_fn(producer);
    bit_or_fn(producer);
    bit_xor_fn(producer);
    logic_and_fn(producer);
    logic_or_fn(producer);
    logic_not_fn(producer);
}
