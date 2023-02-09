pub mod llvm_code_generator;

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::Map;
use std::rc::Rc;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::{AggregateValue, AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, GlobalValue, InstructionValue, IntMathValue, IntValue, PointerValue, StructValue};
use inkwell::context::{Context, ContextRef};
use inkwell::IntPredicate::EQ;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::types::{AnyType, AnyTypeEnum, BasicType, FunctionType, IntType, PointerType, StringRadix, StructType, VoidType};

use crate::components::*;


pub type LLVMInstruction<'a> = AnyValueEnum<'a>;

pub struct LLVMProducer {
    pub context: Box<Context>,
    pub field_tracking: Vec<String>,
}

impl Default for LLVMProducer {
    fn default() -> Self {
        let context = Box::new(Context::create());
        LLVMProducer {
            context,
            field_tracking: vec![]
        }
    }
}

pub type LLVMAdapter<'a> = Rc<RefCell<LLVM<'a>>>;
pub type BigIntType<'a> = IntType<'a>; // i256

pub struct LLVM<'a> {
    module: Module<'a>,
    builder: Builder<'a>,
    template_signals: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    constant_fields: Option<GlobalValue<'a>>,
    stacks: HashMap<usize, PointerValue<'a>>,
    template_variables: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    template_subcomponents: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    constraint_count: u64
}

impl<'a> LLVM<'a> {
    pub fn from_context(context: &'a Context, name: &str) -> Self {
        LLVM {
            module: context.create_module(name),
            builder: context.create_builder(),
            template_signals: HashMap::new(),
            constant_fields: None,
            stacks: HashMap::new(),
            template_variables: HashMap::new(),
            template_subcomponents: HashMap::new(),
            constraint_count: 0
        }
    }

    pub fn merge_module(&self, other: Module<'a>) {
        self.module.link_in_module(other).expect(format!("Cannot merge with {}", self.module.get_name().to_str().unwrap()).as_str());
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), ()> {
        self.module.print_to_stderr();
        self.module.print_to_file(path).map_err(|_| {})
    }

