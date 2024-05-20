use super::environment_utils::{

    slice_types::{
        AExpressionSlice, ArithmeticExpression as ArithmeticExpressionGen, ComponentRepresentation,
        ComponentSlice, MemoryError, TypeInvalidAccess, TypeAssignmentError, MemorySlice, 
        SignalSlice, SliceCapacity, TagInfo, TagState, TagDefinitions, BusSlice, BusRepresentation
    },
};

use std::mem;


// Utils for assigning tags

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