use program_structure::ast::Access;

use super::slice_types::{AExpressionSlice, FieldTypes, MemoryError, TypeInvalidAccess, TypeAssignmentError, SignalSlice, BusSlice, SliceCapacity,TagInfo, TagDefinitions, TagState};
use crate::execution_data::type_definitions::{NodePointer, AccessingInformationBus};
use crate::execution_data::ExecutedProgram;
use std::collections::{BTreeMap,HashMap, HashSet};
use crate::ast::Meta;
use std::mem;
use num_bigint_dig::BigInt;

use crate::assignment_utils::*;

pub struct BusRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub meta: Option<Meta>,
    fields: BTreeMap<String, FieldTypes>,
    pub field_tags: BTreeMap<String, (TagDefinitions, TagInfo)>,
    unassigned_fields: HashMap<String, SliceCapacity>,
    has_assignment: bool
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
    ) -> Result<(), MemoryError> {
        let possible_node = ExecutedProgram::get_bus_node(scheme, node_pointer);
        assert!(possible_node.is_some());
        let node = possible_node.unwrap();

        // Distinguir si es bus o señal y crear la Slice correspondiente
        // En caso de los buses, crear e inicializar componentRepresentation de todos

        for (symbol, route) in node.signal_fields() {
            let signal_slice = SignalSlice::new_with_route(route, &false);
            let signal_slice_size = SignalSlice::get_number_of_cells(&signal_slice);
            if signal_slice_size > 0{
                component.unassigned_fields
                    .insert(symbol.clone(), signal_slice_size);
            }
            let field_signal = FieldTypes::Signal(signal_slice);
            component.fields.insert(symbol.clone(), field_signal);
        }

        let bus_connexions = node.bus_connexions();

        for (symbol, route) in node.bus_fields() {
            
            let bus_node = bus_connexions.get(symbol).unwrap().inspect.goes_to;
            let mut bus_field = BusRepresentation::default();
            BusRepresentation::initialize_bus(
                &mut bus_field,
                bus_node,
                scheme,
            )?;
            let bus_slice = BusSlice::new_with_route(route, &bus_field);
            let bus_slice_size = BusSlice::get_number_of_cells(&bus_slice);
            if bus_slice_size > 0{
                component.unassigned_fields
                    .insert(symbol.clone(), bus_slice_size);
            }
            let field_bus = FieldTypes::Bus(bus_slice);
            component.fields.insert(symbol.clone(), field_bus);
        }

        component.node_pointer = Option::Some(node_pointer);


        Result::Ok(())
    }

    pub fn get_field_signal(
        &self, 
        field_name: &str,
        remaining_access: &AccessingInformationBus
    ) -> Result<((TagDefinitions, TagInfo), SignalSlice), MemoryError> {
        // TODO: REMOVE CLONE

        let field = self.fields.get(field_name).unwrap(); 
        let field_tags = self.field_tags.get(field_name).unwrap();
        if remaining_access.field_access.is_some(){
            // we are still considering a bus
            match field{
                FieldTypes::Bus(bus_slice)=>{

                    let memory_response = BusSlice::access_values(
                    &bus_slice, 
                        &remaining_access.array_access
                    );
                    match memory_response{
                        Result::Ok(bus_slice) =>{
                            assert!(bus_slice.is_single());
                            let resulting_bus = 
                                BusSlice::unwrap_to_single(bus_slice);
                            resulting_bus.get_field_signal( 
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
 
        } else{
            match field{
                FieldTypes::Signal(signals) =>{
                    // Case it is just a signal or an array of signals, 
                    // in this case there is no need for recursion
                    assert!(remaining_access.field_access.is_none());
                    Ok((field_tags.clone(), signals.clone()))
                }
                FieldTypes::Bus(_) => unreachable!(),
            }
        }

        // Returns the tags and a SignalSlice with true/false values
    }

    pub fn get_field_bus(
        &self,
        field_name: &str,
        remaining_access: &AccessingInformationBus
    ) -> Result<((TagDefinitions, TagInfo), BusSlice), MemoryError> {
        // TODO: REMOVE CLONE
        
        let field = self.fields.get(field_name).unwrap();
        let field_tags = self.field_tags.get(field_name).unwrap();
        
        if remaining_access.field_access.is_some(){
            // we are still considering an intermediate bus
            match field{
                FieldTypes::Bus(bus_slice)=>{

                let memory_response = BusSlice::access_values(
                &bus_slice, 
                    &remaining_access.array_access
                );
                match memory_response{
                    Result::Ok(bus_slice) =>{
                        assert!(bus_slice.is_single());
                        let resulting_bus = 
                            BusSlice::unwrap_to_single(bus_slice);
                        resulting_bus.get_field_bus( 
                        remaining_access.field_access.as_ref().unwrap(), 
                        &remaining_access.remaining_access.as_ref().unwrap())
                    }
                    Result::Err(err)=>{
                        return Err(err);
                    }
                }
            }
            FieldTypes::Signal(_) => unreachable!(),
        }       
        } else{
            match field{
                FieldTypes::Bus(buses) =>{
                    // Case it is the final array of buses that we must return

                    assert!(remaining_access.field_access.is_none());
                    Ok((field_tags.clone(), buses.clone()))
                }
                FieldTypes::Signal(_) => unreachable!(),
            }
        }
    }

    pub fn has_unassigned_fields(&self) -> bool{
        self.node_pointer.is_none() || !self.unassigned_fields.is_empty()
    }

    pub fn assign_value_to_field_signal(
        &mut self,
        field_name: &str,
        remaining_access: &AccessingInformationBus,
        slice_route: &[SliceCapacity],
        tags: TagInfo,
    ) -> Result<(), MemoryError> {

        self.has_assignment = true;

        let field: &mut FieldTypes = self.fields.get_mut(field_name).unwrap();

        // TODO: add quick check if completely assigned

        if remaining_access.field_access.is_some(){
            // we are still considering a bus
            match field{
                FieldTypes::Bus(bus_slice)=>{

                    let memory_response = BusSlice::access_values_by_mut_reference(
                    bus_slice, 
                        &remaining_access.array_access
                    );
                    match memory_response{
                        Result::Ok(mut bus_slice) =>{
                            assert!(bus_slice.len() == 1);
                            let resulting_bus = bus_slice.get_mut(0).unwrap();
                            resulting_bus.assign_value_to_field_signal( 
                                remaining_access.field_access.as_ref().unwrap(), 
                                &remaining_access.remaining_access.as_ref().unwrap(),
                                slice_route,
                                tags,
                            )
                        }
                        Result::Err(err)=>{
                            return Err(err);
                        }
                    }
                }
                FieldTypes::Signal(_) => unreachable!(),
            }
 
        } else{
            // in this case we are in a signal
            match field{
                FieldTypes::Signal(ref mut signal_slice) =>{
                    
                    // First we add the tags --> similar to what we do in execute
                    let (tags_definitions, tags_info) = self.field_tags.get_mut(field_name).unwrap();
     
                    let signal_is_init = SignalSlice::get_number_of_inserts(&signal_slice) > 0;

                    perform_tag_propagation(tags_info, tags_definitions, &tags, signal_is_init);

                    // Perform the signal assignment
                    perform_signal_assignment(signal_slice, &remaining_access.array_access, slice_route)?;

                    // Update the value of unnasigned fields
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
                        None => {}
                    }

                    // Update the value of the signal tags it is complete
                    let signal_is_completely_initialized = 
                        SignalSlice::get_number_of_inserts(signal_slice) == 
                        SignalSlice::get_number_of_cells(signal_slice);

                    if signal_is_completely_initialized {

                        for (tag, _value) in tags_info{
                            let tag_state = tags_definitions.get_mut(tag).unwrap();
                            tag_state.complete = true;
                            
                        }
                    }

                    Result::Ok(())
                }
                FieldTypes::Bus(_) => unreachable!(),
            }
        }
    }



    pub fn assign_value_to_field_tag(
        &mut self,
        field_name: &str,
        remaining_access: &AccessingInformationBus,
        value: BigInt,
    ) -> Result<(), MemoryError> {

        let field: &mut FieldTypes = self.fields.get_mut(field_name).unwrap();
        
        // we need to stop when there is something like bus.field.tag
        // distance 2 between the tag and where we add it

        // the access with distance 2
        let next_access = remaining_access.remaining_access.as_ref().unwrap();

        if next_access.field_access.is_some(){
            // we are still considering a bus
            match field{
                FieldTypes::Bus(bus_slice)=>{

                    let memory_response = BusSlice::access_values_by_mut_reference(
                        bus_slice, 
                        &remaining_access.array_access
                    );
                    match memory_response{
                        Result::Ok(mut bus_slice) =>{
                            assert!(bus_slice.len() == 1);
                            let resulting_bus = bus_slice.get_mut(0).unwrap();
                            resulting_bus.assign_value_to_field_tag( 
                                remaining_access.field_access.as_ref().unwrap(), 
                                &remaining_access.remaining_access.as_ref().unwrap(),
                                value
                            )
                        }
                        Result::Err(err)=>{
                            return Err(err);
                        }
                    }
                }
                FieldTypes::Signal(_) => unreachable!(),
            }
 
        } else{
            // just add the tag to the field
            // distance 2 (self.field.tag)
            let tag = remaining_access.field_access.as_ref().unwrap();

            let (tags_status, tags_value) = self.field_tags.get_mut(field_name).unwrap();

            match field{
                FieldTypes::Signal(s) =>{
                    let signal_is_init = SignalSlice::get_number_of_inserts(&s) > 0;
                    if signal_is_init{
                        return Result::Err(MemoryError::AssignmentTagAfterInit)
                    }
                }
                FieldTypes::Bus(s) =>{
                    // TODO, include info about assignments, no recorrer todo
                    for i in 0..BusSlice::get_number_of_cells(s){
                        let accessed_bus = BusSlice::get_reference_to_single_value_by_index(&s, i)?;
                        if accessed_bus.has_assignment(){
                            return Result::Err(MemoryError::AssignmentTagAfterInit)
                        }
                    }
                }   
            }

            
            let possible_tag = tags_value.get_mut(tag);
            if let Some(val) = possible_tag {
                if let Some(_) = val {
                    Result::Err(MemoryError::AssignmentTagTwice)
                } else { // we add the info saying that the tag is defined
                    let tag_state = tags_status.get_mut(tag).unwrap();
                    tag_state.value_defined = true;
                    tags_value.insert(tag.clone(), Option::Some(value));
                    Result::Ok(())

                }   
            } else{
                unreachable!("Tag does not exist");
            }
        }
    }


    pub fn assign_value_to_field_bus(
        &mut self,
        field_name: &str,
        remaining_access: &AccessingInformationBus,
        slice_route: &[SliceCapacity],
        assigned_bus: &BusSlice,
        tags: TagInfo
    ) -> Result<(), MemoryError> {

        self.has_assignment = true;


        let field: &mut FieldTypes = self.fields.get_mut(field_name).unwrap();

        // TODO: add quick check if completely assigned

        if remaining_access.field_access.is_some(){
            // we are still considering a bus
            match field{
                FieldTypes::Bus(bus_slice)=>{

                    let memory_response = BusSlice::access_values_by_mut_reference(
                    bus_slice, 
                        &remaining_access.array_access
                    );
                    match memory_response{
                        Result::Ok(mut bus_slice) =>{
                            assert!(bus_slice.len() == 1);
                            let resulting_bus = bus_slice.get_mut(0).unwrap();
                            resulting_bus.assign_value_to_field_bus( 
                                remaining_access.field_access.as_ref().unwrap(), 
                                &remaining_access.remaining_access.as_ref().unwrap(),
                                slice_route,
                                assigned_bus,
                                tags
                            )
                        }
                        Result::Err(err)=>{
                            return Err(err);
                        }
                    }
                }
                FieldTypes::Signal(_) => unreachable!(),
            }
 
        } else{
            // in this case we are in a signal
            match field{
                FieldTypes::Bus(ref mut bus_slice) =>{
                    
                    // First we add the tags --> similar to what we do in execute
                    let (tags_definitions, tags_info) = self.field_tags.get_mut(field_name).unwrap();
                    
                    let mut bus_is_init = false;
                    for i in 0..BusSlice::get_number_of_cells(bus_slice){
                        match BusSlice::get_reference_to_single_value_by_index(bus_slice, i){
                            Ok(bus) => {
                                bus_is_init |= bus.has_assignment();
                            }
                            Err(_) => unreachable!()
                        }
                    }

                    perform_tag_propagation(tags_info, tags_definitions, &tags, bus_is_init);
                    

                    // We completely assign each one of the buses

                    let bus_previous_value = BusSlice::access_values_by_mut_reference(
                         bus_slice,
                         &remaining_access.array_access,
                    )?;

                    let mut index = 0;
                    let dim_slice = bus_previous_value.len();

                    for bus_assigned in bus_previous_value{
                        let value = BusSlice::get_reference_to_single_value_by_index(&assigned_bus, index)?;

                        bus_assigned.completely_assign_bus(&value)?;
                        index += 1;
                    }

                    // Update the value of unnasigned fields
                    match self.unassigned_fields.get_mut(field_name){
                        Some(left) => {
                            *left -= dim_slice;
                            if *left == 0 {
                                self.unassigned_fields.remove(field_name);
                            }
                        }
                        None => {}
                    }

                    // Update the value of the signal tags it is complete
                    let mut bus_is_completely_init = true;
                    for i in 0..BusSlice::get_number_of_cells(bus_slice){
                        match BusSlice::get_reference_to_single_value_by_index(bus_slice, i){
                            Ok(bus) => {
                                bus_is_completely_init &= bus.has_assignment();
                            }
                            Err(_) => unreachable!()
                        }
                    }

                    if bus_is_completely_init {

                        for (tag, _value) in tags_info{
                            let tag_state = tags_definitions.get_mut(tag).unwrap();
                            tag_state.complete = true;
                            
                        }
                    }

                    Result::Ok(())
                }
                FieldTypes::Signal(_) => unreachable!(),
            }
        }
    }


    pub fn completely_assign_bus(&mut self, assigned_bus: &BusRepresentation)-> Result<(), MemoryError>{
        if self.has_assignment{
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments));
        }
        self.has_assignment = true;
        for (field_name, value)  in &mut self.fields{
            
            // update the tags
            
            let (tags_definition, tags_info) = self.field_tags.get_mut(field_name).unwrap();
            let (tags_assigned_definition, tags_assigned_info) =  assigned_bus.field_tags.get(field_name).unwrap();
                
            let mut tags_propagated = TagInfo::new();
            for (tag, value) in tags_assigned_info{
                let state = tags_assigned_definition.get(tag).unwrap();
                if state.value_defined || state.complete{
                    tags_propagated.insert(tag.clone(), value.clone());
                } else if state.defined{
                    tags_propagated.insert(tag.clone(), None);
                }
            }

            let is_init = match value{
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
                },
                FieldTypes::Signal(signal_slice)=>{
                    SignalSlice::get_number_of_inserts(&signal_slice) > 0
                }
            };

            // perform the tag assignment
            perform_tag_propagation(tags_info, tags_definition, &tags_propagated, is_init);


            match value{
                FieldTypes::Bus(ref mut bus_slice) =>{

                    let bus_slice_assigned = match assigned_bus.fields.get(field_name).unwrap(){
                        FieldTypes::Bus(bs) => bs,
                        FieldTypes::Signal(_) => unreachable!(),
                    };

                    let assignment_result = perform_bus_assignment(bus_slice, &[], bus_slice_assigned);

                    if assignment_result.is_err(){
                        return Err(assignment_result.err().unwrap());
                    }
                },
                FieldTypes::Signal(signal_slice)=>{
                    // check if not assigned yet
                    // set to true
                    // updated unassigned_fields

                    let new_value_slice = &SignalSlice::new_with_route(signal_slice.route(), &true);
           
                    // Not needed because we know that it has not been assigned?
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
            for (tag, _value) in tags_info{
                let tag_state = tags_definition.get_mut(tag).unwrap();
                tag_state.complete = true;
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
