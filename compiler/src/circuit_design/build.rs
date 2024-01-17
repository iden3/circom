use crate::circuit_design::circuit::{Circuit, CompilationFlags};
use crate::circuit_design::function::FunctionCodeInfo;
use crate::circuit_design::template::TemplateCodeInfo;
use crate::hir::very_concrete_program::*;
use crate::intermediate_representation::translate;
use crate::intermediate_representation::translate::{CodeInfo, FieldTracker, TemplateDB, ParallelClusters};
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;
use program_structure::file_definition::FileLibrary;
use std::collections::{BTreeMap, HashMap};

#[cfg(debug_assertions)]
fn matching_lengths_and_offsets(list: &InputOutputList) {
    let mut prev = 0;
    let mut offset = 0;
    for signal in list {
        debug_assert_eq!(signal.offset, prev + offset);
        prev = signal.offset;
        offset = signal.lengths.iter().fold(1, |p, c| p * (*c));
    }
}

fn build_template_instances(
    circuit: &mut Circuit,
    c_info: &CircuitInfo,
    ti: Vec<TemplateInstance>,
    mut field_tracker: FieldTracker,
) -> (FieldTracker, HashMap<String,usize>) {

    fn compute_jump(lengths: &Vec<usize>, indexes: &[usize]) -> usize {
        let mut jump = 0;
        let mut full_length = lengths.iter().fold(1, |p, c| p * (*c));
        let mut lengths = lengths.clone();
        lengths.reverse();
        for index in indexes {
            let length = lengths.pop().unwrap();
            full_length /= length;
            jump += (*index) * full_length;
        }
        jump
    }
    let mut cmp_id = 0;
    let mut tmp_id = 0;
    let mut string_table = HashMap::new();
    for template in ti {
        let header = template.template_header;
        let name = template.template_name;
        let instance_values = template.header;
        let msg = format!("Error in template {}", header);
        let number_of_components = template.number_of_components;
        let mut cmp_to_type = HashMap::new();
        for cluster in &template.clusters {
            let name = cluster.cmp_name.clone();
            let xtype = cluster.xtype.clone();
            cmp_to_type.insert(name, xtype);
        }
        circuit.wasm_producer.message_list.push(msg);
        circuit.c_producer.has_parallelism |= template.is_parallel || template.is_parallel_component;

        let mut component_to_parallel: HashMap<String, ParallelClusters> = HashMap::new();
        for trigger in &template.triggers{
            match component_to_parallel.get_mut(&trigger.component_name){
                Some(parallel_info) => {
                    parallel_info.positions_to_parallel.insert(trigger.indexed_with.clone(), trigger.is_parallel);
                    if parallel_info.uniform_parallel_value.is_some(){
                        if parallel_info.uniform_parallel_value.unwrap() != trigger.is_parallel{
                            parallel_info.uniform_parallel_value = None;
                        }
                    }
                },
                None => {
                    let mut positions_to_parallel = BTreeMap::new();
                        positions_to_parallel.insert(trigger.indexed_with.clone(), trigger.is_parallel);
                    let new_parallel_info = ParallelClusters {
                        positions_to_parallel,
                        uniform_parallel_value: Some(trigger.is_parallel),
                    };
                    component_to_parallel.insert(trigger.component_name.clone(), new_parallel_info);
                },
            }
        }
        
        let code_info = CodeInfo {
            cmp_to_type,
            field_tracker,
            component_to_parallel,
            message_id: tmp_id,
            params: Vec::new(),
            header: header.clone(),
            signals: template.signals,
            constants: instance_values,
            files: &c_info.file_library,
            triggers: template.triggers,
            clusters: template.clusters,
            functions: &c_info.functions,
            fresh_cmp_id: cmp_id,
            components: template.components,
            template_database: &c_info.template_database,
            string_table : string_table,
            signals_to_tags: template.signals_to_tags,
        };
        let mut template_info = TemplateCodeInfo {
            name,
            header,
            number_of_components,
            id: tmp_id,
            is_parallel: template.is_parallel,
            is_parallel_component: template.is_parallel_component,
            is_not_parallel_component: template.is_not_parallel_component,
            number_of_inputs: template.number_of_inputs,
            number_of_outputs: template.number_of_outputs,
            number_of_intermediates: template.number_of_intermediates,
            has_parallel_sub_cmp: template.has_parallel_sub_cmp,
            ..TemplateCodeInfo::default()
        };
        let code = template.code;
        let out = translate::translate_code(code, code_info);
        field_tracker = out.constant_tracker;
        template_info.body = out.code;
        template_info.expression_stack_depth = out.expression_depth;
        template_info.var_stack_depth = out.stack_depth;
        template_info.signal_stack_depth = out.signal_depth;
        string_table = out.string_table;
        cmp_id = out.next_cmp_id;
        circuit.add_template_code(template_info);
        tmp_id += 1;
    }
    (field_tracker, string_table)
}

