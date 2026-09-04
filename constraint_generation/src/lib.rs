extern crate num_bigint_dig as num_bigint;
extern crate num_traits;

mod compute_constants;
mod environment_utils;
mod execute;
mod execution_data;
mod assignment_utils;

use ansi_term::Colour;
use circom_algebra::algebra::{ArithmeticError, ArithmeticExpression};
use compiler::hir::very_concrete_program::VCP;
use constraint_list::ConstraintList;
use constraint_writers::ConstraintExporter;
use dag::{CircuitTemplateInfo, DAG, TreeConstraints};
use execution_data::executed_program::ExportResult;
use execution_data::ExecutedProgram;
use program_structure::ast::{self};
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::FileID;
use program_structure::program_archive::ProgramArchive;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap,BTreeMap};


pub struct BuildConfig {
    pub no_rounds: usize,
    pub flag_json_sub: bool,
    pub json_substitutions: String,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_verbose: bool,
    pub flag_old_heuristics: bool,
    pub inspect_constraints: bool,
    pub prime: String,
    pub print_tree_info: bool,
    pub print_template_info: bool,
    pub initial_constraints_file: String,
    pub structure_file: String,
    pub template_info_file: String,
}

#[derive(Debug, Copy, Clone)]
pub struct FlagsExecution{
    pub verbose: bool,
    pub inspect: bool,
}

pub type ConstraintWriter = Box<dyn ConstraintExporter>;
type BuildResponse = Result<(ConstraintWriter, VCP), ()>;
pub fn build_circuit(program: ProgramArchive, config: BuildConfig) -> BuildResponse {
    let files = program.file_library.clone();
    let flags = FlagsExecution{
        verbose: config.flag_verbose,
        inspect: config.inspect_constraints,
    };
    let (exe, warnings) = instantiation(&program, flags, &config.prime).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    Report::print_reports(&warnings, &files);
    let (mut dag, mut vcp, warnings) = export(exe, program, flags).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    if config.inspect_constraints {
        Report::print_reports(&warnings, &files);
    }
    if config.print_tree_info {
        let tree_constraints = dag.map_to_constraint_tree();
        print_tree_info(
            &tree_constraints,
            &config.initial_constraints_file,
            &config.structure_file
        );
    }
    if config.print_template_info {
        let template_info = dag.map_to_template_info();
        print_template_info(&template_info, &config.template_info_file);
    }
    if config.flag_f {
        sync_dag_and_vcp(&mut vcp, &mut dag);
        if config.flag_json_sub { 
            use constraint_writers::json_writer::SubstitutionJSON;
            let substitution_log = SubstitutionJSON::new(&config.json_substitutions).unwrap();
            let _ = substitution_log.end();
            println!("{} {}", Colour::Green.paint("Written successfully:"), config.json_substitutions);
        };

        Result::Ok((Box::new(dag), vcp))
    } else {
        let list = simplification_process(&mut vcp, dag, &config);
        if config.flag_json_sub { 
            println!("{} {}", Colour::Green.paint("Written successfully:"), config.json_substitutions);
        };
        Result::Ok((Box::new(list), vcp))
    }
}

type InstantiationResponse = Result<(ExecutedProgram, ReportCollection), ReportCollection>;
fn instantiation(program: &ProgramArchive, flags: FlagsExecution, prime: &String) -> InstantiationResponse {
    let execution_result = execute::constraint_execution(&program, flags, prime);
    match execution_result {
        Ok((program_exe, warnings)) => {
            let no_nodes = program_exe.number_of_nodes();
            let success = Colour::Green.paint("template instances");
            let nodes_created = format!("{}: {}", success, no_nodes);
            println!("{}", &nodes_created);
            InstantiationResponse::Ok((program_exe,warnings))
        }
        Err(reports) => InstantiationResponse::Err(reports),
    }
}

fn export(exe: ExecutedProgram, program: ProgramArchive, flags: FlagsExecution) -> ExportResult {
    let exported = exe.export(program, flags);
    exported
}

fn sync_dag_and_vcp(vcp: &mut VCP, dag: &mut DAG) {
    let witness = Rc::new(DAG::produce_witness(dag));
    VCP::add_witness_list(vcp, Rc::clone(&witness));
}

fn simplification_process(vcp: &mut VCP, dag: DAG, config: &BuildConfig) -> ConstraintList {
    use dag::SimplificationFlags;
    let flags = SimplificationFlags {
        flag_s: config.flag_s,
        parallel_flag: config.flag_p,
        port_substitution: config.flag_json_sub,
        json_substitutions: config.json_substitutions.clone(),
        no_rounds: config.no_rounds,
        flag_old_heuristics: config.flag_old_heuristics,
        prime : config.prime.clone(),
    };
    let list = DAG::map_to_list(dag, flags);
    VCP::add_witness_list(vcp, Rc::new(list.get_witness_as_vec()));
    list
}


