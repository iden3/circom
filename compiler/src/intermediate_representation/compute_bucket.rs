use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::wasm_elements::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OperatorType {
    Mul,
    Div,
    Add,
    Sub,
    Pow,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    LesserEq,
    GreaterEq,
    Lesser,
    Greater,
    Eq(usize),
    NotEq,
    BoolOr,
    BoolAnd,
    BitOr,
    BitAnd,
    BitXor,
    PrefixSub,
    BoolNot,
    Complement,
    ToAddress,
    MulAddress,
    AddAddress,
}

impl OperatorType {
    pub fn is_address_op(&self) -> bool {
        *self == OperatorType::ToAddress
            || *self == OperatorType::AddAddress
            || *self == OperatorType::MulAddress
    }

    pub fn is_multiple_eq(&self) -> bool {
        match self {
            OperatorType::Eq(n) => *n > 1,
            _ => false
        }
    }
}

impl ToString for OperatorType {
    fn to_string(&self) -> String {
        use OperatorType::*;
	if let Eq(n) = self {
	    format!("EQ({})", n)
	} else {
        match self {
            Mul => "MUL",
            Div => "DIV",
            Add => "ADD",
            Sub => "SUB",
            Pow => "POW",
            IntDiv => "INT_DIV",
            Mod => "MOD",
            ShiftL => "SHIFT_L",
            ShiftR => "SHIFT_R",
            LesserEq => "LESSER_EQ",
            GreaterEq => "GREATER_EQ",
            Lesser => "LESSER",
            Greater => "GREATER",
            NotEq => "NOT_EQ",
            BoolOr => "BOOL_OR",
            BoolAnd => "BOOL_AND",
            BitOr => "BITOR",
            BitAnd => "BITAND",
            BitXor => "BITXOR",
            PrefixSub => "PREFIX_SUB",
            BoolNot => "BOOL_NOT",
            Complement => "COMPLEMENT",
            ToAddress => "TO_ADDRESS",
            MulAddress => "MUL_ADDRESS",
            AddAddress => "ADD_ADDRESS",
	    _ => "",
        }
        .to_string()
	}
    }
}

#[derive(Clone)]
pub struct ComputeBucket {
    pub line: usize,
    pub message_id: usize,
    pub op: OperatorType,
    pub op_aux_no: usize,
    pub stack: Vec<InstructionPointer>,
}

impl IntoInstruction for ComputeBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Compute(self)
    }
}

