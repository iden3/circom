use super::slice_types::{
    AExpressionSlice, 
    ComponentRepresentation, 
    BusRepresentation,
    ComponentSlice,
    SignalSlice, 
    SliceCapacity,
    TagInfo, 
    TagDefinitions,
    TagState,
    BusSlice,
    BusTagInfo,
    SignalTagInfo
};
use crate::environment_utils::slice_types::AssignmentState::*;
use super::{ArithmeticExpression, CircomEnvironment, CircomEnvironmentError};
use program_structure::memory_slice::MemoryError;
use crate::execution_data::type_definitions::TagWire;
use crate::ast::Meta;
use std::collections::BTreeMap;
use crate::environment_utils::slice_types::BigInt;


pub type ExecutionEnvironmentError = CircomEnvironmentError;
pub type ExecutionEnvironment = CircomEnvironment<ComponentSlice, (SignalTagInfo, SignalSlice), (TagInfo, AExpressionSlice), (BusTagInfo, BusSlice)>;

pub fn environment_shortcut_add_component(
    environment: &mut ExecutionEnvironment,
    component_name: &str,
    dimensions: &[SliceCapacity],
) {
    let slice = ComponentSlice::new_with_route(dimensions, &ComponentRepresentation::default());
    environment.add_component(component_name, slice);
}

pub fn environment_shortcut_add_input(
    environment: &mut ExecutionEnvironment,
    input_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = SignalSlice::new_with_route(dimensions, &Assigned(None));
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some()});
    }
    let tag_info = SignalTagInfo{
        tags: tags.clone(),
        definitions: tags_defined,
        remaining_inserts: 0,
        is_init: true
    };


    environment.add_input(input_name, (tag_info,  slice));
}
pub fn environment_shortcut_add_output(
    environment: &mut ExecutionEnvironment,
    output_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = SignalSlice::new_with_route(dimensions, &NoAssigned);
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some()});
    }
    let size = dimensions.iter().fold(1, |acc, dim| acc * dim);
    let tag_info = SignalTagInfo{
        tags: tags.clone(),
        definitions: tags_defined,
        remaining_inserts: size,
        is_init: false
    };
    environment.add_output(output_name, (tag_info, slice));
}
pub fn environment_shortcut_add_intermediate(
    environment: &mut ExecutionEnvironment,
    intermediate_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = SignalSlice::new_with_route(dimensions, &NoAssigned);
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some()});
    }
    let size = dimensions.iter().fold(1, |acc, dim| acc * dim);
    let tag_info = SignalTagInfo{
        tags: tags.clone(),
        definitions: tags_defined,
        remaining_inserts: size,
        is_init: false
    };
    environment.add_intermediate(intermediate_name, (tag_info, slice));
}
pub fn environment_shortcut_add_bus_input(
    environment: &mut ExecutionEnvironment,
    input_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagWire,
) {
    fn generate_tags_data(tags: &TagWire)-> BusTagInfo{
        let mut tags_defined = TagDefinitions::new();
    
        for (t, value) in &tags.tags{
            tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some()});
        }
        let mut fields = BTreeMap::new();
        if tags.fields.is_some(){
            for (field_name, info_field) in tags.fields.as_ref().unwrap(){
                let field_tag_info = generate_tags_data(info_field);
                fields.insert(field_name.clone(), field_tag_info);
            }
        }

        BusTagInfo{
            definitions: tags_defined,
            tags: tags.tags.clone(),
            remaining_inserts: 0,
            size: 0, // in this case we never use it
            is_init: true,
            fields
        }
    }
    
    // In this case we need to set all the signals of the bus to known -> in the default method
    let slice = BusSlice::new_with_route(dimensions, &BusRepresentation::default());

    
    environment.add_input_bus(input_name, (generate_tags_data(tags),  slice));
}
pub fn environment_shortcut_add_bus_output(
    environment: &mut ExecutionEnvironment,
    output_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = BusSlice::new_with_route(dimensions, &BusRepresentation::default());
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some()});
    }
    let size = dimensions.iter().fold(1, |aux, val| aux * val);
    let tag_info= BusTagInfo{
        definitions: tags_defined,
        tags: tags.clone(),
        remaining_inserts: size,
        size,
        is_init: false,
        fields: BTreeMap::new(),
    };
    environment.add_output_bus(output_name, (tag_info, slice));
}
pub fn environment_shortcut_add_bus_intermediate(
    environment: &mut ExecutionEnvironment,
    intermediate_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = BusSlice::new_with_route(dimensions, &BusRepresentation::default());
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some()});
    }
    let size = dimensions.iter().fold(1, |aux, val| aux * val);

    let tag_info= BusTagInfo{
        definitions: tags_defined,
        tags: tags.clone(),
        remaining_inserts: size,
        size,
        is_init: false,
        fields: BTreeMap::new(),
    };
    environment.add_intermediate_bus(intermediate_name, (tag_info, slice));
}
pub fn environment_shortcut_add_variable(
    environment: &mut ExecutionEnvironment,
    variable_name: &str,
    dimensions: &[SliceCapacity],
) {
    let slice = AExpressionSlice::new_with_route(dimensions, &ArithmeticExpression::default());
    environment.add_variable(variable_name, (TagInfo::new(), slice));
}

pub fn environment_check_all_components_assigned(environment: &ExecutionEnvironment)-> Result<(), (MemoryError, Meta)>{
    use program_structure::memory_slice::MemorySlice;
    for (name, slice) in environment.get_components_ref(){
        for i in 0..MemorySlice::get_number_of_cells(slice){
            let component = MemorySlice::get_reference_to_single_value_by_index_or_break(slice, i);
            if component.is_preinitialized() && component.has_unassigned_inputs(){
                return Result::Err((MemoryError::MissingInputs(name.clone()), component.meta.as_ref().unwrap().clone()));
            } 
        }
    }
    Result::Ok(())
}


pub fn environment_get_value_tags_signal(environment: &ExecutionEnvironment, name: &String) -> Vec<(Vec<String>, BigInt)>{
    let mut to_add = Vec::new();
    let (tag_data, _) = environment.get_signal(name).unwrap();
    for (tag, value) in &tag_data.tags{
        if value.is_some(){
            let state = tag_data.definitions.get(tag).unwrap();
            if state.defined && (state.value_defined || tag_data.remaining_inserts == 0){
                to_add.push((vec![name.clone(), tag.clone()], value.clone().unwrap()));
            }
        }
    }
    to_add
}

pub fn environment_get_value_tags_bus(environment: &ExecutionEnvironment, name: &String) -> Vec<(Vec<String>, BigInt)>{
    fn get_value_tags_data(tag_data: &BusTagInfo, name: &Vec<String>)-> Vec<(Vec<String>, BigInt)>{
        let mut  to_add = Vec::new();
        for (tag, value) in &tag_data.tags{
            if value.is_some(){
                let state = tag_data.definitions.get(tag).unwrap();
                if state.defined && (state.value_defined || tag_data.remaining_inserts == 0){
                    let mut aux = name.clone();
                    aux.push(tag.clone());
                    to_add.push((aux, value.clone().unwrap()));
                }
            }
        }
        for (field, field_data) in &tag_data.fields{
            let mut aux = name.clone();
            aux.push(field.clone());
            let mut new_tags = get_value_tags_data(field_data, &aux);
            to_add.append(&mut new_tags);
        }
        to_add
    }
    let (tag_data, _) = environment.get_bus(name).unwrap();

    get_value_tags_data(tag_data, &vec![name.clone()])
}