use super::slice_types::{AssignmentState, BusSlice, FoldedArgument, FoldedResult, MemoryError, SignalSlice, SliceCapacity, TagInfo, TypeAssignmentError, TypeInvalidAccess};
use crate::execution_data::type_definitions::AccessingInformationBus;
use crate::{environment_utils::slice_types::BusRepresentation, execution_data::type_definitions::NodePointer};
use crate::execution_data::ExecutedProgram;
use std::collections::{HashMap, HashSet};
use crate::ast::Meta;
use crate::execution_data::type_definitions::{TagNames,TagWire};
use crate::assignment_utils::*;
use num_bigint_dig::BigInt;
use crate::environment_utils::slice_types::AssignmentState::*;

pub struct ComponentRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub is_parallel: bool,
    pub meta: Option<Meta>,
    unassigned_inputs: HashMap<String, SliceCapacity>,
    unassigned_tags: HashSet<Vec<String>>,
    to_assign_inputs: Vec<(String, Vec<SliceCapacity>, Vec<SliceCapacity>, AssignmentState)>,
    to_assign_input_buses: Vec<(String, Vec<SliceCapacity>, BusSlice, AssignmentState)>,
    to_assign_input_bus_fields: Vec<(String, AccessingInformationBus, FoldedResult, AssignmentState)>,
    inputs: HashMap<String, SignalSlice>,
    input_buses: HashMap<String, BusSlice>,
    pub inputs_tags: HashMap<String, TagWire>,
    outputs: HashMap<String, SignalSlice>,
    output_buses: HashMap<String, BusSlice>,
    pub outputs_tags: HashMap<String, TagWire>,
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
            inputs_tags: HashMap::new(),
            outputs: HashMap::new(),
            outputs_tags: HashMap::new(),
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
        let mut inputs_tags = HashMap::new();
        let mut outputs_tags = HashMap::new();
        pub fn collect_info_tag(tags: &TagNames, complete_name: &mut Vec<String>, unassigned_tags: &mut HashSet<Vec<String>>, is_input:bool)-> TagWire{
            let mut new_tags = TagInfo::new();
            if !tags.tag_names.is_empty() && is_input{
                unassigned_tags.insert(complete_name.clone());
            }
            for t in &tags.tag_names{
                new_tags.insert(t.clone(), Option::None);
            }
            let new_tag_fields = if tags.fields.is_some(){
                let mut info = HashMap::new();
                for (name, tags) in tags.fields.as_ref().unwrap(){
                    complete_name.push(name.clone());
                    info.insert(
                        name.clone(), 
                        collect_info_tag(tags, complete_name, unassigned_tags, is_input)
                    );
                    complete_name.pop();
                }
                Some(info)
            } else{
                None
            };
            TagWire{
                tags: new_tags,
                fields: new_tag_fields,
            }
        }
        for (symbol, tags) in node.inputs() {
            let mut complete_name = vec![symbol.clone()];
            if !tags.tag_names.is_empty() {
                unassigned_tags.insert(complete_name.clone());
            }
            let tag_info = collect_info_tag(tags, &mut complete_name, &mut unassigned_tags, true);
            inputs_tags.insert(symbol.clone(), tag_info);

        }

        for (symbol, tags) in node.outputs() {
            let mut complete_name = vec![symbol.clone()];
            let tag_info = collect_info_tag(tags, &mut complete_name, &mut unassigned_tags, false);
            outputs_tags.insert(symbol.clone(), tag_info);
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
                let signal_slice = SignalSlice::new_with_route(route, &NoAssigned);
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


        fn insert_tags_output(node: &crate::execution_data::ExecutedTemplate, component: &mut ComponentRepresentation) {
            
            for (tag_name, value) in &node.signal_to_tags{
                if component.outputs_tags.contains_key(&tag_name[0]){
                    // in this case we have to store the value
                    let mut info_output_tags = component.outputs_tags.get_mut(&tag_name[0]).unwrap();
                    for i in 1..tag_name.len()-1{
                        info_output_tags = info_output_tags.fields.as_mut().unwrap().get_mut(&tag_name[i]).unwrap();
                    }
                    info_output_tags.tags.insert(tag_name.last().unwrap().clone(), Some(value.clone()));
                }
            }
            
        }

        for info_wire in node.outputs() {
            let symbol = &info_wire.name;
            let route = &info_wire.length;
            if !info_wire.is_bus{
                component.outputs.insert(symbol.clone(), SignalSlice::new_with_route(route, &Assigned(None)));
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
        }
        // To insert the info of the output tags
        insert_tags_output(node, component);

        
        component.node_pointer = Option::Some(node_pointer);

        let to_assign = std::mem::replace(&mut component.to_assign_inputs, vec![]);

        for (signal_name, access, route, conditions_assignment) in &to_assign{
            let tags_input = component.inputs_tags[signal_name].clone();
            component.assign_value_to_signal_init(signal_name, access, route, &tags_input, conditions_assignment)?;
        }

        let to_assign = std::mem::replace(&mut component.to_assign_input_buses, vec![]);
        for (signal_name, access, bus_slice, conditions_assignment) in &to_assign{
            let tags_input = component.inputs_tags[signal_name].clone();
            component.assign_value_to_bus_init(signal_name, access, bus_slice, &tags_input, conditions_assignment)?;
        }

        let to_assign = std::mem::replace(&mut component.to_assign_input_bus_fields, vec![]);
        for (signal_name, access, field_value, conditions_assignment) in &to_assign{
            let mut tags_input = &component.inputs_tags[signal_name].clone();
            let mut aux_access = access;
            while aux_access.field_access.is_some(){
                tags_input = tags_input.fields.as_ref().unwrap().get(aux_access.field_access.as_ref().unwrap()).unwrap();
                aux_access = aux_access.remaining_access.as_ref().unwrap();
            }
            component.assign_value_to_bus_field_init(&signal_name, &access, &field_value, tags_input, conditions_assignment)?;
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
            return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::MissingInputTags(ex_signal[0].clone()))); // TODO: improve, show complete trace
        }
        Result::Ok(())
    }
    
    pub fn get_io_value(&self, field_name: &str, remaining_access: &AccessingInformationBus) ->Result<(TagWire, FoldedResult), MemoryError>{
        if let Result::Err(value) = self.check_initialized_inputs(field_name) {
            return Err(value);
        }
    
        if self.inputs.contains_key(field_name) || self.outputs.contains_key(field_name){
            // in this case we are accessing a signal
            
            // We get the info of the tags
            let (tag_info, signal_slice) = if self.inputs.contains_key(field_name) {
                (self.inputs_tags.get(field_name).unwrap(), self.inputs.get(field_name).unwrap())
            } else {
                (self.outputs_tags.get(field_name).unwrap(), self.outputs.get(field_name).unwrap())
            };

            /* NOT VALID: to access a tag of a subcomponent
            if remaining_access.field_access.is_some(){
                // in case it is a tag access
                assert!(remaining_access.array_access.len() == 0);
                let value_tag = tag_info.tags.get(remaining_access.field_access.as_ref().unwrap()).unwrap();
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
            } else{*/
                // case signals
                // We access to the selected signal if it is an array
                let accessed_slice_result = SignalSlice::access_values(signal_slice, &remaining_access.array_access);
                match accessed_slice_result{
                    Ok(slice) =>{
                        let folded_slice = FoldedResult::Signal(slice);
                        Result::Ok((tag_info.clone(), folded_slice))
                    },
                    Err(err) => Err(err)
                }
            //}
        } else{
            // in this case we are accessing a bus
            let (mut tag_info, bus_slice) = if self.input_buses.contains_key(field_name) {
                (self.inputs_tags.get(field_name).unwrap(), self.input_buses.get(field_name).unwrap())
            } else {
                (self.outputs_tags.get(field_name).unwrap(), self.output_buses.get(field_name).unwrap())
            };
    
            let result = if remaining_access.field_access.is_some(){
                // In this case we need to access to values of the bus 
                let next_field_access = remaining_access.field_access.as_ref().unwrap();
                let next_remaining_access = remaining_access.remaining_access.as_ref().unwrap();
                
                /* NOT VALID-> case tags: if tag_info.tags.contains_key(remaining_access.field_access.as_ref().unwrap()){
                    // in this case we are returning a tag
                    assert!(next_array_access.len() == 0);
                    let value_tag = tag_info.tags.get(next_field_access).unwrap();
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
                } else{ */
                    // we are returning a field of the bus
    
                    let accessed_slice_result = BusSlice::access_values(bus_slice, &remaining_access.array_access);
                    let accessed_bus = match accessed_slice_result{
                        Ok(slice) =>{
                            BusSlice::unwrap_to_single(slice)
                        },
                        Err(err) => return Err(err)
                    };
                    accessed_bus.get_field(next_field_access, next_remaining_access)?
                //}
            } else{

                // In this case we are accessing the complete bus
                let slice = BusSlice::access_values(bus_slice, &remaining_access.array_access)?;
                FoldedResult::Bus(slice)
            };

            // Finally, get the tags
            let mut to_access = remaining_access;
            while to_access.field_access != None {
                let acc = to_access.field_access.as_ref().unwrap();

                tag_info = tag_info.fields.as_ref().unwrap().get(acc).unwrap();
                to_access = to_access.remaining_access.as_ref().unwrap();
            }
            Ok((tag_info.clone(), result))
        }
    
    }

    pub fn get_tag_value(&self, field_name: &str, remaining_access: &AccessingInformationBus)-> Result<BigInt, MemoryError>{
        
        if let Result::Err(value) = self.check_initialized_inputs(field_name) {
            return Err(value);
        }
        
        let mut tags_info = if self.inputs_tags.contains_key(field_name){
            self.inputs_tags.get(field_name).unwrap()
        } else{
            self.outputs_tags.get(field_name).unwrap()
        };

        let mut to_access = remaining_access;
        let mut next_access = remaining_access.remaining_access.as_ref().unwrap();
        while next_access.field_access.is_some(){
            let field = to_access.field_access.as_ref().unwrap();
            tags_info = tags_info.fields.as_ref().unwrap().get(field).unwrap();
            to_access = to_access.remaining_access.as_ref().unwrap();
            next_access = next_access.remaining_access.as_ref().unwrap();
        }
        let tag_value = tags_info.tags.get(to_access.field_access.as_ref().unwrap()).unwrap();
        if let Some(value_tag) = tag_value { // tag has value
                Result::Ok(value_tag.clone() )
        } else {
            let error = MemoryError::TagValueNotInitializedAccess;
            return Result::Err(error);
        }
    }

    // Assign signals: Operations to assign signals -> case init and no init

    pub fn assign_value_to_signal(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_signal_no_init(
                component, 
                signal_name, 
                access, 
                slice_route,
                tags,
                conditions_assignment
            )
        } else {
            ComponentRepresentation::assign_value_to_signal_init(
                component,
                signal_name, 
                access, 
                slice_route,
                tags,
                conditions_assignment
            )
        }
    }

    pub fn assign_value_to_signal_no_init(
        component: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {

        // check that the tags are correct and update values
        ComponentRepresentation::handle_tag_assignment_no_init(component, &vec![signal_name.to_string()], tags)?;
        component.to_assign_inputs.push((signal_name.to_string(), access.to_vec(), slice_route.to_vec(), conditions_assignment.clone()));
        
        Result::Ok(())
    }

    pub fn assign_value_to_signal_init(
        self: &mut ComponentRepresentation,
        signal_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {

        if !self.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !self.inputs.contains_key(signal_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // Check that the assignment satisfies the tags requisites
        ComponentRepresentation::handle_tag_assignment_init(self, &vec![signal_name.to_string()], tags)?;

        
        // Perform the assignment
        let inputs_response = self.inputs.get_mut(signal_name).unwrap();
        perform_signal_assignment(inputs_response, &access, slice_route, conditions_assignment)?;
        
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
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_bus_no_init(
                component, 
                bus_name, 
                access, 
                bus_slice,
                tags,
                conditions_assignment
            )
        } else {
            ComponentRepresentation::assign_value_to_bus_init(
                component,
                bus_name, 
                access, 
                &bus_slice,
                tags,
                conditions_assignment
            )
        }
    }

    pub fn assign_value_to_bus_no_init(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        bus_slice: BusSlice,
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {

        // check that the tags are correct and update values
        ComponentRepresentation::handle_tag_assignment_no_init(component, &vec![bus_name.to_string()], tags)?;
        component.to_assign_input_buses.push((bus_name.to_string(), access.to_vec(), bus_slice, conditions_assignment.clone()));
        
        Result::Ok(())
    }

    pub fn assign_value_to_bus_init(
        self: &mut ComponentRepresentation,
        bus_name: &str,
        access: &[SliceCapacity],
        bus_slice: &BusSlice,
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {

        if !self.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !self.input_buses.contains_key(bus_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // Check that the assignment satisfies the tags requisites
        ComponentRepresentation::handle_tag_assignment_init(self, &vec![bus_name.to_string()], tags)?;
        
        // Perform the assignment
        let inputs_response = self.input_buses.get_mut(bus_name).unwrap();
        perform_bus_assignment(inputs_response, &access, bus_slice, true, conditions_assignment)?;
        
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
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {
        if !component.is_initialized{
            ComponentRepresentation::assign_value_to_bus_field_no_init(
                component, 
                bus_name, 
                access, 
                field_value,
                tags,
                conditions_assignment
            )
        } else {
            ComponentRepresentation::assign_value_to_bus_field_init(
                component,
                bus_name, 
                access, 
                &field_value,
                tags,
                conditions_assignment
            )
        }
    }

    pub fn assign_value_to_bus_field_no_init(
        component: &mut ComponentRepresentation,
        bus_name: &str,
        access: &AccessingInformationBus,
        field_value: FoldedResult,
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {

        fn build_name(complete_bus_name: &mut Vec<String>, access: &AccessingInformationBus){
            match &access.field_access{
                Option::None =>{},
                Option::Some(name) =>{
                    complete_bus_name.push(name.clone());
                }
            }
            match &access.remaining_access{
                Option::None =>{},
                Option::Some(remaining) =>{
                    build_name(complete_bus_name, &remaining)
                }
            }
        }
        let mut complete_name = vec![bus_name.to_string()];
        build_name(&mut complete_name, access);

        // check that the tags are correct and update values
        ComponentRepresentation::handle_tag_assignment_no_init(
            component, 
            &complete_name, 
            &tags
        )?;
        
        component.to_assign_input_bus_fields.push((
            bus_name.to_string(), 
            access.clone(), 
            field_value,
            conditions_assignment.clone()
        )
        );
        
        Result::Ok(())
    }

    pub fn assign_value_to_bus_field_init(
        self: &mut ComponentRepresentation,
        bus_name: &str,
        access: &AccessingInformationBus,
        field_value: &FoldedResult,
        tags: &TagWire,
        conditions_assignment: &AssignmentState
    ) -> Result<(), MemoryError> {

        if !self.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        
        if !self.input_buses.contains_key(bus_name){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // Check that the assignment satisfies the tags requisites
        let mut complete_bus_name = vec![bus_name.to_string()];
        build_name(&mut complete_bus_name, access);
        ComponentRepresentation::handle_tag_assignment_init(self, &complete_bus_name, &tags)?;
        
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
        };

        single_bus.assign_value_to_field(
            access.field_access.as_ref().unwrap(),
            access.remaining_access.as_ref().unwrap(),
            folded_arg,
            true, 
            conditions_assignment
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
        !self.unassigned_tags.is_empty () || !self.unassigned_inputs.is_empty() 
    }
     

    /*
        Tags:
        - If an input receives a value that does not contain a expected tag ==> error
        - If an input receives a tag whose value is different to the expected (the one received earlier) ==> error
    
     */


    fn handle_tag_assignment_no_init(
        component: &mut ComponentRepresentation, 
        signal_name: &Vec<String>, 
        tags: &TagWire
    ) -> Result<(), MemoryError> {
        
        if !component.is_preinitialized() {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::NoInitializedComponent));
        }
        if !component.inputs_tags.contains_key(&signal_name[0]){
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentOutput));
        }

        // perform all the intermediate accesses to the field
        let mut accessed_info = &mut component.inputs_tags;
        for i in 0..signal_name.len()-1{
            accessed_info = accessed_info.get_mut(&signal_name[i]).unwrap().fields.as_mut().unwrap();
        }
        let input_tags = accessed_info.get_mut(&signal_name[signal_name.len()-1]).unwrap();

        // We copy tags in any case, complete or incomplete assignment
        // The values of the tags must be the same than the ones stored before
        
        fn check_tags(
            input_tags: &mut TagWire, 
            unassigned_tags: &mut HashSet<Vec<String>>,
            tags_info: &TagWire,
            signal_name: &mut Vec<String>
        )-> Result<(), MemoryError> {

             // remove the signal name of unassigned inputs
            let is_first_assignment_signal = unassigned_tags.contains(signal_name);
            unassigned_tags.remove(signal_name);

            for (t, value) in &mut input_tags.tags{
                if !tags_info.tags.contains_key(t){
                    // TODO: change error message to consider Vec<String>
                    return Result::Err(MemoryError::AssignmentMissingTags(signal_name[0].to_string(), t.clone()));
                } else{
                    if is_first_assignment_signal{
                        *value = tags_info.tags.get(t).unwrap().clone();
                    }
                    else{
                        // already given a value, check that it is the same
                        if value != tags_info.tags.get(t).unwrap(){
                            return Result::Err(MemoryError::AssignmentTagInputTwice(signal_name[0].to_string(), t.clone()));
                        }
                    }
                }
            }
            if input_tags.fields.is_some(){
                let input_fields = input_tags.fields.as_mut().unwrap();
                for (field_name, input_field) in input_fields{
                    let mut tags_assigned = &TagWire::default();
                    if tags_info.fields.is_some(){
                        let tags_fields = tags_info.fields.as_ref().unwrap();
                        if tags_fields.contains_key(field_name){
                            tags_assigned = tags_fields.get(field_name).unwrap();
                        }
                    }
                    signal_name.push(field_name.clone());
                    check_tags(
                        input_field,
                        unassigned_tags,
                        tags_assigned, 
                        signal_name
                    )?;
                    signal_name.pop();
                }
            }
            Result::Ok(())
        }

        check_tags(
            input_tags,
            &mut component.unassigned_tags,
            tags, 
            &mut signal_name.clone()
        )

    
    }

    fn handle_tag_assignment_init(
        component: &ComponentRepresentation,
        signal_name: &Vec<String>,
        tags: &TagWire
    )-> Result<(), MemoryError>{
        // perform all the intermediate accesses to the field
        let mut accessed_info = &component.inputs_tags;
        for i in 0..signal_name.len()-1{
            accessed_info = accessed_info.get(&signal_name[i]).unwrap().fields.as_ref().unwrap();
        }
        let input_tags = accessed_info.get(&signal_name[signal_name.len()-1]).unwrap();

        fn check_tags(input_tags: &TagWire, tags: &TagWire, signal_name: &mut Vec<String>)-> Result<(), MemoryError>{
            for (t, value) in &input_tags.tags{
                if !tags.tags.contains_key(t){
                    return Result::Err(MemoryError::AssignmentMissingTags(signal_name[0].to_string(), t.clone()));
                } else{            
                    // We are in the case where the component is initialized, so we 
                    // assume that all tags already have their value and check if it is
                    // the same as the one we are receiving
                    if value != tags.tags.get(t).unwrap(){
                        return Result::Err(MemoryError::AssignmentTagInputTwice(signal_name[0].to_string(), t.clone()));
                    }
                }
            }
            if input_tags.fields.is_some(){
                for (field_name, input_field) in input_tags.fields.as_ref().unwrap(){
                    
                    let mut tags_assigned = &TagWire::default();
                    if tags.fields.is_some(){
                        let tags_fields = tags.fields.as_ref().unwrap();
                        if tags_fields.contains_key(field_name){
                            tags_assigned = tags_fields.get(field_name).unwrap();
                        }
                    }
                    signal_name.push(field_name.clone());
                    check_tags(
                        input_field, 
                        tags_assigned, 
                        signal_name                    
                    )?;
                    signal_name.pop();
                }

            }
            Ok(())
        }
        let mut aux_name = signal_name.clone();
        check_tags(input_tags, tags, &mut aux_name)
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


// Auxiliar function to build the name of a bus field
fn build_name(complete_bus_name: &mut Vec<String>, access: &AccessingInformationBus){
    match &access.field_access{
        Option::None =>{},
        Option::Some(name) =>{
            complete_bus_name.push(name.clone());
        }
    }
    match &access.remaining_access{
        Option::None =>{},
        Option::Some(remaining) =>{
            build_name(complete_bus_name, &remaining)
        }
    }
}

