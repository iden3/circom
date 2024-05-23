use super::slice_types::{BusSlice, MemoryError, SignalSlice, SliceCapacity, TagDefinitions, TagInfo, TypeAssignmentError, TypeInvalidAccess};
use crate::execution_data::type_definitions::AccessingInformationBus;
use crate::{environment_utils::slice_types::BusRepresentation, execution_data::type_definitions::NodePointer};
use crate::execution_data::ExecutedProgram;
use std::collections::{BTreeMap,HashMap, HashSet};
use crate::ast::Meta;

use crate::assignment_utils::*;

pub struct ComponentRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub is_parallel: bool,
    pub meta: Option<Meta>,
    unassigned_inputs: HashMap<String, SliceCapacity>,
    unassigned_tags: HashSet<String>,
    to_assign_inputs: Vec<(String, Vec<SliceCapacity>, Vec<SliceCapacity>)>,
    to_assign_input_buses: Vec<(String, Vec<SliceCapacity>, BusSlice)>,
    to_assign_input_bus_buses: Vec<(String, AccessingInformationBus, Vec<SliceCapacity>, BusSlice)>,
    to_assign_input_signal_buses: Vec<(String, AccessingInformationBus, Vec<SliceCapacity>)>,
    inputs: HashMap<String, SignalSlice>,
    input_buses: HashMap<String, BusSlice>,
    pub inputs_tags: BTreeMap<String, TagInfo>,
    outputs: HashMap<String, SignalSlice>,
    output_buses: HashMap<String, BusSlice>,
    pub outputs_tags: BTreeMap<String, TagInfo>,
    pub is_initialized: bool,
}

impl Default for ComponentRepresentation {
    fn default() -> Self {
        ComponentRepresentation {
            node_pointer: Option::None,
            is_parallel: false,
            unassigned_inputs: HashMap::new(),
            unassigned_tags: HashSet::new(),
            to_assign_inputs: Vec::new(),
            inputs: HashMap::new(),
            inputs_tags: BTreeMap::new(),
            outputs: HashMap::new(),
            outputs_tags: BTreeMap::new(),
            is_initialized: false,
            meta: Option::None,
            to_assign_input_buses: Vec::new(),
            input_buses: HashMap::new(),
            output_buses: HashMap::new(),
            to_assign_input_bus_buses: Vec::new(),
            to_assign_input_signal_buses: Vec::new(),
        }
    }
}
impl Clone for ComponentRepresentation {
    fn clone(&self) -> Self {
        ComponentRepresentation {
            node_pointer: self.node_pointer,
            is_parallel: self.is_parallel,
            unassigned_inputs: self.unassigned_inputs.clone(),
            unassigned_tags: self.unassigned_tags.clone(),
            to_assign_inputs: self.to_assign_inputs.clone(),
            inputs: self.inputs.clone(),
            inputs_tags: self.inputs_tags.clone(),
            outputs: self.outputs.clone(),
            outputs_tags: self.outputs_tags.clone(),
            is_initialized: self.is_initialized,
            meta : self.meta.clone(),
            to_assign_input_buses: self.to_assign_input_buses.clone(),
            input_buses: self.input_buses.clone(),
            output_buses: self.output_buses.clone(),
            to_assign_input_bus_buses: Vec::new(),
            to_assign_input_signal_buses: Vec::new(),
        }
    }
}