#[derive(Deserialize,Serialize, Debug)]
pub struct TimingInfo{
    pub graph_construction: f32,
    pub clustering: f32,
    pub dag_construction: f32,
    pub equivalency: f32,
    pub total: f32,
}

#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct NodeInfo{
    pub node_name: String,
    pub component_name: String,
    pub node_id: usize,
    pub constraints: Vec<usize>, //ids of the constraints
    pub input_signals: Vec<usize>,
    pub output_signals: Vec<usize>,
    pub signals: Vec<usize>, 
    pub is_custom: bool,
    pub is_deterministic: bool, // to add the info saying if the node is verified or not
    pub successors: Vec<usize>, //ids of the successors 
    pub predecessors: Vec<usize>, //ids of the predecessors, can be filled after building the structure
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StructureInfo {
    pub timing: TimingInfo,
    pub nodes: Vec<NodeInfo>, //all the nodes of the circuit, position of the node is not the position.
    pub equivalency_local: Vec<Vec<usize>>, //equivalence classes, each inner vector is a class
    pub equivalency_structural: Vec<Vec<usize>>, //equivalence classes, each inner vector is a class
}



fn print_tree_info(
    tree_constraints: &TreeConstraints,
    initial_constraints_file: &String,
    structure_file: &String,
){
    let mut init_constraint_to_node =  BTreeMap::new();
    
    let mut equivalence_nodes = HashMap::new();
    let mut node_info = Vec::new();

    let mut init_c = 0;
    let mut node_id = 0;
    build_structure_nodes(&tree_constraints, &mut node_id, &mut init_c, &mut node_info, &mut equivalence_nodes, &mut init_constraint_to_node, None);
    let aux_timing = TimingInfo{
        graph_construction: 0.0,
        clustering: 0.0,
        dag_construction: 0.0,
        equivalency: 0.0,
        total: 0.0
    };

    let equiv_to_vec: Vec<Vec<usize>> = equivalence_nodes.into_iter()
                                        .map(|(_id, class)| class)
                                        .collect();
    let structure = StructureInfo{
        timing: aux_timing,
        nodes: node_info,
        equivalency_local: equiv_to_vec.clone(),
        equivalency_structural: equiv_to_vec
    };

    let _ = std::fs::write(
        initial_constraints_file,
        serde_json::to_string_pretty(&init_constraint_to_node).unwrap(),
    );
     
    let _ = std::fs::write(
        structure_file,
        serde_json::to_string_pretty(&structure).unwrap(),
    );
}


/// Longest instantiation name shown in the table. Names carrying a big
/// parameter, such as `CompConstant(2188...5616)`, are cut in the middle: the
/// whole name is always kept in the JSON file.
const NAME_LIMIT: usize = 46;

fn shorten(name: &str, limit: usize) -> String {
    let characters: Vec<char> = name.chars().collect();
    if characters.len() <= limit || limit < 5 {
        return name.to_string();
    }
    let kept = limit - 3;
    let head = kept - kept / 2;
    let mut shortened: String = characters[..head].iter().collect();
    shortened.push_str("...");
    shortened.extend(characters[characters.len() - (kept - head)..].iter());
    shortened
}

fn center(text: &str, width: usize) -> String {
    let length = text.chars().count();
    if length >= width {
        return text.to_string();
    }
    let left = (width - length) / 2;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(width - length - left))
}

