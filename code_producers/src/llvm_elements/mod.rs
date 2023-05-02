pub mod llvm_code_generator;
pub mod template;
pub mod types;
pub mod functions;
pub mod instructions;
pub mod fr;
pub mod values;

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::{AggregateValue, AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, GlobalValue, InstructionValue, IntMathValue, IntValue, PhiValue, PointerValue};
use inkwell::context::{Context, ContextRef};
use inkwell::IntPredicate::{EQ, NE, SLT};
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::types::{AnyType, AnyTypeEnum, ArrayType, FunctionType, IntType, PointerType, StringRadix, VoidType};
use template::TemplateCtx;
use crate::llvm_elements::template::TemplateLLVMIRProducer;
use crate::llvm_elements::types::bool_type;
use crate::llvm_elements::values::ValueProducer;

pub type LLVMInstruction<'a> = AnyValueEnum<'a>;
pub trait LLVMIRProducer<'a>: LLVMContext<'a> + ValueProducer<'a> {}

pub trait LLVMContext<'a> {
    fn llvm(&self) -> &LLVM<'a>;
    fn context(&self) -> ContextRef<'a>;
    fn set_current_bb(&self, bb: BasicBlock<'a>);
    fn template_ctx(&self) -> &TemplateCtx<'a>;
    fn current_function(&self) -> FunctionValue<'a>;
    fn builder(&self) -> &Builder<'a>;
    fn constant_fields(&self) -> Option<GlobalValue<'a>>;
}

#[derive(Default)]
pub struct LLVMCircuitData {
    pub field_tracking: Vec<String>,
}

pub struct TopLevelLLVMIRProducer<'a> {
    pub context: &'a Context,
    current_module: LLVM<'a>
}

impl<'a> LLVMContext<'a> for TopLevelLLVMIRProducer<'a> {
    fn llvm(&self) -> &LLVM<'a> {
        &self.current_module
    }

    fn context(&self) -> ContextRef<'a> {
        self.current_module.module.get_context()
    }

    fn set_current_bb(&self, bb: BasicBlock<'a>) {
        self.llvm().builder.position_at_end(bb);
    }

    fn template_ctx(&self) -> &TemplateCtx<'a> {
        panic!("The top level llvm producer does not hold a template context!");
    }

    fn current_function(&self) -> FunctionValue<'a> {
        panic!("The top level llvm producer does not have a current function");
    }

    fn builder(&self) -> &Builder<'a> {
        &self.llvm().builder
    }

    fn constant_fields(&self) -> Option<GlobalValue<'a>> {
        self.llvm().constant_fields
    }
}

impl<'a> LLVMIRProducer<'a> for TopLevelLLVMIRProducer<'a> {}
impl<'a> ValueProducer<'a> for TopLevelLLVMIRProducer<'a> {}

impl<'a> TopLevelLLVMIRProducer<'a> {
    pub fn write_to_file(&self, path: &str) -> Result<(), ()> {
        self.current_module.write_to_file(path)
    }
}

pub fn create_context() -> Context {
    Context::create()
}

impl<'a> TopLevelLLVMIRProducer<'a> {
    pub fn new(context: &'a Context, name: &str) -> Self {
        TopLevelLLVMIRProducer {
            context,
            current_module: LLVM::from_context(context, name)
        }
    }
}

pub type LLVMAdapter<'a> = &'a Rc<RefCell<LLVM<'a>>>;
pub type BigIntType<'a> = IntType<'a>; // i256

pub struct LLVM<'a> {
    module: Module<'a>,
    builder: Builder<'a>,
    template_signals: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    constant_fields: Option<GlobalValue<'a>>,
    stacks: HashMap<usize, PointerValue<'a>>,
    template_variables: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    template_subcomponents: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    constraint_count: u64,
    current_function: Option<FunctionValue<'a>>,
    subcmp_counters: HashMap<usize, HashMap<IntValue<'a>, PointerValue<'a>>>,
    template_ctx: Option<TemplateCtx<'a>>
}

pub fn new_constraint<'a>(producer: &dyn LLVMIRProducer<'a>) -> AnyValueEnum<'a> {
    let v = producer.llvm().module.add_global(bool_type(producer), None, "constraint");
    v.as_any_value_enum()
}

pub fn any_value_wraps_basic_value(v: AnyValueEnum) -> bool {
    match BasicValueEnum::try_from(v) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn any_value_to_basic(v: AnyValueEnum) -> BasicValueEnum {
    BasicValueEnum::try_from(v).expect("Attempted to convert a non basic value!")
}

pub fn to_enum<'a, T: AnyValue<'a>>(v: T) -> AnyValueEnum<'a> {
    v.as_any_value_enum()
}

pub fn to_basic_metadata_enum(value: AnyValueEnum) -> BasicMetadataValueEnum {
    match BasicMetadataValueEnum::try_from(value) {
        Ok(v) => v,
        Err(_) => panic!("Attempted to convert a value that does not support BasicMetadataValueEnum")
    }
}

