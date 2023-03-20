pub mod wasm_code_generator;

use crate::components::*;

type WasmInstruction = String;

pub struct WASMProducer {
    pub main_signal_offset: usize,
    pub fr_memory_size: usize, // depending of the prime; missing in build.rs
    pub size_32_bit: usize,
    pub size_32_shift: usize,
    pub number_of_main_outputs: usize,
    pub number_of_main_inputs: usize,
    pub signals_in_witness: usize,
    pub total_number_of_signals: usize,
    pub size_of_component_tree: usize,
    pub number_of_components: usize,
    pub main_header: String,
    pub prime: String,
    pub prime_str: String,
    pub main_input_list: InputList,
    pub witness_to_signal_list: SignalList,
    pub io_map: TemplateInstanceIOMap,
    pub template_instance_list: TemplateList,
    pub message_list: MessageList,
    pub field_tracking: Vec<String>,
    pub wat_flag: bool,
    pub major_version: usize,
    pub minor_version: usize,
    pub patch_version: usize,
    stack_free_pos: usize,
    local_info_size_u32: usize,
    size_of_message_buffer_in_bytes: usize,
    size_of_message_in_bytes: usize,
    offset_tag: String,
    signal_offset_tag: String,
    signal_start_tag: String,
    sub_cmp_tag: String,
    sub_cmp_load_tag: String,
    io_info_tag: String,
    result_address_tag: String,
    result_size_tag: String,
    cstack_tag: String,
    lvar_tag: String,
    call_lvar_tag: String,
    expaux_tag: String,
    temp_tag: String,
    aux_0_tag: String,
    aux_1_tag: String,
    aux_2_tag: String,
    counter_tag: String,
    store_aux_1_tag: String,
    store_aux_2_tag: String,
    copy_counter_tag: String,
    create_loop_sub_cmp_tag: String,
    create_loop_offset_tag: String,
    create_loop_counter_tag: String,
    merror_tag: String,
    string_table:  Vec<String>,
}

impl Default for WASMProducer {
    fn default() -> Self {
        //let mut my_map = TemplateInstanceIOMap::new();
        //my_map.insert(0,[(0,0),(1,2),(2,4)].to_vec());
        //my_map.insert(1,[(0,0),(1,1)].to_vec());
        //my_map.insert(2,[(0,0),(1,1),(2,3)].to_vec());
        WASMProducer {
            main_header: "Main_0".to_string(),
            main_signal_offset: 1,
            prime: "21888242871839275222246405745257275088548364400416034343698204186575808495617"
                .to_string(),
            prime_str: "bn128".to_string(),
            fr_memory_size: 1948,
            size_32_bit: 8,
            size_32_shift: 5,
            number_of_main_outputs: 0, //2,
            number_of_main_inputs: 0,  // 4,
            main_input_list: [("in1".to_string(), 1, 1), ("in2".to_string(), 2, 1)].to_vec(), //("inpair".to_string(),2),
            signals_in_witness: 0,                                                      //20,
            witness_to_signal_list: [].to_vec(), //[0,1,2,3,4,5,6,12,16,19,24,27,33,42,46,50,51,65,78,79].to_vec(),
            message_list: [].to_vec(), //["Main".to_string(),"Hola Herme".to_string(),"Hola Albert".to_string()].to_vec(),
            total_number_of_signals: 0, //80,
            number_of_components: 1,   //3,
            size_of_component_tree: 3, //10,
            io_map: TemplateInstanceIOMap::new(), //my_map,
            template_instance_list: [].to_vec(),
            field_tracking: [].to_vec(),
            wat_flag: true,
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
            stack_free_pos: 0,
            local_info_size_u32: 0, // in the future we can add some info like pointer to run father or text father
            size_of_message_buffer_in_bytes: 256,
            size_of_message_in_bytes: 240,
            offset_tag: "$offset".to_string(),
            signal_offset_tag: "$signaloffset".to_string(),
            signal_start_tag: "$signalstart".to_string(),
            sub_cmp_tag: "$subcmp".to_string(),
            sub_cmp_load_tag: "$subcmpload".to_string(),
            io_info_tag: "$ioinfo".to_string(),
            result_address_tag: "$resultaddress".to_string(),
            result_size_tag: "$resultsize".to_string(),
            cstack_tag: "$cstack".to_string(),
            lvar_tag: "$lvar".to_string(),
            call_lvar_tag: "$calllvar".to_string(),
            expaux_tag: "$expaux".to_string(),
            temp_tag: "$temp".to_string(),
            aux_0_tag: "$aux0".to_string(),
            aux_1_tag: "$aux1".to_string(),
            aux_2_tag: "$aux2".to_string(),
            counter_tag: "$counter".to_string(),
            store_aux_1_tag: "$storeaux1".to_string(),
            store_aux_2_tag: "$storeaux2".to_string(),
            copy_counter_tag: "$copycounter".to_string(),
            create_loop_sub_cmp_tag: "$createloopsubcmp".to_string(),
            create_loop_offset_tag: "$createloopoffset".to_string(),
            create_loop_counter_tag: "$createloopcounter".to_string(),
	        merror_tag: "$merror".to_string(),
            string_table: Vec::new(),
        }
    }
}

