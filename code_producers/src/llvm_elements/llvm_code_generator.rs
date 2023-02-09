use std::cell::RefCell;
use std::rc::Rc;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use crate::llvm_elements::{LLVMInstruction, LLVMProducer, LLVM, LLVMAdapter};

pub fn create_llvm<'a>(producer: &'a LLVMProducer, name: &str) -> LLVMAdapter<'a> {
    Rc::new(RefCell::new(LLVM::from_context(&producer.context, name)))
}

pub const FR_ADD_FN_NAME: &str = "fr_add";
pub const FR_MUL_FN_NAME: &str = "fr_mul";
pub const FR_EQ_FN_NAME: &str = "fr_eq";

mod fr {
    use std::convert::TryInto;
    use inkwell::values::AnyValue;
    use crate::llvm_elements::{LLVMAdapter, LLVMProducer};
    use crate::llvm_elements::llvm_code_generator::{FR_ADD_FN_NAME, FR_EQ_FN_NAME, FR_MUL_FN_NAME};

    pub fn add_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let bigint_ty = llvm.borrow().bigint_type();
        let args = &[bigint_ty.into(), bigint_ty.into()];
        let func = llvm.borrow().create_function(FR_ADD_FN_NAME, bigint_ty.fn_type(args, false));
        let main = llvm.borrow().create_bb(func, FR_ADD_FN_NAME);
        llvm.borrow().set_current_bb(main);

        let lhs = func.get_nth_param(0).unwrap();
        let rhs = func.get_nth_param(1).unwrap();
        let add = llvm.borrow().create_add(lhs.into_int_value(), rhs.into_int_value(), "");
        let _ = llvm.borrow().create_return(add.into_int_value());
    }

    pub fn mul_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let bigint_ty = llvm.borrow().bigint_type();
        let args = &[bigint_ty.into(), bigint_ty.into()];
        let func = llvm.borrow().create_function(FR_MUL_FN_NAME, bigint_ty.fn_type(args, false));
        let main = llvm.borrow().create_bb(func, FR_MUL_FN_NAME);
        llvm.borrow().set_current_bb(main);

        let lhs = func.get_nth_param(0).unwrap();
        let rhs = func.get_nth_param(1).unwrap();
        let mul = llvm.borrow().create_mul(lhs.into_int_value(), rhs.into_int_value(), "");
        let _ = llvm.borrow().create_return(mul.into_int_value());
    }

    pub fn eq_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let bigint_ty = llvm.borrow().bigint_type();
        let args = &[bigint_ty.into(), bigint_ty.into()];
        let func = llvm.borrow().create_function(FR_EQ_FN_NAME, llvm.borrow().bool_type().fn_type(args, false));
        let main = llvm.borrow().create_bb(func, FR_EQ_FN_NAME);
        llvm.borrow().set_current_bb(main);

        let lhs = func.get_nth_param(0).unwrap();
        let rhs = func.get_nth_param(1).unwrap();
        let eq = llvm.borrow().create_eq(lhs.into_int_value(), rhs.into_int_value(), "");
        let _ = llvm.borrow().create_return(eq.into_int_value());
    }
}

pub fn load_fr<'a>(producer: &'a LLVMProducer, module: LLVMAdapter<'a>) {
    fr::add_fn(producer, module.clone());
    fr::mul_fn(producer, module.clone());
    fr::eq_fn(producer, module.clone());
}

pub const CONSTRAINT_VALUES_FN_NAME: &str = "__constraint_values";
pub const CONSTRAINT_VALUE_FN_NAME: &str = "__constraint_value";
pub const ASSERT_FN_NAME: &str = "__assert";

mod stdlib {
    use inkwell::types::{AnyType, BasicType};
    use inkwell::values::AnyValue;
    use crate::llvm_elements::{LLVMAdapter, LLVMProducer};
    use crate::llvm_elements::llvm_code_generator::{ASSERT_FN_NAME, CONSTRAINT_VALUE_FN_NAME, CONSTRAINT_VALUES_FN_NAME};

    pub fn constraint_values_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let bigint_ty = llvm.borrow().bigint_type();
        let args = &[bigint_ty.into(), bigint_ty.into(), llvm.borrow().bool_type().ptr_type(Default::default()).into()];
        let void_ty = llvm.borrow().void_type();
        let func = llvm.borrow().create_function(CONSTRAINT_VALUES_FN_NAME, void_ty.fn_type(args, false));
        let main = llvm.borrow().create_bb(func, "main");
        llvm.borrow().set_current_bb(main);

        let lhs = func.get_nth_param(0).unwrap();
        let rhs = func.get_nth_param(1).unwrap();
        let constr = func.get_nth_param(2).unwrap();

        let cmp = llvm.borrow().create_eq(lhs.into_int_value(), rhs.into_int_value(), "");
        let _ = llvm.borrow().create_store(constr.into_pointer_value(), cmp);
        let _ = llvm.borrow().create_return_void();
    }

    pub fn constraint_value_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let args = &[llvm.borrow().bool_type().into(), llvm.borrow().bool_type().ptr_type(Default::default()).into()];
        let void_ty = llvm.borrow().void_type();
        let func = llvm.borrow().create_function(CONSTRAINT_VALUE_FN_NAME, void_ty.fn_type(args, false));
        let main = llvm.borrow().create_bb(func, "main");
        llvm.borrow().set_current_bb(main);

        let bool = func.get_nth_param(0).unwrap();
        let constr = func.get_nth_param(1).unwrap();

        let _ = llvm.borrow().create_store(constr.into_pointer_value(), bool.as_any_value_enum());
        let _ = llvm.borrow().create_return_void();
    }

    pub fn assert_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let func = llvm.borrow().create_function(ASSERT_FN_NAME, llvm.borrow().void_type().fn_type(&[llvm.borrow().bool_type().into()], false));
        let main = llvm.borrow().create_bb(func, "main");
        let if_false = llvm.borrow().create_bb(func, "if.assert.fails");
        let end = llvm.borrow().create_bb(func, "end");
        let bool = func.get_nth_param(0).unwrap();
        llvm.borrow().set_current_bb(main);
        let _ = llvm.borrow().create_conditional_branch(bool.into_int_value(), end, if_false);
        llvm.borrow().set_current_bb(if_false);
        let _ = llvm.borrow().create_call("__abort", &[]);
        let _ = llvm.borrow().create_br(end);
        llvm.borrow().set_current_bb(end);
        let _ = llvm.borrow().create_return_void();
    }

    pub fn abort_declared_fn<'a>(_producer: &'a LLVMProducer, llvm: LLVMAdapter<'a>) {
        let _ = llvm.borrow().create_function("__abort", llvm.borrow().void_type().fn_type(&[], false));
    }
}

pub fn load_stdlib<'a>(producer: &'a LLVMProducer, module: LLVMAdapter<'a>) {
    stdlib::constraint_values_fn(producer, module.clone());
    stdlib::constraint_value_fn(producer, module.clone());
    stdlib::abort_declared_fn(producer, module.clone());
    stdlib::assert_fn(producer, module.clone());
}

pub fn run_fn_name(name: String) -> String {
    format!("{}_run", name)
}

pub fn build_fn_name(name: String) -> String {
    format!("{}_build", name)
}

