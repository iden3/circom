
use super::slice_types::{BusSlice, FieldTypes, FoldedArgument, FoldedResult, MemoryError, SignalSlice, SliceCapacity, TagDefinitions, TagInfo, TagState, TypeAssignmentError};
use crate::execution_data::type_definitions::{NodePointer, AccessingInformationBus};
use crate::execution_data::ExecutedProgram;
use std::collections::{BTreeMap,HashMap};
use crate::ast::Meta;

use crate::assignment_utils::*;

pub struct BusRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub meta: Option<Meta>,
    fields: BTreeMap<String, FieldTypes>,
    pub field_tags: BTreeMap<String, (TagDefinitions, TagInfo)>,
    unassigned_fields: HashMap<String, SliceCapacity>,
    has_assignment: bool,
}

impl Default for BusRepresentation {
    fn default() -> Self {
        BusRepresentation {
            node_pointer: Option::None,
            fields: BTreeMap::new(),
            field_tags: BTreeMap::new(),
            meta: Option::None,
            unassigned_fields: HashMap::new(),
            has_assignment: false
        }
    }
}
impl Clone for BusRepresentation {
    fn clone(&self) -> Self {
        BusRepresentation {
            node_pointer: self.node_pointer,
            fields: self.fields.clone(),
            field_tags: self.field_tags.clone(),
            meta : self.meta.clone(),
            unassigned_fields: self.unassigned_fields.clone(),
            has_assignment: self.has_assignment
        }
    }
}

impl BusRepresentation {
 
        
    pub fn initialize_bus(
        component: &mut BusRepresentation,
        node_pointer: NodePointer,
        scheme: &ExecutedProgram,
        is_initialized: bool,
    ) -> Result<(), MemoryError> {
        let possible_node = ExecutedProgram::get_bus_node(scheme, node_pointer);
        assert!(possible_node.is_some());
        let node = possible_node.unwrap();


        // if input bus all signals are set initialize to true, else to false
        if is_initialized{
            component.has_assignment = true;
        }
        // initialice the signals
        for info_field in node.fields() {
            let symbol = &info_field.name;
            let route = &info_field.length;
            if !info_field.is_bus{
                let signal_slice = SignalSlice::new_with_route(&route, &is_initialized);
                let signal_slice_size = SignalSlice::get_number_of_cells(&signal_slice);
                if signal_slice_size > 0{
                    component.unassigned_fields
                        .insert(symbol.clone(), signal_slice_size);
                }
                let field_signal = FieldTypes::Signal(signal_slice);
                component.fields.insert(symbol.clone(), field_signal);
            } else{
                let bus_connexions = node.bus_connexions();
                let bus_node = bus_connexions.get(symbol).unwrap().inspect.goes_to;
                let mut bus_field = BusRepresentation::default();
                BusRepresentation::initialize_bus(
                    &mut bus_field,
                    bus_node,
                    scheme,
                    is_initialized
                )?;
                let bus_slice = BusSlice::new_with_route(&route, &bus_field);
                let bus_slice_size = BusSlice::get_number_of_cells(&bus_slice);
                if bus_slice_size > 0{
                    component.unassigned_fields
                        .insert(symbol.clone(), bus_slice_size);
                }
                let field_bus = FieldTypes::Bus(bus_slice);
                component.fields.insert(symbol.clone(), field_bus);
            }
            

            // add the tags 
            if node.signal_to_tags.get(symbol).is_some(){
                let defined_tags = node.signal_to_tags.get(symbol).unwrap();
                let mut definitions = BTreeMap::new();
                let mut values = BTreeMap::new();
                for (tag, value) in defined_tags{
                    let tag_state = TagState{defined:true, value_defined: value.is_some(), complete: true};
                    definitions.insert(tag.clone(), tag_state);
                    values.insert(tag.clone(), value.clone());

                }
                component.field_tags.insert(symbol.clone(), (definitions, values));
            } else{
                component.field_tags.insert(symbol.clone(), (BTreeMap::new(), BTreeMap::new()));
            }
            if is_initialized{
                component.unassigned_fields.remove(symbol);
            }
        }


        component.node_pointer = Option::Some(node_pointer);


        Result::Ok(())
    }