impl WASMProducer {
    /*
        pub fn set_constant(&self,value: &str) -> WasmInstruction {
            set_constant(value)
        }
        pub fn set_constant_64(&self,value: &str) -> WasmInstruction {
            set_constant_64(value)
        }
        pub fn get_local(&self,value: &str) -> WasmInstruction {
            get_local(value)
        }
        pub fn set_local(&self,value: &str) -> WasmInstruction {
            set_local(value)
        }
        pub fn add32(&self,value_0: &str,value_1: &str) -> WasmInstruction {
            let instructions = vec![
                value_0.to_string(),
                value_1.to_string(),
                add32()];
            merge_code(instructions)
        }
        pub fn mul32(&self,value_0: &str,value_1: &str) -> WasmInstruction {
            let instructions = vec![
                value_0.to_string(),
                value_1.to_string(),
                mul32()];
            merge_code(instructions)
        }
    */
    pub fn get_version(&self) -> usize {
        self.major_version
    }
    pub fn get_minor_version(&self) -> usize {
        self.minor_version
    }
    pub fn get_patch_version(&self) -> usize {
        self.patch_version
    }
    pub fn get_main_header(&self) -> &str {
        &self.main_header
    }
    pub fn get_main_signal_offset(&self) -> usize {
        self.main_signal_offset
    }
    pub fn get_prime(&self) -> &str {
        &self.prime
    }
    pub fn get_number_of_main_outputs(&self) -> usize {
        self.number_of_main_outputs
    }
    pub fn get_number_of_main_inputs(&self) -> usize {
        self.number_of_main_inputs
    }
    pub fn get_main_input_list(&self) -> &InputList {
        &self.main_input_list
    }
    pub fn get_number_of_witness(&self) -> usize {
        self.signals_in_witness
    }
    pub fn get_witness_to_signal_list(&self) -> &SignalList {
        &self.witness_to_signal_list
    }
    pub fn get_total_number_of_signals(&self) -> usize {
        self.total_number_of_signals
    }
    pub fn get_size_of_component_tree(&self) -> usize {
        self.size_of_component_tree
    }
    pub fn get_size_of_message_buffer_in_bytes(&self) -> usize {
        self.size_of_message_buffer_in_bytes
    }
    pub fn get_size_of_message_in_bytes(&self) -> usize {
        self.size_of_message_in_bytes
    }
    pub fn get_number_of_components(&self) -> usize {
        self.number_of_components
    }
    pub fn get_size_32_bit(&self) -> usize {
        self.size_32_bit
    }
    pub fn get_size_32_shift(&self) -> usize {
        self.size_32_shift
    }
    pub fn get_fr_memory_size(&self) -> usize {
        self.fr_memory_size
    }
    pub fn get_stack_free_pos(&self) -> usize {
        self.stack_free_pos
    }
    pub fn get_local_info_size_u32(&self) -> usize {
        self.local_info_size_u32
    }
    pub fn get_io_map(&self) -> &TemplateInstanceIOMap {
        &self.io_map
    }
    pub fn get_template_instance_list(&self) -> &TemplateList {
        &self.template_instance_list
    }
    pub fn get_number_of_template_instances(&self) -> usize {
        self.template_instance_list.len()
    }
    pub fn get_number_of_io_signals(&self) -> usize {
        let mut n = 0;
        for (_c, v) in &self.io_map {
            n += v.len();
        }
        n
    }
    pub fn get_io_signals_info_size(&self) -> usize {
        let mut n = 0;
        for (_c, v) in &self.io_map {
            for s in v {
                // since we take offset and all lenghts but last one
                if s.lengths.len() == 0 {
                    n += 1;
                } else {
                    n += s.lengths.len();
                }
            }
        }
        n * 4
    }
    pub fn get_message_list(&self) -> &MessageList {
        &self.message_list
    }
    pub fn get_field_constant_list(&self) -> &Vec<String> {
        &self.field_tracking
    }
    pub fn get_raw_prime_start(&self) -> usize {
        4 + self.fr_memory_size
    }
    pub fn get_shared_rw_memory_start(&self) -> usize {
        (4 * self.size_32_bit) + 8 + self.get_raw_prime_start()
    }
    pub fn get_input_signals_hashmap_start(&self) -> usize {
        (4 * self.size_32_bit) + 8 + self.get_shared_rw_memory_start()
    }
    pub fn get_remaining_input_signal_counter(&self) -> usize {
        self.get_input_signals_hashmap_start() + 4096 // 256*(8(h)+4(pos)+4(size))
    }
    pub fn get_input_signal_set_map_start(&self) -> usize {
        self.get_remaining_input_signal_counter() + 4
    }
    pub fn get_witness_signal_id_list_start(&self) -> usize {
        self.get_input_signal_set_map_start() + (4 * self.get_number_of_main_inputs())
    }
    pub fn get_signal_free_pos(&self) -> usize {
        self.get_witness_signal_id_list_start() + (4 * self.get_number_of_witness())
    }
    pub fn get_signal_memory_start(&self) -> usize {
        self.get_signal_free_pos() + 4
    }
    pub fn get_component_free_pos(&self) -> usize {
        let a = 2 + self.get_size_32_bit();
        let b = 4 * self.get_total_number_of_signals();
        let c = self.get_signal_memory_start();
        a * b + c
    }
    pub fn get_component_tree_start(&self) -> usize {
        self.get_component_free_pos() + 4
    }
    pub fn get_signal_start_address_in_component(&self) -> usize {
        4
    } //template id
    pub fn get_input_counter_address_in_component(&self) -> usize {
        8
    } //template id + signal address
    pub fn get_sub_component_start_in_component(&self) -> usize {
        12
    } //template id + signal address + input counter
    pub fn get_template_instance_to_io_signal_start(&self) -> usize {
        let a = self.get_component_tree_start();
        let b = self.get_size_of_component_tree() * 4;
        a + b
    }
    pub fn get_io_signals_to_info_start(&self) -> usize {
        let a = self.get_template_instance_to_io_signal_start();
        let b = self.get_number_of_template_instances() * 4;
        a + b
    }
    pub fn get_io_signals_info_start(&self) -> usize {
        let a = self.get_io_signals_to_info_start();
        let b = self.get_number_of_io_signals() * 4;
        a + b
    }
    pub fn get_message_buffer_counter_position(&self) -> usize {
        self.get_io_signals_info_start() + self.get_io_signals_info_size()
    }
    pub fn get_message_buffer_start(&self) -> usize {
        self.get_message_buffer_counter_position() + 4
    }
    pub fn get_message_list_start(&self) -> usize {
        self.get_message_buffer_start() + self.size_of_message_buffer_in_bytes
    }
    pub fn get_string_list_start(&self) -> usize {
        self.get_message_list_start() + self.size_of_message_in_bytes * self.message_list.len()
    }