fn print_template_info(template_info: &CircuitTemplateInfo, template_info_file: &String) {
    // One row per template instance: `Num2Bits(135)` and `Num2Bits(254)` are
    // two instances of Num2Bits and get a row each. The first group of columns
    // is what a single component of the instance costs and the second one is
    // that cost times the number of components of the instance.
    let names: Vec<String> =
        template_info.instances.iter().map(|i| shorten(&i.name, NAME_LIMIT)).collect();
    let name_width =
        names.iter().map(|n| n.chars().count()).fold("instance".len(), std::cmp::max);
    let all_constraints = template_info.circuit.constraints.total;
    let percentage = |part: usize| {
        if all_constraints == 0 { 0.0 } else { (part as f64 * 100.0) / all_constraints as f64 }
    };

    println!("{}", Colour::Green.paint("template information (before simplification)"));
    let group_header = format!(
        "{:<width$} {:>10} {} {}",
        "",
        "",
        center("per component", 39),
        center("total (per component x components)", 55),
        width = name_width
    );
    println!("{}", group_header.trim_end());
    println!(
        "{:<width$} {:>10} {:>9} {:>9} {:>9} {:>9} {:>11} {:>7} {:>11} {:>11} {:>11}",
        "instance",
        "components",
        "constrs",
        "non-lin",
        "linear",
        "signals",
        "constrs",
        "%",
        "non-lin",
        "linear",
        "signals",
        width = name_width
    );
    for (instance, name) in template_info.instances.iter().zip(&names) {
        println!(
            "{:<width$} {:>10} {:>9} {:>9} {:>9} {:>9} {:>11} {:>6.1}% {:>11} {:>11} {:>11}",
            name,
            instance.components,
            instance.own.constraints.total,
            instance.own.constraints.non_linear,
            instance.own.constraints.linear,
            instance.own.signals.total,
            instance.total_own.constraints.total,
            percentage(instance.total_own.constraints.total),
            instance.total_own.constraints.non_linear,
            instance.total_own.constraints.linear,
            instance.total_own.signals.total,
            width = name_width
        );
    }
    println!(
        "{:<width$} {:>10} {:>9} {:>9} {:>9} {:>9} {:>11} {:>6.1}% {:>11} {:>11} {:>11}",
        "TOTAL",
        template_info.number_of_components,
        "-",
        "-",
        "-",
        "-",
        all_constraints,
        percentage(all_constraints),
        template_info.circuit.constraints.non_linear,
        template_info.circuit.constraints.linear,
        template_info.circuit.signals.total,
        width = name_width
    );

    let written = std::fs::write(
        template_info_file,
        serde_json::to_string_pretty(template_info).unwrap(),
    );
    if written.is_ok() {
        println!("{} {}", Colour::Green.paint("Written successfully:"), template_info_file);
    } else {
        eprintln!(
            "{} {}",
            Colour::Red.paint("Could not write the template information in:"),
            template_info_file
        );
    }
}


fn build_structure_nodes(
    tree_constraints: &TreeConstraints,
    node_id: &mut usize,
    init_c: &mut usize,
    node_info: &mut Vec<NodeInfo>,
    equivalence_nodes: &mut HashMap<usize, Vec<usize>>,
    init_constraint_to_node: &mut BTreeMap<usize, String>,
    predecessor: Option<usize>
) -> usize{
    
    let my_node_id = *node_id;
    *node_id += 1;

    let equivalence_node_id = tree_constraints.node_id;
    if equivalence_nodes.contains_key(&equivalence_node_id){
        let ref_equiv = equivalence_nodes.get_mut(&equivalence_node_id).unwrap();
        ref_equiv.push(my_node_id);

    } else{
        equivalence_nodes.insert(equivalence_node_id, vec![my_node_id]);
    }

    init_constraint_to_node.insert(*init_c, tree_constraints.template_name.clone());
    let mut constraints = Vec::new();
    for i in 0..tree_constraints.number_constraints{
        constraints.push(*init_c + i);
    }
    *init_c += tree_constraints.number_constraints;

    let mut output_signals = Vec::new();
    for i in 0..tree_constraints.number_outputs{
        output_signals.push(tree_constraints.initial_signal + i);
    } 

    let mut input_signals = Vec::new();
    for i in 0..tree_constraints.number_inputs{
        input_signals.push(tree_constraints.initial_signal + tree_constraints.number_outputs + i);
    } 

    let mut signals = Vec::new();
    for i in 0..tree_constraints.number_signals{
        signals.push(tree_constraints.initial_signal + i);
    } 

    let predecessors = if predecessor.is_some(){
        vec![predecessor.unwrap()]
    } else{
        Vec::new()
    };

    let new_node = NodeInfo{
        node_name: tree_constraints.template_name.clone(),
        component_name: tree_constraints.component_name.clone(),
        node_id: my_node_id,
        constraints,
        input_signals,
        output_signals,
        signals,
        is_deterministic: tree_constraints.is_deterministic,
        is_custom: tree_constraints.is_custom,
        successors: Vec::new(),
        predecessors
    };
    node_info.push(new_node);

    let mut successors = Vec::new();
    for subcomponent in &tree_constraints.subcomponents{
        successors.push(
            build_structure_nodes(subcomponent, node_id, init_c, node_info, equivalence_nodes, init_constraint_to_node, Some(my_node_id))
        );
    }
    node_info[my_node_id].successors = successors;

    my_node_id
}