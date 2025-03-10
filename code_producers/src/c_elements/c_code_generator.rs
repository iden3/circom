use super::*;
use num_bigint_dig::{BigInt, Sign};
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

// Types
const T_U64: &str = "u64";
const T_U32: &str = "u32";
const T_U8: &str = "u8";
pub const T_P_FR_ELEMENT: &str = "PFrElement";
const T_FR_ELEMENT: &str = "FrElement";

// Structs
const S_CIRCOM_HASH_ENTRY: &str = "Circom_HashEntry";
const CIRCOM_HASH_ENTRY_FIELDS: [&str; 3] = ["hash", "signalid", "signalsize"];
const S_CIRCOM_COMPONENT: &str = "Circom_Component";
const CIRCOM_COMPONENT_FIELDS: [&str; 4] =
    ["templateID", "signalStart", "inputCounter", "subcomponents"];
const S_IOFIELD_DEF: &str = "IOFieldDef";
const IOFIELD_DEF_FIELDS: [&str; 4] = ["offset", "size", "lengths", "busid"];


// Global variables
//pub const SIZE_INPUT_HASHMAP: usize = 256;
const G_INPUT_HASHMAP: &str = "inputHashMap"; // type HashSignalInfo[max(256,needed)]
const G_RM_INPUT_SIGNAL_COUNTER: &str = "remainingInputSignalCounter"; // type u32
const G_INPUT_SIGNAL_SET: &str = "inputSignalSetMap"; // type bool[M]
const G_WITNESS_TO_SIGNAL: &str = "witness2signalList"; // type u64[W]
const G_SIGNAL_MEM: &str = "signalMemory"; // type PFrElements[S]
const G_COMPONENT_MEMORY: &str = "componentMemory"; // type Circom_component[C]
const G_COMPONENT_ID_TO_IO_SIGNAL: &str = "componentID2IOSignalInfo"; // type IODef[][C]
const G_TEMPLATE_INSTANCE_FUNCTIONS: &str = "listOfTemplateInstanceFunctions"; // type Circom_templateFunction*[TI]
const G_TEMPLATE_MESSAGES: &str = "listOfTemplateMessages"; // type string[T]

// Local to functions
pub const L_INTERMEDIATE_COMPUTATIONS_STACK: &str = "expaux"; // type PFrElements[]
pub fn declare_expaux(size: usize) -> CInstruction {
    format!("{} {}[{}]", T_FR_ELEMENT, L_INTERMEDIATE_COMPUTATIONS_STACK, size)
}
pub fn declare_64bit_expaux(size: usize) -> CInstruction {
    format!("{} {}[{}]", T_U64, L_INTERMEDIATE_COMPUTATIONS_STACK, size)
}
pub fn expaux(at: CInstruction) -> CInstruction {
    format!("{}[{}]", L_INTERMEDIATE_COMPUTATIONS_STACK, at)
}
pub fn store_expaux(at: CInstruction, value: CInstruction) -> CInstruction {
    format!("{} = {}", expaux(at), value)
}

pub const L_VAR_FUNC_CALL_STORAGE: &str = "lvarcall"; // type PFrElements[]
pub fn declare_lvar_func_call(size: usize) -> CInstruction {
    format!("{} {}[{}]", T_FR_ELEMENT, L_VAR_FUNC_CALL_STORAGE, size)
}
pub fn declare_64bit_lvar_func_call(size: usize) -> CInstruction {
    format!("{} {}[{}]", T_U64, L_VAR_FUNC_CALL_STORAGE, size)
}

pub const L_VAR_STORAGE: &str = "lvar"; // type PFrElements[]
pub fn declare_lvar(size: usize) -> CInstruction {
    format!("{} {}[{}]", T_FR_ELEMENT, L_VAR_STORAGE, size)
}
pub fn declare_64bit_lvar(size: usize) -> CInstruction {
    format!("{} {}[{}]", T_U64, L_VAR_STORAGE, size)
}
pub fn declare_lvar_pointer() -> CInstruction {
    format!("{}* {}", T_FR_ELEMENT, L_VAR_STORAGE)
}
pub fn declare_64bit_lvar_pointer() -> CInstruction {
    format!("{}* {}", T_U64, L_VAR_STORAGE)
}
pub fn declare_64bit_lvar_array() -> CInstruction {
    format!("{} {}[]", T_U64, L_VAR_STORAGE)
}
pub fn lvar(at: CInstruction) -> CInstruction {
    format!("{}[{}]", L_VAR_STORAGE, at)
}
pub fn store_lvar(at: CInstruction, value: CInstruction) -> CInstruction {
    format!("{} = {}", lvar(at), value)
}

pub const SUBCOMPONENT_AUX: &str = "sub_component_aux"; // type PFrElements[]
pub fn declare_sub_component_aux() -> CInstruction {
    format!("uint {}", SUBCOMPONENT_AUX)
}

pub const INDEX_MULTIPLE_EQ: &str = "index_multiple_eq"; // type PFrElements[]
pub fn declare_index_multiple_eq() -> CInstruction {
    format!("uint {}", INDEX_MULTIPLE_EQ)
}
pub fn index_multiple_eq() -> CInstruction {
    format!("{}", INDEX_MULTIPLE_EQ)
}

pub const FUNCTION_DESTINATION: &str = "destination"; // type PFrElements[]
pub fn declare_dest_pointer() -> CInstruction {
    format!("{}* {}", T_FR_ELEMENT, FUNCTION_DESTINATION)
}
pub fn declare_64bit_dest_reference() -> CInstruction {
    format!("{}& {}", T_U64, FUNCTION_DESTINATION)
}
pub const FUNCTION_DESTINATION_SIZE: &str = "destination_size"; // type PFrElements[]
pub fn declare_dest_size() -> CInstruction {
    format!("int {}", FUNCTION_DESTINATION_SIZE)
}

pub const CTX_INDEX: &str = "ctx_index";
pub fn declare_ctx_index() -> CInstruction {
    format!("uint {}", CTX_INDEX)
}

pub fn ctx_index() -> CInstruction {
    format!("{}", CTX_INDEX)
}
pub fn store_ctx_index(value: CInstruction) -> CInstruction {
    format!("{} = {}", ctx_index(), value)
}

pub const SIGNAL_OFFSET: &str = "soffset";
pub fn declare_signal_offset() -> CInstruction {
    format!("uint {}", SIGNAL_OFFSET)
}
pub fn signal_offset() -> CInstruction {
    SIGNAL_OFFSET.to_string()
}