impl ComponentRepresentation {
    pub fn preinitialize_component(
        component: &mut ComponentRepresentation,
        is_parallel: bool,
        prenode_pointer: NodePointer,
        scheme: &ExecutedProgram,
        is_anonymous_component: bool,
        meta: &Meta,
    ) -> Result<(), MemoryError>{
        if !is_anonymous_component && component.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments));
        }
        let possible_node = ExecutedProgram::get_prenode(scheme, prenode_pointer);
        assert!(possible_node.is_some());
        let node = possible_node.unwrap();

        let mut unassigned_tags = HashSet::new();
        let mut inputs_tags = BTreeMap::new();
        let mut outputs_tags = BTreeMap::new();
        for (symbol, tags) in node.inputs() {
            if !tags.is_empty() {
                unassigned_tags.insert(symbol.clone());
            }
            let mut new_tags = TagInfo::new();
            for t in tags{
                new_tags.insert(t.clone(), Option::None);
            }
            inputs_tags.insert(symbol.clone(), new_tags);
        }

        for (symbol, tags) in node.outputs() {
            let mut new_tags = TagInfo::new();
            for t in tags{
                new_tags.insert(t.clone(), Option::None);
            }
            outputs_tags.insert(symbol.clone(), new_tags);
        }

        *component = ComponentRepresentation {
            node_pointer: Option::Some(prenode_pointer),
            unassigned_inputs: HashMap::new(),
            unassigned_tags,
            to_assign_inputs: Vec::new(),
            inputs_tags,
            outputs_tags,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            is_initialized: false,
            is_parallel,
            meta: Some(meta.clone()),
            to_assign_input_buses: Vec::new(),
            input_buses: HashMap::new(),
            output_buses: HashMap::new(),
            to_assign_input_bus_buses: Vec::new(),
            to_assign_input_signal_buses: Vec::new(),
        };
        Result::Ok(())
    }

    pub fn initialize_component(
        component: &mut ComponentRepresentation,
        node_pointer: NodePointer,
        scheme: &ExecutedProgram,
    ) -> Result<(), MemoryError> {
        let possible_node = ExecutedProgram::get_node(scheme, node_pointer);
        assert!(possible_node.is_some());
        let node = possible_node.unwrap();
        component.is_initialized = true;

        for (symbol, route) in node.inputs() {
            let signal_slice = SignalSlice::new_with_route(route, &false);
            let signal_slice_size = SignalSlice::get_number_of_cells(&signal_slice);
            if signal_slice_size > 0{
                component.unassigned_inputs
                    .insert(symbol.clone(), signal_slice_size);
            }
            component.inputs.insert(symbol.clone(), signal_slice);
        }

        for (symbol, route) in node.bus_inputs() {
            let mut initial_value_bus = BusRepresentation::default();
            let bus_node = node.bus_connexions.get(symbol).unwrap().inspect.goes_to;
            BusRepresentation::initialize_bus(
                &mut initial_value_bus,
                bus_node,
                scheme,
                false // it is not initialized at the begining
            )?;
            let bus_slice = BusSlice::new_with_route(route, &initial_value_bus);
            let bus_slice_size = BusSlice::get_number_of_cells(&bus_slice);
            if bus_slice_size > 0{
                component.unassigned_inputs
                    .insert(symbol.clone(), bus_slice_size);
            }
            component.input_buses.insert(symbol.clone(), bus_slice);
        }

        fn insert_tags_output(node: &crate::execution_data::ExecutedTemplate, symbol: &String, component: &mut ComponentRepresentation) {
            let tags_output = node.signal_to_tags.get(symbol);
            let component_tags_output = component.outputs_tags.get_mut(symbol);
            if tags_output.is_some() && component_tags_output.is_some(){
                let result_tags_output = tags_output.unwrap();
                let result_component_tags_output = component_tags_output.unwrap();
                for (tag, value) in result_tags_output{
                    // only update the output tag in case it contains the tag in the definition
                    if result_component_tags_output.contains_key(tag){
                        result_component_tags_output.insert(tag.clone(), value.clone());
                    }
                }
            }
        }

        for (symbol, route) in node.outputs() {
            component.outputs.insert(symbol.clone(), SignalSlice::new_with_route(route, &true));
            insert_tags_output(node, symbol, component);
        }
        
        for (symbol, route) in node.bus_outputs() {
            component.output_buses.insert(symbol.clone(), BusSlice::new_with_route(route, &BusRepresentation::default()));
            insert_tags_output(node, symbol, component);
        }

        component.node_pointer = Option::Some(node_pointer);

        let to_assign = component.to_assign_inputs.clone();
        for s in to_assign{
            let tags_input = component.inputs_tags.get(&s.0).unwrap();
            ComponentRepresentation::assign_value_to_signal_init(component, &s.0, &s.1, &s.2, tags_input.clone())?;
        }

        let to_assign = component.to_assign_input_buses.clone();
        for s in to_assign{
            let tags_input = component.inputs_tags.get(&s.0).unwrap();
            ComponentRepresentation::assign_value_to_bus_init(component, &s.0, &s.1, tags_input.clone(), &s.2)?;
        }

        Result::Ok(())
    }
