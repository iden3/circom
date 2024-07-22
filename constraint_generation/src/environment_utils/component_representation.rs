use super::slice_types::{FoldedResult, FoldedArgument, BusSlice, MemoryError, SignalSlice, SliceCapacity, TagInfo, TypeAssignmentError, TypeInvalidAccess};
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
    to_assign_input_bus_fields: Vec<(String, AccessingInformationBus, FoldedResult, TagInfo)>,
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
            to_assign_input_bus_fields: Vec::new(),
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
            to_assign_input_bus_fields: self.to_assign_input_bus_fields.clone(),
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
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignmentsComponent));
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
            to_assign_input_bus_fields: Vec::new(),
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

        for info_wire in node.inputs() {
            let symbol = &info_wire.name;
            let route = &info_wire.length;
            if !info_wire.is_bus{
                let signal_slice = SignalSlice::new_with_route(route, &false);
                let signal_slice_size = SignalSlice::get_number_of_cells(&signal_slice);
                if signal_slice_size > 0{
                    component.unassigned_inputs
                        .insert(symbol.clone(), signal_slice_size);
                }
                component.inputs.insert(symbol.clone(), signal_slice);
            } else{
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

        for info_wire in node.outputs() {
            let symbol = &info_wire.name;
            let route = &info_wire.length;
            if !info_wire.is_bus{
                component.outputs.insert(symbol.clone(), SignalSlice::new_with_route(route, &true));
            } else{
                let mut initial_value_bus = BusRepresentation::default();
                let bus_node = node.bus_connexions.get(symbol).unwrap().inspect.goes_to;
                BusRepresentation::initialize_bus(
                    &mut initial_value_bus,
                    bus_node,
                    scheme,
                    true // the outputs of the component are initialized at the begining
                )?;
                let bus_slice = BusSlice::new_with_route(route, &initial_value_bus);
    
                component.output_buses.insert(symbol.clone(), bus_slice);
            }
            insert_tags_output(node, symbol, component);
        }
        
        component.node_pointer = Option::Some(node_pointer);

        let to_assign = std::mem::replace(&mut component.to_assign_inputs, vec![]);

        for (signal_name, access, route) in &to_assign{
            let tags_input = component.inputs_tags[signal_name].clone();
            component.assign_value_to_signal_init(signal_name, access, route, &tags_input)?;
        }

        let to_assign = std::mem::replace(&mut component.to_assign_input_buses, vec![]);
        for (signal_name, access, bus_slice) in &to_assign{
            let tags_input = component.inputs_tags[signal_name].clone();
            component.assign_value_to_bus_init(signal_name, access, bus_slice, &tags_input)?;
        }

        let to_assign = std::mem::replace(&mut component.to_assign_input_bus_fields, vec![]);
        for (signal_name, access, field_value, tags_input) in to_assign{
            component.assign_value_to_bus_field_init(&signal_name, &access, &field_value, tags_input)?;
        }

        Result::Ok(())
    }


    fn check_initialized_inputs(&self, bus_name: &str) -> Result<(), MemoryError> {
        if self.node_pointer.is_none() {
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedComponent));
        }
        // in case it is an output signal or bus
        if (self.outputs.contains_key(bus_name) || self.output_buses.contains_key(bus_name)) && !self.unassigned_inputs.is_empty() {
            // we return the name of an input that has not been assigned
            let ex_signal = self.unassigned_inputs.iter().next().unwrap().0.clone();
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputs(ex_signal)));
        }
    
        if !self.is_initialized {
            // we return the name of an input with tags that has not been assigned
            let ex_signal = self.unassigned_tags.iter().next().unwrap().clone();
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputTags(ex_signal)));
        }
        Result::Ok(())
    }
    
    pub fn get_io_value(&self, field_name: &str, remaining_access: &AccessingInformationBus) ->Result<(Option<TagInfo>, FoldedResult), MemoryError>{
        if let Result::Err(value) = self.check_initialized_inputs(field_name) {
            return Err(value);
        }
    
        if self.inputs.contains_key(field_name) || self.outputs.contains_key(field_name){
            // in this case we are accessing a signal
            let (tag_info, signal_slice) = if self.inputs.contains_key(field_name) {
                (self.inputs_tags.get(field_name).unwrap(), self.inputs.get(field_name).unwrap())
            } else {
                (self.outputs_tags.get(field_name).unwrap(), self.outputs.get(field_name).unwrap())
            };
    
            if remaining_access.field_access.is_some(){
                // in case it is a tag access
                assert!(remaining_access.array_access.len() == 0);
                let value_tag = tag_info.get(remaining_access.field_access.as_ref().unwrap()).unwrap();
                match value_tag{
                    Option::None =>{
                        let error = MemoryError::TagValueNotInitializedAccess;
                        Result::Err(error)
                    },
                    Some(v) =>{
                        let folded_tag = FoldedResult::Tag(v.clone());
                        Result::Ok((None, folded_tag))
                    }
                }
            } else{
                // case signals
                // We access to the selected signal if it is an array
                let accessed_slice_result = SignalSlice::access_values(signal_slice, &remaining_access.array_access);
                match accessed_slice_result{
                    Ok(slice) =>{
                        let folded_slice = FoldedResult::Signal(slice);
                        Result::Ok((Some(tag_info.clone()), folded_slice))
                    },
                    Err(err) => Err(err)
                }
            }
        } else{
            // in this case we are accessing a bus
            let (tag_info, bus_slice) = if self.input_buses.contains_key(field_name) {
                (self.inputs_tags.get(field_name).unwrap(), self.input_buses.get(field_name).unwrap())
            } else {
                (self.outputs_tags.get(field_name).unwrap(), self.output_buses.get(field_name).unwrap())
            };
    
            if remaining_access.field_access.is_some(){
                // In this case we need to access to values of the bus or one of its tags
                let next_array_access = &remaining_access.array_access;
                let next_field_access = remaining_access.field_access.as_ref().unwrap();
                let next_remaining_access = remaining_access.remaining_access.as_ref().unwrap();
                
                // we distingish between tags or buses 
                if tag_info.contains_key(remaining_access.field_access.as_ref().unwrap()){
                    // in this case we are returning a tag
                    assert!(next_array_access.len() == 0);
                    let value_tag = tag_info.get(next_field_access).unwrap();
                    match value_tag{
                        Option::None =>{
                            let error = MemoryError::TagValueNotInitializedAccess;
                            Result::Err(error)
                        },
                        Some(v) =>{
                            let folded_tag = FoldedResult::Tag(v.clone());
                            Result::Ok((None, folded_tag))
                        }
                    }
                } else{
                    // in this case we are returning a field of the bus
    
                    let accessed_slice_result = BusSlice::access_values(bus_slice, &remaining_access.array_access);
                    let accessed_bus = match accessed_slice_result{
                        Ok(slice) =>{
                            BusSlice::unwrap_to_single(slice)
                        },
                        Err(err) => return Err(err)
                    };
                    accessed_bus.get_field(next_field_access, next_remaining_access)
                }
            } else{

                // In this case we are accessing the complete bus
                let accessed_slice_result = BusSlice::access_values(bus_slice, &remaining_access.array_access);
                
                match accessed_slice_result{
                    Ok(slice) =>{
                        let folded_slice = FoldedResult::Bus(slice);
                        Result::Ok((Some(tag_info.clone()), folded_slice))
                    },
                    Err(err) => Err(err)
                }
            }
        }
    
    }

    // Assign signals: Operations to assign signals -> case init and no init

    pub fn assign_value_to_signal(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: &TagInfo,
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

    pub fn assign_value_to_signal_no_init(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {

        // check that the tags are correct and update values
        ComponentRepresentation::handle_tag_assignment_no_init(component, signal_name, tags)?;
        component.to_assign_inputs.push((signal_name.to_string(), access.to_vec(), slice_route.to_vec()));
        
        Result::Ok(())
    }

    pub fn assign_value_to_signal_init(
        self: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {

        if !self.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !self.inputs.contains_key(signal_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // Check that the assignment satisfies the tags requisites
        ComponentRepresentation::handle_tag_assignment_init(self, signal_name, tags)?;

        
        // Perform the assignment
        let inputs_response = self.inputs.get_mut(signal_name).unwrap();
        perform_signal_assignment(inputs_response, &access, slice_route)?;
        
        // Update the value of unnasigned fields
        ComponentRepresentation::update_unassigned_inputs(self, signal_name, slice_route);

        Result::Ok(())

    }

    // Assign buses: Operations to assign buses -> case init and no init

    pub fn assign_value_to_bus(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        bus_slice: BusSlice,
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_bus_no_init(
                component, 
                bus_name, 
                access, 
                bus_slice,
                tags
            )
        } else {
            ComponentRepresentation::assign_value_to_bus_init(
                component,
                bus_name, 
                access, 
                &bus_slice,
                tags
            )
        }
    }

    pub fn assign_value_to_bus_no_init(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        bus_slice: BusSlice,
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {

        // check that the tags are correct and update values
        ComponentRepresentation::handle_tag_assignment_no_init(component, bus_name, tags)?;
        component.to_assign_input_buses.push((bus_name.to_string(), access.to_vec(), bus_slice));
        
        Result::Ok(())
    }

    pub fn assign_value_to_bus_init(
        self: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        bus_slice: &BusSlice,
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {

        if !self.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !self.input_buses.contains_key(bus_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // Check that the assignment satisfies the tags requisites
        ComponentRepresentation::handle_tag_assignment_init(self, bus_name, tags)?;
        
        // Perform the assignment
        let inputs_response = self.input_buses.get_mut(bus_name).unwrap();
        perform_bus_assignment(inputs_response, &access, bus_slice, true)?;
        
        // Update the value of unnasigned fields
        ComponentRepresentation::update_unassigned_inputs(self, bus_name, bus_slice.route());

        Result::Ok(())

    }
        


    // Assign bus field: Operations to assign bus fields -> case init and no init

    pub fn assign_value_to_bus_field(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &AccessingInformationBus,
        field_value: FoldedResult,
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_bus_field_no_init(
                component, 
                bus_name, 
                access, 
                field_value,
                tags
            )
        } else {
            ComponentRepresentation::assign_value_to_bus_field_init(
                component,
                bus_name, 
                access, 
                &field_value,
                tags.clone()
            )
        }
    }

    pub fn assign_value_to_bus_field_no_init(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &AccessingInformationBus,
        field_value: FoldedResult,
        tags: &TagInfo,
    ) -> Result<(), MemoryError> {

        // check that the tags are correct and update values, in this case none inputs
        // are assigned to the complete bus
        ComponentRepresentation::handle_tag_assignment_no_init(
            component, 
            bus_name, 
            &TagInfo::new())?;
        
        component.to_assign_input_bus_fields.push((
            bus_name.to_string(), 
            access.clone(), 
            field_value,
            tags.clone()
        )
        );
        
        Result::Ok(())
    }

    pub fn assign_value_to_bus_field_init(
        self: &mut ComponentRepresentation,
        bus_name: &str,
        access: &AccessingInformationBus,
        field_value: &FoldedResult,
        tags: TagInfo,
    ) -> Result<(), MemoryError> {

        if !self.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !self.input_buses.contains_key(bus_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        
        // Get the assigned input bus
        let inputs_slice = self.input_buses.get_mut(bus_name).unwrap();
    
        let access_result = BusSlice::access_values_by_mut_reference(inputs_slice, &access.array_access);
        let mut accessed_bus = match access_result{
            Ok(value) => value,
            Err(err) => return Err(err)
        };
        
        assert!(accessed_bus.len() == 1);
        let single_bus = accessed_bus.get_mut(0).unwrap();
        assert!(access.field_access.is_some());

        // call to bus representation to perform the assignment
        let route_signal;

        let folded_arg = match field_value{
            FoldedResult::Signal (ss)=>{
                route_signal = ss.route().to_vec();
                FoldedArgument::Signal(&route_signal)
            },
            FoldedResult::Bus (bs)=>{
                FoldedArgument::Bus(&bs)
            },
            FoldedResult::Tag(_) =>{
                unreachable!()
            }
        };

        single_bus.assign_value_to_field(
            access.field_access.as_ref().unwrap(),
            access.remaining_access.as_ref().unwrap(),
            folded_arg,
            Some(tags),
            true, // it is an input so check tags instead of propagate
        )?;
        
        
        // In case it is completely assigned update unassigned
        
        if !single_bus.has_unassigned_fields(){
            ComponentRepresentation::update_unassigned_inputs(self, bus_name, &[1]);
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
     

    /*
        Tags:
        - If an input receives a value that does not contain a expected tag ==> error
        - If an input receives a tag whose value is different to the expected (the one received earlier) ==> error
    
     */


     fn handle_tag_assignment_no_init(component: &mut ComponentRepresentation, signal_name: &str, tags: &TagInfo) -> Result<(), MemoryError> {
        
        if !component.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        if !component.inputs_tags.contains_key(signal_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        let tags_input = component.inputs_tags.get_mut(signal_name).unwrap();
        let is_first_assignment_signal = component.unassigned_tags.contains(signal_name);
        component.unassigned_tags.remove(signal_name);

        // We copy tags in any case, complete or incomplete assignment
        // The values of the tags must be the same than the ones stored before

        for (t, value) in tags_input{
            if !tags.contains_key(t){
                return Result::Err(MemoryError::AssignmentMissingTags(signal_name.to_string(), t.clone()));
            } else{
                if is_first_assignment_signal{
                    *value = tags.get(t).unwrap().clone();
                }
                else{
                    // already given a value, check that it is the same
                    if value != tags.get(t).unwrap(){
                        return Result::Err(MemoryError::AssignmentTagInputTwice(signal_name.to_string(), t.clone()));
                    }
                }
            }
        }
        Result::Ok(())
    }

    fn handle_tag_assignment_init(component: &ComponentRepresentation, signal_name: &str, tags: &TagInfo)-> Result<(), MemoryError>{
        let tags_input = component.inputs_tags.get(signal_name).unwrap();
        for (t, value) in tags_input{
            if !tags.contains_key(t){
                return Result::Err(MemoryError::AssignmentMissingTags(signal_name.to_string(), t.clone()));
            } else{            
                // We are in the case wher.e the component is initialized, so we 
                // assume that all tags already have their value and check if it is
                // the same as the one we are receiving
                if value != tags.get(t).unwrap(){
                    return Result::Err(MemoryError::AssignmentTagInputTwice(signal_name.to_string(), t.clone()));
                }
            }
        }
        Ok(())
    }

    // Auxiliar function to update the unassigned inputs

    fn update_unassigned_inputs(component: &mut ComponentRepresentation, signal_name: &str, slice_route: &[usize]){
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
            Option::None => {}
        }
    }
}