fn build_function_instances(
    circuit: &mut Circuit,
    c_info: &CircuitInfo,
    instances: Vec<VCF>,
    mut field_tracker: FieldTracker,
    mut string_table : HashMap<String,usize>
) -> (FieldTracker, HashMap<String, usize>, HashMap<String, usize>) {
    let mut function_to_arena_size = HashMap::new();
    for instance in instances {
        let msg = format!("Error in function {}", instance.header);
        let header = instance.header;
        let name = instance.name;
        let params = instance.params_types;
        let returns = instance.return_type;
        let id = circuit.wasm_producer.message_list.len();
        circuit.wasm_producer.message_list.push(msg);
        let code_info = CodeInfo {
            field_tracker,
            header: header.clone(),
            message_id: id,
            files: &c_info.file_library,
            functions: &c_info.functions,
            params: params.clone(),
            fresh_cmp_id: 0,
            signals: Vec::with_capacity(0),
            triggers: Vec::with_capacity(0),
            clusters: Vec::with_capacity(0),
            constants: Vec::with_capacity(0),
            components: Vec::with_capacity(0),
            cmp_to_type: HashMap::with_capacity(0),
            component_to_parallel: HashMap::with_capacity(0),
            template_database: &c_info.template_database,
            string_table : string_table,
            signals_to_tags: BTreeMap::new(),
        };
        let mut function_info = FunctionCodeInfo {
            name,
            params,
            returns,
            header: header.clone(),
            ..FunctionCodeInfo::default()
        };
        let code = instance.body;
        let out = translate::translate_code(code, code_info);
        string_table = out.string_table;
        field_tracker = out.constant_tracker;
        function_info.body = out.code;
        function_info.max_number_of_ops_in_expression = out.expression_depth;
        function_info.max_number_of_vars = out.stack_depth;
        function_to_arena_size.insert(header, function_info.max_number_of_vars);
        circuit.add_function_code(function_info);
    }
    (field_tracker, function_to_arena_size, string_table)
}

// WASM producer builder
fn initialize_wasm_producer(vcp: &VCP, database: &TemplateDB, wat_flag:bool, version: &str) -> WASMProducer {
    use program_structure::utils::constants::UsefulConstants;
    let initial_node = vcp.get_main_id();
    let prime = UsefulConstants::new(&vcp.prime).get_p().clone();
    let mut producer = WASMProducer::default();
    let stats = vcp.get_stats();
    producer.main_header = vcp.get_main_instance().unwrap().template_header.clone();
    producer.main_signal_offset = 1;
    producer.prime = prime.to_str_radix(10);
    producer.prime_str = vcp.prime.clone();
    producer.fr_memory_size = match vcp.prime.as_str(){
        "goldilocks" => 412,
        "bn128" => 1948,
        "bls12381" => 1948,
        "grumpkin" => 1948,
        "pallas" => 1948,
        "vesta" => 1948,
        "secq256r1" => 1948,
        _ => unreachable!()
    };
    //producer.fr_memory_size = 412 if goldilocks and 1948 for bn128 and bls12381
    // for each created component we store three u32, for each son we store a u32 in its father
    producer.size_of_component_tree = stats.all_created_components * 3 + stats.all_needed_subcomponents_indexes;
    producer.total_number_of_signals = stats.all_signals + 1;
    producer.size_32_bit = prime.bits() / 32 + if prime.bits() % 32 != 0 { 1 } else { 0 };
    producer.size_32_shift = 0;
    let mut pow = 1;
    while pow < producer.size_32_bit {
        pow *= 2;
        producer.size_32_shift += 1;
    }
    producer.size_32_shift += 2;
    producer.number_of_components = stats.all_created_components;
    producer.witness_to_signal_list = vcp.get_witness_list().clone();
    producer.signals_in_witness = producer.witness_to_signal_list.len();
    producer.number_of_main_inputs = vcp.templates[initial_node].number_of_inputs;
    producer.number_of_main_outputs = vcp.templates[initial_node].number_of_outputs;
    producer.main_input_list = main_input_list(&vcp.templates[initial_node]);
    producer.io_map = build_io_map(vcp, database);
    producer.template_instance_list = build_template_list(vcp);
    producer.field_tracking.clear();
    producer.wat_flag = wat_flag;
    (producer.major_version, producer.minor_version, producer.patch_version) = get_number_version(version);
    producer
}