/* 
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
        if !component.is_initialized{
            return Result::Err(MemoryError::InvalidAccess);
        }

        let slice = if component.inputs.contains_key(signal_name) {
            component.inputs.get(signal_name).unwrap()
        } else {
            component.outputs.get(signal_name).unwrap()
        };

        let enabled_slice = SignalSlice::access_values(&slice, &access)?;
        let mut enabled = false;
        for i in 0..SignalSlice::get_number_of_cells(&enabled_slice) {
            enabled |= SignalSlice::get_reference_to_single_value_by_index(&enabled_slice, i)?;
        }
        Result::Ok(enabled)
    }
*/

fn check_initialized_inputs(&self, bus_name: &str) -> Result<(), MemoryError> {
    if self.node_pointer.is_none() {
        return (Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedComponent)));
    }
    if self.output_buses.contains_key(bus_name) && !self.unassigned_inputs.is_empty() {
        // we return the name of an input that has not been assigned
        let ex_signal = self.unassigned_inputs.iter().next().unwrap().0.clone();
        return (Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputs(ex_signal))));
    }

    if !self.is_initialized {
        // we return the name of an input with tags that has not been assigned
        let ex_signal = self.unassigned_tags.iter().next().unwrap().clone();
        return (Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputTags(ex_signal))));
    }
    Result::Ok(())
}
    pub fn get_signal_field_bus(&self, bus_name: &str,remaining_access: &AccessingInformationBus) -> Result<((TagDefinitions, TagInfo), SignalSlice), MemoryError> {
        if let Result::Err(value) = self.check_initialized_inputs(bus_name) {
            return Err(value);
        }

        let bus_slice = if self.input_buses.contains_key(bus_name) {
            (self.inputs_tags.get(bus_name).unwrap(), self.input_buses.get(bus_name).unwrap())
        } else {
            (self.outputs_tags.get(bus_name).unwrap(), self.output_buses.get(bus_name).unwrap())
        };
        let initial_value = BusSlice::get_reference_to_single_value(bus_slice.1, &[])?;
        let (tags,slice) = initial_value.get_field_signal(
            bus_name,
            remaining_access
        )?;
        Result::Ok((tags,slice))
    }

    pub fn get_bus_field_bus(&self, bus_name: &str,remaining_access: &AccessingInformationBus) -> Result<((TagDefinitions, TagInfo), BusSlice), MemoryError> {
        if let Result::Err(value) = self.check_initialized_inputs(bus_name) {
            return Err(value);
        }

        let bus_slice = if self.input_buses.contains_key(bus_name) {
            (self.inputs_tags.get(bus_name).unwrap(), self.input_buses.get(bus_name).unwrap())
        } else {
            (self.outputs_tags.get(bus_name).unwrap(), self.output_buses.get(bus_name).unwrap())
        };
        let initial_value = BusSlice::get_reference_to_single_value(bus_slice.1, &[])?;
        let (tags,slice) = initial_value.get_field_bus(
            bus_name,
            remaining_access
        )?;
        Result::Ok((tags,slice))
    }
    
    pub fn get_complete_bus(&self, bus_name: &str) -> Result<(&TagInfo, &BusSlice), MemoryError> {
        if let Result::Err(value) = self.check_initialized_inputs(bus_name) {
            return Err(value);
        }

        let slice = if self.input_buses.contains_key(bus_name) {
            (self.inputs_tags.get(bus_name).unwrap(), self.input_buses.get(bus_name).unwrap())
        } else {
            (self.outputs_tags.get(bus_name).unwrap(), self.output_buses.get(bus_name).unwrap())
        };
        Result::Ok(slice)
    }

    pub fn get_signal(&self, signal_name: &str) -> Result<(&TagInfo, &SignalSlice), MemoryError> {

        if self.node_pointer.is_none() {
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedComponent));
        }
        if self.outputs.contains_key(signal_name) && !self.unassigned_inputs.is_empty() {
            // we return the name of an input that has not been assigned
            let ex_signal = self.unassigned_inputs.iter().next().unwrap().0.clone();
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputs(ex_signal)));
        }

        if !self.is_initialized {
            // we return the name of an input with tags that has not been assigned
            let ex_signal = self.unassigned_tags.iter().next().unwrap().clone();
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputTags(ex_signal)));
        }
    
        let slice = if self.inputs.contains_key(signal_name) {
            (self.inputs_tags.get(signal_name).unwrap(), self.inputs.get(signal_name).unwrap())
        } else {
            (self.outputs_tags.get(signal_name).unwrap(), self.outputs.get(signal_name).unwrap())
        };
        Result::Ok(slice)
    }

    pub fn assign_value_to_signal(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: TagInfo,
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_signal_no_init(
                component, 
                signal_name, 
                access, 
                slice_route,
                tags
            )
        } else {
            ComponentRepresentation::assign_value_to_signal_init(
                component,
                signal_name, 
                access, 
                slice_route,
                tags
            )
        }
    }

    /*
        Tags:
        - If an input receives a value that does not contain a expected tag ==> error
        - If an input receives a tag whose value is different to the expected (the one received earlier) ==> error
    
     */


    fn handle_tag_assignment(component: &mut ComponentRepresentation, signal_name: &str, tags: TagInfo) -> Option<Result<(), MemoryError>> {
            if !component.is_preinitialized() {
            return Some(Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent)));
        }
        if !component.inputs_tags.contains_key(signal_name){
            return Some(Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput)));
        }
        let tags_input = component.inputs_tags.get_mut(signal_name).unwrap();
        let is_first_assignment_signal = component.unassigned_tags.contains(signal_name);
        component.unassigned_tags.remove(signal_name);
        // We copy tags in any case, complete or incomplete assignment
        // The values of the tags must be the same than the ones stored before

        for (t, value) in tags_input{
            if !tags.contains_key(t){
                return Some(Result::Err(MemoryError::AssignmentMissingTags(signal_name.to_string(), t.clone())));
            } else{
                if is_first_assignment_signal{
                    *value = tags.get(t).unwrap().clone();
                }
                else{
                    // already given a value, check that it is the same
                    if value != tags.get(t).unwrap(){
                        return Some(Result::Err(MemoryError::AssignmentTagInputTwice(signal_name.to_string(), t.clone())));
                    }
                }
            }
        }
        None
    }



    pub fn assign_value_to_signal_no_init(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: TagInfo,
    ) -> Result<(), MemoryError> {

        if let Some(value) = ComponentRepresentation::handle_tag_assignment(component, signal_name, tags) {
            //if Some, then an error was detected.
            return value;
        }
        component.to_assign_inputs.push((signal_name.to_string(), access.to_vec(), slice_route.to_vec()));
        Result::Ok(())
    }

        //assign_value_to_bus_bus //y hay que llamar a assign_bus_field.
    pub fn assign_value_to_bus_complete(
            component: &mut ComponentRepresentation,
            signal_name: &str,
            access: &[SliceCapacity],
            tags: TagInfo,
            bus_slice: BusSlice
        ) -> Result<(), MemoryError> {
            if !component.is_initialized{
                ComponentRepresentation::assign_value_to_bus_no_init(
                    component, 
                    signal_name, 
                    access, 
                    tags,
                    bus_slice
                )
            } else {
                ComponentRepresentation::assign_value_to_bus_init(
                    component,
                    signal_name, 
                    access, 
                    tags,
                    &bus_slice
                )
            }
        }

    pub fn assign_value_to_bus_no_init(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        tags: TagInfo,
        bus_slice: BusSlice
    ) -> Result<(), MemoryError> {

        if let Some(value) = ComponentRepresentation::handle_tag_assignment(component, bus_name, tags) {
            return value;
        }
        component.to_assign_input_buses.push((bus_name.to_string(), access.to_vec(),bus_slice));
        Result::Ok(())
    }

    pub fn assign_value_to_signal_init(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: TagInfo,
    ) -> Result<(), MemoryError> {

        if !component.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !component.inputs.contains_key(signal_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // Check that the assignment satisfies the tags requisites

        let tags_input = component.inputs_tags.get_mut(signal_name).unwrap();
        for (t, value) in tags_input{
            if !tags.contains_key(t){
                return Result::Err(MemoryError::AssignmentMissingTags(signal_name.to_string(), t.clone()));
            } else{            
                // We are in the case where the component is initialized, so we 
                // assume that all tags already have their value and check if it is
                // the same as the one we are receiving
                if value != tags.get(t).unwrap(){
                    return Result::Err(MemoryError::AssignmentTagInputTwice(signal_name.to_string(), t.clone()));
                }
            }
        }

        
        // Perform the assignment
        let inputs_response = component.inputs.get_mut(signal_name).unwrap();
        
        perform_signal_assignment(inputs_response, &access, slice_route)?;
        
        // Update the value of unnasigned fields
        let mut dim_slice = 1;
        for i in slice_route {
            dim_slice *= *i;
        }
        match component.unassigned_inputs.get_mut(signal_name){
            Some(left) => {
                *left -= dim_slice;
                if *left == 0 {
                    component.unassigned_inputs.remove(signal_name);
                }
            }
            None => {}
        }

        Result::Ok(())

    }

    pub fn assign_value_to_bus_init(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        tags: TagInfo,
        bus_slice : &BusSlice,
    ) -> Result<(), MemoryError> {

        if !component.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !component.input_buses.contains_key(bus_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        let tags_input = component.inputs_tags.get_mut(bus_name).unwrap();
        for (t, value) in tags_input{
            if !tags.contains_key(t){
                return Result::Err(MemoryError::AssignmentMissingTags(bus_name.to_string(), t.clone()));
            } else{            
                // We are in the case where the component is initialized, so we 
                // assume that all tags already have their value and check if it is
                // the same as the one we are receiving
                if value != tags.get(t).unwrap(){
                    return Result::Err(MemoryError::AssignmentTagInputTwice(bus_name.to_string(), t.clone()));
                }
            }
        }

        // Perform the bus assignment

        let inputs_response = component.input_buses.get_mut(bus_name).unwrap();
        let assignment_result = perform_bus_assignment(inputs_response, &access, bus_slice);
        match assignment_result {
            Ok(_) =>{},
            Err(err) => return Err(err)
        }

        // Update the unassigned inputs
        let dim = BusSlice::get_number_of_cells(&bus_slice);
        match component.unassigned_inputs.get_mut(bus_name){
            Some(left) => {
                *left -= dim;
                if *left == 0 {
                    component.unassigned_inputs.remove(bus_name);
                }
            }
            None => {}
        }

        Result::Ok(())

    }

    pub fn is_preinitialized(&self) -> bool {
        self.node_pointer.is_some()
    }

    pub fn is_ready_initialize(&self) -> bool {
        self.unassigned_tags.is_empty()
    }

    pub fn has_unassigned_inputs(&self) -> bool{
        !self.unassigned_inputs.is_empty()
    }

    fn check_input_errors(component: &mut ComponentRepresentation, bus_name: &str, tags: &TagInfo) -> Option<Result<(), MemoryError>> {
        if !component.is_preinitialized() {
            return Some(Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent)));
        }
        if !component.input_buses.contains_key(bus_name){
            return Some(Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput)));
        }
        let tags_input = component.inputs_tags.get_mut(bus_name).unwrap();
            
    
        for (t, value) in tags_input{
            if !tags.contains_key(t){
                return Some(Result::Err(MemoryError::AssignmentMissingTags(bus_name.to_string(), t.clone())));
            } else{            
                // We are in the case where the component is initialized, so we 
                // assume that all tags already have their value and check if it is
                // the same as the one we are receiving
                if value != tags.get(t).unwrap(){
                    return Some(Result::Err(MemoryError::AssignmentTagInputTwice(bus_name.to_string(), t.clone())));
                }
            }
        }
        None
    }

      //assign_value_to_bus_signal y hay que llamar a assign_signal_field.
      pub fn assign_value_to_bus_bus(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        remaining_access: AccessingInformationBus,
        slice_route: &[SliceCapacity],
        tags: TagInfo,
        bus_slice: BusSlice
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_bus_bus_no_init(
                component, 
                signal_name, 
                remaining_access,
                slice_route,
                tags,
                bus_slice
            )
        } else {
            ComponentRepresentation::assign_value_to_bus_bus_init(
                component,
                signal_name, 
                remaining_access,
                slice_route,
                tags,
                &bus_slice
            )
        }
    }

    fn assign_value_to_bus_bus_init(component: &mut ComponentRepresentation, bus_name: &str, remaining_access: AccessingInformationBus, slice_route: &[usize], tags: TagInfo, bus_slice: &BusSlice) -> Result<(), MemoryError> {
        if let Some(value) = ComponentRepresentation::check_input_errors(component, bus_name, &tags) {
            return value;
        }

        let inputs_response = component.input_buses.get_mut(bus_name).unwrap();

        let initial_value = BusSlice::get_mut_reference_to_single_value(inputs_response, &remaining_access.array_access)?;
        initial_value.assign_value_to_field_bus(bus_name, &remaining_access, slice_route, bus_slice, tags)

    }

    fn assign_value_to_bus_bus_no_init(component: &mut ComponentRepresentation, bus_name: &str, remaining_access: AccessingInformationBus, slice_route: &[usize], tags: TagInfo, bus_slice: BusSlice) -> Result<(), MemoryError> {
        if let Some(value) = ComponentRepresentation::handle_tag_assignment(component, bus_name, tags) {
            return value;
        }
        component.to_assign_input_bus_buses.push((bus_name.to_string(), remaining_access, slice_route.to_vec(),bus_slice));
        Result::Ok(())
    }

    pub fn assign_value_to_bus_signal(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        remaining_access: AccessingInformationBus,
        slice_route: &[SliceCapacity],
        tags: TagInfo,
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_bus_signal_no_init(
                component, 
                signal_name, 
                remaining_access,
                slice_route,
                tags
            )
        } else {
            ComponentRepresentation::assign_value_to_bus_signal_init(
                component,
                signal_name, 
                remaining_access,
                slice_route,
                tags
            )
        }
    }

    fn assign_value_to_bus_signal_init(component: &mut ComponentRepresentation, bus_name: &str, remaining_access: AccessingInformationBus, slice_route: &[usize], tags: TagInfo) -> Result<(), MemoryError> {
        if let Some(value) = ComponentRepresentation::check_input_errors(component, bus_name, &tags) {
            return value;
        }

        let inputs_response = component.input_buses.get_mut(bus_name).unwrap();

        let initial_value = BusSlice::get_mut_reference_to_single_value(inputs_response, &remaining_access.array_access)?;
        initial_value.assign_value_to_field_signal(bus_name, &remaining_access, slice_route, tags)

    }

    fn assign_value_to_bus_signal_no_init(component: &mut ComponentRepresentation, bus_name: &str, remaining_access: AccessingInformationBus, slice_route: &[usize], tags: TagInfo) -> Result<(), MemoryError> {
        if let Some(value) = ComponentRepresentation::handle_tag_assignment(component, bus_name, tags) {
            return value;
        }
        component.to_assign_input_signal_buses.push((bus_name.to_string(), remaining_access, slice_route.to_vec()));
        Result::Ok(())
    }
}

