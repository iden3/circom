use crate::intermediate_representation::InstructionList;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use std::default::Default;
use code_producers::llvm_elements::{AnyType, AnyValue, build_fn_name, LLVMInstruction, LLVMIRProducer, run_fn_name, to_basic_type_enum};
use code_producers::llvm_elements::functions::{create_bb, create_function};
use code_producers::llvm_elements::instructions::{create_alloca, create_br, create_gep, create_return_void, create_store, pointer_cast};
use code_producers::llvm_elements::template::{create_template_struct, TemplateLLVMIRProducer};
use code_producers::llvm_elements::types::{bigint_type, i32_type, void_type};
use code_producers::llvm_elements::values::{create_literal_u32, zero};
use code_producers::wasm_elements::*;

pub type TemplateID = usize;
pub type TemplateCode = Box<TemplateCodeInfo>;

#[derive(Default, Clone)]
pub struct TemplateCodeInfo {
    pub id: TemplateID,
    pub header: String,
    pub name: String,
    pub is_parallel: bool,
    pub is_parallel_component: bool,
    pub is_not_parallel_component: bool,
    pub has_parallel_sub_cmp: bool,
    pub number_of_inputs: usize,
    pub number_of_outputs: usize, 
    pub number_of_intermediates: usize, // Not used now
    pub body: InstructionList,
    pub var_stack_depth: usize,
    pub expression_stack_depth: usize,
    pub signal_stack_depth: usize, // Not used now
    pub number_of_components: usize,
}
impl ToString for TemplateCodeInfo {
    fn to_string(&self) -> String {
        let mut body = "".to_string();
        for i in &self.body {
            body = format!("{}{}\n", body, i.to_string());
        }
        format!("TEMPLATE({})(\n{})", self.header, body)
    }
}



impl WriteLLVMIR for TemplateCodeInfo {
    fn produce_llvm_ir<'ctx, 'prod>(&self, producer: &'prod dyn LLVMIRProducer<'ctx>) -> Option<LLVMInstruction<'ctx>> {
        let void = void_type(producer);
        let n_signals = self.number_of_inputs + self.number_of_outputs;
        let template_struct = create_template_struct(producer, n_signals);

        ///////////////////////////////////////////////////////////////////////////////////////////////
        // Build function
        let bigint_ptr = bigint_type(producer).array_type(0);
        // Set the type of the component memory: signals array + signals counter
        let component_memory = producer.context().struct_type(&[
            to_basic_type_enum(bigint_ptr.ptr_type(Default::default())),
            to_basic_type_enum(i32_type(producer))
        ], false);
        let build_function = create_function(producer, build_fn_name(self.header.clone()).as_str(), void_type(producer).fn_type(&[component_memory.ptr_type(Default::default()).into()], false));
        let main = create_bb(producer,build_function, "main");
        producer.set_current_bb(main);

        let cmp_mem = build_function.get_nth_param(0).unwrap();
        // Allocate memory for the component
        let alloca = create_alloca(producer, bigint_type(producer).array_type(n_signals as u32).as_any_type_enum(), "").into_pointer_value();
        // Get the counter as a pointer
        let counter_ptr = create_gep(producer, cmp_mem.into_pointer_value(), &[zero(producer), create_literal_u32(producer, 1)]).into_pointer_value();
        // Create a literal value of the initial value of the counter
        let initial_counter_value = create_literal_u32(producer, self.number_of_inputs as u64);
        // Write that value in the counter
        create_store(producer, counter_ptr, initial_counter_value.as_any_value_enum());
        let signals_mem = create_gep(producer, cmp_mem.into_pointer_value(), &[zero(producer), zero(producer)]).into_pointer_value();
        //let signals = create_load(producer, alloca);
        let ptr = pointer_cast(producer, alloca, bigint_ptr.ptr_type(Default::default()));
        create_store(producer, signals_mem, ptr.into());
        // Return that memory
        create_return_void(producer);

        ///////////////////////////////////////////////////////////////////////////////////////////////
        // Run function