fn initialize_c_producer(vcp: &VCP, database: &TemplateDB, version: &str) -> CProducer {
    use program_structure::utils::constants::UsefulConstants;
    let initial_node = vcp.get_main_id();
    let prime = UsefulConstants::new(&vcp.prime).get_p().clone();
    let mut producer = CProducer::default();
    let stats = vcp.get_stats();
    producer.main_header = vcp.get_main_instance().unwrap().template_header.clone();
    producer.main_signal_offset = 1;
    producer.prime = prime.to_str_radix(10);
    producer.prime_str = vcp.prime.clone();
    producer.size_of_component_tree = stats.all_created_components * 3 + stats.all_needed_subcomponents_indexes;
    producer.total_number_of_signals = stats.all_signals + 1;
    producer.size_32_bit = prime.bits() / 32 + if prime.bits() % 32 != 0 { 1 } else { 0 };
    producer.size_32_shift = 0;
    let mut pow = 1;
    while pow < producer.size_32_bit {
        pow *= 2;
        producer.size_32_shift += 1;
    }
    producer.size_32_shift += 2;
    producer.number_of_components = stats.all_created_components;
    producer.witness_to_signal_list = vcp.get_witness_list().clone();
    producer.signals_in_witness = producer.witness_to_signal_list.len();
    producer.number_of_main_inputs = vcp.templates[initial_node].number_of_inputs;
    producer.number_of_main_outputs = vcp.templates[initial_node].number_of_outputs;
    producer.main_input_list = main_input_list(&vcp.templates[initial_node]);   
    producer.io_map = build_io_map(vcp, database);
    producer.template_instance_list = build_template_list_parallel(vcp);
    producer.field_tracking.clear();
    (producer.major_version, producer.minor_version, producer.patch_version) = get_number_version(version);
    producer
}

fn main_input_list(main: &TemplateInstance) -> InputList {
    use program_structure::ast::SignalType::*;
    let mut input_list = vec![];
    for s in &main.signals {
        if s.xtype == Input {
            input_list.push((s.name.clone(), s.dag_local_id, s.size()));
        }
    }
    input_list
}

fn build_template_list(vcp: &VCP) -> TemplateList {
    let mut tmp_list = MessageList::new();
    for instance in &vcp.templates {
        tmp_list.push(instance.template_header.clone());
    }
    tmp_list
}

fn build_template_list_parallel(vcp: &VCP) -> TemplateListParallel {
    let mut tmp_list = TemplateListParallel::new();
    for instance in &vcp.templates {
        tmp_list.push(InfoParallel{
            name: instance.template_header.clone(), 
            is_parallel: instance.is_parallel || instance.is_parallel_component,
            is_not_parallel: !instance.is_parallel && instance.is_not_parallel_component,
        });
    }
    tmp_list
}

