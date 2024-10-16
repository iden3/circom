use super::type_definitions::*;

use std::collections::{HashMap, BTreeMap};
use compiler::hir::very_concrete_program::*;



pub struct BusConnexion{
    pub full_name: String,
    pub inspect: BusData,
    pub dag_offset: usize,
    pub dag_jump: usize,
}


pub struct ExecutedBus {
    pub bus_name: String,
    pub report_name: String,
    pub fields: WireCollector,
    pub parameter_instances: ParameterContext,
    pub bus_connexions: HashMap<String, BusConnexion>,
    pub size: usize, 
    pub bus_id: Option<usize>,
    pub tag_names: HashMap<String, Vec<String>>,
}

impl ExecutedBus {
    pub fn new(
        name: String,
        report_name: String,
        instance: ParameterContext,
    ) -> ExecutedBus {
        ExecutedBus {
            report_name,
            bus_name: name,
            parameter_instances: instance,
            fields: Vec::new(),
            bus_connexions: HashMap::new(),
            size: 0,
            bus_id: None,
            tag_names: HashMap::new(),
        }
    }

    pub fn is_equal(&self, name: &str, context: &ParameterContext) -> bool {
        self.bus_name == name 
            && self.parameter_instances == *context
    }

    pub fn add_bus_arrow(&mut self, component_name: String, data: BusData) {
        
        let mut dimensions = &vec![];
        for wire_data in &self.fields{
            if *wire_data.name == component_name{
                dimensions = &wire_data.length;
            }
        }
        let mut total_size = data.size;
        for v in dimensions{
            total_size *= v;
        }
        self.size += total_size;

        let cnn =
            BusConnexion { full_name: component_name.clone(), inspect: data, dag_offset: 0, dag_jump: 0};
        self.bus_connexions.insert(component_name, cnn);
    }

    pub fn add_signal(&mut self, signal_name: &str, dimensions: &[usize], tags: Vec<String>) {
        let info_signal = WireData{
            name: signal_name.to_string(),
            length: dimensions.to_vec(),
            is_bus: false
        };
        self.fields.push(info_signal);
        let mut total_size = 1;
        for v in dimensions{
            total_size *= v;
        }
        self.tag_names.insert(signal_name.to_string(), tags);
        self.size += total_size;
    }

    pub fn add_bus(&mut self, bus_name: &str, dimensions: &[usize], tags: Vec<String>) {
        let info_bus = WireData{
            name: bus_name.to_string(),
            length: dimensions.to_vec(),
            is_bus: true
        };
        self.fields.push(info_bus);
        self.tag_names.insert(bus_name.to_string(), tags);

    }

    pub fn bus_name(&self) -> &String {
        &self.bus_name
    }

    pub fn parameter_instances(&self) -> &ParameterContext {
        &self.parameter_instances
    }

    pub fn fields(&self) -> &WireCollector {
        &self.fields
    }

    pub fn bus_connexions(&self) -> &HashMap<String, BusConnexion>{
        &self.bus_connexions
    }

    pub fn build_bus_info(
        &self,
        bus_id: usize,
        bus_table: &mut Vec<Option<BusInstance>>,
        buses_info: &Vec<ExecutedBus>
    ){
        if bus_table[bus_id].is_none(){
            let mut total_size = 0;
            let mut offset = 0;
            let mut wires = BTreeMap::new();
            let mut field_id = 0;
            for info_field in &self.fields{
                let (name, lengths) = (&info_field.name, &info_field.length);
                if !info_field.is_bus{
                    // Case signal
                    let size = lengths.iter().fold(1, |p, c| p * (*c));
                    let signal = FieldInfo {
                        field_id, 
                        dimensions: lengths.clone(),
                        size,
                        offset,
                        bus_id: None
                    };
                    wires.insert(name.clone(), signal);
                    total_size += size;
                    offset += size;
                    field_id += 1;

                } else{
                    let bus_node = self.bus_connexions.get(name).unwrap().inspect.goes_to;
                    if bus_table[bus_node].is_none(){
                        let exe_bus = buses_info.get(bus_node).unwrap();
                        exe_bus.build_bus_info(bus_node, bus_table, buses_info);
                    }
                    let bus_instance = bus_table.get(bus_node).unwrap().as_ref().unwrap();
                    
                    let size = lengths.iter().fold(bus_instance.size, |p, c| p * (*c));
                    let bus = FieldInfo {
                        field_id, 
                        dimensions: lengths.clone(),
                        size,
                        offset,
                        bus_id: Some(bus_node)
                    };
                    wires.insert(name.clone(), bus);
                    total_size += size;
                    offset += size;
                    field_id += 1;
                                  
                }
            }
            bus_table[bus_id] = Some(
                BusInstance{
                    name: self.bus_name.clone(),
                    size: total_size,
                    fields: wires
                }
            )
            
        }
    }
}