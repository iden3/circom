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
    //New for buses
    pub num_of_bus_instances: usize,  //total number of different bus instances
    //pub size_of_bus_fields: usize,  //total number of fields in all differen bus intances
    pub busid_field_info: FieldMap, //for every busId (0..num-1) provides de offset, size, dimensions and busId of each field (0..n-1) in it
}

impl Default for CProducer {
    fn default() -> Self {
        let mut my_map = TemplateInstanceIOMap::new();
        my_map.insert(
            0,
            vec![
                IODef { code: 0, offset: 0, lengths: [2, 3].to_vec(), size: 6, bus_id:None },
                IODef { code: 1, offset: 6, lengths: [].to_vec(), size: 1, bus_id:None },
                IODef { code: 2, offset: 7, lengths: [2].to_vec(), size: 2, bus_id:None },
            ],
        );
        my_map.insert(
            1,
            vec![
                IODef { code: 0, offset: 0, lengths: [3].to_vec(), size: 3, bus_id:None },
                IODef { code: 1, offset: 3, lengths: [4, 8, 6].to_vec(), size: 192, bus_id:None },
            ],
        );
        my_map.insert(
            2,
            vec![
                IODef { code: 0, offset: 0, lengths: [].to_vec(), size: 1, bus_id:None },
                IODef { code: 1, offset: 1, lengths: [4].to_vec(), size: 4, bus_id:None },
                IODef { code: 2, offset: 5, lengths: [2, 6].to_vec(), size: 12, bus_id:None },
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
            main_input_list: [
                InputInfo{
                    name:"in1".to_string(), 
                    size:1, 
		    dimensions: Vec::new(),
                    start: 2, 
                    bus_id: None
                },
                InputInfo{
                    name:"in2".to_string(), 
                    size:1,
		    dimensions: Vec::new(),
                    start: 3, 
                    bus_id: None
                },
            ].to_vec(),
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
            //New for buses
	        num_of_bus_instances: 0,
//	        size_of_bus_fields: 0,
	        busid_field_info: Vec::new(), 
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
<<<<<<< HEAD
    fn get_accesses(&self, pos: usize, dims: &Vec<usize>) -> Vec<(String,usize)> {
	if pos >= dims.len() {
	    vec![("".to_string(),0)]
	} else {
	    let mut res: Vec<(String,usize)> = vec![];
	    let res1 = self.get_accesses(pos+1, dims);
	    let mut elems:usize = 1;
	    let mut epos = pos + 1;
	    while epos < dims.len() {
		elems *= dims[epos];
		epos += 1;
	    }
	    let mut jump = 0;
	    for i in 0..dims[pos] {
		for j in 0..res1.len() {
		    let (a,s) = &res1[j];
		    res.push((format!("[{}]{}",i,a),jump+s));
		}
		jump += elems;
	    }
	    res
	}
    }
    fn get_qualified_names (&self, busid: usize, start: usize, prefix: String) -> InputList {
	let mut buslist = vec![];
	//println!("BusId: {}", busid);
	for io in &self.busid_field_info[busid] {
	    let name = format!("{}.{}",prefix.clone(),io.name);
	    let new_start = start + io.offset;
	    //print!("name: {}, start: {}", name, new_start);
	    if let Some(value) = io.bus_id {
		let accesses = self.get_accesses(0,&io.dimensions);
		//println!("accesses list: {:?}", accesses);
		for (a,s) in &accesses {
		    let prefix = format!("{}{}",name.clone(),a);
		    let mut ios = self.get_qualified_names (value,new_start+s*io.size,prefix);
		    buslist.append(&mut ios);
		}
	    }
	    else {
		//println!("");
		let mut total_size = io.size;
		for i in &io.dimensions {
		    total_size *= i;
		}
		let ioinfo = {
		    InputInfo{
			name: name,
			dimensions: io.dimensions.clone(),
			size: total_size,
			start: new_start,
			bus_id: None
		    } };
		buslist.push(ioinfo);
	    }
	}
	buslist
    }
    
    pub fn get_main_input_list_with_qualifiers(&self) -> InputList {
	let mut iolist = vec![];
        for io in &self.main_input_list {
	    if let Some(value) = io.bus_id {
		let mut elems:usize = 1;
		for i in &io.dimensions {
		    elems *= i;
		}
		let size:usize = io.size/elems;
		let accesses = self.get_accesses(0,&io.dimensions);
		for (a,s) in &accesses {
		    let prefix = format!("{}{}",io.name.clone(),a);
		    let mut ios = self.get_qualified_names (value,io.start+s*size,prefix);
		    iolist.append(&mut ios);
		}
	    }
	    else {
		iolist.push(io.clone());
	    }
	}
	iolist
    }
    
=======
    pub fn get_input_hash_map_entry_size(&self) -> usize {
        std::cmp::max(usize::pow(2,(self.main_input_list.len() as f32).log2().ceil() as u32),256)
    }    
>>>>>>> 9f3da35a8ac3107190f8c85c8cf3ea1a0f8780a4
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
    
    //New for buses
    pub fn get_number_of_bus_instances(&self) -> usize {
        self.num_of_bus_instances
    }
    
    pub fn get_busid_field_info(&self) -> &FieldMap {
        &self.busid_field_info
    }
    // end
}
