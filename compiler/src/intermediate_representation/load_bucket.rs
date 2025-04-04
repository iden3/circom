use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Clone)]
pub struct LoadBucket {
    pub line: usize,
    pub message_id: usize,
    pub address_type: AddressType,
    pub src: LocationRule,
    pub context: InstrContext,
}

impl IntoInstruction for LoadBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Load(self)
    }
}

impl Allocate for LoadBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for LoadBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for LoadBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let address = self.address_type.to_string();
        let src = self.src.to_string();
        format!(
            "LOAD(line:{},template_id:{},address_type:{},src:{})",
            line, template_id, address, src
        )
    }
}
impl WriteWasm for LoadBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; load bucket".to_string());
	}
        match &self.src {
            LocationRule::Indexed { location, .. } => {
                let mut instructions_src = location.produce_wasm(producer);
                instructions.append(&mut instructions_src);
                let size = producer.get_size_32_bits_in_memory() * 4;
                instructions.push(set_constant(&size.to_string()));
                instructions.push(mul32());
                match &self.address_type {
                    AddressType::Variable => {
                        instructions.push(get_local(producer.get_lvar_tag()).to_string());
                    }
                    AddressType::Signal => {
                        instructions.push(get_local(producer.get_signal_start_tag()).to_string());
                    }
                    AddressType::SubcmpSignal { cmp_address, .. } => {
			if producer.needs_comments() {
			    instructions.push(";; is subcomponent".to_string());
			}
                        instructions.push(get_local(producer.get_offset_tag()));
                        instructions.push(set_constant(
                            &producer.get_sub_component_start_in_component().to_string(),
                        ));
                        instructions.push(add32());
                        let mut instructions_sci = cmp_address.produce_wasm(producer);
                        instructions.append(&mut instructions_sci);
                        instructions.push(set_constant("4")); //size in byte of i32
                        instructions.push(mul32());
                        instructions.push(add32());
                        instructions.push(load32(None)); //subcomponent block
                        instructions.push(set_constant(
                            &producer.get_signal_start_address_in_component().to_string(),
                        ));
                        instructions.push(add32());
                        instructions.push(load32(None)); //subcomponent start_of_signals
                    }
                }
                instructions.push(add32());
		if producer.needs_comments() {
                    instructions.push(";; end of load bucket".to_string());
		}
            }
            LocationRule::Mapped { signal_code, indexes} => {
                match &self.address_type {
                    AddressType::SubcmpSignal { cmp_address, .. } => {
			if producer.needs_comments() {
                            instructions.push(";; is subcomponent mapped".to_string());
			}
                        instructions.push(get_local(producer.get_offset_tag()));
                        instructions.push(set_constant(
                            &producer.get_sub_component_start_in_component().to_string(),
                        ));
                        instructions.push(add32());
                        let mut instructions_sci = cmp_address.produce_wasm(producer);
                        instructions.append(&mut instructions_sci);
                        instructions.push(set_constant("4")); //size in byte of i32
                        instructions.push(mul32());
                        instructions.push(add32());
                        instructions.push(load32(None)); //subcomponent block
                        instructions.push(tee_local(producer.get_sub_cmp_load_tag()));
                        instructions.push(load32(Some(
                            &producer.get_signal_start_address_in_component().to_string()
                        ))); //subcomponent start_of_signals
                        // and now, we compute the offset
                        instructions.push(get_local(producer.get_sub_cmp_load_tag()));
                        instructions.push(load32(None)); // get template id
                        instructions.push(set_constant("4")); //size in byte of i32
                        instructions.push(mul32());
                        instructions.push(load32(Some(
                            &producer.get_template_instance_to_io_signal_start().to_string(),
                        ))); // get position in component io signal to info list
                        let signal_code_in_bytes = signal_code * 4; //position in the list of the signal code
                        instructions.push(load32(Some(&signal_code_in_bytes.to_string()))); // get where the info of this signal is
                        //now we have first the offset, and then the all size dimensions but the last one
			if indexes.len() == 0 {
			    //instructions.push(";; has no indexes".to_string());
			    instructions.push(load32(None)); // get signal offset (it is already the actual one in memory);
			} else {
			    //instructions.push(";; has indexes".to_string());
			    instructions.push(tee_local(producer.get_io_info_tag()));
			    instructions.push(load32(None)); // get offset; first slot in io_info (to start adding offsets)
			    // if the first access is qualified we place the address of the bus_id
			    if let AccessType::Qualified(_) = &indexes[0] {
				instructions.push(get_local(producer.get_io_info_tag()));
				instructions.push(load32(Some("4"))); // it is a bus, so the bus_id is in the second position
			    }
			    let mut idxpos = 0;			    
			    while idxpos < indexes.len() {
				if let AccessType::Indexed(index_info) = &indexes[idxpos] {				    
                                    let index_list = &index_info.indexes;
                                    let dim = index_info.symbol_dim;
                                    let mut infopos = 0;
				    assert!(index_list.len() > 0);
				    //We first compute the number of elements as
				    //((index_list[0] * length_of_dim[1]) + index_list[1]) * length_of_dim[2] + ... )* length_of_dim[n-1] + index_list[n-1]
				    //first position in the array access
				    let mut instructions_idx0 = index_list[0].produce_wasm(producer);				    
				    instructions.append(&mut instructions_idx0);				    
				    for i in 1..index_list.len() {
					instructions.push(get_local(producer.get_io_info_tag()));
					infopos += 4;	//position in io or bus info of dimension of [1] (recall that first dimension is not added)
					instructions.push(load32(Some(&infopos.to_string()))); // second dimension
					instructions.push(mul32());
					let mut instructions_idxi = index_list[i].produce_wasm(producer);				    
					instructions.append(&mut instructions_idxi);				    
					instructions.push(add32());
				    }
				    assert!(index_list.len() <= dim);
				    let diff = dim - index_list.len();
				    if diff > 0 {
					//println!("There is difference: {}",diff);
					//instructions.push(format!(";; There is a difference {}", diff));
					// must be last access
					assert!(idxpos+1 == indexes.len());
					instructions.push(get_local(producer.get_io_info_tag()));
					infopos += 4; //position in io or bus info of the next dimension 
					instructions.push(load32(Some(&infopos.to_string()))); // length of next dimension					
					for _i in 1..diff {
					    //instructions.push(format!(";; Next dim {}", i));
					    instructions.push(get_local(producer.get_io_info_tag()));
					    infopos += 4; //position in io or bus info of the next dimension 
					    instructions.push(load32(Some(&infopos.to_string()))); // length of next dimension					
					    instructions.push(mul32()); // multiply with previous dimensions
					}
				    } // after this we have the product of the remaining dimensions				    
				    let field_size = producer.get_size_32_bits_in_memory() * 4;
				    instructions.push(set_constant(&field_size.to_string()));
				    instructions.push(get_local(producer.get_io_info_tag()));
				    infopos += 4; //position in io or bus info of size 
				    instructions.push(load32(Some(&infopos.to_string()))); // size
				    instructions.push(mul32()); // size mult by size of field in bytes
				    if diff > 0 {
					//instructions.push(format!(";; Multiply dimensions"));
					instructions.push(mul32()); // total size of the content according to the missing dimensions
				    }
				    instructions.push(mul32()); // total offset in the array
				    instructions.push(add32()); // to the current offset
				    idxpos += 1;
				    if idxpos < indexes.len() {
					//next must be Qualified
					if let AccessType::Indexed(_) = &indexes[idxpos] {
					    assert!(false);
					}
					// we add the type of bus it is
					instructions.push(get_local(producer.get_io_info_tag()));
					infopos += 4;
					instructions.push(load32(Some(&infopos.to_string()))); // bus_id
				    }
				} else if let AccessType::Qualified(field_no) = &indexes[idxpos] {
				    //we have on the stack the bus_id
				    instructions.push(set_constant("4")); //size in byte of i32
				    instructions.push(mul32()); //maybe better in the memory like this
				    instructions.push(load32(Some(
					&producer.get_bus_instance_to_field_start().to_string()
				    ))); // get position in the bus to field in memory
				    let field_no_bytes = field_no * 4;
				    instructions.push(load32(Some(&field_no_bytes.to_string()))); // get position in the field info in memory
				    if idxpos +1 < indexes.len() {				    
					instructions.push(tee_local(producer.get_io_info_tag()));
				    }
				    //let field_size = producer.get_size_32_bits_in_memory() * 4;
				    //instructions.push(set_constant(&field_size.to_string()));
				    instructions.push(load32(None)); // get the offset
				    //instructions.push(mul32()); // mult by size of field in bytes
				    instructions.push(add32()); // add to the current offset
				    idxpos += 1;
				    if idxpos < indexes.len() {
					if let AccessType::Qualified(_) = &indexes[idxpos] {
					    instructions.push(get_local(producer.get_io_info_tag()));
					    instructions.push(load32(Some("4"))); // bus_id
					}
				    }
				} else {
				    assert!(false);
				}
			    }
			}
                        //after this we have  the offset on top of the stack and the subcomponent start_of_signals just below
                        instructions.push(add32()); // we get the position of the signal (with indexes) in memory
			if producer.needs_comments() {
                            instructions.push(";; end of load bucket".to_string());
			}
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }
        }
        instructions
    }
}