    pub fn bool_type(&self) -> IntType<'a> {
        self.module.get_context().bool_type()
    }
    pub fn void_type(&self) -> VoidType<'a> {
        self.module.get_context().void_type()
    }
    pub fn i32_type(&self) -> IntType<'a> {
        self.module.get_context().i32_type()
    }
    pub fn bigint_type(&self) -> BigIntType<'a> {
        self.module.get_context().custom_width_int_type(256)
    }

    pub fn create_function(&self, name: &str, ty: FunctionType<'a>) -> FunctionValue<'a> {
        self.module.add_function(name, ty, None)
    }

    pub fn create_bb(&self, func: FunctionValue<'a>, name: &str) -> BasicBlock<'a> {
       self.module.get_context().append_basic_block(func, name)
    }

    pub fn set_current_bb(&self, bb: BasicBlock<'a>) {
        self.builder.position_at_end(bb);
    }

    pub fn create_literal_u32(&self, val: u64) -> AnyValueEnum<'a> {
        self.module.get_context().i32_type().const_int(val, false).as_any_value_enum()
    }

    pub fn zero(&self) -> IntValue<'a> {
        self.module.get_context().i32_type().const_zero()
    }

    pub fn create_template_struct(&self, n_signals: usize) -> PointerType<'a> {
        let context = self.module.get_context();
        let fields: Vec<_> = (0..n_signals).map(|_| {
            self.bigint_type().as_basic_type_enum()
        }).collect();
        let str = context.struct_type(&fields, false);
        let ptr = str.ptr_type(Default::default());
        ptr
    }

    pub fn create_signal_geps(&mut self, id: usize, n_signals: usize) {
        let mut signal_ptrs= HashMap::new();
        let template_arg = self.get_template_arg().unwrap();
        let zero = self.create_literal_u32(0);
        for i in 0..n_signals {
            let idx = self.create_literal_u32(i as u64);
            let gep = self.create_gep(template_arg, &[zero.into_int_value(), idx.into_int_value()], "");
            signal_ptrs.insert(idx.into_int_value(), gep.into_pointer_value());
        }
        self.template_signals.insert(id, signal_ptrs);
    }

    pub fn get_signal(&self, id: usize, idx: IntValue<'a>) -> PointerValue<'a> {
        match self.template_signals.get(&id).unwrap().get(&idx) {
            None => panic!("Signal value not found!"),
            Some(ptr) => *ptr
        }
    }

    pub fn get_variable(&self, id: usize, idx: IntValue<'a>) -> AnyValueEnum<'a> {
        self.template_variables.get(&id).unwrap().get(&idx).unwrap().as_any_value_enum()
    }

    fn template_arg_id(&self) -> u32 {
        0
    }

    fn get_template_arg(&self) -> Option<PointerValue<'a>> {
        if let Some(bb) = self.builder.get_insert_block() {
            if let Some(func) = bb.get_parent() {
                if let Some(val) = func.get_nth_param(self.template_arg_id()) {
                    return Some(val.into_pointer_value());
                }
            }
        }
        None
    }

    pub fn create_gep(&self, ptr: PointerValue<'a>, indices: &[IntValue<'a>], name: &str) -> AnyValueEnum<'a> {
        unsafe { self.builder.build_in_bounds_gep(ptr, indices, name) }.as_instruction().unwrap().as_any_value_enum()
    }

    pub fn create_load(&self, ptr: PointerValue<'a>, name: &str) -> AnyValueEnum<'a> {
        self.builder.build_load(ptr, name).as_any_value_enum()
    }

    pub fn create_return_void(&self) -> AnyValueEnum<'a> {
        self.builder.build_return(None).as_any_value_enum()
    }

    pub fn create_return<V: BasicValue<'a>>(&self, val: V) -> AnyValueEnum<'a> {
        self.builder.build_return(Some(&val)).as_any_value_enum()
    }

    pub fn create_br(&self, bb: BasicBlock<'a>) -> AnyValueEnum<'a> {
        self.builder.build_unconditional_branch(bb).as_any_value_enum()
    }

    pub fn create_alloca(&self, ty: AnyTypeEnum<'a>, name: &str) -> AnyValueEnum<'a> {
        match ty {
            AnyTypeEnum::ArrayType(ty) => self.builder.build_alloca(ty, name),
            AnyTypeEnum::FloatType(ty) => self.builder.build_alloca(ty, name),
            AnyTypeEnum::IntType(ty) => self.builder.build_alloca(ty, name),
            AnyTypeEnum::PointerType(ty) => self.builder.build_alloca(ty, name),
            AnyTypeEnum::StructType(ty) => self.builder.build_alloca(ty, name),
            AnyTypeEnum::VectorType(ty) => self.builder.build_alloca(ty, name),
            AnyTypeEnum::FunctionType(ty) => panic!("We cannot allocate a function type!"),
            AnyTypeEnum::VoidType(ty) => panic!("We cannot allocate a void type!")
        }.as_any_value_enum()
    }

    pub fn create_store(&self, ptr: PointerValue<'a>, value: AnyValueEnum<'a>) -> AnyValueEnum<'a> {
        match value {
            AnyValueEnum::ArrayValue(v) => self.builder.build_store(ptr, v),
            AnyValueEnum::IntValue(v) => self.builder.build_store(ptr, v),
            AnyValueEnum::FloatValue(v)  => self.builder.build_store(ptr, v),
            AnyValueEnum::PointerValue(v) => self.builder.build_store(ptr, v),
            AnyValueEnum::StructValue(v) => self.builder.build_store(ptr, v),
            AnyValueEnum::VectorValue(v) => self.builder.build_store(ptr, v),
            _ => panic!("We cannot create a store from a non basic value! There is a bug somewhere.")
        }.as_any_value_enum()
    }

    pub fn create_eq<T: IntMathValue<'a>>(&self, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
        self.builder.build_int_compare(EQ, lhs, rhs, name).as_any_value_enum()
    }

    pub fn create_add<T: IntMathValue<'a>>(&self, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
        self.builder.build_int_add(lhs, rhs, name).as_any_value_enum()
    }

    pub fn create_mul<T: IntMathValue<'a>>(&self, lhs: T, rhs: T, name: &str) -> AnyValueEnum<'a> {
        self.builder.build_int_mul(lhs, rhs, name).as_any_value_enum()
    }

    pub fn create_conditional_branch(&self, comparison: IntValue<'a>, then_block: BasicBlock<'a>, else_block: BasicBlock< 'a>) -> AnyValueEnum<'a> {
        self.builder.build_conditional_branch(comparison, then_block, else_block).as_any_value_enum()
    }

    pub fn create_call(&self, name: &str, arguments: &[BasicMetadataValueEnum<'a>]) -> AnyValueEnum<'a> {
        let f = self.module.get_function(name).expect(format!("Cannot find function {}", name).as_str());
        self.builder.build_call(f, arguments, "").as_any_value_enum()
    }

    pub fn new_constraint(&mut self) -> AnyValueEnum<'a> {
        let v = self.module.add_global(self.bool_type(), None, format!("constraint.{}", self.constraint_count).as_str());
        self.constraint_count += 1;
        v.as_any_value_enum()
    }

    pub fn to_enum<T: AnyValue<'a>>(&self, v: T) -> AnyValueEnum<'a> {
        v.as_any_value_enum()
    }

    pub fn get_const(&self, value: usize) -> AnyValueEnum<'a> {
        let arr = self.constant_fields.expect("Access to constant before initialization!").get_initializer().unwrap().into_array_value();
        let mut idx = vec![value as u32];
        let gep = arr.const_extract_value(&mut idx);
        gep.as_any_value_enum()
    }

    pub fn create_consts(&mut self, fields: &Vec<String>) -> AnyValueEnum<'a> {
        let bigint_ty = self.bigint_type();
        let vals: Vec<_> = fields.into_iter().map(|f| {
            bigint_ty.const_int_from_string(f.as_str(), StringRadix::Decimal).unwrap()
        }).collect();
        let values_arr = bigint_ty.const_array(&vals);
        let global = self.module.add_global(values_arr.get_type(), None, "constant_fields");
        global.set_initializer(&values_arr);
        self.constant_fields = Some(global);
        global.as_any_value_enum()
    }

    pub fn create_stack(&mut self, id: usize, depth: usize) -> AnyValueEnum<'a> {
        let mut var_ptrs= HashMap::new();
        let bigint_ty = self.bigint_type();
        let stack = self.create_alloca(bigint_ty.array_type(depth as u32).as_any_type_enum(), "stack");
        self.stacks.insert(id, stack.into_pointer_value());
        for i in 0..depth {
            let idx = self.create_literal_u32(i as u64);
            let gep = self.create_gep(stack.into_pointer_value(), &[idx.into_int_value()], "");
            var_ptrs.insert(idx.into_int_value(), gep.into_pointer_value());
        }
        self.template_variables.insert(id, var_ptrs);
        stack
    }

    pub fn to_basic_metadata_enum(&self, value: AnyValueEnum<'a>) -> BasicMetadataValueEnum<'a> {
        match BasicMetadataValueEnum::try_from(value) {
            Ok(v) => v,
            Err(_) => panic!("Attempted to convert a value that does not support BasicMetadataValueEnum")
        }
    }

    pub fn add_subcomponent(&mut self, id: usize, cmp_id: IntValue<'a>, ptr: PointerValue<'a>) {
        if !self.template_subcomponents.contains_key(&id) {
            self.template_subcomponents.insert(id, HashMap::new());
        }
        let mut tmp = self.template_subcomponents.get_mut(&id).unwrap();
        tmp.insert(cmp_id, ptr);
    }

    pub fn get_subcomponent(&self, id: usize, cmp_id: IntValue<'a>) -> AnyValueEnum<'a> {
        self.template_subcomponents
            .get(&id)
            .expect("Access to component before initialization!")
            .get(&cmp_id)
            .expect("Access to subcomponent before initialization!")
            .as_any_value_enum()
    }

    pub fn get_arg(&self, inst: InstructionValue<'a>, idx: u32) -> AnyValueEnum<'a> {
        let r = inst.get_operand(idx).unwrap();
        if r.is_left() {
            r.unwrap_left().as_any_value_enum()
        } else {
            r.unwrap_right().get_last_instruction().unwrap().as_any_value_enum()
        }
    }
}