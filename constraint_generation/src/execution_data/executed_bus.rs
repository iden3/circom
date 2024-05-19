use super::type_definitions::*;
use circom_algebra::algebra::ArithmeticExpression;
use compiler::hir::very_concrete_program::*;
use dag::DAG;
use num_bigint::BigInt;
use program_structure::ast::{SignalType, Statement};
use std::collections::{HashMap, HashSet};
use crate::execution_data::AExpressionSlice;
use crate::execution_data::TagInfo;


pub struct BusConnexion{
    pub full_name: String,
    pub inspect: BusData,
    pub dag_offset: usize,
    pub dag_jump: usize,
}


pub struct ExecutedBus {
    pub bus_name: String,
    pub report_name: String,
    pub signal_fields: SignalCollector,
    pub bus_fields: BusCollector, 
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
            signal_fields: Vec::new(),
            bus_fields: Vec::new(),
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
        let cnn =
            BusConnexion { full_name: component_name.clone(), inspect: data, dag_offset: 0, dag_jump: 0};
        self.bus_connexions.insert(component_name, cnn);
    }

    pub fn add_signal(&mut self, signal_name: &str, dimensions: &[usize]) {
        self.signal_fields.push((signal_name.to_string(), dimensions.to_vec()));
    }

    pub fn add_bus(&mut self, bus_name: &str, dimensions: &[usize]) {
        self.bus_fields.push((bus_name.to_string(), dimensions.to_vec()));
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

    fn build_signals(&self, dag: &mut DAG) {
        
    }

    pub fn bus_name(&self) -> &String {
        &self.bus_name
    }

    pub fn parameter_instances(&self) -> &ParameterContext {
        &self.parameter_instances
    }

    pub fn signal_fields(&self) -> &SignalCollector {
        &self.signal_fields
    }
    pub fn bus_fields(&self) -> &BusCollector {
        &self.bus_fields
    }

    pub fn bus_connexions(&self) -> &HashMap<String, BusConnexion>{
        &self.bus_connexions
    }
   
}