impl Allocate for ComputeBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for ComputeBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for ComputeBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let op = self.op.to_string();
        let op_number = self.op_aux_no.to_string();
        let mut stack = "".to_string();
        for i in &self.stack {
            stack = format!("{}{};", stack, i.to_string());
        }
        format!(
            "COMPUTE(line:{},template_id:{},op_number:{},op:{},stack:{})",
            line, template_id, op_number, op, stack
        )
    }
}
impl WriteWasm for ComputeBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(";; compute bucket".to_string());
	}
        match &self.op {
            OperatorType::AddAddress => {}
            OperatorType::MulAddress => {}
            OperatorType::ToAddress => {}
            _ => {
                //address of the result for the Fr operations
                instructions.push(get_local(producer.get_expaux_tag()));
                let size = self.op_aux_no * producer.get_size_32_bits_in_memory() * 4;
                instructions.push(set_constant(&size.to_string()));
                instructions.push(add32());
            }
        }
        for e in &self.stack {
            let mut instructions_exp = e.produce_wasm(producer);
            instructions.append(&mut instructions_exp);
        }
        if producer.needs_comments() {
            instructions.push(format!(";; OP({})", self.op.to_string()));
	}
        match &self.op {
            OperatorType::AddAddress => {
                instructions.push(add32());
            }
            OperatorType::MulAddress => {
                instructions.push(mul32());
            }
            OperatorType::ToAddress => {
                instructions.push(call("$Fr_toInt"));
            }
            _ => {
                match self.op {
                    OperatorType::Add => {
                        instructions.push(call("$Fr_add")); // Result, Argument, Argument
                    }
                    OperatorType::Div => {
                        instructions.push(call("$Fr_div")); // Result, Argument, Argument
                    }
                    OperatorType::Mul => {
                        instructions.push(call("$Fr_mul")); // Result, Argument, Argument
                    }
                    OperatorType::Sub => {
                        instructions.push(call("$Fr_sub")); // Result, Argument, Argument
                    }
                    OperatorType::Pow => {
                        instructions.push(call("$Fr_pow"));
                    }
                    OperatorType::IntDiv => {
                        instructions.push(call("$Fr_idiv"));
                    }
                    OperatorType::Mod => {
                        instructions.push(call("$Fr_mod"));
                    }
                    OperatorType::ShiftL => {
                        instructions.push(call("$Fr_shl"));
                    }
                    OperatorType::ShiftR => {
                        instructions.push(call("$Fr_shr"));
                    }
                    OperatorType::LesserEq => {
                        instructions.push(call("$Fr_leq"));
                    }
                    OperatorType::GreaterEq => {
                        instructions.push(call("$Fr_geq"));
                    }
                    OperatorType::Lesser => {
                        instructions.push(call("$Fr_lt"));
                    }
                    OperatorType::Greater => {
                        instructions.push(call("$Fr_gt"));
                    }
                    OperatorType::Eq(n) => {
			assert!(n != 0);
			if n == 1 {
                            instructions.push(call("$Fr_eq"));
                        } else {
                            instructions.push(set_local(producer.get_aux_2_tag()));
			    instructions.push(set_local(producer.get_aux_1_tag()));
			    instructions.push(set_local(producer.get_aux_0_tag()));
                            instructions.push(set_constant(&n.to_string()));
                            instructions.push(set_local(producer.get_counter_tag()));
                            instructions.push(add_block());
                            instructions.push(add_loop());
                            instructions.push(get_local(producer.get_aux_0_tag()));
                            instructions.push(get_local(producer.get_aux_1_tag()));
                            instructions.push(get_local(producer.get_aux_2_tag()));
                            instructions.push(call("$Fr_eq"));
                            instructions.push(get_local(producer.get_aux_0_tag()));
                            instructions.push(call("$Fr_isTrue"));
                            instructions.push(eqz32());
			    instructions.push(br_if("1"));
                            instructions.push(get_local(producer.get_counter_tag()));
                            instructions.push(set_constant("1"));
                            instructions.push(sub32());
                            instructions.push(tee_local(producer.get_counter_tag()));
                            instructions.push(eqz32());
                            instructions.push(br_if("1"));
                            instructions.push(get_local(producer.get_aux_1_tag()));
                            let s = producer.get_size_32_bits_in_memory() * 4;
                            instructions.push(set_constant(&s.to_string()));
                            instructions.push(add32());
                            instructions.push(set_local(producer.get_aux_1_tag()));
                            instructions.push(get_local(producer.get_aux_2_tag()));
                            instructions.push(set_constant(&s.to_string()));
                            instructions.push(add32());
                            instructions.push(set_local(producer.get_aux_2_tag()));
                            instructions.push(br("0"));
                            instructions.push(add_end());
                            instructions.push(add_end());
                        }
                    }
                    OperatorType::NotEq => {
                        instructions.push(call("$Fr_neq"));
                    }
                    OperatorType::BoolOr => {
                        instructions.push(call("$Fr_lor"));
                    }
                    OperatorType::BoolAnd => {
                        instructions.push(call("$Fr_land"));
                    }
                    OperatorType::BitOr => {
                        instructions.push(call("$Fr_bor"));
                    }
                    OperatorType::BitAnd => {
                        instructions.push(call("$Fr_band"));
                    }
                    OperatorType::BitXor => {
                        instructions.push(call("$Fr_bxor"));
                    }
                    OperatorType::PrefixSub => {
                        instructions.push(call("$Fr_neg"));
                    }
                    OperatorType::BoolNot => {
                        instructions.push(call("$Fr_lnot"));
                    }
                    OperatorType::Complement => {
                        instructions.push(call("$Fr_bnot"));
                    }
                    _ => (), //$Fr_inv? Does not exists
                }
                instructions.push(get_local(producer.get_expaux_tag()));
                let size = self.op_aux_no * producer.get_size_32_bits_in_memory() * 4;
                instructions.push(set_constant(&size.to_string()));
                instructions.push(add32());
            }
        }
        if producer.needs_comments() {
            instructions.push(";; end of compute bucket".to_string());
	}
        instructions
    }
}

