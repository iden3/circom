use super::slice_types::{MemoryError, TypeInvalidAccess, TypeAssignmentError, SignalSlice, BusSlice, SliceCapacity,TagInfo};
use crate::execution_data::type_definitions::NodePointer;
use crate::execution_data::ExecutedProgram;
use std::collections::{BTreeMap,HashMap, HashSet};
use crate::ast::Meta;

#[derive(Clone)]
struct FieldTypes { // For each field, we store the info depending on if it is a signal o a bus
                    // Depending on the case we store a different slice
    pub signal: Option<SignalSlice>,
    pub bus: Option<BusSlice>,
}

pub struct BusRepresentation {
    pub node_pointer: Option<NodePointer>,
    pub meta: Option<Meta>,
    pub fields: BTreeMap<String, FieldTypes>,
    pub field_tags: BTreeMap<String, TagInfo>,
}

impl Default for BusRepresentation {
    fn default() -> Self {
        BusRepresentation {
            node_pointer: Option::None,
            fields: BTreeMap::new(),
            field_tags: BTreeMap::new(),
            meta: Option::None,
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

        }
        for (symbol, route) in node.bus_fields() {

        }

        component.node_pointer = Option::Some(node_pointer);


        Result::Ok(())
    }

    pub fn get_field(&self, field_name: &str) -> Result<(&TagInfo, &SignalSlice), MemoryError> {

        // Devuelve las tags y la SignalSlice con los valores
        // Si es un bus, llamar a que cada uno devuelva todo (get_all_fields)
        unreachable!()
    }

    fn get_all_fields(&self) -> Result<&SignalSlice, MemoryError>{
        // TODO
        unreachable!()
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



}