pub const COMPONENT_OFFSET: &str = "coffset";
pub fn declare_component_offset() -> CInstruction {
    format!("uint {}", COMPONENT_OFFSET)
}
pub fn component_offset() -> CInstruction {
    COMPONENT_OFFSET.to_string()
}

pub const COMPONENT_NAME: &str = "componentName";
pub fn declare_component_name() -> CInstruction {
    format!("std::string {}", COMPONENT_NAME)
}
pub fn component_name() -> CInstruction {
    COMPONENT_NAME.to_string()
}

pub const COMPONENT_FATHER: &str = "componentFather";
pub fn declare_component_father() -> CInstruction {
    format!("uint {}", COMPONENT_FATHER)
}
pub fn component_father() -> CInstruction {
    COMPONENT_FATHER.to_string()
}

pub const CIRCOM_CALC_WIT: &str = "ctx";
pub fn declare_circom_calc_wit() -> CInstruction {
    format!("Circom_CalcWit* {}", CIRCOM_CALC_WIT)
}
pub fn circom_calc_wit() -> CInstruction {
    format!("{}", CIRCOM_CALC_WIT)
}

pub const TEMP_INS_2_IO_INFO: &str = "templateInsId2IOSignalInfo";
pub fn template_ins_2_io_info() -> CInstruction {
    format!("{}", TEMP_INS_2_IO_INFO)
}

pub const BUS_INS_2_FIELD_INFO: &str = "busInsId2FieldInfo";
pub fn bus_ins_2_field_info() -> CInstruction {
    format!("{}", BUS_INS_2_FIELD_INFO)
}