impl WriteC for ComputeBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::*;
        fn get_fr_op(op_type: OperatorType) -> String {
            match op_type {
                OperatorType::Add => "Fr_add".to_string(),
                OperatorType::Div => "Fr_div".to_string(),
                OperatorType::Mul => "Fr_mul".to_string(),
                OperatorType::Sub => "Fr_sub".to_string(),
                OperatorType::Pow => "Fr_pow".to_string(),
                OperatorType::IntDiv => "Fr_idiv".to_string(),
                OperatorType::Mod => "Fr_mod".to_string(),
                OperatorType::ShiftL => "Fr_shl".to_string(),
                OperatorType::ShiftR => "Fr_shr".to_string(),
                OperatorType::LesserEq => "Fr_leq".to_string(),
                OperatorType::GreaterEq => "Fr_geq".to_string(),
                OperatorType::Lesser => "Fr_lt".to_string(),
                OperatorType::Greater => "Fr_gt".to_string(),
                OperatorType::Eq(_) => "Fr_eq".to_string(),
                OperatorType::NotEq => "Fr_neq".to_string(),
                OperatorType::BoolOr => "Fr_lor".to_string(),
                OperatorType::BoolAnd => "Fr_land".to_string(),
                OperatorType::BitOr => "Fr_bor".to_string(),
                OperatorType::BitAnd => "Fr_band".to_string(),
                OperatorType::BitXor => "Fr_bxor".to_string(),
                OperatorType::PrefixSub => "Fr_neg".to_string(),
                OperatorType::BoolNot => "Fr_lnot".to_string(),
                OperatorType::Complement => "Fr_bnot".to_string(),
                _ => unreachable!(),
            }
        }

        let mut compute_c = vec![];
        let mut operands = vec![];

        let result;
        for instr in &self.stack {
            let (mut instr_c, operand) = instr.produce_c(producer, parallel);
            operands.push(operand);
            compute_c.append(&mut instr_c);
        }
        match &self.op {
            OperatorType::AddAddress => {
                result = format!("({} + {})", operands[0], operands[1]);
            }
            OperatorType::MulAddress => {
                result = format!("({} * {})", operands[0], operands[1]);
            }
            OperatorType::ToAddress => {
                result = build_call("Fr_toInt".to_string(), operands);
            }

            OperatorType::Eq(n) => {
                let exp_aux_index = self.op_aux_no.to_string();
                let operator = get_fr_op(self.op);
                let result_ref = format!("&{}", expaux(exp_aux_index.clone()));
                let mut arguments = vec![result_ref.clone()];
                let operands_copy = operands.clone();
                arguments.append(&mut operands);
                compute_c.push(format!("{}; // line circom {}", build_call(operator.clone(), arguments),self.line.to_string()));
                if *n > 1 {
                    compute_c.push(format!("uint index = 1;"));
                    compute_c.push(format!("while(index < {} && Fr_isTrue({})) {{", n, result_ref));
                    operands = vec![];
                    arguments = vec![result_ref.clone()];
                    for operand in &operands_copy {
                        operands.push(format!("{} + index", operand));
                    }
                    arguments.append(&mut operands);
                    compute_c.push(format!("{}; // line circom {}", build_call(operator.clone(), arguments),self.line.to_string()));
                    compute_c.push(format!("index++;"));
                    compute_c.push(format!("}}"));
                    
                }
                result = result_ref;

                
            }

            _ => {
                let exp_aux_index = self.op_aux_no.to_string();
                // build assign
                let operator = get_fr_op(self.op);
                let result_ref = format!("&{}", expaux(exp_aux_index.clone()));
                let mut arguments = vec![result_ref.clone()];
                arguments.append(&mut operands);
                compute_c.push(format!("{}; // line circom {}", build_call(operator, arguments),self.line.to_string()));

                //value address
                result = result_ref;
            }
        }
	//compute_c.push(format!("// end of compute with result {}",result));
        (compute_c, result)
    }
}
