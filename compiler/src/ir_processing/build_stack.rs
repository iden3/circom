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
        Load(b) => build_load(b, fresh).0,
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
            let (v_0, _) = build_address_type(&mut data.dest_address_type, fresh);
            let (v_1, _) = build_location(&mut data.dest, fresh);
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

// returns the depth and the updated fresh variable to be used in the rest of the expression
pub fn build_instruction_compute(instruction: &mut Instruction, fresh: usize) ->(usize, usize){
    use Instruction::*;
    match instruction {
        Compute(b) =>
             (build_compute(b, fresh), fresh + 1), // needs 1 expaux to store the result
        Load(b) => 
            build_load(b, fresh), // returns the number of expaux needed
        Value(b) => 
            (build_value(b, fresh), fresh + 1), // needs 1 expaux to store the result
        _ => unreachable!(), // only possible instructions inside a compute
    }
}


pub fn build_compute(bucket: &mut ComputeBucket, mut fresh: usize) -> usize {
    
    if bucket.op.is_address_op(){
        println!("Bucket: {}", bucket.to_string());
        unreachable!(); // just to check that addresses do not enter here
    }

    bucket.op_aux_no = fresh;
    fresh += 1;
    let mut max_stack = fresh;

    for i in &mut bucket.stack {
        let (depth, new_fresh) = build_instruction_compute(i, fresh);
        max_stack = std::cmp::max(max_stack, depth);
        fresh = new_fresh;
    }
    max_stack
}



pub fn build_load(bucket: &mut LoadBucket, fresh: usize) -> (usize, usize) {
    let (_v0, f0) = build_address_type(&mut bucket.address_type, fresh);
    let (v1, f1) = build_location(&mut bucket.src, f0);
    (v1, f1)
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
    let mut in_log = 0;
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
    let (f_1, _) = build_location(&mut bucket.dest, fresh);
    let (f_2, _) = build_address_type(&mut bucket.dest_address_type, fresh);
    std::cmp::max(std::cmp::max(f_0, f_1), f_2)
}

pub fn build_value(bucket: &mut ValueBucket, fresh: usize) -> usize {
    bucket.op_aux_no = fresh;
    fresh + 1
}

pub fn build_location(bucket: &mut LocationRule, mut fresh: usize) -> (usize, usize) {
    use LocationRule::*;
    match bucket {
        Indexed { location, .. } => build_instruction_address(location, fresh),
        Mapped { indexes, .. } => {
            let mut max_stack = fresh;
            for acc in indexes{
                match acc{
                    AccessType::Indexed(ind) =>{
                        for i in &mut ind.indexes{
                            let (depth, new_fresh) = build_instruction_address(i, fresh);
                            max_stack = std::cmp::max(max_stack, depth);
                            fresh = new_fresh;
                        }
                    },
                    AccessType::Qualified(_) =>{

                    }
                }
                
            }
            (max_stack, fresh)
        }
             
    }
}

pub fn build_address_type(xtype: &mut AddressType, mut fresh: usize) -> (usize, usize) {
    use AddressType::*;
    let mut max = fresh;
    if let SubcmpSignal { cmp_address, .. } = xtype {
        let (cmp_stack, new_fresh) = build_instruction_address(cmp_address, fresh);
        max = std::cmp::max(max, cmp_stack);
        fresh = new_fresh
    }
    (max, fresh)
}

//////////////////////////////////////////////////////////////////////////
////////////////// INSTRUCTIONS FOR ADDRESSES OPERATIONS /////////////////
//////////////////////////////////////////////////////////////////////////


// returns the depth and the updated fresh variable to be used in the rest of the expression
pub fn build_instruction_address(instruction: &mut Instruction, fresh: usize) ->(usize, usize){
    use Instruction::*;
    match instruction {
        Instruction::Compute(b) => {
            build_compute_address(b, fresh)
        }
        Value(_) =>{
            // we do not need to update the stack and fresh
            (0, fresh)
        }
        _ => unreachable!(),
    }
}

pub fn build_compute_address(bucket: &mut ComputeBucket, mut fresh: usize) -> (usize, usize) {
    use crate::ir_processing::build_stack::OperatorType::{AddAddress, MulAddress, ToAddress};
    let mut max_stack = fresh;
    if bucket.op == AddAddress || bucket.op == MulAddress{ // in case it is ADD or MUL address
        for i in &mut bucket.stack{
            let (depth, new_fresh) = build_instruction_address(i, fresh);
            max_stack = std::cmp::max(max_stack, depth);
            fresh = new_fresh;
        }
    } else if bucket.op == ToAddress{
        for i in &mut bucket.stack{
            let (depth, new_fresh) = build_instruction_compute(i, fresh);
            max_stack = std::cmp::max(max_stack, depth);
            fresh = new_fresh;
        }
    } else{
        unreachable!() // just to check that fr do not enter here
    }
    (max_stack, fresh)
    
}