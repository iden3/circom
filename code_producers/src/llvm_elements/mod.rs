pub mod llvm_code_generator;

use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Map;
use std::rc::Rc;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValue, InstructionValue, IntValue, PointerValue, StructValue};
use inkwell::context::{Context, ContextRef};
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::types::{BasicType, FunctionType, IntType, PointerType, StructType, VoidType};

use crate::components::*;


pub type LLVMInstruction<'a> = AnyValueEnum<'a>;

pub struct LLVMProducer {
    pub context: Box<Context>
}

impl Default for LLVMProducer {
    fn default() -> Self {
        let context = Box::new(Context::create());
        LLVMProducer {
            context
        }
    }
}

pub type ModuleWrapper<'a> = Rc<RefCell<ModuleWrapperStruct<'a>>>;

pub struct ModuleWrapperStruct<'a> {
    module: Module<'a>,
    builder: Builder<'a>,
    template_signals: HashMap<usize, PointerType<'a>>,
    gep_count: usize
}

impl<'a> ModuleWrapperStruct<'a> {
    pub fn from_context(context: &'a Context, name: &str) -> Self {
        ModuleWrapperStruct {
            module: context.create_module(name),
            builder: context.create_builder(),
            template_signals: HashMap::new(),
            gep_count: 0
        }
    }

    pub fn gep_count(&self) -> usize { self.gep_count }
    pub fn inc_geps(&mut self) { self.gep_count+=1; }

    pub fn write_to_file(&self, path: &str) -> Result<(), ()> {
        self.module.print_to_file(path).map_err(|_| {})
    }

    pub fn void_type(&self) -> VoidType<'a> {
        self.module.get_context().void_type()
    }
    pub fn i32_type(&self) -> IntType<'a> {
        self.module.get_context().i32_type()
    }
    pub fn i128_type(&self) -> IntType<'a> {
        self.module.get_context().i128_type()
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

    pub fn create_return(&self, _: Option<i32>) -> InstructionValue<'a> {
        self.builder.build_return(None)
    }

    pub fn create_literal_u32(&self, val: u64) -> AnyValueEnum<'a> {
        self.module.get_context().i32_type().const_int(val, false).as_any_value_enum()
    }

    pub fn create_template_struct(&mut self, id: usize, n_signals: usize) -> PointerType<'a> {
        let context = self.module.get_context();
        let fields: Vec<_> = (0..n_signals).map(|_| {
            context.i128_type().as_basic_type_enum()
        }).collect();
        let str = context.struct_type(&fields, false);
        let ptr = str.ptr_type(Default::default());
        self.template_signals.insert(id, ptr);
        ptr
    }

    pub fn template_arg_id(&self) -> u32 { 0 }

    pub fn get_template_arg(&self) -> Option<PointerValue<'a>> {
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

    pub fn to_enum<T: AnyValue<'a>>(&self, v: T) -> AnyValueEnum<'a> {
        v.as_any_value_enum()
    }
}