fn build_io_map(vcp: &VCP, database: &TemplateDB) -> TemplateInstanceIOMap {
    let mut cmp_io = TemplateInstanceIOMap::new();
    for m in &vcp.templates_in_mixed {
        let tmp_instance = &vcp.templates[*m];
        let io_list = build_input_output_list(tmp_instance, database);
        cmp_io.insert(*m, io_list);
    }
    cmp_io
}

fn build_input_output_list(instance: &TemplateInstance, database: &TemplateDB) -> InputOutputList {
    use program_structure::ast::SignalType::*;
    let mut io_list = vec![];
    for s in &instance.signals {
        if s.xtype != Intermediate {
            let def = IODef {
                code: TemplateDB::get_signal_id(database, &instance.template_name, &s.name),
                offset: s.local_id,
                lengths: s.lengths.clone(),
            };
            io_list.push(def);
        }
    }

    #[cfg(debug_assertions)]
    matching_lengths_and_offsets(&io_list);

    io_list
}

fn write_main_inputs_log(vcp: &VCP) {
    use program_structure::ast::SignalType::*;
    use std::fs::File;
    use std::io::{BufWriter, Write};

    const INPUT_LOG: &str = "./log_input_signals.txt";
    let main = vcp.get_main_instance().unwrap();
    let mut writer = BufWriter::new(File::create(INPUT_LOG).unwrap());
    for signal in &main.signals {
        if signal.xtype == Input {
            let name = format!("main.{}", &signal.name);
            let length = signal.size();
            let msg = format!("{} {}\n", name, length);
            writer.write_all(msg.as_bytes()).unwrap();
        }
        writer.flush().unwrap();
    }
}

fn get_number_version(version: &str) -> (usize, usize, usize) {
    use std::str::FromStr;
    let version_splitted: Vec<&str> = version.split(".").collect();
    (
        usize::from_str(version_splitted[0]).unwrap(),
        usize::from_str(version_splitted[1]).unwrap(),
        usize::from_str(version_splitted[2]).unwrap(),
    )
}

struct CircuitInfo {
    file_library: FileLibrary,
    functions: HashMap<String, Vec<usize>>,
    template_database: TemplateDB,
}

pub fn build_circuit(vcp: VCP, flag: CompilationFlags, version: &str) -> Circuit {
    use crate::ir_processing::set_arena_size_in_calls;
    if flag.main_inputs_log {
        write_main_inputs_log(&vcp);
    }
    let template_database = TemplateDB::build(&vcp.templates);
    let mut circuit = Circuit::default();
    circuit.wasm_producer = initialize_wasm_producer(&vcp, &template_database, flag.wat_flag, version);
    circuit.c_producer = initialize_c_producer(&vcp, &template_database, version);

    let field_tracker = FieldTracker::new();
    let circuit_info = CircuitInfo {
        template_database,
        file_library: vcp.file_library,
        functions: vcp.quick_knowledge,
    };

    let (field_tracker, string_table) =
        build_template_instances(&mut circuit, &circuit_info, vcp.templates, field_tracker);
    let (field_tracker, function_to_arena_size, table_string_to_usize) =
        build_function_instances(&mut circuit, &circuit_info, vcp.functions, field_tracker,string_table);

    let table_usize_to_string = create_table_usize_to_string(table_string_to_usize);
    circuit.wasm_producer.set_string_table(table_usize_to_string.clone());
    circuit.c_producer.set_string_table(table_usize_to_string);
    for i in 0..field_tracker.next_id() {
        let constant = field_tracker.get_constant(i).unwrap().clone();
        circuit.wasm_producer.field_tracking.push(constant.clone());
        circuit.c_producer.field_tracking.push(constant);
    }
    for fun in &mut circuit.functions {
        set_arena_size_in_calls(&mut fun.body, &function_to_arena_size);
    }
    for tem in &mut circuit.templates {
        set_arena_size_in_calls(&mut tem.body, &function_to_arena_size);
    }

    circuit
}

pub fn create_table_usize_to_string( string_table : HashMap<String,usize>) -> Vec<String> {
    let size = string_table.len();
    let mut table_usize_to_string =  vec![String::new(); size];

    for (string, us) in string_table {
        table_usize_to_string[us] = string;
    } 
    table_usize_to_string
}
