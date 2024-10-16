use super::analysis::Analysis;
use crate::FlagsExecution;
use super::executed_template::{ExecutedTemplate, PreExecutedTemplate};
use super::executed_bus::ExecutedBus;

use super::type_definitions::*;
use compiler::hir::very_concrete_program::{Stats, VCPConfig, VCP};
use dag::DAG;
use program_structure::program_archive::ProgramArchive;
use program_structure::program_library::error_definition::ReportCollection;
use std::collections::HashMap;

pub type ExportResult = Result<(DAG, VCP, ReportCollection), ReportCollection>;

#[derive(Default)]
pub struct ExecutedProgram {
    pub model: Vec<ExecutedTemplate>,
    pub model_pretemplates: Vec<PreExecutedTemplate>,
    pub model_buses: Vec<ExecutedBus>,
    pub template_to_nodes: HashMap<String, Vec<NodePointer>>,
    pub bus_to_nodes: HashMap<String, Vec<NodePointer>>,
    pub prime: String,
}

impl ExecutedProgram {
    pub fn new(prime: &String) -> ExecutedProgram {
        ExecutedProgram{
            model: Vec::new(),
            template_to_nodes: HashMap::new(),
            prime: prime.clone(),
            model_pretemplates: Vec::new(),
            model_buses: Vec::new(),
            bus_to_nodes: HashMap::new(),
        }
    }

    pub fn identify_node(&self, name: &str, context: &ParameterContext, tag_context: &HashMap<String, TagWire>) -> Option<NodePointer> {
        if !self.template_to_nodes.contains_key(name) {
            return Option::None;
        }
        let related_nodes = self.template_to_nodes.get(name).unwrap();
        for index in related_nodes {
            let existing_node = &self.model[*index];
            if ExecutedTemplate::is_equal(existing_node, name, context, tag_context) {
                return Option::Some(*index);
            }
        }
        Option::None
    }
    pub fn identify_bus_node(&self, name: &str, context: &ParameterContext) -> Option<NodePointer> {
        if !self.bus_to_nodes.contains_key(name) {
            return Option::None;
        }
        let related_nodes = self.bus_to_nodes.get(name).unwrap();
        for index in related_nodes {
            let existing_node = &self.model_buses[*index];
            if ExecutedBus::is_equal(existing_node, name, context) {
                return Option::Some(*index);
            }
        }
        Option::None
    }

    pub fn number_of_nodes(&self) -> usize {
        self.model.len()
    }
    pub fn get_node(&self, node_pointer: NodePointer) -> Option<&ExecutedTemplate> {
        if node_pointer >= self.model.len() {
            return Option::None;
        }
        Option::Some(&self.model[node_pointer])
    }

    pub fn get_prenode(&self, node_pointer: NodePointer) -> Option<&PreExecutedTemplate> {
        if node_pointer >= self.model_pretemplates.len() {
            return Option::None;
        }
        Option::Some(&self.model_pretemplates[node_pointer])
    }

    pub fn get_prenode_value(&self, node_pointer: NodePointer) -> Option<PreExecutedTemplate> {
        if node_pointer >= self.model_pretemplates.len() {
            return Option::None;
        }
        Option::Some(self.model_pretemplates[node_pointer].clone())
    }
    pub fn get_bus_node(&self, node_pointer: NodePointer) -> Option<&ExecutedBus> {
        if node_pointer >= self.model_buses.len() {
            return Option::None;
        }
        Option::Some(&self.model_buses[node_pointer])
    }

    pub fn add_prenode_to_scheme(
        &mut self,
        node: PreExecutedTemplate,
    ) -> NodePointer {
        // Insert pretemplate
        let node_index = self.model_pretemplates.len();
        self.model_pretemplates.push(node);
        node_index
    }

    pub fn add_node_to_scheme(
        &mut self,
        mut node: ExecutedTemplate,
        analysis: Analysis,
    ) -> NodePointer {
        use super::filters::*;
        // Clean code
        apply_unused(&mut node.code, &analysis, &self.prime);
        apply_computed(&mut node.code, &analysis);
        // Insert template
        let possible_index = self.identify_node(
            node.template_name(), 
            node.parameter_instances(),
            node.tag_instances(),
        );
        if let Option::Some(index) = possible_index {
            return index;
        }
        self.template_to_nodes.entry(node.template_name().clone()).or_insert_with(|| vec![]);
        let nodes_for_template = self.template_to_nodes.get_mut(node.template_name()).unwrap();
        let node_index = self.model.len();
        self.model.push(node);
        nodes_for_template.push(node_index);
        node_index
    }


