use super::type_definitions::*;
use circom_algebra::algebra::ArithmeticExpression;
use compiler::hir::very_concrete_program::*;
use dag::DAG;
use num_bigint::BigInt;
use program_structure::ast::{SignalType, Statement};
use std::collections::{HashMap, HashSet};
use crate::execution_data::AExpressionSlice;
use crate::execution_data::TagInfo;


struct BusConnexion{
    full_name: String,
    inspect: BusData,
    dag_offset: usize,
    dag_jump: usize,
}


pub struct ExecutedBus {
    pub bus_name: String,
    pub report_name: String,
    pub signal_fields: SignalCollector,
    pub bus_fields: BusCollector, 
    pub parameter_instances: ParameterContext,
    bus_connexions: HashMap<String, BusConnexion>,
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
            bus_connexions: HashMap::new(),
        }
    }

    pub fn is_equal(&self, name: &str, context: &ParameterContext, tag_context: &TagContext) -> bool {
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



    fn build_signals(&self, dag: &mut DAG) {
        
    }

    pub fn signal_fields(&self) -> &SignalCollector {
        &self.signal_fields
    }
    pub fn bus_fields(&self) -> &BusCollector {
        &self.bus_fields
    }
   
}