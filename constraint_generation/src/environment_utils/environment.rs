use super::slice_types::{
    AExpressionSlice, ComponentRepresentation, ComponentSlice, SignalSlice, SliceCapacity,
};
use super::{ArithmeticExpression, CircomEnvironment, CircomEnvironmentError};

pub type ExecutionEnvironmentError = CircomEnvironmentError;
pub type ExecutionEnvironment = CircomEnvironment<ComponentSlice, SignalSlice, AExpressionSlice>;

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
) {
    let slice = SignalSlice::new_with_route(dimensions, &true);
    environment.add_input(input_name, slice);
}
pub fn environment_shortcut_add_output(
    environment: &mut ExecutionEnvironment,
    output_name: &str,
    dimensions: &[SliceCapacity],
) {
    let slice = SignalSlice::new_with_route(dimensions, &false);
    environment.add_output(output_name, slice);
}
pub fn environment_shortcut_add_intermediate(
    environment: &mut ExecutionEnvironment,
    intermediate_name: &str,
    dimensions: &[SliceCapacity],
) {
    let slice = SignalSlice::new_with_route(dimensions, &false);
    environment.add_intermediate(intermediate_name, slice);
}
pub fn environment_shortcut_add_variable(
    environment: &mut ExecutionEnvironment,
    variable_name: &str,
    dimensions: &[SliceCapacity],
) {
    let slice = AExpressionSlice::new_with_route(dimensions, &ArithmeticExpression::default());
    environment.add_variable(variable_name, slice);
}
