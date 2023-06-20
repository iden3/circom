pub mod mutable_env;
pub mod immutable_env;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;



pub type TemplatesLibrary = Rc<RefCell<HashMap<String, TemplateCode>>>;
pub type FunctionsLibrary = Rc<RefCell<HashMap<String, FunctionCode>>>;
