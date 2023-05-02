use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::ContextRef;
use inkwell::types::{AnyType, BasicType, PointerType};
use inkwell::values::{AnyValue, AnyValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue};

use crate::llvm_elements::{LLVM, LLVMIRProducer};
use crate::llvm_elements::instructions::{create_alloca, create_gep};
use crate::llvm_elements::types::{bigint_type, i32_type, subcomponent_type};
use crate::llvm_elements::values::{create_literal_u32, zero};

pub struct TemplateLLVMIRProducer<'ctx: 'prod, 'prod> {
    parent: &'prod dyn LLVMIRProducer<'ctx>,
    template_ctx: TemplateCtx<'ctx>,
}

impl<'a, 'b> LLVMIRProducer<'a> for TemplateLLVMIRProducer<'a, 'b> {
    fn llvm(&self) -> &LLVM<'a> {
        self.parent.llvm()
    }

    fn context(&self) -> ContextRef<'a> {
        self.parent.context()
    }

    fn set_current_bb(&self, bb: BasicBlock<'a>) {
        self.parent.set_current_bb(bb)
    }

    fn template_ctx(&self) -> &TemplateCtx<'a> {
        &self.template_ctx
    }

    fn current_function(&self) -> FunctionValue<'a> {
        self.template_ctx.current_function
    }

    fn builder(&self) -> &Builder<'a> {
        self.parent.builder()
    }

    fn constant_fields(&self) -> &Vec<String> {
        self.parent.constant_fields()
    }
}

impl<'a, 'b> TemplateLLVMIRProducer<'a, 'b> {
    pub fn new(parent: &'b dyn LLVMIRProducer<'a>, stack_depth: usize, number_subcmps: usize, current_function: FunctionValue<'a>, template_type: PointerType<'a>, signals_arg_offset: usize) -> Self {
        TemplateLLVMIRProducer {
            parent,
            template_ctx: TemplateCtx::new(
                parent,
                stack_depth,
                number_subcmps,
                current_function,
                template_type,
                signals_arg_offset
            ),
        }
    }
}

pub struct TemplateCtx<'a> {
    pub stack: HashMap<IntValue<'a>, PointerValue<'a>>,
    subcmps: PointerValue<'a>,
    pub current_function: FunctionValue<'a>,
    pub template_type: PointerType<'a>,
    pub signals_arg_offset: usize
}

#[inline]
fn setup_subcmps<'a>(producer: &dyn LLVMIRProducer<'a>, number_subcmps: usize) -> PointerValue<'a> {
    // [{void*, int} x number_subcmps]
    let signals_ptr = subcomponent_type(producer).ptr_type(Default::default());
    let counter_ty = i32_type(producer);
    let subcmp_ty = producer.context().struct_type(&[signals_ptr.as_basic_type_enum(), counter_ty.as_basic_type_enum()], false);
    let subcmps_ty = subcmp_ty.array_type(number_subcmps as u32);
    create_alloca(producer, subcmps_ty.as_any_type_enum(), "subcmps").into_pointer_value()
}

#[inline]
fn setup_stack<'a>(producer: &dyn LLVMIRProducer<'a>, stack_depth: usize) -> HashMap<IntValue<'a>, PointerValue<'a>> {
    let mut stack = HashMap::new();
    // TODO: Might need to change this to an array depending on how array variables are handled
    let bigint_ty = bigint_type(producer);
    for i in 0..stack_depth {
        let idx = create_literal_u32(producer, i as u64);
        let alloca = create_alloca(producer, bigint_ty.as_any_type_enum(), format!("var{}", i).as_str());
        stack.insert(idx, alloca.into_pointer_value());
    }
    stack
}

impl<'a> TemplateCtx<'a> {
    pub fn new(producer: &dyn LLVMIRProducer<'a>, stack_depth: usize, number_subcmps: usize, current_function: FunctionValue<'a>, template_type: PointerType<'a>, signals_arg_offset: usize) -> Self {
        TemplateCtx {
            stack: setup_stack(producer, stack_depth),
            subcmps: setup_subcmps(producer, number_subcmps),
            current_function,
            template_type,
            signals_arg_offset
        }
    }

    /// Creates the necessary code to load a subcomponent given the expression used as id
    pub fn load_subcmp_addr(&self, producer: &dyn LLVMIRProducer<'a>, id: AnyValueEnum<'a>) -> PointerValue<'a> {
        create_gep(producer, self.subcmps, &[zero(producer), id.into_int_value(), zero(producer)]).into_pointer_value()
    }

    /// Creates the necessary code to load a subcomponent counter given the expression used as id
    pub fn load_subcmp_counter(&self, producer: &dyn LLVMIRProducer<'a>, id: AnyValueEnum<'a>) -> PointerValue<'a> {
        create_gep(producer, self.subcmps, &[zero(producer), id.into_int_value(), create_literal_u32(producer, 1)]).into_pointer_value()
    }

    /// Returns a reference to the local variable associated to the index
    pub fn get_variable(&self, index: IntValue<'a>) -> AnyValueEnum<'a> {
        self.stack.get(&index).unwrap().as_any_value_enum()
    }

    /// Returns a pointer to the signal associated to the index
    pub fn get_signal(&self, producer: &dyn LLVMIRProducer<'a>, index: IntValue<'a>) -> AnyValueEnum<'a> {
        let signals = self.current_function.get_nth_param(self.signals_arg_offset as u32).unwrap();
        create_gep(producer, signals.into_pointer_value(), &[zero(producer), index])
    }
}

pub fn create_template_struct<'a>(producer: &dyn LLVMIRProducer<'a>, n_signals: usize) -> PointerType<'a> {
    bigint_type(producer).array_type(n_signals as u32).ptr_type(Default::default())
}