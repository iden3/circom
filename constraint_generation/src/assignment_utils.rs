use super::environment_utils::

    slice_types::{MemoryError, TypeAssignmentError, 
        SignalSlice, SliceCapacity, TagInfo, TagState, TagDefinitions, 
        BusSlice, BusTagInfo
    };
use crate::environment_utils::slice_types::AssignmentState;
use crate::execution_data::type_definitions::TagWire;
use std::mem;
use std::collections::HashMap;

// Utils for assigning tags

pub fn compute_propagated_tags_bus(tag_data: &BusTagInfo) -> TagWire{
    let tags_propagated = compute_propagated_tags(
        &tag_data.tags, 
        &tag_data.definitions,
        tag_data.remaining_inserts
    );
    let mut fields_propagated = HashMap::new();
    for (field_name, field_tags) in &tag_data.fields{
        fields_propagated.insert(field_name.clone(), compute_propagated_tags_bus(field_tags));
    }
    TagWire{
        tags: tags_propagated,
        fields: Some(fields_propagated),
    }
}

pub fn compute_propagated_tags(
    tags_values: &TagInfo, 
    tags_definitions: &TagDefinitions, 
    remaining_inserts: usize
)-> TagInfo{
    let mut tags_propagated = TagInfo::new();
    for (tag, value) in tags_values{
        let state = tags_definitions.get(tag).unwrap();
        if state.value_defined || remaining_inserts == 0{
            tags_propagated.insert(tag.clone(), value.clone());
        } else if state.defined{
            tags_propagated.insert(tag.clone(), None);
        }
    }
    tags_propagated
}

pub fn perform_tag_propagation(tags_values: &mut TagInfo, tags_definitions: &mut TagDefinitions, assigned_tags: &TagInfo, is_init: bool){
        // Study the tags: add the new ones and copy their content.
        /*
        Cases:

            Inherance in arrays => We only have a tag in case it inherites the tag in all positions
            
            - Tag defined by user: 
                * Already with value defined by user => do not copy new values
                * No value defined by user
                   - Already initialized:
                     * If same value as previous preserve
                     * If not set value to None
                   - No initialized:
                     * Set value to new one
            - Tag not defined by user:
                * Already initialized:
                   - If contains same tag with same value preserve
                   - No tag or different value => do not save tag or loose it
                * No initialized:
                   - Save tag


        */
        let previous_tags = mem::take(tags_values);

        for (tag, value) in previous_tags{
            let tag_state =  tags_definitions.get(&tag).unwrap();
            if tag_state.defined{// is signal defined by user
                if tag_state.value_defined{
                    // already with value, store the same value
                    tags_values.insert(tag, value);
                } else{
                    if is_init {
                        // only keep value if same as previous
                        let to_store_value = if assigned_tags.contains_key(&tag){
                            let value_new = assigned_tags.get(&tag).unwrap();
                            if value != *value_new{
                                None
                            } else{
                                value
                            }
                        } else{
                            None
                        };
                        tags_values.insert(tag, to_store_value);
                    } else{
                        // always keep
                        if assigned_tags.contains_key(&tag){
                            let value_new = assigned_tags.get(&tag).unwrap();
                            tags_values.insert(tag, value_new.clone());
                        } else{
                            tags_values.insert(tag, None);
                        }
                    }
                }
            } else{
                // it is not defined by user
                if assigned_tags.contains_key(&tag){
                    let value_new = assigned_tags.get(&tag).unwrap();
                    if value == *value_new{
                        tags_values.insert(tag, value);
                    } else{
                        tags_values.remove(&tag);
                    }
                } else{
                    tags_values.remove(&tag);
                }
            }
        } 

        if !is_init{ // first init, add new tags
            for (tag, value) in assigned_tags{
                if !tags_values.contains_key(tag){ // in case it is a new tag (not defined by user)
                    tags_values.insert(tag.clone(), value.clone());
                    let state = TagState{defined: false, value_defined: false};
                    tags_definitions.insert(tag.clone(), state);
                }
            }
        }

}

pub fn perform_tag_propagation_bus(tag_data: &mut BusTagInfo, assigned_tags: &TagWire, n_inserts: usize){
    perform_tag_propagation(&mut tag_data.tags, &mut tag_data.definitions, &assigned_tags.tags, tag_data.is_init);
    tag_data.remaining_inserts -= n_inserts; 
    tag_data.is_init = true;

    for (field_name, field_data) in &mut tag_data.fields{
        // if the field does not appear in the assigned tags we take an empty TagWire
        let mut field_assigned = &TagWire::default();
        
        if assigned_tags.fields.is_some() {
            let assigned_tag_fields = assigned_tags.fields.as_ref().unwrap();
            if assigned_tag_fields.contains_key(field_name){ // check if it appears in the fields
                field_assigned = assigned_tag_fields.get(field_name).unwrap();
            } 
        }
        let field_n_inserts = field_data.size * n_inserts;
        perform_tag_propagation_bus(field_data, field_assigned, field_n_inserts);
    }
}