pub fn template_id_in_component(idx: CInstruction) -> CInstruction {
    format!("{}->componentMemory[{}].templateId", CIRCOM_CALC_WIT, idx)
}
pub const MY_SIGNAL_START: &str = "mySignalStart";
pub fn declare_my_signal_start() -> CInstruction {
    format!(
        "u64 {} = {}->componentMemory[{}].signalStart",
        MY_SIGNAL_START, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn my_signal_start() -> CInstruction {
    format!("{}", MY_SIGNAL_START)
}

pub const MY_TEMPLATE_NAME: &str = "myTemplateName";
pub fn declare_my_template_name() -> CInstruction {
    format!(
        "std::string {} = {}->componentMemory[{}].templateName",
        MY_TEMPLATE_NAME, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn declare_my_template_name_function(name: &String) -> CInstruction {
    format!(
        "std::string {} = \"{}\"",
        MY_TEMPLATE_NAME, name.to_string()
    )
}
pub fn my_template_name() -> CInstruction {
    format!("{}", MY_TEMPLATE_NAME)
}


pub const MY_COMPONENT_NAME: &str = "myComponentName";
pub fn declare_my_component_name() -> CInstruction {
    format!(
        "std::string {} = {}->componentMemory[{}].componentName",
        MY_COMPONENT_NAME, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn my_component_name() -> CInstruction {
    format!("{}", MY_COMPONENT_NAME)
}

pub const MY_FATHER: &str = "myFather";
pub fn declare_my_father() -> CInstruction {
    format!(
        "u64 {} = {}->componentMemory[{}].idFather",
        MY_FATHER, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn my_father() -> CInstruction {
    format!("{}", MY_FATHER)
}

pub const MY_ID: &str = "myId";
pub fn declare_my_id() -> CInstruction {
    format!(
        "u64 {} = {}",
        MY_ID, CTX_INDEX
    )
}
pub fn my_id() -> CInstruction {
    format!("{}", MY_ID)
}

pub const FUNCTION_TABLE: &str = "_functionTable";
pub fn function_table() -> CInstruction {
    format!("{}", FUNCTION_TABLE)
}

pub const FUNCTION_TABLE_PARALLEL: &str = "_functionTableParallel";
pub fn function_table_parallel() -> CInstruction {
    format!("{}", FUNCTION_TABLE_PARALLEL)
}

pub const SIGNAL_VALUES: &str = "signalValues";
pub fn declare_signal_values() -> CInstruction {
    format!("FrElement* {} = {}->{}", SIGNAL_VALUES, CIRCOM_CALC_WIT, SIGNAL_VALUES)
}
pub fn declare_64bit_signal_values() -> CInstruction {
    format!("u64* {} = {}->{}", SIGNAL_VALUES, CIRCOM_CALC_WIT, SIGNAL_VALUES)
}
pub fn signal_values(at: CInstruction) -> CInstruction {
    format!("{}[{} + {}]", SIGNAL_VALUES, MY_SIGNAL_START, at)
}
pub fn store_signal_values(at: CInstruction, value: CInstruction) -> CInstruction {
    format!("{} = {}", signal_values(at), value)
}

/*
pub const MY_MEMORY: &str = "myMemory";
pub fn declare_my_memory() -> CInstruction {
    format!("Circom_Component {} = {}->componentMemory[{}]", MY_MEMORY, CIRCOM_CALC_WIT, CTX_INDEX)
}
pub fn my_memory() -> CInstruction {
    format!("{}", MY_MEMORY)
}

pub const MY_TEMPLATE_ID: &str = "myTemplateId";
pub fn declare_my_template_id() -> CInstruction {
    format!(
        "u32 {} = {}->componentMemory[{}].templateId",
        MY_TEMPLATE_ID, CIRCOM_CALC_WIT, CTX_INDEX
    )
}

pub fn my_template_id() -> CInstruction {
    format!("{}", MY_TEMPLATE_ID)
}

pub const MY_INPUT_COUNTER: &str = "myInputCounter";
pub fn declare_my_input_counter() -> CInstruction {
    format!(
        "u32 {} = {}->componentMemory[{}].inputCounter",
        MY_INPUT_COUNTER, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn my_input_counter() -> CInstruction {
    format!("{}", MY_INPUT_COUNTER)
}

pub const TEMPLATE_INS_ID_2_IO_SIGNAL_INFO: &str = "templateInsId2IOSignalInfo";
pub fn declare_template_ins_id_2_io_signal_info() -> CInstruction {
    format!(
        "std::map<u32,IOFieldDefPair> {} = {}->{}",
        TEMPLATE_INS_ID_2_IO_SIGNAL_INFO, CIRCOM_CALC_WIT, TEMPLATE_INS_ID_2_IO_SIGNAL_INFO
    )
}
pub fn template_ins_id_2_io_signal_info() -> CInstruction {
    format!("{}", TEMPLATE_INS_ID_2_IO_SIGNAL_INFO)
}

pub const LIST_OF_TEMPLATE_INSTANCE_FUNCTIONS: &str = "listOfTemplateInstanceFunctions";
pub fn declare_list_of_template_instance_functions() -> CInstruction {
    format!(
        "Circom_TemplateFunction* {} = {}->{}",
        LIST_OF_TEMPLATE_INSTANCE_FUNCTIONS, CIRCOM_CALC_WIT, LIST_OF_TEMPLATE_INSTANCE_FUNCTIONS
    )
}
pub fn list_of_template_instance_functions() -> CInstruction {
    format!("{}", LIST_OF_TEMPLATE_INSTANCE_FUNCTIONS)
}
*/

pub const MY_SUBCOMPONENTS: &str = "mySubcomponents";
pub fn declare_my_subcomponents() -> CInstruction {
    format!(
        "u32* {} = {}->componentMemory[{}].subcomponents",
        MY_SUBCOMPONENTS, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn my_subcomponents() -> CInstruction {
    format!("{}", MY_SUBCOMPONENTS)
}

pub const MY_SUBCOMPONENTS_PARALLEL: &str = "mySubcomponentsParallel";
pub fn declare_my_subcomponents_parallel() -> CInstruction {
    format!(
        "bool* {} = {}->componentMemory[{}].subcomponentsParallel",
        MY_SUBCOMPONENTS_PARALLEL, CIRCOM_CALC_WIT, CTX_INDEX
    )
}
pub fn my_subcomponents_parallel() -> CInstruction {
    format!("{}", MY_SUBCOMPONENTS_PARALLEL)
}

pub const CIRCUIT_CONSTANTS: &str = "circuitConstants";
pub fn declare_circuit_constants() -> CInstruction {
    format!("FrElement* {} = {}->{}", CIRCUIT_CONSTANTS, CIRCOM_CALC_WIT, CIRCUIT_CONSTANTS)
}
pub fn circuit_constants(at: CInstruction) -> CInstruction {
    format!("{}[{}]", CIRCUIT_CONSTANTS, at)
}
pub fn store_circuit_constants(at: CInstruction, value: CInstruction) -> CInstruction {
    format!("{} = {}", circuit_constants(at), value)
}
pub const FREE_IN_COMPONENT_MEM_MUTEX: &str = "freePositionInComponentMemoryMutex"; // type u32
pub const FREE_IN_COMPONENT_MEM: &str = "freePositionInComponentMemory"; // type u32
pub fn declare_free_position_in_component_memory() -> CInstruction {
    format!("u32 {} = {}->{}", FREE_IN_COMPONENT_MEM, CIRCOM_CALC_WIT, FREE_IN_COMPONENT_MEM)
}
pub fn free_position_in_component_memory() -> CInstruction {
    format!("{}", FREE_IN_COMPONENT_MEM)
}
pub fn store_free_position_in_component_memory(value: String) -> CInstruction {
    format!("{} = {}", FREE_IN_COMPONENT_MEM, value)
}

pub const LIST_OF_TEMPLATE_MESSAGES: &str = "listOfTemplateMessages";
pub fn declare_list_of_template_messages_use() -> CInstruction {
    format!(
        "std::string* {} = {}->{}",
        LIST_OF_TEMPLATE_MESSAGES, CIRCOM_CALC_WIT, LIST_OF_TEMPLATE_MESSAGES
    )
}
pub fn list_of_template_messages_use() -> CInstruction {
    format!("{}", LIST_OF_TEMPLATE_MESSAGES)
}

pub fn build_callable(header: String, params: Vec<String>, body: Vec<String>) -> String {
    let mut params_string = "".to_string();
    for param in params {
        params_string = format!("{}{},", params_string, param);
    }
    params_string.pop();
    format!("{}({}){{\n{}}}\n", header, params_string, merge_code(body))
}

pub fn argument_list(args: Vec<String>) -> String {
    let mut args_string = "".to_string();
    for arg in args {
        args_string = format!("{}{},", args_string, arg);
    }
    args_string.pop();
    args_string
}

pub fn build_call(header: String, arguments: Vec<String>) -> String {
    format!("{}({})", header, argument_list(arguments))
}

pub fn set_list(elems: Vec<usize>) -> String {
    let mut set_string = "{".to_string();
    for elem in elems {
        set_string = format!("{}{},", set_string, elem);
    }
    set_string.pop();
    set_string .push('}');
    set_string
}

pub fn set_list_tuple(elems: Vec<(usize, usize)>) -> String {
    let mut set_string = "{".to_string();
    for (elem_a, elem_b) in elems {
        set_string = format!("{}{{{},{}}},", set_string, elem_a, elem_b);
    }
    set_string.pop();
    set_string .push('}');
    set_string
}

pub fn set_list_bool(elems: Vec<bool>) -> String {
    let mut set_string = "{".to_string();
    for elem in elems {
        set_string = format!("{}{},", set_string, elem);
    }
    set_string.pop();
    set_string.push('}');
    set_string
}

pub fn add_return() -> String {
    "return;".to_string()
}

pub fn generate_my_array_position(aux_dimensions: String, len_dimensions: String, param: String) -> String {
    format!("{}->generate_position_array({}, {}, {})", CIRCOM_CALC_WIT, aux_dimensions, len_dimensions, param)
}

pub fn generate_my_trace() -> String {
    format!("{}->getTrace({})", CIRCOM_CALC_WIT, MY_ID)
}

pub fn build_failed_assert_message(line: usize) -> String{
    
    format!("std::cout << \"Failed assert in template/function \" << {} << \" line {}. \" <<  \"Followed trace of components: \" << {} << std::endl" ,
        MY_TEMPLATE_NAME,
        line,
        generate_my_trace()
     )
}


pub fn build_conditional(
    cond: Vec<String>,
    if_body: Vec<String>,
    else_body: Vec<String>,
) -> String {
    assert_eq!(cond.len(), 1);
    let mut conditional = format!("if({}){{\n{}\n}}", cond[0], merge_code(if_body));
    if !else_body.is_empty() {
        conditional.push_str(&format!("else{{\n{}\n}}", merge_code(else_body)));
    }
    conditional
}

pub fn merge_code(instructions: Vec<String>) -> String {
    let code = format!("{}\n", instructions.join("\n"));
    code
}

pub fn collect_template_headers(instances: &TemplateListParallel) -> Vec<String> {
    let mut template_headers = vec![];
    for instance in instances {
        let params_run = vec![declare_ctx_index(), declare_circom_calc_wit()];
        let params_run = argument_list(params_run);
        let params_create = vec![
            declare_signal_offset(), 
            declare_component_offset(), 
            declare_circom_calc_wit(),
            declare_component_name(),
            declare_component_father(),
        ];
        let params_create = argument_list(params_create);
        if instance.is_parallel{
            let run_header = format!("void {}_run_parallel({});", instance.name, params_run);
            let create_header = format!("void {}_create_parallel({});", instance.name, params_create);
            template_headers.push(create_header);
            template_headers.push(run_header);
        }
        if instance.is_not_parallel{
            let run_header = format!("void {}_run({});", instance.name, params_run);
            let create_header = format!("void {}_create({});", instance.name, params_create);
            template_headers.push(create_header);
            template_headers.push(run_header);
        }
    }
    template_headers
}

pub fn collect_function_headers(producer: &CProducer, functions: Vec<String>) -> Vec<String> {
    let mut function_headers = vec![];
    for function in functions {
        let params = vec![
            declare_circom_calc_wit(),
            if producer.prime_str != "goldilocks" { declare_lvar_pointer()
            } else { declare_64bit_lvar_array() },
            declare_component_father(),
            if producer.prime_str != "goldilocks" { declare_dest_pointer()
            } else { declare_64bit_dest_reference() },
            declare_dest_size(),
        ];
        let params = argument_list(params);
        let header = format!("void {}({});", function, params);
        function_headers.push(header);
    }
    function_headers
}

//--------------- generate all kinds of Data for the .dat file ---------------

pub fn generate_hash_map(signal_name_list: &Vec<InputInfo>, size: usize) -> Vec<(u64, u64, u64)> {
    assert!(signal_name_list.len() <= size);
    let mut hash_map = vec![(0, 0, 0); size];
    for i in 0..signal_name_list.len() {
        let h = hasher(&signal_name_list[i].name);
        let mut p = h as usize % size;
        while hash_map[p].1 != 0 {
            p = (p + 1) % size;
        }
        hash_map[p] = (h, signal_name_list[i].start as u64, signal_name_list[i].size as u64);
    }
    hash_map
}

pub fn generate_dat_from_hash_map(map: &Vec<(u64, u64, u64)>) -> Vec<u8> {
    let mut hash_map_data = vec![];
    for (h, p, s) in map {
        let mut v: Vec<u8> = h.to_be_bytes().to_vec();
        v.reverse();
        hash_map_data.append(&mut v);
        v = p.to_be_bytes().to_vec();
        v.reverse();
        hash_map_data.append(&mut v);
        v = s.to_be_bytes().to_vec();
        v.reverse();
        hash_map_data.append(&mut v);
    }
    hash_map_data
}

pub fn generate_dat_witness_to_signal_list(signal_list: &Vec<usize>) -> Vec<u8> {
    let mut signal_list_data = vec![];
    for s in signal_list {
        let s64 = *s as u64;
        let mut v: Vec<u8> = s64.to_be_bytes().to_vec();
        v.reverse();
        signal_list_data.append(&mut v);
    }
    signal_list_data
}

pub fn generate_dat_constant_list(producer: &CProducer, constant_list: &Vec<String>) -> Vec<u8> {
    let mut constant_list_data = vec![];
    for s in constant_list {
        //      For sort/long or short/montgomery
        let mut n = s.parse::<BigInt>().unwrap();
        let min_int = BigInt::from(-2147483648);
        let max_int = BigInt::from(2147483647);
        let p = producer.get_prime().parse::<BigInt>().unwrap();
        let b = ((p.bits() + 63) / 64) * 64;
        let mut r = BigInt::from(1);
        r = r << b;
        n = n % BigInt::clone(&p);
        n = n + BigInt::clone(&p);
        n = n % BigInt::clone(&p);
        let hp = BigInt::clone(&p) / 2;
        let mut nn;
        if BigInt::clone(&n) > hp {
            nn = BigInt::clone(&n) - BigInt::clone(&p);
        } else {
            nn = BigInt::clone(&n);
        }

        if min_int <= nn && nn <= max_int {
            // It is short. We have it in short & Montgomery
            if nn < BigInt::from(0) {
                nn = BigInt::parse_bytes(b"100000000", 16).unwrap() + nn;
            }
            let (snn, bnn) = nn.to_bytes_be();
            assert_ne!(snn, Sign::Minus);
            let mut v: Vec<u8> = bnn.to_vec();
            v.reverse();
            constant_list_data.append(&mut v);
            for _i in 0..4 - bnn.len() {
                constant_list_data.push(0);
            }
            //short Montgomery
            let sm = 0x40000000 as u32;
            let mut v: Vec<u8> = sm.to_be_bytes().to_vec();
            v.reverse();
            constant_list_data.append(&mut v);
        } else {
            //It is long. Only Montgomery
            for _i in 0..4 {
                constant_list_data.push(0);
            }
            let lm = 0xC0000000 as u32;
            let mut v: Vec<u8> = lm.to_be_bytes().to_vec();
            v.reverse();
            constant_list_data.append(&mut v);
        }
        // Montgomery
        // n*R mod P
        n = (n * BigInt::clone(&r)) % BigInt::clone(&p);
        let (sn, bn) = n.to_bytes_be();
        assert_ne!(sn, Sign::Minus);
        let mut v: Vec<u8> = bn.to_vec();
        v.reverse();
        constant_list_data.append(&mut v);
        for _i in 0..(producer.get_size_32_bit() * 4) - bn.len() {
            constant_list_data.push(0);
        }
    }
    constant_list_data
}

pub fn generate_dat_io_signals_info(
    _producer: &CProducer,
    io_map: &TemplateInstanceIOMap,
) -> Vec<u8> {
    // println!("size: {}",io_map.len());
    let mut io_signals_info = vec![];
    for (t_ins, _) in io_map {
        //println!("info: {}",t_ins);
        let t32 = *t_ins as u32;
        let mut v: Vec<u8> = t32.to_be_bytes().to_vec();
        v.reverse();
        io_signals_info.append(&mut v);
    }
    for (_, l_io_def) in io_map {
        //println!("io_def_len: {}",l_io_def.len());
        let l32 = l_io_def.len() as u32;
        let mut v: Vec<u8> = l32.to_be_bytes().to_vec();
        v.reverse();
        io_signals_info.append(&mut v);
        for s in l_io_def {
            //println!("offset: {}",s.offset);
            let l32 = s.offset as u32;
            let mut v: Vec<u8> = l32.to_be_bytes().to_vec();
            v.reverse();
            io_signals_info.append(&mut v);
            let n32: u32;
            if s.lengths.len() > 0 {
                n32 = (s.lengths.len() - 1) as u32;
            } else {
                n32 = 0;
            }
            // println!("dims-1: {}",n32);
            let mut v: Vec<u8> = n32.to_be_bytes().to_vec();
            v.reverse();
            io_signals_info.append(&mut v);
            for i in 1..s.lengths.len() {
                // println!("dims {}: {}",i,s.lengths[i]);
                let pos = s.lengths[i] as u32;
                let mut v: Vec<u8> = pos.to_be_bytes().to_vec();
                v.reverse();
                io_signals_info.append(&mut v);
            }
            let s32 = s.size as u32;
            let mut v: Vec<u8> = s32.to_be_bytes().to_vec();
            v.reverse();
            io_signals_info.append(&mut v);
	    let b32 = if let Some(value) = s.bus_id {
		value as u32
	    } else {
		0 as u32
	    };
	    let mut v = b32.to_be_bytes().to_vec();
	    v.reverse();
	    io_signals_info.append(&mut v);
        }
    }
    io_signals_info
}

pub fn generate_dat_bus_field_info(
    _producer: &CProducer,
    field_map: &FieldMap,
) -> Vec<u8> {
    // println!("size: {}",field_map.len());
    let mut bus_field_info = vec![];
    //let t32 = field_map.len() as u32;
    //let mut v: Vec<u8> = t32.to_be_bytes().to_vec();
    //v.reverse();
    //bus_field_info.append(&mut v);
    for c in 0..field_map.len() {
        //println!("field_def_len: {}",field_map[c].len());
        let l32 = field_map[c].len() as u32;
        let mut v: Vec<u8> = l32.to_be_bytes().to_vec();
        v.reverse();
        bus_field_info.append(&mut v);
        for s in &field_map[c] {
            //println!("offset: {}",s.offset);
            let l32 = s.offset as u32;
            let mut v: Vec<u8> = l32.to_be_bytes().to_vec();
            v.reverse();
            bus_field_info.append(&mut v);
            let n32: u32;
            if s.dimensions.len() > 0 {
                n32 = (s.dimensions.len() - 1) as u32;
            } else {
                n32 = 0;
            }
            // println!("dims-1: {}",n32);
            let mut v: Vec<u8> = n32.to_be_bytes().to_vec();
            v.reverse();
            bus_field_info.append(&mut v);
            for i in 1..s.dimensions.len() {
                // println!("dims {}: {}",i,s.lengths[i]);
                let pos = s.dimensions[i] as u32;
                let mut v: Vec<u8> = pos.to_be_bytes().to_vec();
                v.reverse();
                bus_field_info.append(&mut v);
            }
            let s32 = s.size as u32;
            let mut v: Vec<u8> = s32.to_be_bytes().to_vec();
            v.reverse();
            bus_field_info.append(&mut v);
	    let b32 = if let Some(value) = s.bus_id {
		value as u32
	    } else {
		0 as u32
	    };
	    let mut v = b32.to_be_bytes().to_vec();
	    v.reverse();
	    bus_field_info.append(&mut v);
        }
    }
    bus_field_info
}

// in main fix one to 1

/*
- witness2signal: u64[u8,8] for list length
           [u8,8] for all elements of the given length
- constants

- prime: u32 for string length + [u8] of string as byte
- hashmap: u32[u8,4] for hashtable length
           a pair [u8,8],[u8,8] for all entries given by the length
- io_map:  u64[u8,8] for io_map length
           for every element of the given length
              u64[u8,8] for the template_instance_id
              u64[u8,8] for template_instance info length
              for every signal in the info (given by length)
                u64[u8,8] with offset
                u64[u8,4] num of dimensions (except first is any)
                for every given dimension
                    u64[u8,8] size of dimension

 */

pub fn generate_dat_file(dat_file: &mut dyn Write, producer: &CProducer) -> std::io::Result<()> {
    //let p = producer.get_prime().as_bytes();
    //let pl = p.len() as u32;
    //dfile.write_all(&pl.to_be_bytes())?;
    //dfile.write_all(&p)?;
    //dfile.flush()?;

//<<<<<<< HEAD
//=======
    let aux = producer.get_main_input_list();
    let map = generate_hash_map(&aux,producer.get_input_hash_map_entry_size());
//>>>>>>> 9f3da35a8ac3107190f8c85c8cf3ea1a0f8780a4
    let hashmap = generate_dat_from_hash_map(&map); //bytes u64 --> u64
                                                    //let hml = producer.get_input_hash_map_entry_size() as u32;
                                                    //dfile.write_all(&hml.to_be_bytes())?;
    dat_file.write_all(&hashmap)?;
    //dat_file.flush()?;
    let s = generate_dat_witness_to_signal_list(producer.get_witness_to_signal_list()); // list of bytes u64
                                                                                        //let sl = s.len() as u64; //8 bytes
                                                                                        //dfile.write_all(&sl.to_be_bytes())?;
    dat_file.write_all(&s)?;
    //dat_file.flush()?;
    if producer.prime_str != "goldilocks" { // if field number is not goldilocks
        let s = generate_dat_constant_list(producer, producer.get_field_constant_list()); // list of bytes Fr
        dat_file.write_all(&s)?;
    }
    //dat_file.flush()?;
    //let ioml = producer.get_io_map().len() as u64;
    //dfile.write_all(&ioml.to_be_bytes())?;
    let iomap = generate_dat_io_signals_info(&producer, producer.get_io_map());
    dat_file.write_all(&iomap)?;
    if producer.get_io_map().len() > 0 {
	//otherwise it is not used
	let fieldmap = generate_dat_bus_field_info(&producer, producer.get_busid_field_info());
	dat_file.write_all(&fieldmap)?;
    }
    /*
        let ml = producer.get_message_list();
        let mll = ml.len() as u64;
        dfile.write_all(&mll.to_be_bytes())?;
        for i in 0..ml.len() {
            let m = ml[i].as_bytes();
            let mlen = m.len() as u32;
            dfile.write_all(&mlen.to_be_bytes())?;
            dfile.write_all(m)?;
            dfile.flush()?;
        }
     */
    dat_file.flush()?;
    Ok(())
}
pub fn generate_function_list(_producer: &CProducer, list: &TemplateListParallel) -> (String, String) {
    let mut func_list= "".to_string();
    let mut func_list_parallel= "".to_string();
    if list.len() > 0 {
        if list[0].is_parallel{
            func_list_parallel.push_str(&format!("\n{}_run_parallel",list[0].name));
        }else{
            func_list_parallel.push_str(&format!("\nNULL"));
        }
        if list[0].is_not_parallel{
            func_list.push_str(&format!("\n{}_run",list[0].name));
        }else{
            func_list.push_str(&format!("\nNULL"));
        }
	    for i in 1..list.len() {
            if list[i].is_parallel{
                func_list_parallel.push_str(&format!(",\n{}_run_parallel",list[i].name));
            }else{
                func_list_parallel.push_str(&format!(",\nNULL"));
            }
            if list[i].is_not_parallel{
                func_list.push_str(&format!(",\n{}_run",list[i].name));
            }else{
                func_list.push_str(&format!(",\nNULL"));
            }
	    }
    }
    (func_list, func_list_parallel)
}

pub fn generate_message_list_def(_producer: &CProducer, message_list: &MessageList) -> Vec<String> {
    let mut instructions = vec![];
    let list_of_messages = "listOfTemplateMessages".to_string();
    let start = format!("std::string {}1 [] = {{\n", list_of_messages);
    // let start = format!("{}1 [] = {{\n",producer.get_list_of_messages_name());
    instructions.push(start);
    if message_list.len() > 0 {
        instructions.push(format!("\"{}\"", message_list[0]));
        for i in 1..message_list.len() {
            instructions.push(format!(",\n\"{}\"", message_list[i]));
        }
        instructions.push("\n".to_string());
    }
    instructions.push("};\n".to_string());
    //instructions.push(format!("#define {} = {}1;\n", list_of_messages, list_of_messages));
    instructions
}

pub fn generate_function_release_memory_component() -> Vec<String>{
    let mut instructions = vec![];
    instructions.push("void release_memory_component(Circom_CalcWit* ctx, uint pos) {{\n".to_string());
    instructions.push("if (pos != 0){{\n".to_string());
    instructions.push("if(ctx->componentMemory[pos].subcomponents)".to_string());
    instructions.push("delete []ctx->componentMemory[pos].subcomponents;\n".to_string());
    instructions.push("if(ctx->componentMemory[pos].subcomponentsParallel)".to_string());
    instructions.push("delete []ctx->componentMemory[pos].subcomponentsParallel;\n".to_string());
    instructions.push("if(ctx->componentMemory[pos].outputIsSet)".to_string());
    instructions.push("delete []ctx->componentMemory[pos].outputIsSet;\n".to_string());
    instructions.push("if(ctx->componentMemory[pos].mutexes)".to_string());
    instructions.push("delete []ctx->componentMemory[pos].mutexes;\n".to_string());
    instructions.push("if(ctx->componentMemory[pos].cvs)".to_string());
    instructions.push("delete []ctx->componentMemory[pos].cvs;\n".to_string());
    instructions.push("if(ctx->componentMemory[pos].sbct)".to_string());
    instructions.push("delete []ctx->componentMemory[pos].sbct;\n".to_string());
    instructions.push("}}\n\n".to_string());
    instructions.push("}}\n\n".to_string());
    instructions
}

pub fn generate_function_release_memory_circuit() -> Vec<String>{ 
    // deleting each one of the components
    let mut instructions = vec![];
    instructions.push("void release_memory(Circom_CalcWit* ctx) {{\n".to_string());
    instructions.push("for (int i = 0; i < get_number_of_components(); i++) {{\n".to_string());
    instructions.push("release_memory_component(ctx, i);\n".to_string());
    instructions.push("}}\n".to_string());
    instructions.push("}}\n".to_string());
    instructions
  }

pub fn generate_main_cpp_file(c_folder: &PathBuf, producer: &CProducer) -> std::io::Result<()> {
    use std::io::BufWriter;
    let mut code = "".to_string();
    if producer.prime_str != "goldilocks" { // if field number is not goldilocks   
        let file = include_str!("common/main.cpp");
        for line in file.lines() {
            code = format!("{}{}\n", code, line);
        }
    } else {
        let main_template: &str = include_str!("common64/main.cpp");
        let template = handlebars::Handlebars::new();
        code = template
            .render_template(
                main_template,
                &json!({
                    "prime": format!("{}ull",producer.get_prime())
                }),
            )
            .expect("must render");
    }
    let mut file_path = c_folder.clone();
    file_path.push("main");
    file_path.set_extension("cpp");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    c_file.write_all(code.as_bytes())?;
    c_file.flush()?;
    Ok(())
}

pub fn generate_circom_hpp_file(c_folder: &PathBuf, producer: &CProducer) -> std::io::Result<()> {
    use std::io::BufWriter;
    let mut file_path = c_folder.clone();
    file_path.push("circom");
    file_path.set_extension("hpp");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    let mut code = "".to_string();
    let file = if producer.prime_str != "goldilocks" {
        include_str!("common/circom.hpp")
    } else { include_str!("common64/circom.hpp")};
    for line in file.lines() {
        code = format!("{}{}\n", code, line);
    }
    c_file.write_all(code.as_bytes())?;
    c_file.flush()?;
    Ok(())
}

pub fn generate_fr_hpp_file(c_folder: &PathBuf, prime: &String, producer: &CProducer) -> std::io::Result<()> {
    use std::io::BufWriter;
    let mut file_path = c_folder.clone();
    file_path.push("fr");
    file_path.set_extension("hpp");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    let mut code = "".to_string();
    if producer.prime_str != "goldilocks" && producer.no_asm {
        let p = producer.get_prime().parse::<BigInt>().unwrap();
        let n64 = (p.bits() + 63) / 64;
        let fr_hpp_template: &str = include_str!("generic/fr.hpp");
        let template = handlebars::Handlebars::new();
        code = template
            .render_template(
                fr_hpp_template,
                &json!({
                    "fr_n64": n64,
                }),
            )
            .expect("must render");
        c_file.write_all(code.as_bytes())?;
        c_file.flush()?;
    } else {
        let file = match prime.as_ref(){
            "bn128" => include_str!("bn128/fr.hpp"),
            "bls12381" => include_str!("bls12381/fr.hpp"),
            //"goldilocks" => include_str!("goldilocks/fr.hpp"),
            "goldilocks" => include_str!("goldilocks/fr.hpp"),
            "grumpkin" => include_str!("grumpkin/fr.hpp"),
            "pallas" => include_str!("pallas/fr.hpp"),
            "vesta" => include_str!("vesta/fr.hpp"),
            "secq256r1" => include_str!("secq256r1/fr.hpp"),
            "bls12377" => include_str!("bls12377/fr.hpp"),
            _ => unreachable!(),
        };
        for line in file.lines() {
            code = format!("{}{}\n", code, line);
        }
        c_file.write_all(code.as_bytes())?;
        c_file.flush()?;
    }
    Ok(())
}

pub fn generate_calcwit_hpp_file(c_folder: &PathBuf, producer: &CProducer) -> std::io::Result<()> {
    use std::io::BufWriter;
    let mut file_path = c_folder.clone();
    file_path.push("calcwit");
    file_path.set_extension("hpp");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    let mut code = "".to_string();
    let file = if producer.prime_str != "goldilocks" { include_str!("common/calcwit.hpp")
    } else { include_str!("common64/calcwit.hpp")};
    for line in file.lines() {
        code = format!("{}{}\n", code, line);
    }
    c_file.write_all(code.as_bytes())?;
    c_file.flush()?;
    Ok(())
}

fn get_vector_of_u64(bytes: &Vec<u8>, n64: usize ) -> Vec<String> {
    let mut bytes1 = vec![];
    assert!(bytes.len() <= n64*8);
    for _i in 0..n64*8-bytes.len() {
        bytes1.push(0 as u8);
    }
    bytes1.append(&mut bytes.clone());
    // println!("{:?}\n", bytes1);
    let mut v = vec![];
    let n = bytes1.len()/8;
    for i in (0..n).rev() {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes1[i*8..i*8+8]);
        v.push(format!("0x{:x}",u64::from_be_bytes(buf)));
    }
    v
}

pub fn generate_fr_cpp_file(c_folder: &PathBuf, prime: &String,  producer: &CProducer) -> std::io::Result<()> {
    if prime != "goldilocks" {
        use std::io::BufWriter;
        let mut file_path = c_folder.clone();
        file_path.push("fr");
        file_path.set_extension("cpp");
        let file_name = file_path.to_str().unwrap();
        let mut c_file = BufWriter::new(File::create(file_name).unwrap());
        let mut code = "".to_string();
        if producer.no_asm {
            use circom_algebra::num_traits::ToPrimitive;
            //use circom_algebra::modular_arithmetic;
            use circom_algebra::num_bigint::{ModInverse};
            let p = producer.get_prime().parse::<BigInt>().unwrap();
            let pbits = p.bits();
            let half = p.clone() / BigInt::from(2);
            let inv = p.clone().mod_inverse(&(BigInt::from(1) << 64)).unwrap();
            let np = ((BigInt::from(1) << 64) - inv).to_u64().unwrap();
            let n64 = (pbits + 63) / 64;
            let nbits = n64*64;
            let lbo_mask = ((BigInt::parse_bytes(b"10000000000000000", 16).unwrap()) >> (nbits - pbits)) - BigInt::from(1);
            //let r = (BigInt::from(1) << nbits) % p.clone();
            let r2 = (BigInt::from(1) << (nbits*2)) % p.clone();
            let r3 = (BigInt::from(1) << (nbits*3)) % p.clone();
            use handlebars::handlebars_helper;
            let fr_cpp_template: &str = include_str!("generic/fr.cpp");
            handlebars_helper!(inc: |x : u32| x + 1);
            handlebars_helper!(dec: |x : u32| x - 1);
            handlebars_helper!(elements: |v : Vec<String>| v.join(","));
            let mut template = handlebars::Handlebars::new();
            template.register_helper("inc", Box::new(inc));
            template.register_helper("dec", Box::new(dec));
            template.register_helper("elements", Box::new(elements));
            //let fr_q_list = p.to_u64_digits().1;
            //println!("{}",fr_q_list.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(","));
 // cannotOptimize = (p >> ((n64-1)*64) ) > (((1<<64)-1) >> 1)-1
           code = template
                .render_template(
                fr_cpp_template,
                &json!({
                    "cannotOptimize": (p.clone() >> ((n64 - 1)*64) ) > (((BigInt::from(1) << 64 ) - BigInt::from(1)) >> 1)-BigInt::from(1),
                    "list0n64": (0..n64).collect::<Vec<usize>>(),
                    "list0n64_1": (0..n64-1).collect::<Vec<usize>>(),
                    "list1n64": (1..n64).collect::<Vec<usize>>(),
                    "n64": n64,
                    "qbits": pbits,
                    "lboMask": format!("0x{:x}",lbo_mask),
                    "fr_np": format!("0x{:x}",np),
                    "fr_q_list":get_vector_of_u64(&p.to_bytes_be().1,n64),
                    "fr_r2_list": get_vector_of_u64(&r2.to_bytes_be().1,n64),
                    "fr_r3_list": get_vector_of_u64(&r3.to_bytes_be().1,n64),
                    "half_list": get_vector_of_u64(&half.to_bytes_be().1,n64),
                }),
            )
            .expect("must render");
            c_file.write_all(code.as_bytes())?;
            c_file.flush()?;
        } else {
            let file = match prime.as_ref(){
                "bn128" => include_str!("bn128/fr.cpp"),
                "bls12381" => include_str!("bls12381/fr.cpp"),
                //"goldilocks" => include_str!("goldilocks/fr.cpp"),
                "grumpkin" => include_str!("grumpkin/fr.cpp"),
                "pallas" => include_str!("pallas/fr.cpp"),
                "vesta" => include_str!("vesta/fr.cpp"),
                "secq256r1" => include_str!("secq256r1/fr.cpp"),
                "bls12377" => include_str!("bls12377/fr.cpp"),
                _ => unreachable!(),
            };
            for line in file.lines() {
                code = format!("{}{}\n", code, line);
            }
            c_file.write_all(code.as_bytes())?;
            c_file.flush()?;
        }
    }
    Ok(())
}

pub fn generate_calcwit_cpp_file(c_folder: &PathBuf, producer: &CProducer) -> std::io::Result<()> {
    use std::io::BufWriter;
    let mut file_path = c_folder.clone();
    file_path.push("calcwit");
    file_path.set_extension("cpp");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    let mut code = "".to_string();
    let file = if producer.prime_str != "goldilocks" { include_str!("common/calcwit.cpp")
    } else { include_str!("common64/calcwit.cpp")};
    for line in file.lines() {
        code = format!("{}{}\n", code, line);
    }
    c_file.write_all(code.as_bytes())?;
    c_file.flush()?;
    Ok(())
}

pub fn generate_fr_asm_file(c_folder: &PathBuf, prime: &String, producer: &CProducer) -> std::io::Result<()> {
    if prime != "goldilocks" && !producer.no_asm {
        use std::io::BufWriter;
        let mut file_path = c_folder.clone();
        file_path.push("fr");
        file_path.set_extension("asm");
        let file_name = file_path.to_str().unwrap();
        let mut c_file = BufWriter::new(File::create(file_name).unwrap());
        let mut code = "".to_string();
        let file = match prime.as_ref(){
            "bn128" => include_str!("bn128/fr.asm"),
            "bls12381" => include_str!("bls12381/fr.asm"),
            //"goldilocks" => include_str!("goldilocks/fr.asm"),
            "grumpkin" => include_str!("grumpkin/fr.asm"),
            "pallas" => include_str!("pallas/fr.asm"),
            "vesta" => include_str!("vesta/fr.asm"),
            "secq256r1" => include_str!("secq256r1/fr.asm"),
            "bls12377" => include_str!("bls12377/fr.asm"),
            
            _ => unreachable!(),
        };    
        for line in file.lines() {
            code = format!("{}{}\n", code, line);
        }
        c_file.write_all(code.as_bytes())?;
        c_file.flush()?;
    }
    Ok(())
}

pub fn generate_make_file(
    c_folder: &PathBuf,
    run_name: &str,
    producer: &CProducer,
) -> std::io::Result<()> {
    use std::io::BufWriter;
    let makefile_template: &str = if producer.prime_str != "goldilocks" && !producer.no_asm { include_str!("common/makefile")
    } else {
        if producer.prime_str == "goldilocks" {include_str!("common64/makefile")
        } else {include_str!("generic/makefile")}
    };
    let template = handlebars::Handlebars::new();
    let code = template
        .render_template(
            makefile_template,
            &json!({
                "run_name": run_name,
                "has_parallelism": producer.has_parallelism,
            }),
        )
        .expect("must render");

    let mut file_path = c_folder.clone();
    file_path.push("Makefile");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    c_file.write_all(code.as_bytes())?;
    c_file.flush()?;
    Ok(())
}

pub fn generate_json2bin64(c_folder: &PathBuf, producer: &CProducer) -> std::io::Result<()> {
    use std::io::BufWriter;
    let mut file_path = c_folder.clone();
    file_path.push("json2bin64");
    file_path.set_extension("cpp");
    let file_name = file_path.to_str().unwrap();
    let mut c_file = BufWriter::new(File::create(file_name).unwrap());
    let mut code = "".to_string();
    assert!(producer.prime_str == "goldilocks");
    let file = include_str!("common64/json2bin64.cpp");
    for line in file.lines() {
        code = format!("{}{}\n", code, line);
    }
    c_file.write_all(code.as_bytes())?;
    c_file.flush()?;
    Ok(())
}


pub fn generate_c_file(name: String, producer: &CProducer) -> std::io::Result<()> {
    let full_name = name + ".cpp";
    let mut cfile = File::create(full_name)?;
    let mut code = vec![];
    let len = producer.get_input_hash_map_entry_size();
    code.push("#include <stdio.h>".to_string());
    code.push("#include <iostream>".to_string());
    code.push("#include <assert.h>".to_string());
    code.push("#include \"circom.hpp\"".to_string());
    code.push("#include \"calcwit.hpp\"".to_string());

    let mut run_defs = collect_template_headers(producer.get_template_instance_list());
    code.append(&mut run_defs);

    let (func_list_no_parallel, func_list_parallel) = generate_function_list(producer, producer.get_template_instance_list());

    code.push(format!(
        "Circom_TemplateFunction _functionTable[{}] = {{ {} }};",
        producer.get_number_of_template_instances(),
        func_list_no_parallel,
    ));

    code.push(format!(
        "Circom_TemplateFunction _functionTableParallel[{}] = {{ {} }};",
        producer.get_number_of_template_instances(),
        func_list_parallel,
    ));

    code.push(format!("uint get_size_of_input_hashmap() {{return {};}}\n", len));
    code.push(format!(
        "uint get_size_of_witness() {{return {};}}\n",
        producer.get_witness_to_signal_list().len()
    ));
    code.push(format!(
        "uint get_size_of_constants() {{return {};}}\n",
        producer.get_field_constant_list().len()
    ));
    code.push(format!("uint get_size_of_io_map() {{return {};}}\n", producer.get_io_map().len()));
    code.push(format!("uint get_size_of_bus_field_map() {{return {};}}\n", producer.get_busid_field_info().len()));

    // let mut ml_def = generate_message_list_def(producer, producer.get_message_list());
    // code.append(&mut ml_def);
    for l in code {
        cfile.write_all(l.as_bytes())?;
    }
    cfile.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    //    use std::io::{BufWriter,BufReader,BufRead};
    use std::path::Path;
    //    use std::fs::File;
    use super::*;
    const LOCATION: &'static str = "../target/code_generator_test";

    fn create_producer() -> CProducer {
        CProducer::default()
    }

    #[test]
    fn produce_dat() {
        if !Path::new(LOCATION).is_dir() {
            std::fs::create_dir(LOCATION).unwrap();
        }
        let path = format!("{}/code", LOCATION);
        let producer = create_producer();
        let mut dat_file = File::create(path + ".dat").unwrap();
        let _rd = generate_dat_file(&mut dat_file, &producer);
        assert!(true);
        let pathc = format!("{}/code", LOCATION);
        let _rc = generate_c_file(pathc, &producer);
        assert!(true);
    }
}
