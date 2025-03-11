
use super::slice_types::{BusSlice, FieldTypes, FoldedArgument, FoldedResult, MemoryError, SignalSlice, SliceCapacity, TypeAssignmentError};
use crate::execution_data::type_definitions::{NodePointer, AccessingInformationBus};
use crate::execution_data::ExecutedProgram;
use std::collections::{BTreeMap,HashMap};
use crate::ast::Meta;
use crate::environment_utils::slice_types::AssignmentState;

use crate::assignment_utils::*;

pub struct BusRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub meta: Option<Meta>,
    fields: BTreeMap<String, FieldTypes>,
    unassigned_fields: HashMap<String, SliceCapacity>,
    has_assignment: bool,
}

impl Default for BusRepresentation {
    fn default() -> Self {
        BusRepresentation {
            node_pointer: Option::None,
            fields: BTreeMap::new(),
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
        let assigned_state = if is_initialized {
            AssignmentState::Assigned(None)
        } else{
            AssignmentState::NoAssigned
        };
        component.has_assignment = is_initialized; 
        // initialice the signals
        for info_field in node.fields() {
            let symbol = &info_field.name;
            let route = &info_field.length;
            if !info_field.is_bus{
                let signal_slice = SignalSlice::new_with_route(&route, &assigned_state);
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
    ) -> Result<FoldedResult, MemoryError>{
        
        let field = self.fields.get(field_name).unwrap(); 
        if remaining_access.field_access.is_some(){
            // we are still considering an intermediate bus, check cases            
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

        } else{
            // in this case there is no need for recursion, final access
            
            match field{
                FieldTypes::Signal(signal_slice) =>{
                    // Case it is just a signal or an array of signals, 
                    // in this case there is no need for recursion
                                        
                    let accessed_slice_result = SignalSlice::access_values(&signal_slice, &remaining_access.array_access);
                    match accessed_slice_result{
                        Ok(slice) =>{
                            let folded_slice = FoldedResult::Signal(slice);
                            Result::Ok(folded_slice)
                        },
                        Err(err) => Err(err)
                    }                
                }
                FieldTypes::Bus(bus_slice) => {
                    // Case it is just a bus or an array of buses, 
                    // in this case there is no need for recursion
                    
                    let accessed_slice_result = BusSlice::access_values(&bus_slice, &remaining_access.array_access);
                    match accessed_slice_result{
                        Ok(slice) =>{
                            let folded_slice = FoldedResult::Bus(slice);
                            Result::Ok( folded_slice)
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
        is_input: bool,
        conditions_assignment: &AssignmentState
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
            };

            // We only update AssignmentState if it has been assigned for sure
            if has_assignment{
                match conditions_assignment{
                    AssignmentState::Assigned(_) =>{
                        self.has_assignment = true;

                    }
                    _ =>{}
                }
            }

            // check if we need to access to another bus or if it is the final access
            let field: &mut FieldTypes = self.fields.get_mut(field_name).unwrap();

            if remaining_access.field_access.is_some(){
                // case still intermediate access

                match field{
                    FieldTypes::Bus(bus_slice)=>{
                        // case bus -> apply recursion

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
                                    is_input,
                                    conditions_assignment
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
            
                    
                    
                

            } else{
                // assign the values to the signal or bus
                match field{
                    FieldTypes::Signal(signal_slice) =>{
                        let route = match assigned_value{
                            FoldedArgument::Signal(signal_slice_route) =>{
                                signal_slice_route
                            },
                            _ => unreachable!()
                        };
                        perform_signal_assignment(signal_slice, &remaining_access.array_access, route, conditions_assignment)?;
                    },
                    FieldTypes::Bus(bus_slice) =>{
                        let assigned_bus_slice = match assigned_value{
                            FoldedArgument::Bus(bus_slice) =>{
                                bus_slice
                            },
                            _ => unreachable!()
                        };
                        perform_bus_assignment(bus_slice, &remaining_access.array_access, assigned_bus_slice, is_input, conditions_assignment)?;

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
                Ok(())
            }
        
    }

    pub fn completely_assign_bus(
        &mut self, 
        assigned_bus: &BusRepresentation, 
        is_input: bool,
        conditions_assignment: &AssignmentState
    )-> Result<(), MemoryError>{
        
        if self.has_assignment{
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::MultipleAssignmentsBus));
        }

        // check that they are the same instance of buses
        if self.node_pointer != assigned_bus.node_pointer{
            return Result::Err(MemoryError::AssignmentError(TypeAssignmentError::DifferentBusInstances));
        }

        // only update if it is assigned for sure
        match conditions_assignment{
            AssignmentState::Assigned(_) =>{
                self.has_assignment = true;

            }
            _ =>{}
        }

        for (field_name, value)  in &mut self.fields{
            
            // perform the assignment
            match value{
                FieldTypes::Bus(ref mut bus_slice) =>{

                    let bus_slice_assigned = match assigned_bus.fields.get(field_name).unwrap(){
                        FieldTypes::Bus(bs) => bs,
                        FieldTypes::Signal(_) => unreachable!(),
                    };

                    let assignment_result = perform_bus_assignment(bus_slice, &[], bus_slice_assigned, is_input, &conditions_assignment);

                    if assignment_result.is_err(){
                        return Err(assignment_result.err().unwrap());
                    }
                },
                FieldTypes::Signal(signal_slice)=>{
                    // we need to check assignments (case conditional assignments)
                    let route = signal_slice.route_value();
                    perform_signal_assignment(
                        signal_slice, 
                        &Vec::new(), 
                        &route, 
                        conditions_assignment
                    )?;
                }
                
            }

            // Update the value of unnasigned fields
            self.unassigned_fields.remove(field_name);
              
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

}