    pub fn get_field(
        &self, 
        field_name: &str,
        remaining_access: &AccessingInformationBus
    ) -> Result<(Option<TagInfo>, FoldedResult), MemoryError>{
        
        let field = self.fields.get(field_name).unwrap(); 
        let (tags_defs, tags_info) = self.field_tags.get(field_name).unwrap();
        if remaining_access.field_access.is_some(){
            // we are still considering an intermediate bus or a tag, check cases
            let next_access = remaining_access.field_access.as_ref().unwrap();
            if tags_info.contains_key(next_access){
                // case tag, return its value
                
                // access only allowed when (1) it is value defined by user or (2) it is completely assigned
                let state = tags_defs.get(next_access).unwrap();
                if state.value_defined || state.complete{
                    let value_tag = tags_info.get(next_access).unwrap();
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
                    let error = MemoryError::TagValueNotInitializedAccess;
                    Result::Err(error)
                }
            } else{
                // case bus, access to the next field
                match field{
                    FieldTypes::Bus(bus_slice)=>{
    
                        let memory_response = BusSlice::access_values_by_reference(
                        &bus_slice, 
                            &remaining_access.array_access
                        );
                        match memory_response{
                            Result::Ok(bus_slice) =>{
                                assert!(bus_slice.len() == 1);
                                let resulting_bus = bus_slice[0];
                                resulting_bus.get_field( 
                                    remaining_access.field_access.as_ref().unwrap(), 
                                    &remaining_access.remaining_access.as_ref().unwrap()
                                )
                            }
                            Result::Err(err)=>{
                                return Err(err);
                            }
                        }
                    }
                    FieldTypes::Signal(_) => unreachable!(),
                }
            }
 
        } else{
            // in this case there is no need for recursion, final access
            
            match field{
                FieldTypes::Signal(signal_slice) =>{
                    // Case it is just a signal or an array of signals, 
                    // in this case there is no need for recursion
                    
                    // compute which tags are propagated
                    let propagated_tags = compute_propagated_tags(tags_info, tags_defs);
                    
                    let accessed_slice_result = SignalSlice::access_values(&signal_slice, &remaining_access.array_access);
                    match accessed_slice_result{
                        Ok(slice) =>{
                            let folded_slice = FoldedResult::Signal(slice);
                            Result::Ok((Some(propagated_tags), folded_slice))
                        },
                        Err(err) => Err(err)
                    }                
                }
                FieldTypes::Bus(bus_slice) => {
                    // Case it is just a bus or an array of buses, 
                    // in this case there is no need for recursion

                    // compute which tags are propagated
                    let propagated_tags = compute_propagated_tags(tags_info, tags_defs);
                    
                    let accessed_slice_result = BusSlice::access_values(&bus_slice, &remaining_access.array_access);
                    match accessed_slice_result{
                        Ok(slice) =>{
                            let folded_slice = FoldedResult::Bus(slice);
                            Result::Ok((Some(propagated_tags), folded_slice))
                        },
                        Err(err) => Err(err)
                    }  
                },
            }
        }
    }

   

    pub fn has_unassigned_fields(&self) -> bool{
        self.node_pointer.is_none() || !self.unassigned_fields.is_empty()
    }

