use std::cell::RefCell;
use std::collections::BTreeMap;
use compiler::circuit_design::function::FunctionCode;
use compiler::circuit_design::template::TemplateCode;
use compiler::compiler_interface::Circuit;
use compiler::intermediate_representation::InstructionPointer;
use compiler::intermediate_representation::ir_interface::{
    Allocate, AssertBucket, BlockBucket, BranchBucket, CallBucket, ComputeBucket,
    ConstraintBucket, CreateCmpBucket, LoadBucket, LocationRule, LogBucket, LoopBucket, NopBucket,
    ReturnBucket, StoreBucket, ValueBucket,
};
use crate::bucket_interpreter::env::{FunctionsLibrary, TemplatesLibrary};
use crate::bucket_interpreter::env::Env;
use crate::bucket_interpreter::BucketInterpreter;
use crate::bucket_interpreter::observer::InterpreterObserver;
use crate::passes::CircuitTransformationPass;
use crate::passes::memory::PassMemory;

pub struct LoopUnrollPass {
    // Wrapped in a RefCell because the reference to the static analysis is immutable but we need mutability
    memory: RefCell<PassMemory>,
    replacements: RefCell<BTreeMap<LoopBucket, InstructionPointer>>,
}

impl LoopUnrollPass {
    pub fn new(prime: &String) -> Self {
        LoopUnrollPass {
            memory: PassMemory::new_cell(prime),
            replacements: Default::default(),
        }
    }
}

impl InterpreterObserver for LoopUnrollPass {
    fn on_value_bucket(&self, _bucket: &ValueBucket, _env: &Env) -> bool {
        true
    }

    fn on_load_bucket(&self, _bucket: &LoadBucket, _env: &Env) -> bool {
        true
    }

    fn on_store_bucket(&self, _bucket: &StoreBucket, _env: &Env) -> bool {
        true
    }

    fn on_compute_bucket(&self, _bucket: &ComputeBucket, _env: &Env) -> bool {
        true
    }

    fn on_assert_bucket(&self, _bucket: &AssertBucket, _env: &Env) -> bool {
        true
    }

    fn on_loop_bucket(&self, bucket: &LoopBucket, env: &Env) -> bool {
        let mem = self.memory.borrow();
        let interpreter = BucketInterpreter::init(&mem.prime, &mem.constant_fields, self);
        // First we run the loop once. If the result is None that means that the condition is unknown
        let (_, cond_result, _) = interpreter.execute_loop_bucket_once(bucket, env, false);
        if cond_result.is_none() {
            return true;
        }
        let mut block_body = vec![];
        let mut cond_result = Some(true);
        let mut env = env.clone();
        while cond_result.unwrap() {
            let (_, new_cond, new_env) = interpreter.execute_loop_bucket_once(bucket, &env, false);
            cond_result = new_cond;
            env = new_env;
            if let Some(true) = new_cond {
                for inst in &bucket.body {
                    block_body.push(inst.clone());
                }
            }
        }
        let block =
            BlockBucket { line: bucket.line, message_id: bucket.message_id, body: block_body }
                .allocate();
        self.replacements.borrow_mut().insert(bucket.clone(), block);
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
}

impl CircuitTransformationPass for LoopUnrollPass {
    fn pre_hook_circuit(&self, circuit: &Circuit) {
        self.memory.borrow_mut().fill_from_circuit(circuit);
    }

    fn pre_hook_template(&self, template: &TemplateCode) {
        self.memory.borrow().run_template(self, template);
    }

    fn get_updated_field_constants(&self) -> Vec<String> {
        self.memory.borrow().constant_fields.clone()
    }

    fn run_on_loop_bucket(&self, bucket: &LoopBucket) -> InstructionPointer {
        if let Some(unrolled_loop) = self.replacements.borrow().get(&bucket) {
            return self.run_on_instruction(unrolled_loop);
        }
        LoopBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            continue_condition: self.run_on_instruction(&bucket.continue_condition),
            body: self.run_on_instructions(&bucket.body),
        }.allocate()
    }
}

#[cfg(test)]
mod test {
    use compiler::circuit_design::template::{TemplateCode, TemplateCodeInfo};
    use compiler::compiler_interface::Circuit;
    use compiler::intermediate_representation::Instruction;
    use compiler::intermediate_representation::ir_interface::{
        AddressType, Allocate, ComputeBucket, InstrContext, LoadBucket, LocationRule, LoopBucket,
        OperatorType, StoreBucket, ValueBucket, ValueType,
    };
    use crate::passes::CircuitTransformationPass;
    use crate::passes::loop_unroll::LoopUnrollPass;

    #[test]
    fn test_loop_unrolling() {
        let prime = "goldilocks".to_string();
        let pass = LoopUnrollPass::new(&prime);
        let circuit = example_program();
        let new_circuit = pass.run_on_circuit(&circuit);
        println!("{}", new_circuit.templates[0].body.last().unwrap().to_string());
        assert_ne!(circuit, new_circuit);
        match new_circuit.templates[0].body.last().unwrap().as_ref() {
            Instruction::Block(b) => assert_eq!(b.body.len(), 5),
            _ => assert!(false),
        }
    }

