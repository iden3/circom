use crate::intermediate_representation::ir_interface::*;
use std::collections::{HashSet};

type ComponentsSet = HashSet<String>;


pub fn visit_list(
    instructions: &mut InstructionList, 
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet, 
    found_unknown_address: bool,
    inside_loop: bool
)-> bool {
    let len_instructions = instructions.len();
    let mut found_unkown_aux = found_unknown_address;
    for i in 0..instructions.len(){
        found_unkown_aux = visit_instruction(
            &mut instructions[len_instructions - 1 - i],
            known_last_component,
            unknown_last_component, 
            found_unkown_aux,
            inside_loop
        );
    }
    found_unkown_aux
}

pub fn visit_instruction(
    instruction: &mut  Instruction, 
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet, 
    found_unknown_address: bool,
    inside_loop: bool
) ->bool {
    use Instruction::*;
    match instruction {
        Branch(b) => visit_branch(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Call(b) => visit_call(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Compute(b) => visit_compute(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Load(b) => visit_load(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Loop(b) => visit_loop(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Return(b) => visit_return(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Store(b) => visit_store(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Value(b) => visit_value(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Assert(b) => visit_assert(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        CreateCmp(b) => visit_create_cmp(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
        Log(b) => visit_log(b, known_last_component, unknown_last_component, found_unknown_address, inside_loop),
    }
}

pub fn visit_branch(
    bucket: &mut BranchBucket,  
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    inside_loop: bool
) -> bool {
  let mut known_last_component_if: ComponentsSet = known_last_component.clone();
    let mut known_last_component_else: ComponentsSet = known_last_component.clone();
    let mut unknown_last_component_if: ComponentsSet = unknown_last_component.clone();
    let mut unknown_last_component_else: ComponentsSet = unknown_last_component.clone();

    let found_unknown_if :bool = visit_list(
        &mut bucket.if_branch, 
        &mut known_last_component_if, 
        &mut unknown_last_component_if, 
        found_unknown_address,
        inside_loop
    );
    let found_unknown_else :bool = visit_list(
        &mut bucket.if_branch, 
        &mut known_last_component_else, 
        &mut unknown_last_component_else, 
        found_unknown_address,
        inside_loop
    );

    let known_component_both_branches: ComponentsSet = known_last_component_if.intersection(& known_last_component_else).map(|s| s.clone()).collect();
    let known_component_one_branch: ComponentsSet = known_last_component_if.symmetric_difference(&known_last_component_else).map(|s| s.clone()).collect();

    let mut new_unknown_component: ComponentsSet = unknown_last_component_if.union(&unknown_last_component_else).map(|s| s.clone()).collect();
    new_unknown_component = new_unknown_component.union(&known_component_one_branch).map(|s| s.clone()).collect(); 

    let joined_unknown_component: ComponentsSet = unknown_last_component.union(&new_unknown_component).map(|s| s.clone()).collect();

    *known_last_component = known_last_component.union(&known_component_both_branches).map(|s| s.clone()).collect();
    *unknown_last_component =  joined_unknown_component.difference(&known_component_both_branches).map(|s| s.clone()).collect();
    found_unknown_if || found_unknown_else
}

pub fn visit_call(
    bucket: &mut  CallBucket, 
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    inside_loop: bool
)-> bool {
    use ReturnType::*;

    if let Final(data) = &mut bucket.return_info {
        visit_address_type(
            &mut data.dest_address_type, 
            known_last_component,
            unknown_last_component,
            found_unknown_address,
            inside_loop
        )
    }
    else{
        found_unknown_address
    }
}

pub fn visit_compute(
    _bucket: &mut ComputeBucket, 
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}
pub fn visit_load(
    _bucket: &mut LoadBucket, 
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}

pub fn visit_loop(
    bucket: &mut LoopBucket,
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
)-> bool{
    let mut known_last_component_loop: ComponentsSet = known_last_component.clone();
    let mut unknown_last_component_loop: ComponentsSet = unknown_last_component.clone();

    let found_unknown_address_new  = visit_list(
        &mut bucket.body, 
        &mut known_last_component_loop, 
        &mut unknown_last_component_loop, 
        found_unknown_address,
        true
    );

    let new_unknown_component: ComponentsSet = known_last_component_loop.union(&unknown_last_component_loop).map(|s| s.clone()).collect();

    *known_last_component = known_last_component.difference(&new_unknown_component).map(|s| s.clone()).collect();
    *unknown_last_component = unknown_last_component.union(&new_unknown_component).map(|s| s.clone()).collect();
    found_unknown_address_new
}

pub fn visit_create_cmp(
    _bucket: &mut CreateCmpBucket,
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}

pub fn visit_return(
    _bucket: &mut ReturnBucket,     
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}

pub fn visit_log(
    _bucket: &mut LogBucket,
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}

pub fn visit_assert(_bucket: &mut AssertBucket,
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}

pub fn visit_value(_bucket: &mut ValueBucket,
    _known_last_component: &mut ComponentsSet, 
    _unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    _inside_loop: bool
) -> bool{
    found_unknown_address
}



pub fn visit_store(
    bucket: &mut StoreBucket,
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    inside_loop: bool
)-> bool{
    visit_address_type(
        &mut bucket.dest_address_type, 
        known_last_component,
        unknown_last_component,
        found_unknown_address,
        inside_loop
    )
}


pub fn visit_address_type(
    xtype: &mut AddressType,
    known_last_component: &mut ComponentsSet, 
    unknown_last_component: &mut ComponentsSet,  
    found_unknown_address: bool,
    inside_loop: bool
) -> bool {
    use AddressType::*;
    use InputInformation::*;
    use StatusInput::*;
    use Instruction::*;

    if let SubcmpSignal { cmp_address, input_information ,..} = xtype {
        if let Input {..} = input_information{

            if known_last_component.contains(&cmp_address.to_string()){
                *input_information = Input{status: NoLast};
                //println!("Poniendo un nolast en {}", cmp_address.to_string());
                found_unknown_address
            }
            else if unknown_last_component.contains(&cmp_address.to_string()){
                *input_information = Input{status: Unknown};
                //println!("Poniendo un unknown en {}", cmp_address.to_string());
                found_unknown_address
            }
            else{
                if let Value {..} = **cmp_address{
                    if found_unknown_address{
                        *input_information = Input{status: Unknown};
                        //println!("Poniendo un unknown en {}", cmp_address.to_string());
                    }
                    else{
                        if inside_loop {
                            *input_information = Input{status: Unknown};
                            //println!("Poniendo un unknown en {}", cmp_address.to_string());
                        }
                        else{
                            *input_information = Input{status: Last};
                            //println!("Poniendo un last en {}", cmp_address.to_string());
                        }
                    }
                    known_last_component.insert(cmp_address.to_string());
                    unknown_last_component.remove(&cmp_address.to_string());
                    found_unknown_address
                }
                else{
                    *input_information = Input{status: Unknown};
                    //println!("Poniendo un unknown en {}", cmp_address.to_string());
                    false
                }
            }
        }
        else{
            found_unknown_address
        }
    }
    else{
        found_unknown_address
    }
}




