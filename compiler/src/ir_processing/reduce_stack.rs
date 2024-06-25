use crate::intermediate_representation::ir_interface::*;

pub fn reduce_list(list: InstructionList) -> InstructionList {
    let mut reduced = InstructionList::with_capacity(InstructionList::len(&list));
    for instr in list {
        InstructionList::push(&mut reduced, Allocate::allocate(reduce_instruction(*instr)));
    }
    reduced
}

pub fn reduce_instruction(instr: Instruction) -> Instruction {
    use Instruction::*;
    match instr {
        Value(b) => IntoInstruction::into_instruction(b),
        Load(b) => reduce_load(b),
        Store(b) => reduce_store(b),
        Call(b) => reduce_call(b),
        Branch(b) => reduce_branch(b),
        Return(b) => reduce_return(b),
        Assert(b) => reduce_assert(b),
        Log(b) => reduce_log(b),
        Loop(b) => reduce_loop(b),
        CreateCmp(b) => reduce_crt_cmp(b),
        Compute(b) => reduce_compute(b),
    }
}

pub fn reduce_compute(mut bucket: ComputeBucket) -> Instruction {
    use OperatorType::*;
    bucket.stack = reduce_list(bucket.stack);
    if !bucket.op.is_address_op() || bucket.op == ToAddress { 
        return IntoInstruction::into_instruction(bucket);
    }
    
    let op0 = *bucket.stack[0].clone();
    let op1 = *bucket.stack[1].clone();
    let res = reduce_operands(op0, op1)
    .map(|(a, b)| match bucket.op {
        MulAddress => a * b,
        AddAddress => a + b,
        _ => unreachable!()
    });
    if let Some(value) = res {
        let v_bucket = ValueBucket {
            line: bucket.line,
            message_id: bucket.message_id,
            parse_as: ValueType::U32,
            op_aux_no: bucket.op_aux_no,
            value,
        };
        IntoInstruction::into_instruction(v_bucket)
    } else {
        IntoInstruction::into_instruction(bucket)
    }
}

pub fn reduce_operands(op0: Instruction, op1: Instruction) -> Option<(usize, usize)> {
    use Instruction::Value;
    match (op0, op1) {
        (Value(op0), Value(op1)) => match (op0.parse_as, op1.parse_as) {
            (ValueType::U32, ValueType::U32) => {
                let v0 = op0.value;
                let v1 = op1.value;
                Some((v0, v1))
            }
            _ => None,
        },
        _ => None,
    }
}

pub fn reduce_crt_cmp(mut bucket: CreateCmpBucket) -> Instruction {
    bucket.sub_cmp_id = Allocate::allocate(reduce_instruction(*bucket.sub_cmp_id));
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_loop(mut bucket: LoopBucket) -> Instruction {
    bucket.continue_condition = Allocate::allocate(reduce_instruction(*bucket.continue_condition));
    bucket.body = reduce_list(bucket.body);
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_log(mut bucket: LogBucket) -> Instruction {
    let mut new_args_prints : Vec<LogBucketArg> = Vec::new();
    for print in bucket.argsprint {
        match print {
            LogBucketArg::LogExp(exp)=> {
                let print_aux = Allocate::allocate(reduce_instruction(*exp));
                new_args_prints.push(LogBucketArg::LogExp(print_aux));

            },
            LogBucketArg::LogStr(s) => {
                new_args_prints.push(LogBucketArg::LogStr(s));
            },
        }
        
    }
    
    bucket.argsprint = new_args_prints;
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_assert(mut bucket: AssertBucket) -> Instruction {
    bucket.evaluate = Allocate::allocate(reduce_instruction(*bucket.evaluate));
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_return(mut bucket: ReturnBucket) -> Instruction {
    bucket.value = Allocate::allocate(reduce_instruction(*bucket.value));
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_branch(mut bucket: BranchBucket) -> Instruction {
    bucket.cond = Allocate::allocate(reduce_instruction(*bucket.cond));
    bucket.if_branch = reduce_list(bucket.if_branch);
    bucket.else_branch = reduce_list(bucket.else_branch);
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_load(mut bucket: LoadBucket) -> Instruction {
    bucket.address_type = reduce_address_type(bucket.address_type);
    bucket.src = reduce_location_rule(bucket.src);
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_store(mut bucket: StoreBucket) -> Instruction {
    bucket.dest_address_type = reduce_address_type(bucket.dest_address_type);
    bucket.dest = reduce_location_rule(bucket.dest);
    bucket.src = Allocate::allocate(reduce_instruction(*bucket.src));
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_call(mut bucket: CallBucket) -> Instruction {
    bucket.arguments = reduce_list(bucket.arguments);
    if let ReturnType::Final(mut data) = bucket.return_info {
        data.dest = reduce_location_rule(data.dest);
        data.dest_address_type = reduce_address_type(data.dest_address_type);
        bucket.return_info = ReturnType::Final(data);
    }
    IntoInstruction::into_instruction(bucket)
}

pub fn reduce_address_type(at: AddressType) -> AddressType {
    use AddressType::*;
    match at {
        Variable => Variable,
        Signal => Signal,
        SubcmpSignal { cmp_address, uniform_parallel_value, is_output, input_information } => {
            let cmp_address = Allocate::allocate(reduce_instruction(*cmp_address));
            SubcmpSignal { cmp_address, uniform_parallel_value, is_output, input_information }
        }
    }
}

pub fn reduce_location_rule(lc: LocationRule) -> LocationRule {
    use LocationRule::*;
    match lc {
        Indexed { location, template_header } => {
            let location = Allocate::allocate(reduce_instruction(*location));
            Indexed { location, template_header }
        }
        Mapped { signal_code, indexes } => {
            let no_indexes = InstructionList::len(&indexes);
            let work = indexes;
            let mut indexes = InstructionList::with_capacity(no_indexes);
            for index in work {
                let index = Allocate::allocate(reduce_instruction(*index));
                InstructionList::push(&mut indexes, index);
            }
            Mapped { signal_code, indexes }
        }
    }
}