    pub fn assign_value_to_field(
        &mut self,
        field_name: &str,
        remaining_access: &AccessingInformationBus,
        assigned_value: FoldedArgument,
        tags: Option<TagInfo>,
        is_input: bool,
    ) -> Result<(), MemoryError> {
            // TODO: move to auxiliar function to do not repeat effort
            // We update the has_assignment value if not tag and not empty
            let has_assignment = match assigned_value{
                FoldedArgument::Signal(dimensions)=>{
                    let total_size = dimensions.iter().fold(1, |acc, x| acc * x);
                    total_size > 0
                },
                FoldedArgument::Bus(slice)=>{
                    let route = slice.route();
                    let total_size = route.iter().fold(1, |acc, x| acc * x);
                    total_size > 0
                },
                FoldedArgument::Tag(_) => false
            };
            if has_assignment{
                self.has_assignment = true;
            }

            // We later distinguish the case of tags
            // check if we need to access to another bus or if it is the final access
            let field: &mut FieldTypes = self.fields.get_mut(field_name).unwrap();
            let (status_tags, info_tags) = self.field_tags.get_mut(field_name).unwrap();

            if remaining_access.field_access.is_some(){
                // case still intermediate access or a tag
                let next_access = remaining_access.field_access.as_ref().unwrap();

                // Distinguish between tag and intermediate access
                if info_tags.contains_key(next_access){
                    
                    // it is tag assignment -> check if valid
                    match field{
                        FieldTypes::Signal(s) =>{
                            let signal_is_init = SignalSlice::get_number_of_inserts(&s) > 0;
                            if signal_is_init{
                                return Result::Err(MemoryError::AssignmentTagAfterInit)
                            }
                        }
                        FieldTypes::Bus(s) =>{
                            for i in 0..BusSlice::get_number_of_cells(s){
                                let accessed_bus = BusSlice::get_reference_to_single_value_by_index(&s, i)?;
                                if accessed_bus.has_assignment(){
                                    return Result::Err(MemoryError::AssignmentTagAfterInit)
                                }
                            }
                        }   
                    }
                    // Get the assigned value
                    let value = match assigned_value{
                        FoldedArgument::Tag(value) =>{
                            value
                        },
                        _ => unreachable!()
                    };
                    
                    let possible_tag = info_tags.get_mut(next_access);
                    if let Some(val) = possible_tag {
                        if let Some(_) = val {
                            // already assigned value, return error
                            Result::Err(MemoryError::AssignmentTagTwice)
                        } else { // we add the info saying that the tag is defined
                            let tag_state = status_tags.get_mut(next_access).unwrap();
                            tag_state.value_defined = true;
                            *val = Option::Some(value.clone());
                            Result::Ok(())
                        }   
                    } else{
                        unreachable!()
                    }
                } else{
                    // it is intermediate access

                    match field{
                        FieldTypes::Bus(bus_slice)=>{
                            // case bus -> apply recursion

                            // no tags assigned to the complete bus 
                            // Check in case input if it is expecting tags, if so return error
                            if is_input{
                                if !info_tags.is_empty(){
                                    let (possible_tag, _) = info_tags.iter().next().unwrap();
                                    return Result::Err(MemoryError::AssignmentMissingTags(field_name.to_string(), possible_tag.clone()));
                                }
                            }


                            let memory_response = BusSlice::access_values_by_mut_reference(
                            bus_slice, 
                                &remaining_access.array_access
                            );                            
                            match memory_response{
                                Result::Ok(mut bus_slice) =>{
                                    assert!(bus_slice.len() == 1);
                                    let resulting_bus = bus_slice.get_mut(0).unwrap();
                                    resulting_bus.assign_value_to_field( 
                                        remaining_access.field_access.as_ref().unwrap(), 
                                        &remaining_access.remaining_access.as_ref().unwrap(),
                                        assigned_value,
                                        tags,
                                        is_input
                                    )?;

                                    // Update from unassigned if it is completely assigned
                                    if !resulting_bus.has_unassigned_fields(){
                                        match self.unassigned_fields.get_mut(field_name){
                                            Some(left) => {
                                                *left -= 1;
                                                if *left == 0 {
                                                    self.unassigned_fields.remove(field_name);
                                                }
                                            }
                                            Option::None => {}
                                        }
                                    }
                                    Result::Ok(())
                                }
                                Result::Err(err)=>{
                                    return Err(err);
                                }
                            }
                        }
                        FieldTypes::Signal(_) => {
                            // no possible, already checked in check_types
                            unreachable!()
                        }
                    }
                }

            } else{
                // case final assignment of signal or bus
                let tags = tags.unwrap();

                // first propagate the tags or check if conditions satisfied if input
                let is_init = match field{
                    FieldTypes::Signal(signal_slice) =>{
                        SignalSlice::get_number_of_inserts(&signal_slice) > 0
                    },
                    FieldTypes::Bus(bus_slice) =>{
                        let mut bus_is_init = false;
                        for i in 0..BusSlice::get_number_of_cells(bus_slice){
                            match BusSlice::get_reference_to_single_value_by_index(bus_slice, i){
                                Ok(bus) => {
                                    bus_is_init |= bus.has_assignment();
                                }
                                Err(_) => unreachable!()
                            }
                        }
                        bus_is_init
                    }
                };
                if !is_input{
                    // case no input, just propagate
                    perform_tag_propagation(info_tags, status_tags, &tags, is_init);
                } else{
                    // in case input check if tags are satisfied
                    for (t, value) in info_tags{
                        if !tags.contains_key(t){
                            return Result::Err(MemoryError::AssignmentMissingTags(field_name.to_string(), t.clone()));
                        } else{
                            if !is_init{
                                // First assignment of input tag
                                *value = tags.get(t).unwrap().clone();
                            }
                            else{
                                // already given a value, check that it is the same
                                // if not return error
                                if value != tags.get(t).unwrap(){
                                    return Result::Err(MemoryError::AssignmentTagInputTwice(field_name.to_string(), t.clone()));
                                }
                            }
                        }
                    }
                }

                // then assign the values to the signal or bus
                match field{
                    FieldTypes::Signal(signal_slice) =>{
                        let route = match assigned_value{
                            FoldedArgument::Signal(signal_slice_route) =>{
                                signal_slice_route
                            },
                            _ => unreachable!()
                        };
                        perform_signal_assignment(signal_slice, &remaining_access.array_access, route)?;
                    },
                    FieldTypes::Bus(bus_slice) =>{
                        let assigned_bus_slice = match assigned_value{
                            FoldedArgument::Bus(bus_slice) =>{
                                bus_slice
                            },
                            _ => unreachable!()
                        };
                        perform_bus_assignment(bus_slice, &remaining_access.array_access, assigned_bus_slice, is_input)?;

                    }
                }

                // Update the value of unnasigned fields
                let slice_route = match assigned_value{
                    FoldedArgument::Signal(signal_slice_route) =>{
                        signal_slice_route
                    },
                    FoldedArgument::Bus(bus_slice) =>{
                        bus_slice.route()
                    },
                    _ => unreachable!()
                };

                let mut dim_slice = 1;
                for i in slice_route {
                    dim_slice *= *i;
                }
                    
                match self.unassigned_fields.get_mut(field_name){
                    Some(left) => {
                        *left -= dim_slice;
                        if *left == 0 {
                            self.unassigned_fields.remove(field_name);
                        }
                    }
                    Option::None => {}
                }

                // Update the value of the signal tags it is complete

                let is_completely_initialized = match field{
                    FieldTypes::Signal(signal_slice) =>{
                        SignalSlice::get_number_of_inserts(signal_slice) == 
                            SignalSlice::get_number_of_cells(signal_slice)
                    },
                    FieldTypes::Bus(bus_slice) =>{
                        let mut bus_is_completely_init = true;
                        for i in 0..BusSlice::get_number_of_cells(bus_slice){
                            match BusSlice::get_reference_to_single_value_by_index(bus_slice, i){
                                Ok(bus) => {
                                    bus_is_completely_init &= bus.has_assignment();
                                }
                                Err(_) => unreachable!()
                            }
                        }
                        bus_is_completely_init
                    }

                };
                    
                if is_completely_initialized && !is_input{

                    for (_tag, state) in status_tags{
                        state.complete = true;
                    }  
                }
                Ok(())
            }
        
    }

