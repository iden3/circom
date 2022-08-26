use crate::intermediate_representation::ir_interface::*;
use std::collections::HashMap;

pub fn visit_list(
    instructions: &mut InstructionList,
    function_to_arena_size: &HashMap<String, usize>,
) {
    for i in instructions {
        visit_instruction(i, function_to_arena_size);
    }
}

pub fn visit_instruction(
    instruction: &mut Instruction,
    function_to_arena_size: &HashMap<String, usize>,
) {
    use Instruction::*;
    match instruction {
        Branch(b) => visit_branch(b, function_to_arena_size),
        Call(b) => visit_call(b, function_to_arena_size),
        Compute(b) => visit_compute(b, function_to_arena_size),
        Load(b) => visit_load(b, function_to_arena_size),
        Loop(b) => visit_loop(b, function_to_arena_size),
        Return(b) => visit_return(b, function_to_arena_size),
        Store(b) => visit_store(b, function_to_arena_size),
        Value(b) => visit_value(b, function_to_arena_size),
        Assert(b) => visit_assert(b, function_to_arena_size),
        CreateCmp(b) => visit_create_cmp(b, function_to_arena_size),
        Log(b) => visit_log(b, function_to_arena_size),
    }
}

pub fn visit_branch(bucket: &mut BranchBucket, function_to_arena_size: &HashMap<String, usize>) {
    visit_instruction(&mut bucket.cond, function_to_arena_size);
    visit_list(&mut bucket.if_branch, function_to_arena_size);
    visit_list(&mut bucket.else_branch, function_to_arena_size);
}

pub fn visit_call(bucket: &mut CallBucket, function_to_arena_size: &HashMap<String, usize>) {
    use ReturnType::*;
    bucket.arena_size = *function_to_arena_size.get(&bucket.symbol).unwrap();
    if let Final(data) = &mut bucket.return_info {
        visit_address_type(&mut data.dest_address_type, function_to_arena_size);
        visit_location(&mut data.dest, function_to_arena_size);
    }

    for i in &mut bucket.arguments {
        visit_instruction(i, function_to_arena_size)
    }
}

pub fn visit_compute(bucket: &mut ComputeBucket, function_to_arena_size: &HashMap<String, usize>) {
    for i in &mut bucket.stack {
        visit_instruction(i, function_to_arena_size);
    }
}

pub fn visit_load(bucket: &mut LoadBucket, function_to_arena_size: &HashMap<String, usize>) {
    visit_location(&mut bucket.src, function_to_arena_size);
    visit_address_type(&mut bucket.address_type, function_to_arena_size);
}

pub fn visit_create_cmp(
    bucket: &mut CreateCmpBucket,
    function_to_arena_size: &HashMap<String, usize>,
) {
    visit_instruction(&mut bucket.sub_cmp_id, function_to_arena_size);
}

pub fn visit_loop(bucket: &mut LoopBucket, function_to_arena_size: &HashMap<String, usize>) {
    visit_instruction(&mut bucket.continue_condition, function_to_arena_size);
    visit_list(&mut bucket.body, function_to_arena_size);
}

pub fn visit_return(bucket: &mut ReturnBucket, function_to_arena_size: &HashMap<String, usize>) {
    visit_instruction(&mut bucket.value, function_to_arena_size);
}

pub fn visit_log(bucket: &mut LogBucket, function_to_arena_size: &HashMap<String, usize>) {
    for print in bucket.argsprint.clone() {
        if let LogBucketArg::LogExp(mut exp) = print { 
            visit_instruction(&mut exp, function_to_arena_size);
        }
    }
    
}

pub fn visit_assert(bucket: &mut AssertBucket, function_to_arena_size: &HashMap<String, usize>) {
    visit_instruction(&mut bucket.evaluate, function_to_arena_size);
}

pub fn visit_store(bucket: &mut StoreBucket, function_to_arena_size: &HashMap<String, usize>) {
    visit_instruction(&mut bucket.src, function_to_arena_size);
    visit_location(&mut bucket.dest, function_to_arena_size);
    visit_address_type(&mut bucket.dest_address_type, function_to_arena_size);
}

pub fn visit_value(_: &mut ValueBucket, _: &HashMap<String, usize>) {}

pub fn visit_location(bucket: &mut LocationRule, function_to_arena_size: &HashMap<String, usize>) {
    use LocationRule::*;
    match bucket {
        Indexed { location, .. } => visit_instruction(location, function_to_arena_size),
        Mapped { indexes, .. } => visit_list(indexes, function_to_arena_size),
    }
}

pub fn visit_address_type(
    xtype: &mut AddressType,
    function_to_arena_size: &HashMap<String, usize>,
) {
    use AddressType::*;
    if let SubcmpSignal { cmp_address, .. } = xtype {
        visit_instruction(cmp_address, function_to_arena_size);
    }
}
