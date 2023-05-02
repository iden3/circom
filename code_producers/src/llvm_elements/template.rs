use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::ContextRef;
use inkwell::types::{AnyType, PointerType};
use inkwell::values::{FunctionValue, GlobalValue, IntValue, PointerValue};

use crate::llvm_elements::{LLVM, LLVMContext, LLVMIRProducer};
use crate::llvm_elements::instructions::create_alloca;
use crate::llvm_elements::types::{bigint_type, subcomponent_type};
use crate::llvm_elements::values::ValueProducer;

pub struct TemplateLLVMIRProducer<'ctx: 'prod, 'prod> {
    parent: &'prod dyn LLVMIRProducer<'ctx>,
    template_ctx: TemplateCtx<'ctx>,
}

impl<'a, 'b> LLVMContext<'a> for TemplateLLVMIRProducer<'a, 'b> {
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

    fn constant_fields(&self) -> Option<GlobalValue<'a>> {
        self.parent.constant_fields()
    }
}

impl<'a, 'b> ValueProducer<'a> for TemplateLLVMIRProducer<'a, 'b> {}

impl<'a, 'b> LLVMIRProducer<'a> for TemplateLLVMIRProducer<'a, 'b> {}

impl<'a, 'b> TemplateLLVMIRProducer<'a, 'b> {
    pub fn new(parent: &'b dyn LLVMIRProducer<'a>, stack_depth: usize, number_subcmps: usize, current_function: FunctionValue<'a>, template_type: PointerType<'a>) -> Self {
        TemplateLLVMIRProducer {
            parent,
            template_ctx: TemplateCtx::new(parent, stack_depth, number_subcmps, current_function, template_type),
        }
    }
}

pub struct TemplateCtx<'a> {
    pub stack: HashMap<IntValue<'a>, PointerValue<'a>>,
    pub subcmps: PointerValue<'a>,
    pub current_function: FunctionValue<'a>,
    pub template_type: PointerType<'a>,
}

#[inline]
fn setup_subcmps<'a>(producer: &dyn LLVMIRProducer<'a>, number_subcmps: usize) -> PointerValue<'a> {
    let subcmps_ty = subcomponent_type(producer).ptr_type(Default::default()).array_type(number_subcmps as u32);
    create_alloca(producer, subcmps_ty.as_any_type_enum(), "subcmps").into_pointer_value()
}

#[inline]
fn setup_stack<'a>(producer: &dyn LLVMIRProducer<'a>, stack_depth: usize) -> HashMap<IntValue<'a>, PointerValue<'a>> {
    let mut stack = HashMap::new();
    // TODO: Might need to change this to an array depending on how array variables are handled
    let bigint_ty = bigint_type(producer);
    for i in 0..stack_depth {
        let idx = producer.create_literal_u32(i as u64);
        let alloca = create_alloca(producer, bigint_ty.as_any_type_enum(), format!("var{}", i).as_str());
        stack.insert(idx, alloca.into_pointer_value());
    }
    stack
}

impl<'a> TemplateCtx<'a> {
    pub fn new(producer: &dyn LLVMIRProducer<'a>, stack_depth: usize, number_subcmps: usize, current_function: FunctionValue<'a>, template_type: PointerType<'a>) -> Self {
        TemplateCtx {
            stack: setup_stack(producer, stack_depth),
            subcmps: setup_subcmps(producer, number_subcmps),
            current_function,
            template_type,
        }
    }
}

pub fn create_template_struct<'a>(producer: &dyn LLVMIRProducer<'a>, n_signals: usize) -> PointerType<'a> {
    bigint_type(producer).array_type(n_signals as u32).ptr_type(Default::default())
}