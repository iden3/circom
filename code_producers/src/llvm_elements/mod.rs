use std::cell::RefCell;

use std::convert::TryFrom;
use std::rc::Rc;
use ansi_term::Colour;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::{Context, ContextRef};
use inkwell::module::Module;
use inkwell::types::{AnyTypeEnum, BasicType, BasicTypeEnum, IntType};
use inkwell::values::{AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValueEnum, IntValue};
use inkwell::values::FunctionValue;

use template::TemplateCtx;

use crate::llvm_elements::types::bool_type;
pub use inkwell::types::AnyType;
pub use inkwell::values::AnyValue;
use crate::llvm_elements::instructions::create_alloca;

pub mod stdlib;
pub mod template;
pub mod types;
pub mod functions;
pub mod instructions;
pub mod fr;
pub mod values;

pub type LLVMInstruction<'a> = AnyValueEnum<'a>;

pub trait BodyCtx<'a> {
    fn get_variable(
        &self,
        producer: &dyn LLVMIRProducer<'a>,
        index: IntValue<'a>,
    ) -> AnyValueEnum<'a>;
}

pub trait LLVMIRProducer<'a> {
    fn llvm(&self) -> &LLVM<'a>;
    fn context(&self) -> ContextRef<'a>;
    fn set_current_bb(&self, bb: BasicBlock<'a>);
    fn template_ctx(&self) -> &TemplateCtx<'a>;
    fn body_ctx(&self) -> &dyn BodyCtx<'a>;
    fn current_function(&self) -> FunctionValue<'a>;
    fn builder(&self) -> &Builder<'a>;
    fn constant_fields(&self) -> &Vec<String>;
    fn get_template_mem_arg(&self, run_fn: FunctionValue<'a>) -> ArrayValue<'a>;
}

#[derive(Default, Eq, PartialEq, Debug)]
pub struct LLVMCircuitData {
    pub field_tracking: Vec<String>,
}

pub struct TopLevelLLVMIRProducer<'a> {
    pub context: &'a Context,
    current_module: LLVM<'a>,
    pub field_tracking: Vec<String>,
}

impl<'a> LLVMIRProducer<'a> for TopLevelLLVMIRProducer<'a> {
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

    fn body_ctx(&self) -> &dyn BodyCtx<'a> {
        panic!("The top level llvm producer does not hold a body context!");
    }

    fn current_function(&self) -> FunctionValue<'a> {
        panic!("The top level llvm producer does not have a current function");
    }

    fn builder(&self) -> &Builder<'a> {
        &self.llvm().builder
    }

    fn constant_fields(&self) -> &Vec<String> {
        &self.field_tracking
    }

    fn get_template_mem_arg(&self, _run_fn: FunctionValue<'a>) -> ArrayValue<'a> {
        panic!("The top level llvm producer can't extract the template argument of a run function!");
    }
}

impl<'a> TopLevelLLVMIRProducer<'a> {
    pub fn write_to_file(&self, path: &str) -> Result<(), ()> {
        self.current_module.write_to_file(path)
    }
}

pub fn create_context() -> Context {
    Context::create()
}

impl<'a> TopLevelLLVMIRProducer<'a> {
    pub fn new(context: &'a Context, name: &str, field_tracking: Vec<String>) -> Self {
        TopLevelLLVMIRProducer {
            context,
            current_module: LLVM::from_context(context, name),
            field_tracking,
        }
    }
}

pub type LLVMAdapter<'a> = &'a Rc<RefCell<LLVM<'a>>>;
pub type BigIntType<'a> = IntType<'a>; // i256



pub fn new_constraint<'a>(producer: &dyn LLVMIRProducer<'a>) -> AnyValueEnum<'a> {
    let alloca = create_alloca(producer, bool_type(producer).into(), "constraint");
    let s = producer.context().metadata_string("constraint");
    let kind = producer.context().get_kind_id("constraint");
    let node = producer.context().metadata_node(&[s.into()]);
    alloca.into_pointer_value().as_instruction().unwrap().set_metadata(node, kind).expect("Could not setup metadata marker for constraint value");
    alloca
}

#[inline]
pub fn any_value_wraps_basic_value(v: AnyValueEnum) -> bool {
    match BasicValueEnum::try_from(v) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[inline]
pub fn any_value_to_basic(v: AnyValueEnum) -> BasicValueEnum {
    BasicValueEnum::try_from(v).expect("Attempted to convert a non basic value!")
}

#[inline]
pub fn to_enum<'a, T: AnyValue<'a>>(v: T) -> AnyValueEnum<'a> {
    v.as_any_value_enum()
}

#[inline]
pub fn to_basic_enum<'a, T: AnyValue<'a>>(v: T) -> BasicValueEnum<'a> {
    any_value_to_basic(to_enum(v))
}

#[inline]
pub fn to_basic_metadata_enum(value: AnyValueEnum) -> BasicMetadataValueEnum {
    match BasicMetadataValueEnum::try_from(value) {
        Ok(v) => v,
        Err(_) => {
            panic!("Attempted to convert a value that does not support BasicMetadataValueEnum")
        }
    }
}

#[inline]
pub fn to_type_enum<'a, T: AnyType<'a>>(ty: T) -> AnyTypeEnum<'a> {
    ty.as_any_type_enum()
}

#[inline]
pub fn to_basic_type_enum<'a, T: BasicType<'a>>(ty: T) -> BasicTypeEnum<'a> {
    ty.as_basic_type_enum()
}

pub struct LLVM<'a> {
    module: Module<'a>,
    builder: Builder<'a>,
}

impl<'a> LLVM<'a> {
    pub fn from_context(context: &'a Context, name: &str) -> Self {
        LLVM {
            module: context.create_module(name),
            builder: context.create_builder(),
        }
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), ()> {
        // Run module verification
        self.module.verify().map_err(|llvm_err| {
            eprintln!("{}: {}", Colour::Red.paint("LLVM Module verification failed"), llvm_err.to_string());
            eprintln!("Generated LLVM:");
            self.module.print_to_stderr();
        })?;
        // Verify that bitcode can be written, parsed, and re-verified
        {
            let buff = self.module.write_bitcode_to_memory();
            let context = Context::create();
            let new_module =
                Module::parse_bitcode_from_buffer(&buff, &context).map_err(|llvm_err| {
                    eprintln!(
                        "{}: {}",
                        Colour::Red.paint("Parsing LLVM bitcode from verification buffer failed"),
                        llvm_err.to_string()
                    );
                })?;
            new_module.verify().map_err(|llvm_err| {
                eprintln!(
                    "{}: {}",
                    Colour::Red.paint("LLVM bitcode verification failed"),
                    llvm_err.to_string()
                );
            })?;
        }
        // Write the output to file
        self.module.print_to_file(path).map_err(|llvm_err| {
            eprintln!("{}: {}", Colour::Red.paint("Writing LLVM Module failed"), llvm_err.to_string());
        })
    }
}

pub fn run_fn_name(name: String) -> String {
    format!("{}_run", name)
}

pub fn build_fn_name(name: String) -> String {
    format!("{}_build", name)
}
