use super::type_definitions::*;

use num_bigint::BigInt;
use std::collections::HashMap;
use crate::execution_data::TagInfo;
use compiler::hir::very_concrete_program::*;
use crate::ast::SignalType;



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
    pub signal_to_tags: TagContext,
    pub bus_connexions: HashMap<String, BusConnexion>,
    pub size: usize, 
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
            signal_to_tags: TagContext::new(),
            bus_connexions: HashMap::new(),
            size: 0,
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

    pub fn add_signal(&mut self, signal_name: &str, dimensions: &[usize]) {
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
        self.size += total_size;
    }

    pub fn add_bus(&mut self, bus_name: &str, dimensions: &[usize]) {
        let info_bus = WireData{
            name: bus_name.to_string(),
            length: dimensions.to_vec(),
            is_bus: true
        };
        self.fields.push(info_bus);
    }

    pub fn add_tag_signal(&mut self, signal_name: &str, tag_name: &str, value: Option<BigInt>){
        let tags_signal = self.signal_to_tags.get_mut(signal_name);
        if tags_signal.is_none(){
            let mut new_tags_signal = TagInfo::new();
            new_tags_signal.insert(tag_name.to_string(), value);
            self.signal_to_tags.insert(signal_name.to_string(), new_tags_signal);
        } else {
            tags_signal.unwrap().insert(tag_name.to_string(), value);
        }
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

    // TODO: use same category for the bus and signal
    pub fn build_bus(
        &self,
        name: String, 
        lengths: Vec<usize>, 
        local_id: usize, 
        dag_local_id: usize, 
        xtype: SignalType, 
        buses_info: &Vec<ExecutedBus>
    ) -> Bus{
        let mut local_id_aux = local_id;
        let mut dag_local_id_aux = dag_local_id;
        let mut signals =  Vec::new();
        let mut buses = Vec::new();

        for info_field in &self.fields{
            let (name, lengths) = (&info_field.name, &info_field.length);
            if !info_field.is_bus{
                let signal = Signal { name: name.clone(), lengths: lengths.clone(), local_id: local_id_aux, dag_local_id: dag_local_id_aux, xtype};
                local_id_aux += signal.size();
                dag_local_id_aux += signal.size();
                signals.push(signal);
            } else{
                let bus_node = self.bus_connexions.get(name).unwrap().inspect.goes_to;
                let exe_bus = buses_info.get(bus_node).unwrap();
                let bus = exe_bus.build_bus(name.clone(), lengths.clone(), local_id_aux, dag_local_id_aux, xtype, buses_info);
                local_id_aux += bus.size();
                dag_local_id_aux += bus.size();
                buses.push(Box::new(bus));                
            }
        }
        Bus{name, lengths, xtype, local_id, dag_local_id, signals, buses}
    }

   
}