    fn example_program() -> Circuit {
        Circuit {
            wasm_producer: Default::default(),
            c_producer: Default::default(),
            llvm_data: Default::default(),
            templates: vec![Box::new(TemplateCodeInfo {
                id: 0,
                header: "test_0".to_string(),
                name: "test".to_string(),
                is_parallel: false,
                is_parallel_component: false,
                is_not_parallel_component: false,
                has_parallel_sub_cmp: false,
                number_of_inputs: 0,
                number_of_outputs: 0,
                number_of_intermediates: 0,
                body: vec![
                    // (store 0 0)
                    StoreBucket {
                        line: 0,
                        message_id: 0,
                        context: InstrContext { size: 0 },
                        dest_is_output: false,
                        dest_address_type: AddressType::Variable,
                        dest: LocationRule::Indexed {
                            location: ValueBucket {
                                line: 0,
                                message_id: 0,
                                parse_as: ValueType::U32,
                                op_aux_no: 0,
                                value: 0,
                            }
                            .allocate(),
                            template_header: Some("test_0".to_string()),
                        },
                        src: ValueBucket {
                            line: 0,
                            message_id: 0,
                            parse_as: ValueType::U32,
                            op_aux_no: 0,
                            value: 0,
                        }
                        .allocate(),
                    }
                    .allocate(),
                    // (store 1 0)
                    StoreBucket {
                        line: 0,
                        message_id: 0,
                        context: InstrContext { size: 0 },
                        dest_is_output: false,
                        dest_address_type: AddressType::Variable,
                        dest: LocationRule::Indexed {
                            location: ValueBucket {
                                line: 0,
                                message_id: 0,
                                parse_as: ValueType::U32,
                                op_aux_no: 0,
                                value: 1,
                            }
                            .allocate(),
                            template_header: Some("test_0".to_string()),
                        },
                        src: ValueBucket {
                            line: 0,
                            message_id: 0,
                            parse_as: ValueType::U32,
                            op_aux_no: 0,
                            value: 0,
                        }
                        .allocate(),
                    }
                    .allocate(),
                    // (loop (compute le (load 1) 5) (
                    LoopBucket {
                        line: 0,
                        message_id: 0,
                        continue_condition: ComputeBucket {
                            line: 0,
                            message_id: 0,
                            op: OperatorType::Lesser,
                            op_aux_no: 0,
                            stack: vec![
                                LoadBucket {
                                    line: 0,
                                    message_id: 0,
                                    address_type: AddressType::Variable,
                                    src: LocationRule::Indexed {
                                        location: ValueBucket {
                                            line: 0,
                                            message_id: 0,
                                            parse_as: ValueType::U32,
                                            op_aux_no: 0,
                                            value: 1,
                                        }
                                        .allocate(),
                                        template_header: Some("test_0".to_string()),
                                    },
                                }
                                .allocate(),
                                ValueBucket {
                                    line: 0,
                                    message_id: 0,
                                    parse_as: ValueType::U32,
                                    op_aux_no: 0,
                                    value: 5,
                                }
                                .allocate(),
                            ],
                        }
                        .allocate(),
                        body: vec![
                            //   (store 0 (compute add (load 0) 2))
                            StoreBucket {
                                line: 0,
                                message_id: 0,
                                context: InstrContext { size: 0 },
                                dest_is_output: false,
                                dest_address_type: AddressType::Variable,
                                dest: LocationRule::Indexed {
                                    location: ValueBucket {
                                        line: 0,
                                        message_id: 0,
                                        parse_as: ValueType::U32,
                                        op_aux_no: 0,
                                        value: 0,
                                    }
                                    .allocate(),
                                    template_header: None,
                                },
                                src: ComputeBucket {
                                    line: 0,
                                    message_id: 0,
                                    op: OperatorType::Add,
                                    op_aux_no: 0,
                                    stack: vec![
                                        LoadBucket {
                                            line: 0,
                                            message_id: 0,
                                            address_type: AddressType::Variable,
                                            src: LocationRule::Indexed {
                                                location: ValueBucket {
                                                    line: 0,
                                                    message_id: 0,
                                                    parse_as: ValueType::U32,
                                                    op_aux_no: 0,
                                                    value: 0,
                                                }
                                                .allocate(),
                                                template_header: Some("test_0".to_string()),
                                            },
                                        }
                                        .allocate(),
                                        ValueBucket {
                                            line: 0,
                                            message_id: 0,
                                            parse_as: ValueType::U32,
                                            op_aux_no: 0,
                                            value: 2,
                                        }
                                        .allocate(),
                                    ],
                                }
                                .allocate(),
                            }
                            .allocate(),
                            //   (store 1 (compute add (load 1) 1))
                            StoreBucket {
                                line: 0,
                                message_id: 0,
                                context: InstrContext { size: 0 },
                                dest_is_output: false,
                                dest_address_type: AddressType::Variable,
                                dest: LocationRule::Indexed {
                                    location: ValueBucket {
                                        line: 0,
                                        message_id: 0,
                                        parse_as: ValueType::U32,
                                        op_aux_no: 0,
                                        value: 1,
                                    }
                                    .allocate(),
                                    template_header: None,
                                },
                                src: ComputeBucket {
                                    line: 0,
                                    message_id: 0,
                                    op: OperatorType::Add,
                                    op_aux_no: 0,
                                    stack: vec![
                                        LoadBucket {
                                            line: 0,
                                            message_id: 0,
                                            address_type: AddressType::Variable,
                                            src: LocationRule::Indexed {
                                                location: ValueBucket {
                                                    line: 0,
                                                    message_id: 0,
                                                    parse_as: ValueType::U32,
                                                    op_aux_no: 0,
                                                    value: 1,
                                                }
                                                .allocate(),
                                                template_header: Some("test_0".to_string()),
                                            },
                                        }
                                        .allocate(),
                                        ValueBucket {
                                            line: 0,
                                            message_id: 0,
                                            parse_as: ValueType::U32,
                                            op_aux_no: 0,
                                            value: 1,
                                        }
                                        .allocate(),
                                    ],
                                }
                                .allocate(),
                            }
                            .allocate(),
                        ],
                    }
                    .allocate(), // ))
                ],
                var_stack_depth: 0,
                expression_stack_depth: 0,
                signal_stack_depth: 0,
                number_of_components: 0,
            })],
            functions: vec![],
        }
    }
}