impl WriteC for LoadBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        let mut prologue = vec![];
	//prologue.push(format!("// start of load line {} bucket {}",self.line.to_string(),self.to_string()));
	let cmp_index_ref;
        if let AddressType::SubcmpSignal { cmp_address, .. } = &self.address_type {
            let (mut cmp_prologue, cmp_index) = cmp_address.produce_c(producer, parallel);
            prologue.append(&mut cmp_prologue);
	    cmp_index_ref = cmp_index;
	} else {
            cmp_index_ref = "".to_string();
	}

        let (mut src_prologue, src_index) =
            if let LocationRule::Indexed { location, .. } = &self.src {
                location.produce_c(producer, parallel)
            } else if let LocationRule::Mapped { signal_code, indexes } = &self.src {
        let mut map_prologue = vec![];
		let sub_component_pos_in_memory = format!("{}[{}]",MY_SUBCOMPONENTS,cmp_index_ref.clone());
		let mut map_access = format!("{}->{}[{}].defs[{}].offset",
					     circom_calc_wit(), template_ins_2_io_info(),
					     template_id_in_component(sub_component_pos_in_memory.clone()),
					     signal_code.to_string());
	        if indexes.len() > 0 {
		    //cur_def contains a string that goes to the definion of the next acces.
		    //The first time it is taken from template_ins_2_io_info
		    let mut cur_def = format!("{}->{}[{}].defs[{}]",
					    circom_calc_wit(), template_ins_2_io_info(),
					    template_id_in_component(sub_component_pos_in_memory.clone()),
					      signal_code.to_string());
		    let mut idxpos = 0;
		    while idxpos < indexes.len() {
			if let AccessType::Indexed(index_info) = &indexes[idxpos] {
			    let index_list = &index_info.indexes;
			    let dim = index_info.symbol_dim;
			    //We first compute the number of elements as
			    //((index_list[0] * length_of_dim[1]) + index_list[1]) * length_of_dim[2] + ... )* length_of_dim[n-1] + index_list[n-1] 
		            let (mut index_code_0, mut map_index) = index_list[0].produce_c(producer, parallel);
		            map_prologue.append(&mut index_code_0);
		            for i in 1..index_list.len() {
				let (mut index_code, index_exp) = index_list[i].produce_c(producer, parallel);
				map_prologue.append(&mut index_code);
				map_index = format!("({})*({}.lengths[{}])+{}",
						    map_index,cur_def,(i-1).to_string(),index_exp);
		            }
			    assert!(index_list.len() <= dim);
			    if dim - index_list.len() > 0 {
				map_prologue.push(format!("//There is a difference {};",dim - index_list.len()));
				// must be last access
				assert!(idxpos+1 == indexes.len());
				for i in index_list.len()..dim {
				    map_index = format!("{}*{}.lengths[{}]",
							map_index, cur_def, (i-1).to_string());
				} // after this we have multiplied by the remaining dimensions
			    }
			    // multiply the offset in the array (after multiplying by the missing dimensions) by the size of the elements
			    // and add it to the access expression 
			    map_access = format!("{}+({})*{}.size", map_access, map_index, cur_def);
			} else if let AccessType::Qualified(field_no) = &indexes[idxpos] {
			    cur_def = format!("{}->{}[{}.busId].defs[{}]",
					      circom_calc_wit(), bus_ins_2_field_info(),
					      cur_def, field_no.to_string());
			    map_access = format!("{}+{}.offset", map_access, cur_def);
			} else {
			    assert!(false);
			}
			idxpos += 1;
	            }
		}
                (map_prologue, map_access)
	    } else {
		assert!(false);
                (vec![], "".to_string())
	    };
        
        
        prologue.append(&mut src_prologue);
        let access = match &self.address_type {
            AddressType::Variable => {
                if producer.prime_str != "goldilocks" {
                    format!("&{}", lvar(src_index))
                } else {
                    format!("{}", lvar(src_index))
                }                    
            }
            AddressType::Signal => {
                if producer.prime_str != "goldilocks" {
                    format!("&{}", signal_values(src_index))
                } else {
                    format!("{}", signal_values(src_index))
                }
            }
            AddressType::SubcmpSignal { uniform_parallel_value, is_output, .. } => {
            // we store the value of the cmp index
            prologue.push(format!("cmp_index_ref_load = {};",cmp_index_ref.clone()));

            // we store the value of the cmp index
            prologue.push(format!("cmp_index_ref_load = {};",cmp_index_ref.clone()));

		if *is_output {
            if uniform_parallel_value.is_some(){
                if uniform_parallel_value.unwrap(){
                    prologue.push(format!("{{"));
                    // We compute the possible sizes, case multiple size
                    let size = match &self.context.size{
                        SizeOption::Single(value) => value.to_string(),
                        SizeOption::Multiple(values) => {
                            prologue.push(format!("std::map<int,int> size_load {};",
                                set_list_tuple(values.clone())
                            ));
                            let sub_component_pos_in_memory = format!("{}[{}]",MY_SUBCOMPONENTS,cmp_index_ref);
                            let temp_id = template_id_in_component(sub_component_pos_in_memory);
                            format!("size_load[{}]", temp_id)
                        }
                    };
		            prologue.push(format!("int aux2 = {};",src_index.clone()));
                    // check each one of the outputs of the assignment, we add i to check them one by one
                    
                    prologue.push(format!("for (int i = 0; i < {}; i++) {{", size));
                    prologue.push(format!("ctx->numThreadMutex.lock();"));
                    prologue.push(format!("ctx->numThread--;"));
                    //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                    prologue.push(format!("ctx->numThreadMutex.unlock();"));
                    prologue.push(format!("ctx->ntcvs.notify_one();"));	 
		            prologue.push(format!(
                        "std::unique_lock<std::mutex> lk({}->componentMemory[{}[cmp_index_ref_load]].mutexes[aux2 + i]);",
                        CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
		            prologue.push(format!(
                        "{}->componentMemory[{}[cmp_index_ref_load]].cvs[aux2 + i].wait(lk, [{},{},cmp_index_ref_load,aux2, i]() {{return {}->componentMemory[{}[cmp_index_ref_load]].outputIsSet[aux2 + i];}});",
			            CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, CIRCOM_CALC_WIT,
			            MY_SUBCOMPONENTS, CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
                    prologue.push(format!("std::unique_lock<std::mutex> lkt({}->numThreadMutex);",CIRCOM_CALC_WIT));
                    prologue.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                    prologue.push(format!("ctx->numThread++;"));
                    //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                    prologue.push(format!("}}"));
		            prologue.push(format!("}}"));
                }
            }
            // Case we only know if it is parallel at execution
            else{
                // We compute the possible sizes, case multiple size
                let size = match &self.context.size{
                    SizeOption::Single(value) => value.to_string(),
                    SizeOption::Multiple(values) => {
                        prologue.push(format!("std::map<int,int> size_load {};",
                            set_list_tuple(values.clone())
                        ));
                        let sub_component_pos_in_memory = format!("{}[{}]",MY_SUBCOMPONENTS,cmp_index_ref);
                        let temp_id = template_id_in_component(sub_component_pos_in_memory);
                        format!("size_load[{}]", temp_id)
                    }
                };
                prologue.push(format!(
                    "if ({}[{}]){{",
                    MY_SUBCOMPONENTS_PARALLEL, 
                    cmp_index_ref
                ));

                // case parallel
                prologue.push(format!("{{"));
		        prologue.push(format!("int aux2 = {};",src_index.clone()));
		        // check each one of the outputs of the assignment, we add i to check them one by one
                prologue.push(format!("for (int i = 0; i < {}; i++) {{", size));
                prologue.push(format!("ctx->numThreadMutex.lock();"));
                prologue.push(format!("ctx->numThread--;"));
                //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                prologue.push(format!("ctx->numThreadMutex.unlock();"));
                prologue.push(format!("ctx->ntcvs.notify_one();"));	 
	            prologue.push(format!(
                        "std::unique_lock<std::mutex> lk({}->componentMemory[{}[cmp_index_ref_load]].mutexes[aux2 + i]);",
                        CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
		        prologue.push(format!(
                        "{}->componentMemory[{}[cmp_index_ref_load]].cvs[aux2 + i].wait(lk, [{},{},cmp_index_ref_load,aux2, i]() {{return {}->componentMemory[{}[cmp_index_ref_load]].outputIsSet[aux2 + i];}});",
			            CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, CIRCOM_CALC_WIT,
			            MY_SUBCOMPONENTS, CIRCOM_CALC_WIT, MY_SUBCOMPONENTS)
                    );
                prologue.push(format!("std::unique_lock<std::mutex> lkt({}->numThreadMutex);",CIRCOM_CALC_WIT));
                prologue.push(format!("{}->ntcvs.wait(lkt, [{}]() {{return {}->numThread <  {}->maxThread; }});",CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT,CIRCOM_CALC_WIT));
                prologue.push(format!("ctx->numThread++;"));
                //prologue.push(format!("printf(\"%i \\n\", ctx->numThread);"));
                prologue.push(format!("}}"));
		        prologue.push(format!("}}"));
                
                // end of case parallel, in case no parallel we do nothing

                prologue.push(format!("}}"));
            }
        }
                let sub_cmp_start = format!(
                    "{}->componentMemory[{}[{}]].signalStart",
                    CIRCOM_CALC_WIT, MY_SUBCOMPONENTS, cmp_index_ref
                );
		if producer.prime_str != "goldilocks" {   
                    format!("&{}->signalValues[{} + {}]", CIRCOM_CALC_WIT, sub_cmp_start, src_index)
                } else {
                    format!("{}->signalValues[{} + {}]", CIRCOM_CALC_WIT, sub_cmp_start, src_index)
                }
            }
        };
        
	//prologue.push(format!("// end of load line {} with access {}",self.line.to_string(),access));
        (prologue, access)
    }
}