    pub fn completely_assign_bus(&mut self, assigned_bus: &BusRepresentation, is_input: bool)-> Result<(), MemoryError>{
        
        if self.has_assignment{
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignmentsBus));
        }

        // check that they are the same instance of buses
        if self.node_pointer != assigned_bus.node_pointer{
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::DifferentBusInstances));
        }

        self.has_assignment = true;
        for (field_name, value)  in &mut self.fields{
            
            // get the tags that are propagated
            let (tags_definition, tags_info) = self.field_tags.get_mut(field_name).unwrap();
            let (tags_assigned_definition, tags_assigned_info) =  assigned_bus.field_tags.get(field_name).unwrap();
            let tags_propagated = compute_propagated_tags(tags_assigned_info, tags_assigned_definition);

            let is_init = false;

            // perform the tag assignment
            if !is_input{
                // case no input, just propagate
                perform_tag_propagation(tags_info, tags_definition, &tags_propagated, is_init);
            } else{

                // in case input check if tags are satisfied
                for (t, value) in tags_info{
                    if !tags_propagated.contains_key(t){
                        return Result::Err(MemoryError::AssignmentMissingTags(field_name.to_string(), t.clone()));
                    } else{
                        // Not needed check, always not init, if not error
                        // First assignment of input tag
                        *value = tags_propagated.get(t).unwrap().clone();
                    }
                }
            }

            // perform the assignment
            match value{
                FieldTypes::Bus(ref mut bus_slice) =>{

                    let bus_slice_assigned = match assigned_bus.fields.get(field_name).unwrap(){
                        FieldTypes::Bus(bs) => bs,
                        FieldTypes::Signal(_) => unreachable!(),
                    };

                    let assignment_result = perform_bus_assignment(bus_slice, &[], bus_slice_assigned, is_input);

                    if assignment_result.is_err(){
                        return Err(assignment_result.err().unwrap());
                    }
                },
                FieldTypes::Signal(signal_slice)=>{
                    // check if not assigned yet
                    // set to true
                    // updated unassigned_fields

                    let new_value_slice = &SignalSlice::new_with_route(signal_slice.route(), &true);
           
                    // : Not needed because we know that it has not been assigned?
                    // let dim_slice: usize = SignalSlice::get_number_of_cells(signal_slice);
                    // for i in 0..dim_slice{
                    //     let signal_was_assigned = match SignalSlice::access_value_by_index(&signal_slice, i){
                    //         Ok(v) => v,
                    //         Err(_) => unreachable!()
                    //     };
                    //     if signal_was_assigned {
                    //         return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments));
                    //     }
                    // }

                    SignalSlice::insert_values(
                        signal_slice,
                        &Vec::new(),
                        &new_value_slice,
                        true
                    )?;
                }
                
            }

            // Update the value of unnasigned fields
            self.unassigned_fields.remove(field_name);
            
            // Update the value of the complete tags
            for (_tag, state) in tags_definition{
                state.complete = true;
            }       
        }
        Ok(())

    }

    pub fn get_accesses_bus(&self, name: &str) -> Vec<String>{

        fn unfold_signals(current: String, dim: usize, lengths: &[usize], result: &mut Vec<String>) {
            if dim == lengths.len() {
                result.push(current);
            } else {
                for i in 0..lengths[dim] {
                    unfold_signals(format!("{}[{}]", current, i), dim + 1, lengths, result)
                }
            }
        }
        
        let mut result = Vec::new();
        for field in &self.fields{
            match field{
                (field_name, FieldTypes::Bus(bus_slice)) => {
                    let accessed_name = format!("{}.{}", name, field_name);
                    let dims = bus_slice.route();
                    let mut prefixes = Vec::new();
                    unfold_signals(accessed_name, 0, dims, &mut prefixes);
                    for i in 0..BusSlice::get_number_of_cells(&bus_slice){
                        let access = BusSlice::get_reference_to_single_value_by_index(&bus_slice, i);

                        match access{
                            Ok(bus) =>{
                                let mut result_field = bus.get_accesses_bus(&prefixes[i]);
                                result.append(&mut result_field);
                            }   
                            Err(_) =>{
                                unreachable!()
                            }
                        }
                    }
                }
                (field_name, FieldTypes::Signal(signal_slice)) =>{
                    let accessed_name = format!("{}.{}", name, field_name);
                    let dims = signal_slice.route();
                    unfold_signals(accessed_name, 0, dims, &mut result);
                }
            }
        }
        result
    }



    pub fn has_assignment(&self)-> bool{
        self.has_assignment
    }

}
