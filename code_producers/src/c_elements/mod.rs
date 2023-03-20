pub mod c_code_generator;

pub use crate::components::*;

pub type CInstruction = String;
pub struct CProducer {
    pub main_header: String,
    pub main_is_parallel: bool,
    //pub fr_memory_size: usize, // depending of the prime; missing in build.rs
    pub has_parallelism: bool,
    pub number_of_main_outputs: usize,
    pub main_signal_offset: usize,
    pub number_of_main_inputs: usize,
    pub signals_in_witness: usize,
    pub total_number_of_signals: usize,
    pub number_of_components: usize,
    pub size_of_component_tree: usize,
    pub size_32_bit: usize,
    pub size_32_shift: usize,
    pub prime: String,
    pub prime_str: String,
    pub main_input_list: InputList,
    pub witness_to_signal_list: SignalList,
    pub io_map: TemplateInstanceIOMap,
    pub template_instance_list: TemplateListParallel,
    pub message_list: MessageList,
    pub field_tracking: Vec<String>,
    pub major_version: usize,
    pub minor_version: usize,
    pub patch_version: usize,
    name_tag: String,
    string_table: Vec<String>,
}

impl Default for CProducer {
    fn default() -> Self {
        let mut my_map = TemplateInstanceIOMap::new();
        my_map.insert(
            0,
            vec![
                IODef { code: 0, offset: 0, lengths: [2, 3].to_vec() },
                IODef { code: 1, offset: 6, lengths: [].to_vec() },
                IODef { code: 2, offset: 7, lengths: [2].to_vec() },
            ],
        );
        my_map.insert(
            1,
            vec![
                IODef { code: 0, offset: 0, lengths: [3].to_vec() },
                IODef { code: 1, offset: 3, lengths: [4, 8, 6].to_vec() },
            ],
        );
        my_map.insert(
            2,
            vec![
                IODef { code: 0, offset: 0, lengths: [].to_vec() },
                IODef { code: 1, offset: 1, lengths: [4].to_vec() },
                IODef { code: 2, offset: 5, lengths: [2, 6].to_vec() },
            ],
        );
        CProducer {
            main_header: "Main_0".to_string(),
            main_is_parallel: false,
            has_parallelism: false,
            main_signal_offset: 1,
            prime: "21888242871839275222246405745257275088548364400416034343698204186575808495617"
                .to_string(),
            prime_str: "bn128".to_string(),
            number_of_main_outputs: 1,
            number_of_main_inputs: 2,
            main_input_list: [("in1".to_string(), 2, 1), ("in2".to_string(), 3, 1)].to_vec(), //[].to_vec(),
            signals_in_witness: 20,
            witness_to_signal_list: [
                0, 1, 2, 3, 4, 5, 6, 12, 16, 19, 24, 27, 33, 42, 46, 50, 51, 65, 78, 79,
            ]
            .to_vec(), //[].to_vec(),
            message_list: ["Main".to_string(), "Hola Herme".to_string(), "Hola Albert".to_string()]
                .to_vec(), //[].to_vec(),
            field_tracking: [
                "1884242871839275222246405745257275088548364400416034343698204186575808495617"
                    .to_string(),
                "21888242871839275222246405745257275088548364400416034343698204186575808495615"
                    .to_string(),
                "31424553576487322".to_string(),
            ]
            .to_vec(),
            total_number_of_signals: 80,
            number_of_components: 4,
            size_of_component_tree: 3,
            size_32_bit: 8,
            size_32_shift: 5,
            io_map: my_map, //TemplateInstanceIOMap::new(),
            template_instance_list: Vec::new(),
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
            name_tag: "name".to_string(),
            string_table: Vec::new(),
        }
    }
}

impl CProducer {
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
    pub fn get_main_is_parallel(&self) -> bool {
        self.main_is_parallel
    }
    pub fn get_has_parallelism(&self) -> bool {
        self.has_parallelism
    }
    pub fn get_main_signal_offset(&self) -> usize {
        self.main_signal_offset
    }
    pub fn get_prime(&self) -> &str {
        &self.prime
    }
    pub fn get_number_of_main_outputs(&self) -> usize {
        self.number_of_main_outputs+1
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
    pub fn get_number_of_components(&self) -> usize {
        self.number_of_components
    }
    pub fn get_io_map(&self) -> &TemplateInstanceIOMap {
        &self.io_map
    }
    pub fn get_template_instance_list(&self) -> &TemplateListParallel {
        &self.template_instance_list
    }
    pub fn get_number_of_template_instances(&self) -> usize {
        self.template_instance_list.len()
    }
    pub fn get_message_list(&self) -> &MessageList {
        &self.message_list
    }
    pub fn get_field_constant_list(&self) -> &Vec<String> {
        &self.field_tracking
    }
    pub fn get_name_tag(&self) -> &str {
        &self.name_tag
    }
    pub fn get_size_32_bit(&self) -> usize {
        self.size_32_bit
    }

    pub fn get_string_table(&self) -> &Vec<String> {
        &self.string_table
    }

    pub fn set_string_table(&mut self, string_table: Vec<String>) {
        self.string_table = string_table;
    }
}
