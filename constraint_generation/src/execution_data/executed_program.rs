use super::analysis::Analysis;
use crate::FlagsExecution;
use super::executed_template::{ExecutedTemplate, PreExecutedTemplate};
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
    pub template_to_nodes: HashMap<String, Vec<NodePointer>>,
    pub prime: String,
}

impl ExecutedProgram {
    pub fn new(prime: &String) -> ExecutedProgram {
        ExecutedProgram{
            model: Vec::new(),
            template_to_nodes: HashMap::new(),
            prime: prime.clone(),
            model_pretemplates: Vec::new(),
        }
    }

    pub fn identify_node(&self, name: &str, context: &ParameterContext, tag_context: &TagContext) -> Option<NodePointer> {
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
            exe.insert_in_dag(&mut dag);
        }

        for exe in self.model {
            let tmp_instance = exe.export_to_circuit(&mut temp_instances);
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
