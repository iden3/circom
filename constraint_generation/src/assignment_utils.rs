use super::environment_utils::

    slice_types::{MemoryError, TypeAssignmentError, 
        SignalSlice, SliceCapacity, TagInfo, TagState, TagDefinitions, 
        BusSlice
    };

use std::mem;


// Utils for assigning tags

pub fn compute_propagated_tags(tags_values: &TagInfo, tags_definitions: &TagDefinitions)-> TagInfo{
    let mut tags_propagated = TagInfo::new();
    for (tag, value) in tags_values{
        let state = tags_definitions.get(tag).unwrap();
        if state.value_defined || state.complete{
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
                    let state = TagState{defined: false, value_defined: false, complete: false};
                    tags_definitions.insert(tag.clone(), state);
                }
            }
        }

}


pub fn perform_signal_assignment(signal_slice: &mut SignalSlice, array_access: &[SliceCapacity], new_route: &[SliceCapacity])-> Result<(), MemoryError>{
    let memory_response_for_signal_previous_value = SignalSlice::access_values(
        signal_slice,
        array_access,
    );
    let signal_previous_value = match memory_response_for_signal_previous_value{
        Ok(v) => v,
        Err(err) => return Err(err)
    };

    let new_value_slice = &SignalSlice::new_with_route(new_route, &true);

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

    for i in 0..SignalSlice::get_number_of_cells(&signal_previous_value){
        let memory_response_access = SignalSlice::access_value_by_index(&signal_previous_value, i);
        let signal_was_assigned = match memory_response_access{
            Ok(v) => v,
            Err(err) => return Err(err)
        };
        if signal_was_assigned {
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignments));
        }
    }

    
    
    let access_response = SignalSlice::insert_values(
        signal_slice,
        array_access,
        &new_value_slice,
        true
    );

    match access_response{
        Ok(_) => {},
        Err(err) => return Err(err)
    };
    Result::Ok(())
}


pub fn perform_bus_assignment(bus_slice: &mut BusSlice, array_access: &[SliceCapacity], assigned_bus_slice: &BusSlice, is_input: bool)-> Result<(), MemoryError>{

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

        match accessed_bus.completely_assign_bus(&assigned_bus, is_input){
            Ok(_) =>{},
            Err(err) => return Err(err)
        };
        index += 1;

    }
    
    Ok(())
}
