use super::slice_types::{AExpressionSlice, FieldTypes, MemoryError, TypeInvalidAccess, TypeAssignmentError, SignalSlice, BusSlice, SliceCapacity,TagInfo};
use crate::execution_data::type_definitions::{NodePointer, AccessingInformationBus};
use crate::execution_data::ExecutedProgram;
use std::collections::{BTreeMap,HashMap, HashSet};
use crate::ast::Meta;

pub struct BusRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub meta: Option<Meta>,
    fields: BTreeMap<String, FieldTypes>,
    pub field_tags: BTreeMap<String, TagInfo>,
    unassigned_fields: HashMap<String, SliceCapacity>,
}

impl Default for BusRepresentation {
    fn default() -> Self {
        BusRepresentation {
            node_pointer: Option::None,
            fields: BTreeMap::new(),
            field_tags: BTreeMap::new(),
            meta: Option::None,
            unassigned_fields: HashMap::new(),
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
    ) -> Result<(TagInfo, SignalSlice), MemoryError> {
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
    ) -> Result<(TagInfo, BusSlice), MemoryError> {
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

    pub fn assign_value_to_field(
        component: &mut BusRepresentation,
        field_name: &str,
        access: &[SliceCapacity],
        slice_route: &[SliceCapacity],
        tags: TagInfo,
    ) -> Result<(), MemoryError> {
        // TODO
        unreachable!()
    }

    pub fn get_accesses_bus(&self, name: &String) -> Vec<String>{

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
                        let access = BusSlice::access_value_by_index(&bus_slice, i);

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
