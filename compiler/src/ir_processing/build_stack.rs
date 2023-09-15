use crate::intermediate_representation::ir_interface::*;

pub fn build_list(instructions: &mut InstructionList, fresh: usize) -> usize {
    let mut max_depth = 0;
    for i in instructions {
        let depth = build_instruction(i, fresh);
        max_depth = std::cmp::max(max_depth, depth);
    }
    max_depth
}

pub fn build_instruction(instruction: &mut Instruction, fresh: usize) -> usize {
    use Instruction::*;
    match instruction {
        Branch(b) => build_branch(b, fresh),
        Call(b) => build_call(b, fresh),
        Compute(b) => build_compute(b, fresh),
        Load(b) => build_load(b, fresh),
        Loop(b) => build_loop(b, fresh),
        Return(b) => build_return(b, fresh),
        Store(b) => build_store(b, fresh),
        Value(b) => build_value(b, fresh),
        Assert(b) => build_assert(b, fresh),
        CreateCmp(b) => build_create_cmp(b, fresh),
        Log(b) => build_log(b, fresh),
    }
}

pub fn build_branch(bucket: &mut BranchBucket, fresh: usize) -> usize {
    let in_cond = build_instruction(&mut bucket.cond, fresh);
    let in_if = build_list(&mut bucket.if_branch, fresh);
    let in_else = build_list(&mut bucket.else_branch, fresh);
    std::cmp::max(in_cond, std::cmp::max(in_if, in_else))
}

pub fn build_call(bucket: &mut CallBucket, mut fresh: usize) -> usize {
    use ReturnType::*;
    let mut max_stack = fresh;
    match &mut bucket.return_info {
        Intermediate { op_aux_no } => {
            *op_aux_no = fresh;
            fresh += 1;
        }
        Final(data) => {
            let v_0 = build_address_type(&mut data.dest_address_type, fresh);
            let v_1 = build_location(&mut data.dest, fresh);
            max_stack = std::cmp::max(v_0, v_1);
        }
    }
    for i in &mut bucket.arguments {
        fresh += 1;
        let depth = build_instruction(i, fresh);
        max_stack = std::cmp::max(max_stack, depth);
    }
    max_stack
}

pub fn build_compute(bucket: &mut ComputeBucket, mut fresh: usize) -> usize {
    use crate::ir_processing::build_stack::OperatorType::{AddAddress, MulAddress};
    let consumes = if bucket.op.is_address_op() { 0 } else { 1 };
    bucket.op_aux_no = if bucket.op.is_address_op() { 0 } else { fresh };
    let mut max_stack = fresh + consumes;
    for i in &mut bucket.stack {
        fresh += consumes;
        let depth = build_instruction(i, fresh);
        max_stack = std::cmp::max(max_stack, depth);

        // in case it is an addition or multiplication between addresses the number of new fresh vars is the number of ToAddress inside the operand
        if bucket.op == AddAddress || bucket.op == MulAddress{
            fresh += get_num_to_address_inside_compute_address(i);
        }
    }
    max_stack
}

pub fn build_load(bucket: &mut LoadBucket, fresh: usize) -> usize {
    let v0 = build_address_type(&mut bucket.address_type, fresh);
    let v1 = build_location(&mut bucket.src, v0);
    v1
}

pub fn build_create_cmp(bucket: &mut CreateCmpBucket, fresh: usize) -> usize {
    build_instruction(&mut bucket.sub_cmp_id, fresh)
}

pub fn build_loop(bucket: &mut LoopBucket, fresh: usize) -> usize {
    let in_cond = build_instruction(&mut bucket.continue_condition, fresh);
    let in_body = build_list(&mut bucket.body, fresh);
    std::cmp::max(in_cond, in_body)
}

pub fn build_return(bucket: &mut ReturnBucket, fresh: usize) -> usize {
    build_instruction(&mut bucket.value, fresh)
}

pub fn build_log(bucket: &mut LogBucket, fresh: usize) -> usize {
    let mut in_log = usize::min_value();
    for arglog in &mut bucket.argsprint {
        match arglog {
            LogBucketArg::LogExp(_) => {
                let new_log = build_instruction(arglog.get_mut_arg_logexp(), fresh);
                in_log = std::cmp::max(in_log, new_log);
            }
            LogBucketArg::LogStr(..) => {}
        }
    }
    in_log
}

pub fn build_assert(bucket: &mut AssertBucket, fresh: usize) -> usize {
    build_instruction(&mut bucket.evaluate, fresh)
}

pub fn build_store(bucket: &mut StoreBucket, fresh: usize) -> usize {
    let f_0 = build_instruction(&mut bucket.src, fresh);
    let f_1 = build_location(&mut bucket.dest, fresh);
    let f_2 = build_address_type(&mut bucket.dest_address_type, fresh);
    std::cmp::max(std::cmp::max(f_0, f_1), f_2)
}

pub fn build_value(bucket: &mut ValueBucket, fresh: usize) -> usize {
    bucket.op_aux_no = fresh;
    fresh + 1
}

pub fn build_location(bucket: &mut LocationRule, fresh: usize) -> usize {
    use LocationRule::*;
    match bucket {
        Indexed { location, .. } => build_instruction(location, fresh),
        Mapped { indexes, .. } => build_list(indexes, fresh),
    }
}

pub fn build_address_type(xtype: &mut AddressType, fresh: usize) -> usize {
    use AddressType::*;
    let mut max = fresh;
    if let SubcmpSignal { cmp_address, .. } = xtype {
        let cmp_stack = build_instruction(cmp_address, fresh);
        max = std::cmp::max(max, cmp_stack);
    }
    max
}


pub fn get_num_to_address_inside_compute_address(instruction: &Instruction) -> usize {
    use Instruction::*;
    match instruction {
        Compute(b) =>{
            match b.op{
                OperatorType::ToAddress => 1,
                OperatorType::AddAddress | OperatorType::MulAddress{} =>{
                    let mut num_to_address = 0;
                    for i in &b.stack{
                        num_to_address += get_num_to_address_inside_compute_address(i);
                    }
                    num_to_address
                },
                _ => unreachable!(),
            }
        },
        Value(_) => 0,
        _ => unreachable!()
    }
}