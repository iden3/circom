use inkwell::basic_block::BasicBlock;
use inkwell::IntPredicate::{EQ, NE, SLT};
use inkwell::types::AnyTypeEnum;
use inkwell::values::{AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValue, BasicValueEnum, InstructionValue, IntMathValue, IntValue, PhiValue, PointerValue};
use crate::llvm_elements::LLVMIRProducer;

pub fn create_add_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_add(lhs, rhs, name).as_any_value_enum()
}

pub fn create_add<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_add_with_name(producer, lhs, rhs, "")
}

pub fn create_sub_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_sub(lhs, rhs, name).as_any_value_enum()
}

pub fn create_sub<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_sub_with_name(producer, lhs, rhs, "")
}

pub fn create_mul_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_mul(lhs, rhs, name).as_any_value_enum()
}

pub fn create_mul<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_mul_with_name(producer, lhs, rhs, "")
}

pub fn create_div_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_unsigned_div(lhs, rhs, name).as_any_value_enum()
}

pub fn create_div<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_div_with_name(producer, lhs, rhs, "")
}

pub fn create_return<'a, V: BasicValue<'a>>(producer: & dyn LLVMIRProducer<'a>, val: V) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_return(Some(&val)).as_any_value_enum()
}

pub fn create_eq_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_compare(EQ, lhs, rhs, name).as_any_value_enum()
}

pub fn create_eq<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_eq_with_name(producer, lhs, rhs, "")
}

pub fn create_neq_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_compare(NE, lhs, rhs, name).as_any_value_enum()
}

pub fn create_neq<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_neq_with_name(producer, lhs, rhs, "")
}

pub fn create_ls_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_compare(SLT, lhs, rhs, name).as_any_value_enum()
}

pub fn create_ls<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, lhs: T, rhs: T) -> AnyValueEnum<'a> {
    create_ls_with_name(producer, lhs, rhs, "")
}

pub fn create_neg_with_name<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, v: T, name: &str) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_int_neg(v, name).as_any_value_enum()
}

pub fn create_neg<'a, T: IntMathValue<'a>>(producer: & dyn LLVMIRProducer<'a>, v: T) -> AnyValueEnum<'a> {
    create_neg_with_name(producer, v, "")
}

pub fn create_store<'a>(producer: & dyn LLVMIRProducer<'a>, ptr: PointerValue<'a>, value: AnyValueEnum<'a>) -> AnyValueEnum<'a> {
    match value {
        AnyValueEnum::ArrayValue(v) => producer.llvm().builder.build_store(ptr, v),
        AnyValueEnum::IntValue(v) => producer.llvm().builder.build_store(ptr, v),
        AnyValueEnum::FloatValue(v)  => producer.llvm().builder.build_store(ptr, v),
        AnyValueEnum::PointerValue(v) => producer.llvm().builder.build_store(ptr, v),
        AnyValueEnum::StructValue(v) => producer.llvm().builder.build_store(ptr, v),
        AnyValueEnum::VectorValue(v) => producer.llvm().builder.build_store(ptr, v),
        _ => panic!("We cannot create a store from a non basic value! There is a bug somewhere.")
    }.as_any_value_enum()
}

pub fn create_return_void<'a>(producer: & dyn LLVMIRProducer<'a>) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_return(None).as_any_value_enum()
}

pub fn create_br<'a>(producer: & dyn LLVMIRProducer<'a>, bb: BasicBlock<'a>) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_unconditional_branch(bb).as_any_value_enum()
}