pub fn to_type_enum<'a, T: AnyType<'a>>(ty: T) -> AnyTypeEnum<'a> {
    ty.as_any_type_enum()
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
            constraint_count: 0,
            current_function: None,
            subcmp_counters: HashMap::new(),
            template_ctx: None
        }
    }

    pub fn merge_module(&self, other: Module<'a>) {
        self.module.link_in_module(other).expect(format!("Cannot merge with {}", self.module.get_name().to_str().unwrap()).as_str());
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), ()> {
        // self.module.print_to_stderr();
        self.module.print_to_file(path).map_err(|_| {})
    }

    pub fn bigint_type(&self) -> BigIntType<'a> {
        self.module.get_context().custom_width_int_type(256)
    }

    pub fn set_current_function(&mut self, func: FunctionValue<'a>) {
        self.current_function = Some(func);
    }

    pub fn create_bb_in_current_function(&self, name: &str) -> BasicBlock<'a> {
        self.module.get_context().append_basic_block(self.current_function.unwrap(), name)
    }

    pub fn set_current_bb(&self, bb: BasicBlock<'a>) {
        self.builder.position_at_end(bb);
    }

    pub fn zero(&self) -> IntValue<'a> {
        self.module.get_context().i32_type().const_zero()
    }

    pub fn create_template_struct(&self, n_signals: usize) -> PointerType<'a> {
        self.bigint_type().array_type(n_signals as u32).ptr_type(Default::default())
    }

    pub fn create_signal_geps(&mut self, id: usize, _n_signals: usize) {
        let signal_ptrs= HashMap::new();
        self.template_signals.insert(id, signal_ptrs);
    }

    pub fn get_signal(&self, id: usize, idx: IntValue<'a>) -> PointerValue<'a> {
        match self.template_signals.get(&id).unwrap().get(&idx) {
            None => {
                let template_arg = self.get_template_arg().unwrap();
                let zero = self.zero();
                let gep = self.create_gep(template_arg, &[zero, idx], "signal.gep.0");
                let ptr = gep.into_pointer_value();
                ptr
            }
            Some(ptr) => *ptr
        }
    }

    pub fn get_variable(&self, _id: usize, idx: IntValue<'a>) -> AnyValueEnum<'a> {
        self.template_ctx.as_ref()
            .expect("Could not find template context")
            .stack.get(&idx)
            .expect(format!("Could not get variable {} from template context", idx).as_str())
            .as_any_value_enum()
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
        unsafe { self.builder.build_gep(ptr, indices, name) }.as_instruction().unwrap().as_any_value_enum()
    }



















    pub fn unwrap_any_value(&self, v: &'a AnyValueEnum<'a>) -> &'a dyn AnyValue<'a> {
        match v {
            AnyValueEnum::ArrayValue(x) => x,
            AnyValueEnum::IntValue(x) => x,
            AnyValueEnum::FloatValue(x) => x,
            AnyValueEnum::PhiValue(x) => x,
            AnyValueEnum::FunctionValue(x) => x,
            AnyValueEnum::PointerValue(x) => x,
            AnyValueEnum::StructValue(x) => x,
            AnyValueEnum::VectorValue(x) => x,
            AnyValueEnum::InstructionValue(x) => x,
            AnyValueEnum::MetadataValue(x) => x
        }
    }

    pub fn unwrap_any_value_into_basic(&self, v: &'a AnyValueEnum<'a>) -> &'a dyn BasicValue<'a> {
        match v {
            AnyValueEnum::ArrayValue(x) => x,
            AnyValueEnum::IntValue(x) => x,
            AnyValueEnum::FloatValue(x) => x,
            AnyValueEnum::PointerValue(x) => x,
            AnyValueEnum::StructValue(x) => x,
            AnyValueEnum::VectorValue(x) => x,
            _ => panic!("Attempted to convert a non basic value!")
        }
    }

    pub fn unwrap_basic_value(&self, v: &'a BasicValueEnum<'a>) -> &'a dyn BasicValue<'a> {
        match v {
            BasicValueEnum::ArrayValue(x) => x,
            BasicValueEnum::IntValue(x) => x,
            BasicValueEnum::FloatValue(x) => x,
            BasicValueEnum::PointerValue(x) => x,
            BasicValueEnum::StructValue(x) => x,
            BasicValueEnum::VectorValue(x) => x
        }
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



    pub fn add_subcomponent(&mut self, id: usize, cmp_id: IntValue<'a>, ptr: PointerValue<'a>, counter: PointerValue<'a>) {
        if !self.template_subcomponents.contains_key(&id) {
            self.template_subcomponents.insert(id, HashMap::new());
        }
        let tmp = self.template_subcomponents.get_mut(&id).unwrap();
        tmp.insert(cmp_id, ptr);
        if !self.subcmp_counters.contains_key(&id) {
            self.subcmp_counters.insert(id, HashMap::new());
        }
        let tmp = self.subcmp_counters.get_mut(&id).unwrap();
        tmp.insert(cmp_id, counter);
    }

    pub fn get_subcomponent(&self, id: usize, cmp_id: IntValue<'a>) -> AnyValueEnum<'a> {
        self.template_subcomponents
            .get(&id)
            .expect("Access to component before initialization!")
            .get(&cmp_id)
            .expect("Access to subcomponent before initialization!")
            .as_any_value_enum()
    }

    pub fn get_subcomponent_counter(&self, id: usize, cmp_id: IntValue<'a>) -> AnyValueEnum<'a> {
        self.subcmp_counters
            .get(&id)
            .expect("Access to component before initialization!")
            .get(&cmp_id)
            .expect("Access to subcomponent before initialization!")
            .as_any_value_enum()
    }


}