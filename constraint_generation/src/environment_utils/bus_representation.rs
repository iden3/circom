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
            let field_signal = FieldTypes{signal: Some(signal_slice), bus:None};
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
            let field_bus = FieldTypes{bus: Some(bus_slice), signal:None};
            component.fields.insert(symbol.clone(), field_bus);
        }

        component.node_pointer = Option::Some(node_pointer);


        Result::Ok(())
    }

    pub fn get_field(&self, field_name: &str) -> Result<(&TagInfo, &SignalSlice), MemoryError> {

        // Devuelve las tags y la SignalSlice con los valores
        // Si es un bus, llamar a que cada uno devuelva todo (get_all_fields)
        unreachable!()
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



}