pub fn perform_signal_assignment(
    signal_slice: &mut SignalSlice, 
    array_access: &[SliceCapacity], 
    new_route: &[SliceCapacity],
    conditions_assignment: &AssignmentState
)-> Result<(), MemoryError>{
    let memory_response_for_signal_previous_value = SignalSlice::access_values(
        signal_slice,
        array_access,
    );
    let signal_previous_value = match memory_response_for_signal_previous_value{
        Ok(v) => v,
        Err(err) => return Err(err)
    };

    let new_value_slice = &SignalSlice::new_with_route(new_route, &conditions_assignment);

    let correct_dims_result = SignalSlice::check_correct_dims(
        &signal_previous_value, 
        &Vec::new(), 
        &new_value_slice, 
        true
    );
    match correct_dims_result{
        Ok(_) => {},
        Err(err) => return Err(err)
    };

    let memory_response_for_signal_previous_value = SignalSlice::access_values_by_mut_reference(
        signal_slice,
        array_access,
    );
    let mut signal_previous_value = match memory_response_for_signal_previous_value{
        Ok(v) => v,
        Err(err) => return Err(err)
    };

    for i in 0..signal_previous_value.len(){
        let signal_was_assigned = signal_previous_value.get_mut(i).unwrap();
        match signal_was_assigned{
            AssignmentState::NoAssigned =>{
                **signal_was_assigned = conditions_assignment.clone();
            }
            AssignmentState::Assigned(meta) =>{
                return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments(meta.as_ref().unwrap().clone())));
            }
            AssignmentState::MightAssigned(cond_old, meta) =>{
                match conditions_assignment{
                    AssignmentState::Assigned(_) =>{
                        // TODO: return warning?
                        return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments(meta.as_ref().unwrap().clone())));
                    }
                    AssignmentState::MightAssigned(cond_new, meta_new) =>{
                        // Possibilities:

                        // If they are subsets of each other
                        //    --> If A subset B or B subset A return error("Multiple assignments")

                        // If they are equal until last condition where one true, other false
                        //    --> remove this last level (if single level then change to assigned)

                        // Other case
                        //    --> Just keep the last one??? 
                        //    --> TODO: Return a warning??? 


                        let mut is_subset = true;
                        let mut eq_until_last = false;
                        let mut different_branches = false;

                        let min = std::cmp::min(cond_old.len(), cond_new.len());

                        for i in 0..min{
                            if cond_old[i] != cond_new[i]{
                                // Probably not needed the is_subset
                                is_subset = false;
                                if cond_old[i].0 == cond_new[i].0{
                                    // Different branches of if
                                    different_branches = true;
                                    if i == min - 1 && cond_old.len() == cond_new.len(){
                                        // last case in both -> update one level less
                                        // Probably not needed
                                        eq_until_last = true;
                                    } 
                                }
                                break;
                            }
                        }

                        if is_subset{
                            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments(meta.as_ref().unwrap().clone())));
                        } else if eq_until_last{
                            if cond_new.len() == 1{ // case single level
                                **signal_was_assigned = AssignmentState::Assigned(meta_new.clone());
                            } else{
                                let new_cond = cond_new[0..min -1].to_vec();
                                **signal_was_assigned = AssignmentState::MightAssigned(new_cond, meta_new.clone());
                            }
                        } else if different_branches{
                            **signal_was_assigned = AssignmentState::MightAssigned(cond_new.clone(), meta_new.clone());
                        } else {
                            // TODO: return warning in this case?
                            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments(meta.as_ref().unwrap().clone())));
                        }
                    }
                    AssignmentState::NoAssigned =>{
                        unreachable!()
                    }

                }
            }  
        }

    }

    Result::Ok(())
}


pub fn perform_bus_assignment(
    bus_slice: &mut BusSlice, 
    array_access: &[SliceCapacity], 
    assigned_bus_slice: &BusSlice, 
    is_input: bool, 
    conditions_assignment: &AssignmentState
)-> Result<(), MemoryError>{

    let correct_dims_result = BusSlice::check_correct_dims(
        &bus_slice, 
        &array_access, 
        &assigned_bus_slice, 
        true
    );
    match correct_dims_result{
        Ok(_) => {},
        Err(err) => return Err(err)
    };


    let value_left = match BusSlice::access_values_by_mut_reference(bus_slice, array_access){
        Ok(value) => value,
        Err(err) => return Err(err)
    };



    let mut index = 0;
    for accessed_bus in value_left{
        // We completely assign each one of them
        let memory_response_assign = BusSlice::get_reference_to_single_value_by_index(&assigned_bus_slice, index);
        let assigned_bus = match memory_response_assign{
            Ok(v) => v,
            Err(err) => return Err(err)
        };

        match accessed_bus.completely_assign_bus(&assigned_bus, is_input, conditions_assignment){
            Ok(_) =>{},
            Err(err) => return Err(err)
        };
        index += 1;

    }
    
    Ok(())
}
