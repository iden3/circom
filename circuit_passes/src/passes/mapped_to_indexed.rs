use std::cell::RefCell;
use std::collections::BTreeMap;

use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::{InstructionPointer};
use compiler::intermediate_representation::ir_interface::{AddressType, Allocate, AssertBucket, BlockBucket, BranchBucket, CallBucket, ComputeBucket, ConstraintBucket, CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket, ReturnBucket, StoreBucket, ValueBucket};

use crate::bucket_interpreter::env::Env;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::observer::InterpreterObserver;
use crate::bucket_interpreter::value::Value::KnownU32;
use crate::passes::CircuitTransformationPass;
use crate::passes::memory::PassMemory;

pub struct MappedToIndexedPass {
    // Wrapped in a RefCell because the reference to the static analysis is immutable but we need mutability
    memory: RefCell<PassMemory>,
    replacements: RefCell<BTreeMap<LocationRule, LocationRule>>,
}

impl MappedToIndexedPass {
    pub fn new(prime: &String) -> Self {
        MappedToIndexedPass { memory: PassMemory::new_cell(prime, "".to_string(), Default::default()), replacements: Default::default() }
    }

    fn transform_mapped_loc_to_indexed_loc(&self,
        cmp_address: &InstructionPointer, indexes: &Vec<InstructionPointer>, signal_code: usize, env: &Env) -> LocationRule {

        let mem = self.memory.borrow();
        let interpreter = BucketInterpreter::init(&mem.current_scope, &mem.prime, &mem.constant_fields, self, &mem.io_map);

        let (resolved_addr, acc_env) = interpreter.execute_instruction(cmp_address, env.clone(), false);

        let resolved_addr = resolved_addr
            .expect("cmp_address instruction in SubcmpSignal must produce a value!")
            .get_u32();

        let name = acc_env.get_subcmp_name(resolved_addr).clone();
        let mut indexes_values = vec![];
        let mut acc_env = acc_env;
        for i in indexes {
            let (val, new_env) = interpreter.execute_instruction(i, acc_env, false);
            indexes_values.push(val.expect("Mapped location must produce a value!").get_u32());
            acc_env = new_env;
        }

        if indexes.len() > 0 {
            if indexes.len() == 1 {
                let map_access = &mem.io_map[&acc_env.get_subcmp_template_id(resolved_addr)][signal_code].offset;
                let value = map_access + indexes_values[0];
                let mut unused = vec![];
                LocationRule::Indexed { location: KnownU32(value).to_value_bucket(&mut unused).allocate(), template_header: Some(name) }
            } else {
                todo!()
            }
        } else {
            unreachable!()
        }
    }

    fn maybe_transform_location_rule(&self, address: &AddressType, location: &LocationRule, env: &Env) -> bool {
        match address {
            AddressType::Variable | AddressType::Signal => {
                match location {
                    LocationRule::Indexed { .. } => true,
                    LocationRule::Mapped { .. } => unreachable!()
                }
            },
            AddressType::SubcmpSignal { cmp_address, .. } => {
                match location {
                    LocationRule::Indexed { .. } => true,
                    LocationRule::Mapped { indexes, signal_code } => {
                        let indexed_rule = self.transform_mapped_loc_to_indexed_loc(cmp_address, indexes, *signal_code, env);
                        self.replacements.borrow_mut().insert(location.clone(), indexed_rule);
                        true
                    }
                }
            }
        }
    }
}

impl InterpreterObserver for MappedToIndexedPass {
    fn on_value_bucket(&self, _bucket: &ValueBucket, _env: &Env) -> bool {
        true
    }

    fn on_load_bucket(&self, bucket: &LoadBucket, env: &Env) -> bool {
        self.maybe_transform_location_rule(&bucket.address_type, &bucket.src, env)
    }

    fn on_store_bucket(&self, bucket: &StoreBucket, env: &Env) -> bool {
        self.maybe_transform_location_rule(&bucket.dest_address_type, &bucket.dest, env)
    }

    fn on_compute_bucket(&self, _bucket: &ComputeBucket, _env: &Env) -> bool {
        true
    }

    fn on_assert_bucket(&self, _bucket: &AssertBucket, _env: &Env) -> bool {
        true
    }

    fn on_loop_bucket(&self, _bucket: &LoopBucket, _env: &Env) -> bool {
        true
    }

    fn on_create_cmp_bucket(&self, _bucket: &CreateCmpBucket, _env: &Env) -> bool {
        true
    }

    fn on_constraint_bucket(&self, _bucket: &ConstraintBucket, _env: &Env) -> bool {
        true
    }

    fn on_block_bucket(&self, _bucket: &BlockBucket, _env: &Env) -> bool {
        true
    }

    fn on_nop_bucket(&self, _bucket: &NopBucket, _env: &Env) -> bool {
        true
    }

    fn on_location_rule(&self, _location_rule: &LocationRule, _env: &Env) -> bool {
        true
    }

    fn on_call_bucket(&self, _bucket: &CallBucket, _env: &Env) -> bool {
        true
    }

    fn on_branch_bucket(&self, _bucket: &BranchBucket, _env: &Env) -> bool {
        true
    }

    fn on_return_bucket(&self, _bucket: &ReturnBucket, _env: &Env) -> bool {
        true
    }

    fn on_log_bucket(&self, _bucket: &LogBucket, _env: &Env) -> bool {
        true
    }

    fn ignore_function_calls(&self) -> bool {
        true
    }

    fn ignore_subcmp_calls(&self) -> bool {
        true
    }
}

impl CircuitTransformationPass for MappedToIndexedPass {
    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().constant_fields.clone()
    }

    /*
        iangneal: Let the interpreter run to see if we can find any replacements.
        If so, yield the replacement. Else, just give the default transformation
    */
    fn transform_location_rule(&self, location_rule: &LocationRule) -> LocationRule {
        // If the interpreter found a viable transformation, do that.
        if let Some(indexed_rule) = self.replacements.borrow().get(&location_rule) {
            println!("MappedToIndexedPass: {:?} --> {:?}", location_rule.to_string(), indexed_rule);
            match indexed_rule {
                LocationRule::Indexed { location, .. } => println!("\tWill output location: {:?}", location),
                LocationRule::Mapped { .. } => unreachable!()
            }
            return indexed_rule.clone();
        }
        match location_rule {
            LocationRule::Indexed { location, template_header } => LocationRule::Indexed {
                location: self.transform_instruction(location),
                template_header: template_header.clone(),
            },
            LocationRule::Mapped { .. } => unreachable!()
        }
    }

    fn pre_hook_circuit(&self, circuit: &Circuit) {
        self.memory.borrow_mut().fill_from_circuit(circuit);
    }

    fn pre_hook_template(&self, template: &TemplateCode) {
        self.memory.borrow().run_template(self, template);
    }
}