    pub fn add_bus_node_to_scheme(
        &mut self,
        node: ExecutedBus,
        _analysis: Analysis, // not needed?
    ) -> NodePointer {
        //use super::filters::*;
        // Clean code???
        //apply_unused(&mut node.code, &analysis, &self.prime);
        //apply_computed(&mut node.code, &analysis);
        // Insert template
        let possible_index = self.identify_bus_node(
            node.bus_name(), 
            node.parameter_instances(),
        );
        if let Option::Some(index) = possible_index {
            return index;
        }
        self.bus_to_nodes.entry(node.bus_name().clone()).or_insert_with(|| vec![]);
        let nodes_for_bus = self.bus_to_nodes.get_mut(node.bus_name()).unwrap();
        let node_index = self.model_buses.len();
        self.model_buses.push(node);
        nodes_for_bus.push(node_index);
        node_index
    }

    pub fn export(mut self, mut program: ProgramArchive, flags: FlagsExecution) -> ExportResult {
        use super::executed_template::templates_in_mixed_arrays;
        fn merge_mixed(org: Vec<bool>, new: Vec<bool>) -> Vec<bool> {
            let mut result = Vec::with_capacity(org.len());
            let mut index = 0;
            while index < org.len() {
                result.push(org[index] || new[index]);
                index += 1;
            }
            result
        }

        let mut warnings = vec![];
        let mut dag = DAG::new(&self.prime);
        let mut temp_instances = Vec::with_capacity(self.model.len());
        let mut mixed_instances = vec![false; self.model.len()];

        for exe in &self.model {
            let mixed = templates_in_mixed_arrays(exe, self.model.len());
            mixed_instances = merge_mixed(mixed_instances, mixed);
        }

        for exe in &mut self.model {
            exe.insert_in_dag(&mut dag, &self.model_buses);
        }

        let mut wrapped_buses_table = vec![None; self.model_buses.len()];
        let mut index = 0;
        for exe_bus in &self.model_buses{
            exe_bus.build_bus_info(index, &mut wrapped_buses_table, &self.model_buses);
            index += 1;
        }

        let mut buses_table = Vec::new();
        for info in wrapped_buses_table{
            buses_table.push(info.unwrap());
        }


        for exe in self.model {
            let tmp_instance = exe.export_to_circuit(&mut temp_instances, &buses_table);
            temp_instances.push(tmp_instance);
        }

        temp_instances[dag.main_id()].is_not_parallel_component = true;
        dag.clean_constraints();
        if flags.inspect{
            let mut w = dag.constraint_analysis()?;
            warnings.append(&mut w);
        }

        let dag_stats = produce_dags_stats(&dag);
        crate::compute_constants::manage_functions(&mut program, flags, &self.prime)?;
        crate::compute_constants::compute_vct(&mut temp_instances, &program, flags, &self.prime)?;
        let mut mixed = vec![];
        let mut index = 0;
        for in_mixed in mixed_instances {
            if in_mixed {
                mixed.push(index);
            }
            index += 1;
        }
        let config = VCPConfig {
            stats: dag_stats,
            main_id: dag.main_id(),
            file_library: std::mem::take(&mut program.file_library),
            templates: temp_instances,
            templates_in_mixed: mixed,
            program,
            prime: self.prime,
            buses: buses_table
        };
        let vcp = VCP::new(config);
        Result::Ok((dag, vcp, warnings))
    }
}

fn produce_dags_stats(dag: &DAG) -> Stats {
    let mut all_created_cmp = vec![0; dag.number_of_nodes()];
    let mut all_needed_subcomponents_indexes = vec![0; dag.number_of_nodes()];
    let mut all_signals = vec![0; dag.number_of_nodes()];
    let mut all_io = vec![0; dag.number_of_nodes()];
    for (index, node) in dag.nodes.iter().enumerate() {
        all_signals[index] += node.number_of_inputs();
        all_signals[index] += node.number_of_outputs();
        all_signals[index] += node.number_of_intermediates();
        all_io[index] += node.number_of_inputs();
        all_io[index] += node.number_of_outputs();
        all_created_cmp[index] += 1;
        all_needed_subcomponents_indexes[index] += node.number_of_subcomponents_indexes();
        for c in dag.get_edges(index).unwrap() {
            all_created_cmp[index] += all_created_cmp[c.get_goes_to()];
            all_needed_subcomponents_indexes[index] += all_needed_subcomponents_indexes[c.get_goes_to()];
            all_signals[index] += all_signals[c.get_goes_to()];
            all_io[index] += all_io[c.get_goes_to()];
        }
    }

    Stats {
        all_signals: all_signals.pop().unwrap(),
        io_signals: all_io.pop().unwrap(),
        // number of components that are really created
        all_created_components: all_created_cmp.pop().unwrap(),
        //number of indexes that we need to store (in case there is an array with subcomponents, we need space to store all of them although some positions may not be created)
        //it is the sum of the number of sons of all created components
        all_needed_subcomponents_indexes: all_needed_subcomponents_indexes.pop().unwrap(),
    }
}
