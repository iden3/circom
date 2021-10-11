mod build_stack;
mod reduce_stack;
mod set_arena_size;
mod build_inputs_info;
use crate::intermediate_representation::ir_interface::InstructionList;
use std::collections::{HashMap, HashSet};

pub fn reduce_intermediate_operations(code: InstructionList) -> InstructionList {
    reduce_stack::reduce_list(code)
}

pub fn build_auxiliary_stack(code: &mut InstructionList) -> usize {
    build_stack::build_list(code, 0)
}

pub fn set_arena_size_in_calls(
    code: &mut InstructionList,
    function_to_arena_size: &HashMap<String, usize>,
) {
    set_arena_size::visit_list(code, function_to_arena_size);
}

pub fn build_inputs_info(code: &mut InstructionList){
    build_inputs_info::visit_list(code, &mut HashSet::new(), &mut HashSet::new(),false, false);
}