pub fn create_call<'a>(producer: & dyn LLVMIRProducer<'a>, name: &str, arguments: &[BasicMetadataValueEnum<'a>]) -> AnyValueEnum<'a> {
    let f = producer.llvm().module.get_function(name).expect(format!("Cannot find function {}", name).as_str());
    producer.llvm().builder.build_call(f, arguments, format!("call.{}", name).as_str()).as_any_value_enum()
}

pub fn create_conditional_branch<'a>(producer: & dyn LLVMIRProducer<'a>, comparison: IntValue<'a>, then_block: BasicBlock<'a>, else_block: BasicBlock< 'a>) -> AnyValueEnum<'a> {
    producer.llvm().builder.build_conditional_branch(comparison, then_block, else_block).as_any_value_enum()
}

pub fn create_return_from_any_value<'a>(producer: & dyn LLVMIRProducer<'a>, val: AnyValueEnum<'a>) -> AnyValueEnum<'a> {
    match val {
        AnyValueEnum::ArrayValue(x) => create_return(producer, x),
        AnyValueEnum::IntValue(x) => create_return(producer, x),
        AnyValueEnum::FloatValue(x) => create_return(producer, x),
        AnyValueEnum::PointerValue(x) => create_return(producer, x),
        AnyValueEnum::StructValue(x) => create_return(producer, x),
        AnyValueEnum::VectorValue(x) => create_return(producer, x),
        _ => panic!("Cannot create a return from a non basic value!")
    }
}

pub fn create_alloca<'a>(producer: & dyn LLVMIRProducer<'a>, ty: AnyTypeEnum<'a>, name: &str) -> AnyValueEnum<'a> {
    match ty {
        AnyTypeEnum::ArrayType(ty) => producer.llvm().builder.build_alloca(ty, name),
        AnyTypeEnum::FloatType(ty) => producer.llvm().builder.build_alloca(ty, name),
        AnyTypeEnum::IntType(ty) => producer.llvm().builder.build_alloca(ty, name),
        AnyTypeEnum::PointerType(ty) => producer.llvm().builder.build_alloca(ty, name),
        AnyTypeEnum::StructType(ty) => producer.llvm().builder.build_alloca(ty, name),
        AnyTypeEnum::VectorType(ty) => producer.llvm().builder.build_alloca(ty, name),
        AnyTypeEnum::FunctionType(_) => panic!("We cannot allocate a function type!"),
        AnyTypeEnum::VoidType(_) => panic!("We cannot allocate a void type!")
    }.as_any_value_enum()
}

pub fn create_phi<'a>(producer: & dyn LLVMIRProducer<'a>, ty: AnyTypeEnum<'a>, name: &str) -> PhiValue<'a> {
    match ty {
        AnyTypeEnum::ArrayType(ty) => producer.builder().build_phi(ty, name),
        AnyTypeEnum::FloatType(ty) => producer.builder().build_phi(ty, name),
        AnyTypeEnum::IntType(ty) => producer.builder().build_phi(ty, name),
        AnyTypeEnum::PointerType(ty) => producer.builder().build_phi(ty, name),
        AnyTypeEnum::StructType(ty) => producer.builder().build_phi(ty, name),
        AnyTypeEnum::VectorType(ty) => producer.builder().build_phi(ty, name),
        _ => panic!("Cannot create a phi node with anything other than a basic type! {}", ty)
    }
}

pub fn create_phi_with_incoming<'a>(producer: & dyn LLVMIRProducer<'a>, ty: AnyTypeEnum<'a>, incoming: &[(BasicValueEnum<'a>, BasicBlock<'a>)], name: &str) -> PhiValue<'a> {
    let phi = create_phi(producer, ty, name);
    // Hack to add the incoming to the phi value
    phi.add_incoming_as_enum(incoming);
    phi
}

pub fn create_load_with_name<'a>(producer: & dyn LLVMIRProducer<'a>, ptr: PointerValue<'a>, name: &str) -> AnyValueEnum<'a> {
    producer.builder().build_load(ptr, name).as_any_value_enum()
}

pub fn create_load<'a>(producer: & dyn LLVMIRProducer<'a>, ptr: PointerValue<'a>) -> AnyValueEnum<'a> {
    create_load_with_name(producer, ptr, "")
}

pub fn get_instruction_arg(inst: InstructionValue, idx: u32) -> AnyValueEnum {
    let r = inst.get_operand(idx).unwrap();
    if r.is_left() {
        r.unwrap_left().as_any_value_enum()
    } else {
        r.unwrap_right().get_last_instruction().unwrap().as_any_value_enum()
    }
}