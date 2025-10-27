use crate::intermediate_representation::ir_interface::*;
use std::collections::{HashSet, HashMap};

type ComponentsSet = HashSet<String>;

pub struct LastInfo{
    needs_decrement: bool,
    found_last: bool,
}

type LastInfoMap = HashMap<String, LastInfo>;


// We store the following information:
// Components where we have found the last assignment and we do not need to decrement
//// Flag -> Last, NoLast
//// Flag -> Decrement: false

// Components where there are multiple possible last assignments -> we need decrement
//// - If there is a loop or if/else where there is one branch with no assignments
//// - If there is an unknown assigment -> affects to all components that might be assigned
//// All possible lasts -> Unknown, NoLast
//// Flag -> Decrement: true
 
// HashMap: ComponentStates -> true if sure assigned, false otherwise 
// HashSet: AssignedUnknownComponent: the names of the components where we do not know 
//          which one has been assigned -> HashSet<String>



pub fn visit_list(
    instructions: &mut InstructionList, 
    assigment_status: &mut LastInfoMap, 
    unknown_component_names: &mut ComponentsSet, 
    inside_loop: bool
) {
    let mut assignments_level = HashSet::new();
    let len_instructions = instructions.len();
    for i in 0..len_instructions{
        visit_instruction(
            &mut instructions[len_instructions - 1 - i],
            assigment_status,
            &mut assignments_level,
            unknown_component_names, 
            inside_loop
        );
    }
}

pub fn visit_instruction(
    instruction: &mut Instruction, 
    assignment_status: &mut LastInfoMap, 
    assignments_level: &mut ComponentsSet,
    unknown_component_names: &mut ComponentsSet, 
    inside_loop: bool
) {
    use Instruction::*;
    match instruction {
        Branch(b) => visit_branch(b, assignment_status, unknown_component_names),
        Call(b) => visit_call(b, assignment_status, assignments_level, unknown_component_names, inside_loop),
        Loop(b) => visit_loop(b, assignment_status, unknown_component_names),
        Store(b) => visit_store(b, assignment_status, assignments_level, unknown_component_names, inside_loop),
        _ => {} // in all other cases we do not need to visit the instructions
    }
}

pub fn visit_branch(
    bucket: &mut BranchBucket,  
    assignment_status: &mut LastInfoMap, 
    unknown_component_names: &mut ComponentsSet, 
) {
    
    // Visit each one of the branches
    visit_list(&mut bucket.if_branch, assignment_status, unknown_component_names, true);
    visit_list(&mut bucket.else_branch, assignment_status, unknown_component_names, true);
    
}

pub fn visit_call(
    bucket: &mut  CallBucket, 
    assignment_status: &mut LastInfoMap, 
    assignments_level: &mut ComponentsSet,
    unknown_component_names: &mut ComponentsSet, 
    inside_loop: bool
) {
    use ReturnType::*;
    if let Final(data) = &mut bucket.return_info {
        match data.context.size{
            SizeOption::Single(value) if value == 0 =>{
                
            }
            _ => {
                visit_address_type(
                    &mut data.dest_address_type, 
                    assignment_status,
                    assignments_level,
                    unknown_component_names,
                    inside_loop
                )
            }
        };

    }
}

pub fn visit_loop(
    bucket: &mut LoopBucket,
    assignment_status: &mut LastInfoMap, 
    unknown_component_names: &mut ComponentsSet, 
) {

    // We visit the list of instructions in the body and update

    visit_list(
        &mut bucket.body, 
        assignment_status, 
        unknown_component_names,
        true
    );
}

pub fn visit_store(
    bucket: &mut StoreBucket,
    assignment_status: &mut LastInfoMap, 
    assigments_level: &mut ComponentsSet,
    unknown_component_names: &mut ComponentsSet, 
    inside_loop: bool
) {
    
    match bucket.context.size{
        SizeOption::Single(value) if value == 0 =>{
        }
        _ => {
            visit_address_type(
                &mut bucket.dest_address_type, 
                assignment_status,
                assigments_level,
                unknown_component_names,
                inside_loop
            )
        }
    };
}