        // Run function prelude
        let run_function = create_function(
            producer,
            run_fn_name(self.header.clone()).as_str(),
            void.fn_type(&[bigint_type(producer).array_type(0).ptr_type(Default::default()).into()], false)
        );
        let prelude = create_bb(producer, run_function, "prelude");
        producer.set_current_bb(prelude);
        let template_producer = TemplateLLVMIRProducer::new(
            producer,
            self.var_stack_depth,
            self.number_of_components,
            run_function,
            template_struct,
            0
        );

        // Run function body
        for t in &self.body {
            println!("{}", t.to_string());
            let bb = create_bb(&template_producer, run_function, t.label_name(run_function.count_basic_blocks()).as_str());
            create_br(&template_producer, bb);
            template_producer.set_current_bb(bb);
            t.produce_llvm_ir(&template_producer);
        }

        // Run function prologue
        let prologue = create_bb(producer, run_function, "prologue");
        create_br(producer, prologue);
        producer.set_current_bb(prologue);

        let ret = create_return_void(producer);
        Some(ret)
    }
}

impl WriteWasm for TemplateCodeInfo {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        // create function code
        let mut instructions = vec![];
        let funcdef1 = format!("(func ${}_create (type $_t_i32ri32)", self.header); //return offset
        instructions.push(funcdef1);
        instructions.push(format!(" (param {} i32)", producer.get_signal_offset_tag()));
        instructions.push("(result i32)".to_string());
        instructions.push(format!(" (local {} i32)", producer.get_offset_tag())); //here is a local var to be returned
        instructions.push(format!(" (local {} i32)", producer.get_merror_tag()));
        instructions.push(set_constant(&producer.get_component_free_pos().to_string()));
        instructions.push(load32(None));
        instructions.push(set_local(producer.get_offset_tag()));
        // set component id
        instructions.push(get_local(producer.get_offset_tag()));
        instructions.push(set_constant(&self.id.to_string()));
        instructions.push(store32(None));
        //set component signal start
        instructions.push(get_local(producer.get_offset_tag()));
        instructions.push(get_local(producer.get_signal_offset_tag()));
        instructions
            .push(store32(Some(&producer.get_signal_start_address_in_component().to_string())));
        //set component inputs_to_be_set
        instructions.push(get_local(producer.get_offset_tag()));
        instructions.push(set_constant(&self.number_of_inputs.to_string()));
        instructions
            .push(store32(Some(&producer.get_input_counter_address_in_component().to_string())));
        //reserve memory for component
        instructions.push(set_constant(&producer.get_component_free_pos().to_string()));
        instructions.push(get_local(producer.get_offset_tag()));
        let nbytes_component =
            producer.get_sub_component_start_in_component() + self.number_of_components * 4;
        instructions.push(set_constant(&nbytes_component.to_string()));
        instructions.push(add32());
        instructions.push(store32(None));
        //add the position of the component in the tree as result
        instructions.push(get_local(producer.get_offset_tag()));
        instructions.push(")".to_string());

        // run function code

        let funcdef2 = format!("(func ${}_run (type $_t_i32ri32)", self.header);
        instructions.push(funcdef2);
        instructions.push(format!(" (param {} i32)", producer.get_offset_tag()));
	instructions.push("(result i32)".to_string()); //state 0 = OK; > 0 error
        instructions.push(format!(" (local {} i32)", producer.get_cstack_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_signal_start_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_sub_cmp_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_sub_cmp_load_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_io_info_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_lvar_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_expaux_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_temp_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_aux_0_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_aux_1_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_aux_2_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_counter_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_store_aux_1_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_store_aux_2_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_copy_counter_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_call_lvar_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_create_loop_sub_cmp_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_create_loop_offset_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_create_loop_counter_tag()));
        instructions.push(format!(" (local {} i32)", producer.get_merror_tag()));
        let local_info_size_u32 = producer.get_local_info_size_u32(); // in the future we can add some info like pointer to run father or text father
                                                                      //set lvar (start of auxiliar memory for vars)
        instructions.push(set_constant("0"));
        instructions.push(load32(None));
        let var_start = local_info_size_u32 * 4; // starts after local info
        if local_info_size_u32 != 0 {
            instructions.push(set_constant(&var_start.to_string()));
            instructions.push(add32());
        }
        instructions.push(set_local(producer.get_lvar_tag()));
        //set expaux (start of auxiliar memory for expressions)
        instructions.push(get_local(producer.get_lvar_tag()));
        let var_stack_size = self.var_stack_depth * 4 * (producer.get_size_32_bit() + 2); // starts after vars
        instructions.push(set_constant(&var_stack_size.to_string()));
        instructions.push(add32());
        instructions.push(set_local(producer.get_expaux_tag()));
        //reserve stack and sets cstack (starts of local var memory)
        let needed_stack_bytes = var_start
            + var_stack_size
            + self.expression_stack_depth * 4 * (producer.get_size_32_bit() + 2);
        let mut reserve_stack_fr_code = reserve_stack_fr(producer, needed_stack_bytes);
        instructions.append(&mut reserve_stack_fr_code);
        if producer.needs_comments() {
            instructions.push(";; start of the template code".to_string());
	}
        //set signalstart local
        instructions.push(get_local(producer.get_offset_tag()));
        instructions
            .push(set_constant(&producer.get_signal_start_address_in_component().to_string()));
        instructions.push(add32());
        instructions.push(load32(None));
        instructions.push(set_local(producer.get_signal_start_tag()));
        //generate code

