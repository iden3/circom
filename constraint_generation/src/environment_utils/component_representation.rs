use super::slice_types::{MemoryError, SignalSlice, SliceCapacity};
use crate::execution_data::type_definitions::NodePointer;
use crate::execution_data::ExecutedProgram;
use std::collections::HashMap;

pub struct ComponentRepresentation {
    pub node_pointer: Option<NodePointer>,
    is_parallel: bool,
    unassigned_inputs: HashMap<String, SliceCapacity>,
    inputs: HashMap<String, SignalSlice>,
    outputs: HashMap<String, SignalSlice>,
}

impl Default for ComponentRepresentation {
    fn default() -> Self {
        ComponentRepresentation {
            node_pointer: Option::None,
            is_parallel: false,
            unassigned_inputs: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
}
impl Clone for ComponentRepresentation {
    fn clone(&self) -> Self {
        ComponentRepresentation {
            node_pointer: self.node_pointer,
            is_parallel: self.is_parallel,
            unassigned_inputs: self.unassigned_inputs.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
        }
    }
}

impl ComponentRepresentation {
    pub fn initialize_component(
        component: &mut ComponentRepresentation,
        is_parallel: bool,
        node_pointer: NodePointer,
        scheme: &ExecutedProgram,
    ) -> Result<(), MemoryError> {
        if component.is_initialized() {
            return Result::Err(MemoryError::AssignmentError);
        }
        let possible_node = ExecutedProgram::get_node(scheme, node_pointer);
        assert!(possible_node.is_some());
        let node = possible_node.unwrap();

        let mut unassigned_inputs = HashMap::new();
        let mut inputs = HashMap::new();
        for (symbol, route) in node.inputs() {
            let signal_slice = SignalSlice::new_with_route(route, &false);
            let signal_slice_size = SignalSlice::get_number_of_cells(&signal_slice);
            if signal_slice_size > 0{
                unassigned_inputs
                    .insert(symbol.clone(), signal_slice_size);
            }
            inputs.insert(symbol.clone(), signal_slice);
        }

        let mut outputs = HashMap::new();
        for (symbol, route) in node.outputs() {
            outputs.insert(symbol.clone(), SignalSlice::new_with_route(route, &true));
        }
        *component = ComponentRepresentation {
            node_pointer: Option::Some(node_pointer),
            is_parallel,
            unassigned_inputs,
            inputs,
            outputs,
        };
        Result::Ok(())
    }
    pub fn signal_has_value(
        component: &ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
    ) -> Result<bool, MemoryError> {
        if component.node_pointer.is_none() {
            return Result::Err(MemoryError::InvalidAccess);
        }
        if component.outputs.contains_key(signal_name) && !component.unassigned_inputs.is_empty() {
            return Result::Err(MemoryError::InvalidAccess);
        }

        let slice = if component.inputs.contains_key(signal_name) {
            component.inputs.get(signal_name).unwrap()
        } else {
            component.outputs.get(signal_name).unwrap()
        };
        let enabled = *SignalSlice::get_reference_to_single_value(slice, access)?;
        Result::Ok(enabled)
    }
    pub fn get_signal(&self, signal_name: &str) -> Result<&SignalSlice, MemoryError> {
        if self.node_pointer.is_none() {
            return Result::Err(MemoryError::InvalidAccess);
        }
        if self.outputs.contains_key(signal_name) && !self.unassigned_inputs.is_empty() {
            return Result::Err(MemoryError::InvalidAccess);
        }

        let slice = if self.inputs.contains_key(signal_name) {
            self.inputs.get(signal_name).unwrap()
        } else {
            self.outputs.get(signal_name).unwrap()
        };
        Result::Ok(slice)
    }

    pub fn assign_value_to_signal(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
    ) -> Result<(), MemoryError> {
        let signal_has_value =
            ComponentRepresentation::signal_has_value(component, signal_name, access)?;
        if signal_has_value {
            return Result::Err(MemoryError::AssignmentError);
        }

        let slice = component.inputs.get_mut(signal_name).unwrap();
        let value = SignalSlice::get_mut_reference_to_single_value(slice, access)?;
        let left = component.unassigned_inputs.get_mut(signal_name).unwrap();
        *left -= 1;
        *value = true;
        if *left == 0 {
            component.unassigned_inputs.remove(signal_name);
        }
        Result::Ok(())
    }
    pub fn is_initialized(&self) -> bool {
        self.node_pointer.is_some()
    }
}