pub fn visit_address_type(
    xtype: &mut AddressType,
    assignment_status: &mut LastInfoMap, 
    assigments_level: &mut ComponentsSet,
    unknown_component_names: &mut ComponentsSet, 
    inside_loop: bool
) {
    use AddressType::*;
    use InputInformation::*;
    use StatusInput::*;
    use Instruction::*;

    // Case known component or anonymous

    // If it the assignment status is Last
    // --> The state is NoLast

    // If the assignment status is Unknown
    // --> If the assignment level contains the component then NoLast
    // --> In other case Unknown and
    //     --> If we are in a loop we just update level_assignments
    //     --> If not then we update both level_assignments and assignment status (to Last)

    // If there is not assignment status and the component is not unknown
    //     --> If we are in a loop then Unknown and just update level_assignments
    //     --> If not then we update both level_assignments and assignment status (to Last)

    // If there is not assignment status and the component is in the list of unknowns
    //     We set the value unknown and
    //     --> If we are in a loop we just update level_assignments
    //     --> If not then we update both level_assignments and assignment status (to Last)

    // Case unknown components
    // Value unknown and we add the cmp id to the list of unknowns

    

    match xtype{
        SubcmpSignal { 
            cmp_address, 
            input_information,
            is_anonymous,
            cmp_name,
            ..
        } => {
            match input_information{
                Input{status, needs_decrement} =>{
                    
                    // case anonymous components
                    if *is_anonymous {
                        if assignment_status.contains_key(cmp_name){
                            assert!(assignment_status.get(cmp_name).unwrap().found_last);
                            *status = NoLast;
                            *needs_decrement = false;
                        } else{
                            *status = Last;
                            *needs_decrement = false;
                            assignment_status.insert(cmp_name.clone(), LastInfo{
                                needs_decrement: false,
                                found_last: true
                            });
                        }
                    } else{
                        if let Value(vb) = *cmp_address.clone(){               
                            let value = format!("cmp_{}", vb.value.clone());
                            if assignment_status.contains_key(&value){
                                let last_info = assignment_status.get_mut(&value).unwrap();
                                if last_info.found_last{
                                    // in this case we have previously found the last
                                    *status = NoLast;
                                    *needs_decrement = last_info.needs_decrement;
                                } else{
                                    // in this case it is unknown, we check if assigned in this level
                                    if assigments_level.contains(&value){
                                        *status = NoLast;
                                        *needs_decrement = true;
                                    } else{
                                        *status = Unknown;
                                        *needs_decrement = true;
                                        if !inside_loop{
                                            last_info.found_last = true;
                                        } else{
                                            assigments_level.insert(value.clone());
                                        }
                                    }
                                }
                            } else{
                                if !unknown_component_names.contains(cmp_name){
                                    // case we know all assignments to the component
                                    if inside_loop{
                                        *status = Unknown;
                                        *needs_decrement = true;
                                        assignment_status.insert(value.clone(), LastInfo{
                                            needs_decrement: true,
                                            found_last: false
                                        });
                                        assigments_level.insert(value);
                                    } else{
                                        *status = Last;
                                        *needs_decrement = false;
                                        assignment_status.insert(value.clone(), LastInfo{
                                            needs_decrement: false,
                                            found_last: true
                                        });
                                    }
                                } else{
                                    // case we do not know all assignments to the component
                                    if inside_loop{
                                        *status = Unknown;
                                        *needs_decrement = true;
                                        assignment_status.insert(value.clone(), LastInfo{
                                            needs_decrement: true,
                                            found_last: false
                                        });
                                        assigments_level.insert(value);
                                    } else{
                                        *status = Unknown;
                                        *needs_decrement = true;
                                        assignment_status.insert(value.clone(), LastInfo{
                                            needs_decrement: true,
                                            found_last: true
                                        });
                                    }
                                }
                            }
                        } else{
                            // case unknown
                            *status = StatusInput::Unknown;
                            *needs_decrement = true;
                            unknown_component_names.insert(cmp_name.clone());
                        }
                    }
                },
                NoInput =>{
                    // no input signal, no needed
                }
            }
        },
        _ =>{
            // no input signal, no needed
        }
    } 

}