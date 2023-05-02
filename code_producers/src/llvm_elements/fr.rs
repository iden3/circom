use crate::llvm_elements::LLVMIRProducer;
use crate::llvm_elements::functions::create_bb;
use crate::llvm_elements::functions::create_function;
use crate::llvm_elements::instructions::{create_add, create_div, create_eq, create_ls, create_mul, create_neg, create_neq, create_return};
use crate::llvm_elements::types::bigint_type;

pub const FR_ADD_FN_NAME: &str = "fr_add";
pub const FR_MUL_FN_NAME: &str = "fr_mul";
pub const FR_EQ_FN_NAME: &str = "fr_eq";
pub const FR_LS_FN_NAME: &str = "fr_ls";
pub const FR_NEQ_FN_NAME: &str = "fr_neq";
pub const FR_DIV_FN_NAME: &str = "fr_div";
pub const FR_NEG_FN_NAME: &str = "fr_neg";

macro_rules! fr_binary_op {
        ($name: expr, $producer: expr) => (
            {
                let bigint_ty = bigint_type($producer);
                let args = &[bigint_ty.into(), bigint_ty.into()];
                let func = create_function($producer, $name, bigint_ty.fn_type(args, false));
                let main = create_bb($producer, func, $name);
                $producer.set_current_bb(main);

                let lhs = func.get_nth_param(0).unwrap();
                let rhs = func.get_nth_param(1).unwrap();
                (lhs, rhs)
            }
        )
    }

macro_rules! fr_unary_op {
        ($name: expr, $producer: expr) => (
            {
                let bigint_ty = bigint_type($producer);
                let args = &[bigint_ty.into()];
                let func = create_function($producer, $name, bigint_ty.fn_type(args, false));
                let main = create_bb($producer, func, $name);
                $producer.set_current_bb(main);

                let lhs = func.get_nth_param(0).unwrap();
                lhs
            }
        )
    }

pub fn add_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op!(FR_ADD_FN_NAME, producer);
    let add = create_add(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, add.into_int_value());
}

pub fn mul_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op!(FR_MUL_FN_NAME, producer);
    let add = create_mul(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, add.into_int_value());
}

pub fn div_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op!(FR_DIV_FN_NAME, producer);
    let div = create_div(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, div.into_int_value());
}

pub fn eq_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op!(FR_EQ_FN_NAME, producer);
    let eq = create_eq(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, eq.into_int_value());
}

pub fn neq_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op!(FR_NEQ_FN_NAME, producer);
    let neq = create_neq(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, neq.into_int_value());
}

pub fn ls_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let (lhs, rhs) = fr_binary_op!(FR_LS_FN_NAME, producer);
    let ls = create_ls(producer, lhs.into_int_value(), rhs.into_int_value());
    create_return(producer, ls.into_int_value());
}

pub fn neg_fn<'a>(producer: &dyn LLVMIRProducer<'a>) {
    let arg = fr_unary_op!(FR_NEG_FN_NAME, producer);
    let neg = create_neg(producer, arg.into_int_value());
    create_return(producer, neg.into_int_value());
}


pub fn load_fr<'a>(producer: &dyn LLVMIRProducer<'a>) {
    add_fn(producer);
    mul_fn(producer);
    eq_fn(producer);
    ls_fn(producer);
    neq_fn(producer);
    div_fn(producer);
    neg_fn(producer);
}