    pub fn get_constant_numbers_start(&self) -> usize {
        self.get_string_list_start() + self.size_of_message_in_bytes * self.string_table.len()
    }
    
    pub fn get_var_stack_memory_start(&self) -> usize {
        self.get_constant_numbers_start() + (self.size_32_bit + 2) * 4 * self.field_tracking.len()
    }
    pub fn get_size_32_bits_in_memory(&self) -> usize {
        self.size_32_bit + 2
    }
    pub fn get_offset_tag(&self) -> &str {
        &self.offset_tag
    }
    pub fn get_signal_offset_tag(&self) -> &str {
        &self.signal_offset_tag
    }
    pub fn get_signal_start_tag(&self) -> &str {
        &self.signal_start_tag
    }
    pub fn get_sub_cmp_tag(&self) -> &str {
        &self.sub_cmp_tag
    }
    pub fn get_sub_cmp_load_tag(&self) -> &str {
        &self.sub_cmp_load_tag
    }
    pub fn get_io_info_tag(&self) -> &str {
        &self.io_info_tag
    }
    pub fn get_result_address_tag(&self) -> &str {
        &self.result_address_tag
    }
    pub fn get_result_size_tag(&self) -> &str {
        &self.result_size_tag
    }
    pub fn get_cstack_tag(&self) -> &str {
        &self.cstack_tag
    }
    pub fn get_lvar_tag(&self) -> &str {
        &self.lvar_tag
    }
    pub fn get_call_lvar_tag(&self) -> &str {
        &self.call_lvar_tag
    }
    pub fn get_expaux_tag(&self) -> &str {
        &self.expaux_tag
    }
    pub fn get_temp_tag(&self) -> &str {
        &self.temp_tag
    }
    pub fn get_aux_0_tag(&self) -> &str {
        &self.aux_0_tag
    }
    pub fn get_aux_1_tag(&self) -> &str {
        &self.aux_1_tag
    }
    pub fn get_aux_2_tag(&self) -> &str {
        &self.aux_2_tag
    }
    pub fn get_counter_tag(&self) -> &str {
        &self.counter_tag
    }
    pub fn get_store_aux_1_tag(&self) -> &str {
        &self.store_aux_1_tag
    }
    pub fn get_store_aux_2_tag(&self) -> &str {
        &self.store_aux_2_tag
    }
    pub fn get_copy_counter_tag(&self) -> &str {
        &self.copy_counter_tag
    }
    pub fn get_create_loop_sub_cmp_tag(&self) -> &str {
        &self.create_loop_sub_cmp_tag
    }
    pub fn get_create_loop_offset_tag(&self) -> &str {
        &self.create_loop_offset_tag
    }
    pub fn get_create_loop_counter_tag(&self) -> &str {
        &self.create_loop_counter_tag
    }
    pub fn get_merror_tag(&self) -> &str {
	&self.merror_tag
    }
    pub fn needs_comments(&self) -> bool{
        self.wat_flag
    }

    pub fn get_string_table(&self) -> &Vec<String> {
        &self.string_table
    }

    pub fn set_string_table(&mut self, string_table: Vec<String>) {
        self.string_table = string_table;
    }
}