        for t in &self.body {
            let mut instructions_body = t.produce_wasm(producer);
            instructions.append(&mut instructions_body);
        }

        //free stack
        let mut free_stack_code = free_stack(producer);
        instructions.append(&mut free_stack_code);
        instructions.push(set_constant("0"));	
        instructions.push(")".to_string());
        instructions
    }
}

impl WriteC for TemplateCodeInfo {
    fn produce_c(&self, producer: &CProducer, _parallel: Option<bool>) -> (Vec<String>, String) {
        let mut produced_c = Vec::new();
        if self.is_parallel || self.is_parallel_component{
            produced_c.append(&mut self.produce_c_parallel_case(producer, true));
        }
        if !self.is_parallel && self.is_not_parallel_component{
            produced_c.append(&mut self.produce_c_parallel_case(producer, false));
        } 
        (produced_c, "".to_string())
    }
}


impl TemplateCodeInfo {
    fn produce_c_parallel_case(&self, producer: &CProducer, parallel: bool) -> Vec<String> {
        use c_code_generator::*;

        let create_header = if parallel {format!("void {}_create_parallel", self.header)}
            else{format!("void {}_create", self.header)} ;
        let mut create_params = vec![];
        create_params.push(declare_signal_offset());
        create_params.push(declare_component_offset());
        create_params.push(declare_circom_calc_wit());
        create_params.push(declare_component_name());
        create_params.push(declare_component_father());
        let mut create_body = vec![];

        create_body.push(format!(
            "{}->componentMemory[{}].templateId = {};",
            CIRCOM_CALC_WIT,
	        component_offset(),
            &self.id.to_string()
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].templateName = \"{}\";",
            CIRCOM_CALC_WIT,
	        component_offset(),
            &self.name.to_string()
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].signalStart = {};",
            CIRCOM_CALC_WIT,
	        component_offset(),
	        SIGNAL_OFFSET
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].inputCounter = {};",
            CIRCOM_CALC_WIT,
	        component_offset(),
            &self.number_of_inputs.to_string()
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].componentName = {};",
            CIRCOM_CALC_WIT,
	        component_offset(),
            COMPONENT_NAME
        ));
        create_body.push(format!(
            "{}->componentMemory[{}].idFather = {};",
            CIRCOM_CALC_WIT,
	        component_offset(),
            COMPONENT_FATHER
        ));
        if self.number_of_components > 0{
            create_body.push(format!(
                "{}->componentMemory[{}].subcomponents = new uint[{}]{{0}};",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_components.to_string()
            ));
        } else{
            create_body.push(format!(
                "{}->componentMemory[{}].subcomponents = new uint[{}];",
                CIRCOM_CALC_WIT,
                component_offset(),
                &self.number_of_components.to_string()
            ));
        }
	if self.has_parallel_sub_cmp {
            create_body.push(format!(
		"{}->componentMemory[{}].sbct = new std::thread[{}];",
		CIRCOM_CALC_WIT,
		component_offset(),
		&self.number_of_components.to_string()
            ));

        create_body.push(format!(
            "{}->componentMemory[{}].subcomponentsParallel = new bool[{}];",
            CIRCOM_CALC_WIT,
            component_offset(),
            &self.number_of_components.to_string()
        ));
	}
	if parallel {
            create_body.push(format!(
		"{}->componentMemory[{}].outputIsSet = new bool[{}]();",
		CIRCOM_CALC_WIT,
		component_offset(),
		&self.number_of_outputs.to_string()
            ));
            create_body.push(format!(
		"{}->componentMemory[{}].mutexes = new std::mutex[{}];",
		CIRCOM_CALC_WIT,
		component_offset(),
		&self.number_of_outputs.to_string()
            ));
            create_body.push(format!(
		"{}->componentMemory[{}].cvs = new std::condition_variable[{}];",
		CIRCOM_CALC_WIT,
		component_offset(),
		&self.number_of_outputs.to_string()
            ));
	}
	// if has no inputs should be runned
	if self.number_of_inputs == 0 {
	    let cmp_call_name = format!("{}_run", self.header);
	    let cmp_call_arguments = vec![component_offset(), CIRCOM_CALC_WIT.to_string()]; 
	    create_body.push(format!("{};",build_call(cmp_call_name, cmp_call_arguments)));
        }
        let create_fun = build_callable(create_header, create_params, create_body);

        let run_header = if parallel {format!("void {}_run_parallel", self.header)}
            else{format!("void {}_run", self.header)} ;
        let mut run_params = vec![];
        run_params.push(declare_ctx_index());
        run_params.push(declare_circom_calc_wit());
        let mut run_body = vec![];
        run_body.push(format!("{};", declare_signal_values()));
        run_body.push(format!("{};", declare_my_signal_start()));
        run_body.push(format!("{};", declare_my_template_name()));
        run_body.push(format!("{};", declare_my_component_name()));
        run_body.push(format!("{};", declare_my_father()));
        run_body.push(format!("{};", declare_my_id()));
        run_body.push(format!("{};", declare_my_subcomponents()));
        run_body.push(format!("{};", declare_my_subcomponents_parallel()));
        run_body.push(format!("{};", declare_circuit_constants()));
        run_body.push(format!("{};", declare_list_of_template_messages_use()));
        run_body.push(format!("{};", declare_expaux(self.expression_stack_depth)));
        run_body.push(format!("{};", declare_lvar(self.var_stack_depth)));
        run_body.push(format!("{};", declare_sub_component_aux()));
        run_body.push(format!("{};", declare_index_multiple_eq()));
        for t in &self.body {
            let (mut instructions_body, _) = t.produce_c(producer, Some(parallel));
            run_body.append(&mut instructions_body);
        }
	// parallelism (join at the end of the function)
	if self.number_of_components > 0 && self.has_parallel_sub_cmp {
            run_body.push(format!("{{"));
	    run_body.push(format!("for (uint i = 0; i < {}; i++) {{",&self.number_of_components.to_string()));
	    run_body.push(format!("if (ctx->componentMemory[ctx_index].sbct[i].joinable()) {{"));
	    run_body.push(format!("ctx->componentMemory[ctx_index].sbct[i].join();"));
	    run_body.push(format!("}}"));
	    run_body.push(format!("}}"));
	    run_body.push(format!("}}"));
	}
	if parallel {
	    // parallelism
	    run_body.push(format!("ctx->numThreadMutex.lock();"));
	    run_body.push(format!("ctx->numThread--;"));
	    run_body.push(format!("ctx->numThreadMutex.unlock();"));
	    run_body.push(format!("ctx->ntcvs.notify_one();"));	     
	}

        // to release the memory of its subcomponents
        run_body.push(format!("for (uint i = 0; i < {}; i++){{", &self.number_of_components.to_string()));
        run_body.push(format!(
            "uint index_subc = {}->componentMemory[{}].subcomponents[i];",
            CIRCOM_CALC_WIT,
            ctx_index(),
        ));
        run_body.push(format!("if (index_subc != 0){};", 
            build_call(
                "release_memory_component".to_string(), 
                vec![CIRCOM_CALC_WIT.to_string(), "index_subc".to_string()]
            )));
        
        run_body.push(format!("}}"));
        let run_fun = build_callable(run_header, run_params, run_body);
        vec![create_fun, run_fun]
    }

    pub fn wrap(self) -> TemplateCode {
        TemplateCode::new(self)
    }
}
