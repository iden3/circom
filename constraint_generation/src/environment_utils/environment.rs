use super::slice_types::{
    AExpressionSlice, 
    ComponentRepresentation, 
    ComponentSlice,
    SignalSlice, 
    SliceCapacity,
    TagInfo, 
    TagDefinitions,
    TagState
};
use super::{ArithmeticExpression, CircomEnvironment, CircomEnvironmentError};
use program_structure::memory_slice::MemoryError;
use crate::ast::Meta;

pub type ExecutionEnvironmentError = CircomEnvironmentError;
pub type ExecutionEnvironment = CircomEnvironment<ComponentSlice, (TagInfo, TagDefinitions, SignalSlice), (TagInfo, AExpressionSlice)>;

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
    let slice = SignalSlice::new_with_route(dimensions, &true);
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some(), complete: true});
    }
    
    environment.add_input(input_name, (tags.clone(), tags_defined,  slice));
}
pub fn environment_shortcut_add_output(
    environment: &mut ExecutionEnvironment,
    output_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = SignalSlice::new_with_route(dimensions, &false);
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some(), complete: false});
    }
    environment.add_output(output_name, (tags.clone(), tags_defined, slice));
}
pub fn environment_shortcut_add_intermediate(
    environment: &mut ExecutionEnvironment,
    intermediate_name: &str,
    dimensions: &[SliceCapacity],
    tags: &TagInfo,
) {
    let slice = SignalSlice::new_with_route(dimensions, &false);
    let mut tags_defined = TagDefinitions::new();
    for (t, value) in tags{
        tags_defined.insert(t.clone(), TagState{defined:true, value_defined: value.is_some(), complete: false});
    }
    environment.add_intermediate(intermediate_name, (tags.clone(), tags_defined, slice));
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