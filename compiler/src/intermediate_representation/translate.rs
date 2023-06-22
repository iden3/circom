use super::ir_interface::*;
use crate::hir::very_concrete_program::*;
use crate::intermediate_representation::log_bucket::LogBucketArg;
use constant_tracking::ConstantTracker;
use num_bigint_dig::BigInt;
use program_structure::ast::*;
use program_structure::file_definition::FileLibrary;
use program_structure::utils::environment::VarEnvironment;
use std::collections::{HashMap, BTreeMap, HashSet};

type Length = usize;
pub type E = VarEnvironment<SymbolInfo>;
pub type FieldTracker = ConstantTracker<String>;
#[derive(Clone)]
pub struct SymbolInfo {
    access_instruction: InstructionPointer,
    dimensions: Vec<Length>,
    is_component: bool,
}

#[derive(Clone)]
pub struct SignalInfo{
    signal_type: SignalType,
    lengths: Vec<usize>,
}

#[derive(Clone)]
pub struct TemplateDB {
    // one per template instance
    pub signal_addresses: Vec<E>,
    // stores the type and the length of signal
    pub signal_info: Vec<HashMap<String, SignalInfo>>,
    // template_name to usize
    pub indexes: HashMap<String, usize>,
    // one per generic template, gives its signal to code correspondence
    pub signals_id: Vec<HashMap<String, usize>>,
}
impl TemplateDB {
    pub fn build(templates: &[TemplateInstance]) -> TemplateDB {
        let mut database = TemplateDB {
            indexes: HashMap::with_capacity(templates.len()),
            signal_addresses: Vec::with_capacity(templates.len()),
            signal_info: Vec::with_capacity(templates.len()),
            signals_id: Vec::with_capacity(templates.len()),
        };
        for tmp in templates {
            TemplateDB::add_instance(&mut database, tmp);
        }
        database
    }

    pub fn get_signal_id(db: &TemplateDB, tmp_name: &str, signal_name: &str) -> usize {
        let index = *db.indexes.get(tmp_name).unwrap();
        *db.signals_id[index].get(signal_name).unwrap()
    }

    pub fn get_instance_addresses(db: &TemplateDB, instance_id: usize) -> &E {
        &db.signal_addresses[instance_id]
    }

    fn add_instance(db: &mut TemplateDB, instance: &TemplateInstance) {
        if !db.indexes.contains_key(&instance.template_name) {
            let index = db.signals_id.len();
            db.indexes.insert(instance.template_name.clone(), index);
            let mut correspondence = HashMap::new();
            for (id, signal) in instance.signals.iter().enumerate() {
                correspondence.insert(signal.name.clone(), id);
            }
            db.signals_id.push(correspondence);
        }
        let mut state = State::new(
            instance.template_id,
            0,
            ConstantTracker::new(),
            HashMap::with_capacity(0),
            instance.signals_to_tags.clone(),
        );
        let mut signal_info = HashMap::new();
        for signal in instance.signals.clone() {
            let info = SignalInfo{ signal_type: signal.xtype, lengths: signal.lengths};
            signal_info.insert(signal.name, info);
        }
        initialize_signals(&mut state, instance.signals.clone());
        db.signal_addresses.push(state.environment);
        db.signal_info.push(signal_info);
    }
}

struct State {
    field_tracker: FieldTracker,
    environment: E,
    component_to_parallel:  HashMap<String, ParallelClusters>,
    component_to_instance: HashMap<String, HashSet<usize>>,
    signal_to_type: HashMap<String, SignalType>,
    signal_to_tags: BTreeMap<String, TagInfo>,
    message_id: usize,
    signal_stack: usize,
    variable_stack: usize,
    max_stack_depth: usize,
    fresh_cmp_id: usize,
    component_address_stack: usize,
    code: InstructionList,
    // string_table
    string_table: HashMap<String, usize>,
}

impl State {
    fn new(
        msg_id: usize,
        cmp_id_offset: usize,
        field_tracker: FieldTracker,
        component_to_parallel:  HashMap<String, ParallelClusters>,
        signal_to_tags: BTreeMap<String, TagInfo>
    ) -> State {
        State {
            field_tracker,
            component_to_parallel,
            signal_to_type: HashMap::new(),
            signal_to_tags,
            component_to_instance: HashMap::new(),
            environment: E::new(),
            message_id: msg_id,
            variable_stack: 0,
            signal_stack: 0,
            component_address_stack: 0,
            fresh_cmp_id: cmp_id_offset,
            max_stack_depth: 0,
            code: vec![],
            string_table : HashMap::new(),
        }
    }
    fn reserve(fresh: &mut usize, size: usize) -> usize {
        let start = *fresh;
        *fresh += size;
        start
    }
    fn reserve_signal(&mut self, size: usize) -> usize {
        State::reserve(&mut self.signal_stack, size)
    }
    fn reserve_variable(&mut self, size: usize) -> usize {
        let ret = State::reserve(&mut self.variable_stack, size);
        self.max_stack_depth = std::cmp::max(self.max_stack_depth, self.variable_stack);
        ret
    }

    fn reserve_component_address(&mut self, size: usize) -> usize {
        State::reserve(&mut self.component_address_stack, size)
    }

    fn reserve_component_ids(&mut self, no_ids: usize) -> usize {
        State::reserve(&mut self.fresh_cmp_id, no_ids)
    }
}

struct Context<'a> {
    translating: String,
    files: &'a FileLibrary,
    tmp_database: &'a TemplateDB,
    functions: &'a HashMap<String, Vec<Length>>,
    cmp_to_type: HashMap<String, ClusterType>,
}

fn initialize_parameters(state: &mut State, params: Vec<Param>) {
    for p in params {
        let lengths = p.length;
        let full_size = lengths.iter().fold(1, |p, s| p * (*s));
        let address = state.reserve_variable(full_size);
        let address_instruction = ValueBucket {
            line: 0,
            message_id: 0,
            parse_as: ValueType::U32,
            value: address,
            op_aux_no: 0,
        };
        let address_instruction = address_instruction.allocate();
        let symbol_info =
            SymbolInfo { dimensions: lengths, access_instruction: address_instruction.clone(), is_component:false };
        state.environment.add_variable(&p.name, symbol_info);
    }
}

fn initialize_constants(state: &mut State, constants: Vec<Argument>) {
    for arg in constants {
        let dimensions = arg.lengths;
        let size = dimensions.iter().fold(1, |p, c| p * (*c));
        let address = state.reserve_variable(size);
        let address_instruction = ValueBucket {
            line: 0,
            message_id: 0,
            parse_as: ValueType::U32,
            value: address,
            op_aux_no: 0,
        }
        .allocate();
        let symbol_info =
            SymbolInfo { access_instruction: address_instruction.clone(), dimensions, is_component:false };
        state.environment.add_variable(&arg.name, symbol_info);
        let mut index = 0;
        for value in arg.values {
            let cid = bigint_to_cid(&mut state.field_tracker, &value);
            let offset_instruction = ValueBucket {
                line: 0,
                message_id: 0,
                parse_as: ValueType::U32,
                value: index,
                op_aux_no: 0,
            }
            .allocate();
            let full_address = ComputeBucket {
                line: 0,
                message_id: 0,
                op: OperatorType::AddAddress,
                stack: vec![address_instruction.clone(), offset_instruction],
                op_aux_no: 0,
            }
            .allocate();
            let content = ValueBucket {
                line: 0,
                message_id: 0,
                parse_as: ValueType::BigInt,
                value: cid,
                op_aux_no: 0,
            }
            .allocate();
            let store_instruction = StoreBucket {
                line: 0,
                message_id: 0,
                dest_is_output: false,
                dest_address_type: AddressType::Variable,
                dest: LocationRule::Indexed { location: full_address, template_header: None },
                context: InstrContext { size: 1 },
                src: content,
            }
            .allocate();
            state.code.push(store_instruction);
            index += 1;
        }
    }
}

fn initialize_signals(state: &mut State, signals: Vec<Signal>) {
    for signal in signals {
        let size = signal.lengths.iter().fold(1, |p, c| p * (*c));
        let address = state.reserve_signal(size);
        let instruction = ValueBucket {
            line: 0,
            message_id: state.message_id,
            parse_as: ValueType::U32,
            value: address,
            op_aux_no: 0,
        }
        .allocate();
        let info = SymbolInfo { access_instruction: instruction, dimensions: signal.lengths, is_component:false };
        state.environment.add_variable(&signal.name, info);
        state.signal_to_type.insert(signal.name.clone(), signal.xtype);
    }
}

fn initialize_components(state: &mut State, components: Vec<Component>) {
    for component in components {
        let size = component.size();
        let address = state.reserve_component_address(size);
        let instruction = ValueBucket {
            line: 0,
            message_id: state.message_id,
            parse_as: ValueType::U32,
            value: address,
            op_aux_no: 0,
        }
        .allocate();
        let info = SymbolInfo { access_instruction: instruction, dimensions: component.lengths, is_component: true };
        state.environment.add_variable(&component.name, info);
    }
}

// Start of component creation utils
fn create_components(state: &mut State, triggers: &[Trigger], clusters: Vec<TriggerCluster>) {
    use ClusterType::*;
    for trigger in triggers {
        let component_info = state.component_to_instance.get_mut(&trigger.component_name);
        match component_info{
            Some(info) =>{
                info.insert(trigger.template_id);
            }
            None =>{
                let mut new_info = HashSet::new();
                new_info.insert(trigger.template_id);
                state.component_to_instance.insert(trigger.component_name.clone(), new_info);
            }
        }
    }
    for cluster in clusters {
        match cluster.xtype.clone() {
            Mixed { .. } => create_mixed_components(state, triggers, cluster),
            Uniform { .. } => create_uniform_components(state, triggers, cluster),
        }
    }
}

fn create_uniform_components(state: &mut State, triggers: &[Trigger], cluster: TriggerCluster) {
    fn compute_number_cmp(lengths: &Vec<usize>) -> usize {
        lengths.iter().fold(1, |p, c| p * (*c))
    }
    fn compute_jump(lengths: &Vec<usize>, indexes: &[usize]) -> usize {
        let mut jump = 0;
        let mut full_length = lengths.iter().fold(1, |p, c| p * (*c));
        let mut lengths = lengths.clone();
        lengths.reverse();
        for index in indexes {
            let length = lengths.pop().unwrap();
            full_length /= length;
            jump += (*index) * full_length;
        }
        jump
    }
    use ClusterType::Uniform;
    if let Uniform { offset_jump, component_offset_jump, .. } = cluster.xtype {
        let id = state.reserve_component_ids(offset_jump);
        let first = cluster.slice.start;
        let c_info = &triggers[first];
        let symbol = state.environment.get_variable(&c_info.component_name).unwrap().clone();
        
        let info_parallel_cluster = state.component_to_parallel.get(&c_info.component_name).unwrap(); 
        let mut defined_positions = Vec::new();
        for (pos, value) in &info_parallel_cluster.positions_to_parallel{
            let flattened_pos = compute_jump(&symbol.dimensions, pos);
            defined_positions.push((flattened_pos, *value));
        }

        let creation_instr = CreateCmpBucket {
            line: 0,
            message_id: state.message_id,
            symbol: c_info.runs.clone(),
            name_subcomponent: c_info.component_name.clone(),
            defined_positions,
            is_part_mixed_array_not_uniform_parallel: false,
            uniform_parallel: info_parallel_cluster.uniform_parallel_value,
            cmp_unique_id: id,
            sub_cmp_id: symbol.access_instruction.clone(),
            template_id: c_info.template_id,
            signal_offset: c_info.offset,
	        component_offset: c_info.component_offset,
            has_inputs: c_info.has_inputs,
            number_of_cmp: compute_number_cmp(&symbol.dimensions),
            dimensions: symbol.dimensions,
            signal_offset_jump: offset_jump,
	        component_offset_jump: component_offset_jump,
        }
        .allocate();
        state.code.push(creation_instr);
    } else {
        unreachable!()
    }
}

fn create_mixed_components(state: &mut State, triggers: &[Trigger], cluster: TriggerCluster) {
    fn compute_jump(lengths: &Vec<usize>, indexes: &[usize]) -> usize {
        let mut jump = 0;
        let mut full_length = lengths.iter().fold(1, |p, c| p * (*c));
        let mut lengths = lengths.clone();
        lengths.reverse();
        for index in indexes {
            let length = lengths.pop().unwrap();
            full_length /= length;
            jump += (*index) * full_length;
        }
        jump
    }
    for index in cluster.slice {
        let id = state.reserve_component_ids(1);
        let c_info = &triggers[index];
        let symbol = state.environment.get_variable(&c_info.component_name).unwrap().clone();
        let value_jump = compute_jump(&symbol.dimensions, &c_info.indexed_with);
        let jump = ValueBucket {
            line: 0,
            message_id: state.message_id,
            parse_as: ValueType::U32,
            value: value_jump,
            op_aux_no: 0,
        }
        .allocate();
        let location = ComputeBucket {
            line: 0,
            op_aux_no: 0,
            message_id: state.message_id,
            op: OperatorType::AddAddress,
            stack: vec![symbol.access_instruction.clone(), jump],
        }
        .allocate();

        let info_parallel_cluster = state.component_to_parallel.get(&c_info.component_name).unwrap(); 
        let parallel_value: bool;
        if info_parallel_cluster.uniform_parallel_value.is_some(){
            parallel_value = info_parallel_cluster.uniform_parallel_value.unwrap();
        }
        else{
            parallel_value = *info_parallel_cluster.
                positions_to_parallel.get(&c_info.indexed_with).unwrap();
        }

        let creation_instr = CreateCmpBucket {
            line: 0,
            message_id: state.message_id,
            symbol: c_info.runs.clone(),
            name_subcomponent: format!("{}{}",c_info.component_name.clone(), c_info.indexed_with.iter().fold(String::new(), |acc, &num| format!("{}[{}]", acc, &num.to_string()))),
            defined_positions: vec![(0, parallel_value)],
            is_part_mixed_array_not_uniform_parallel: info_parallel_cluster.uniform_parallel_value.is_none(),
            uniform_parallel: Some(parallel_value),
            dimensions: symbol.dimensions,
            cmp_unique_id: id,
            sub_cmp_id: location,
            template_id: c_info.template_id,
            signal_offset: c_info.offset,
	        component_offset: c_info.component_offset,
            has_inputs: c_info.has_inputs,
            number_of_cmp: 1,
            signal_offset_jump: 0,
	        component_offset_jump: 0,
        }
        .allocate();
        state.code.push(creation_instr);
    }
}

// Start of translation utils
fn translate_statement(stmt: Statement, state: &mut State, context: &Context) {
    if stmt.is_declaration() {
        translate_declaration(stmt, state, context);
    } else if stmt.is_substitution() {
        translate_substitution(stmt, state, context);
    } else if stmt.is_block() {
        translate_block(stmt, state, context);
    } else if stmt.is_if_then_else() {
        translate_if_then_else(stmt, state, context);
    } else if stmt.is_while() {
        translate_while(stmt, state, context);
    } else if stmt.is_assert() {
        translate_assert(stmt, state, context);
    } else if stmt.is_constraint_equality() {
        translate_constraint_equality(stmt, state, context);
    } else if stmt.is_return() {
        translate_return(stmt, state, context);
    } else if stmt.is_log_call() {
        translate_log(stmt, state, context);
    } else if stmt.is_initialization_block() {
        unreachable!("This statement is syntactic sugar");
    } else {
        unreachable!("Unknown statement");
    }
}

fn translate_if_then_else(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::IfThenElse;
    if let IfThenElse { meta, cond, if_case, else_case, .. } = stmt {
        let starts_at = context.files.get_line(meta.start, meta.get_file_id()).unwrap();
        let main_program = std::mem::replace(&mut state.code, vec![]);
        let cond_translation = translate_expression(cond, state, context);
        translate_statement(*if_case, state, context);
        let if_code = std::mem::replace(&mut state.code, vec![]);
        if let Option::Some(else_case) = else_case {
            translate_statement(*else_case, state, context);
        }
        let else_code = std::mem::replace(&mut state.code, main_program);
        let branch_instruction = BranchBucket {
            line: starts_at,
            message_id: state.message_id,
            cond: cond_translation,
            if_branch: if_code,
            else_branch: else_code,
        }
        .allocate();
        state.code.push(branch_instruction);
    }
}

fn translate_while(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::While;
    if let While { meta, cond, stmt, .. } = stmt {
        let starts_at = context.files.get_line(meta.start, meta.get_file_id()).unwrap();
        let main_program = std::mem::replace(&mut state.code, vec![]);
        let cond_translation = translate_expression(cond, state, context);
        translate_statement(*stmt, state, context);
        let loop_code = std::mem::replace(&mut state.code, main_program);
        let loop_instruction = LoopBucket {
            line: starts_at,
            message_id: state.message_id,
            continue_condition: cond_translation,
            body: loop_code,
        }
        .allocate();
        state.code.push(loop_instruction);
    }
}

fn translate_substitution(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::Substitution;
    if let Substitution { meta, var, access, rhe, .. } = stmt {
        debug_assert!(!meta.get_type_knowledge().is_component());
        let def = SymbolDef { meta: meta.clone(), symbol: var, acc: access };
        let str_info =
            StoreInfo { prc_symbol: ProcessedSymbol::new(def, state, context), src: rhe };
        let store_instruction = if str_info.src.is_call() {
            translate_call_case(str_info, state, context)
        } else {
            translate_standard_case(str_info, state, context)
        };
        state.code.push(store_instruction);
    } else {
        unreachable!();
    }
}

// Start of substitution utils
struct StoreInfo {
    prc_symbol: ProcessedSymbol,
    src: Expression,
}
fn translate_call_case(
    info: StoreInfo,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    use Expression::Call;
    if let Call { id, args, .. } = info.src {
        let args_instr = translate_call_arguments(args, state, context);
        info.prc_symbol.into_call_assign(id, args_instr, &state)
    } else {
        unreachable!()
    }
}

fn translate_standard_case(
    info: StoreInfo,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    let src = translate_expression(info.src, state, context);
    info.prc_symbol.into_store(src, state)
}

// End of substitution utils

fn translate_declaration(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::Declaration;
    if let Declaration { name, meta, .. } = stmt {
        let starts_at = context.files.get_line(meta.start, meta.get_file_id()).unwrap();
        let dimensions = meta.get_memory_knowledge().get_concrete_dimensions().to_vec();
        let size = dimensions.iter().fold(1, |p, c| p * (*c));
        let address = state.reserve_variable(size);
        let instruction = ValueBucket {
            line: starts_at,
            message_id: state.message_id,
            parse_as: ValueType::U32,
            value: address,
            op_aux_no: 0,
        }
        .allocate();
        let info = SymbolInfo { access_instruction: instruction, dimensions, is_component: false };
        state.environment.add_variable(&name, info);
    } else {
        unreachable!()
    }
}

fn translate_block(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        let save_variable_address = state.variable_stack;
        state.environment.add_variable_block();
        for s in stmts {
            translate_statement(s, state, context);
        }
        state.environment.remove_variable_block();
        state.variable_stack = save_variable_address;
    } else {
        unreachable!()
    }
}

fn translate_constraint_equality(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::ConstraintEquality;
    use Expression::Variable;
    if let ConstraintEquality { meta, lhe, rhe } = stmt {
        let starts_at = context.files.get_line(meta.start, meta.get_file_id()).unwrap();

        let length = if let Variable { meta, name, access} = lhe.clone() {
            let def = SymbolDef { meta, symbol: name, acc: access };
            ProcessedSymbol::new(def, state, context).length
        } else {1};
        
        let lhe_pointer = translate_expression(lhe, state, context);
        let rhe_pointer = translate_expression(rhe, state, context);
        let stack = vec![lhe_pointer, rhe_pointer];
        let equality = ComputeBucket {
            line: starts_at,
            message_id: state.message_id,
            op_aux_no: 0,
            op: OperatorType::Eq(length),
            stack,
        }
        .allocate();
        let assert_instruction =
            AssertBucket { line: starts_at, message_id: state.message_id, evaluate: equality }
                .allocate();
        state.code.push(assert_instruction);
    } else {
        unimplemented!()
    }
}

fn translate_assert(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::Assert;
    if let Assert { meta, arg, .. } = stmt {
        let line = context.files.get_line(meta.start, meta.get_file_id()).unwrap();
        let code = translate_expression(arg, state, context);
        let assert = AssertBucket { line, message_id: state.message_id, evaluate: code }.allocate();
        state.code.push(assert);
    }
}

fn translate_log(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::LogCall;
    if let LogCall { meta, args, .. } = stmt {
        let line = context.files.get_line(meta.start, meta.get_file_id()).unwrap();
        let mut logbucket_args = Vec::new();
        for arglog in args {
            match arglog {
                LogArgument::LogExp(arg) => {
                    let code = translate_expression(arg, state, context);
                    logbucket_args.push(LogBucketArg::LogExp(code));
                }
                LogArgument::LogStr(exp) => {
                    match state.string_table.get(&exp) {
                        Some( idx) => {logbucket_args.push(LogBucketArg::LogStr(*idx));},
                        None => {
                            logbucket_args.push(LogBucketArg::LogStr(state.string_table.len()));
                            state.string_table.insert(exp, state.string_table.len());
                        },
                    }
                    
                }
            }
        }
        
        let log = LogBucket {
            line,
            message_id: state.message_id,
            argsprint: logbucket_args,
        }.allocate();
        state.code.push(log);
    }
}

fn translate_return(stmt: Statement, state: &mut State, context: &Context) {
    use Statement::Return;
    if let Return { meta, value, .. } = stmt {
        let return_type = context.functions.get(&context.translating).unwrap();
        let return_bucket = ReturnBucket {
            line: context.files.get_line(meta.start, meta.get_file_id()).unwrap(),
            message_id: state.message_id,
            with_size: return_type.iter().fold(1, |p, c| p * (*c)),
            value: translate_expression(value, state, context),
        }
        .allocate();
        state.code.push(return_bucket);
    }
}

fn translate_expression(
    expression: Expression,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    if expression.is_infix() {
        translate_infix(expression, state, context)
    } else if expression.is_prefix() {
        translate_prefix(expression, state, context)
    } else if expression.is_variable() {
        translate_variable(expression, state, context)
    } else if expression.is_number() {
        translate_number(expression, state, context)
    } else if expression.is_call() {
        translate_call(expression, state, context)
    } else if expression.is_array() {
        unreachable!("This expression is syntactic sugar")
    } else if expression.is_switch() {
        unreachable!("This expression is syntactic sugar")
    } else {
        unreachable!("Unknown expression")
    }
}

fn translate_call(
    expression: Expression,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    use Expression::Call;
    use ReturnType::Intermediate;
    if let Call { id, args, meta, .. } = expression {
        let args_inst = translate_call_arguments(args, state, context);
        CallBucket {
            line: context.files.get_line(meta.start, meta.get_file_id()).unwrap(),
            message_id: state.message_id,
            symbol: id,
            argument_types: args_inst.argument_data,
            arguments: args_inst.arguments,
            arena_size: 200,
            return_info: Intermediate { op_aux_no: 0 },
        }
        .allocate()
    } else {
        unreachable!()
    }
}

fn translate_infix(
    expression: Expression,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    use Expression::InfixOp;
    if let InfixOp { meta, infix_op, rhe, lhe, .. } = expression {
        let lhi = translate_expression(*lhe, state, context);
        let rhi = translate_expression(*rhe, state, context);
        ComputeBucket {
            line: context.files.get_line(meta.start, meta.get_file_id()).unwrap(),
            message_id: state.message_id,
            op: translate_infix_operator(infix_op),
            op_aux_no: 0,
            stack: vec![lhi, rhi],
        }
        .allocate()
    } else {
        unreachable!()
    }
}

fn translate_prefix(
    expression: Expression,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    use Expression::PrefixOp;
    if let PrefixOp { meta, prefix_op, rhe, .. } = expression {
        let rhi = translate_expression(*rhe, state, context);
        ComputeBucket {
            line: context.files.get_line(meta.start, meta.get_file_id()).unwrap(),
            message_id: state.message_id,
            op_aux_no: 0,
            op: translate_prefix_operator(prefix_op),
            stack: vec![rhi],
        }
        .allocate()
    } else {
        unreachable!()
    }
}

fn check_tag_access(name_signal: &String, access: &Vec<Access>, state: &mut State) -> Option<BigInt> {
    use Access::*;

    let symbol_info = state.environment.get_variable(name_signal).unwrap().clone();
    let mut value_tag = None;
    if !symbol_info.is_component{
        for acc in access {
            match acc {
                ArrayAccess(..) => {},
                ComponentAccess(name) => {
                    let tags_signal = state.signal_to_tags.get(name_signal).unwrap();
                    let value = tags_signal.get(name).unwrap();

                    value_tag = if value.is_some() {
                        Some(value.clone().unwrap())
                    } else {
                        unreachable!()
                    };
                }
            }
        }
    }
    value_tag
}

fn translate_variable(
    expression: Expression,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    use Expression::{Variable};
    if let Variable { meta, name, access, .. } = expression {
        let tag_access = check_tag_access(&name, &access, state);
        if tag_access.is_some(){
            translate_number( Expression::Number(meta.clone(), tag_access.unwrap()), state, context)
        } else{
            let def = SymbolDef { meta, symbol: name, acc: access };
            ProcessedSymbol::new(def, state, context).into_load(state)
        }
    } else {
        unreachable!()
    }
}

fn translate_number(
    expression: Expression,
    state: &mut State,
    context: &Context,
) -> InstructionPointer {
    use Expression::Number;
    if let Number(meta, value) = expression {
        let cid = bigint_to_cid(&mut state.field_tracker, &value);
        ValueBucket {
            line: context.files.get_line(meta.start, meta.get_file_id()).unwrap(),
            message_id: state.message_id,
            op_aux_no: 0,
            parse_as: ValueType::BigInt,
            value: cid,
        }
        .allocate()
    } else {
        unreachable!()
    }
}

fn translate_infix_operator(op: ExpressionInfixOpcode) -> OperatorType {
    use ExpressionInfixOpcode::*;
    match op {
        Mul => OperatorType::Mul,
        Div => OperatorType::Div,
        Add => OperatorType::Add,
        Sub => OperatorType::Sub,
        Pow => OperatorType::Pow,
        IntDiv => OperatorType::IntDiv,
        Mod => OperatorType::Mod,
        ShiftL => OperatorType::ShiftL,
        ShiftR => OperatorType::ShiftR,
        LesserEq => OperatorType::LesserEq,
        GreaterEq => OperatorType::GreaterEq,
        Lesser => OperatorType::Lesser,
        Greater => OperatorType::Greater,
        Eq => OperatorType::Eq(1),
        NotEq => OperatorType::NotEq,
        BoolOr => OperatorType::BoolOr,
        BoolAnd => OperatorType::BoolAnd,
        BitOr => OperatorType::BitOr,
        BitAnd => OperatorType::BitAnd,
        BitXor => OperatorType::BitXor,
    }
}

fn translate_prefix_operator(op: ExpressionPrefixOpcode) -> OperatorType {
    use ExpressionPrefixOpcode::*;
    match op {
        Sub => OperatorType::PrefixSub,
        BoolNot => OperatorType::BoolNot,
        Complement => OperatorType::Complement,
    }
}

fn bigint_to_cid(field_tracker: &mut FieldTracker, big: &BigInt) -> usize {
    let constant = big.to_str_radix(10);
    field_tracker.insert(constant)
}

// Code generators

fn build_signal_location(
    signal: &str,
    cmp_name: &str,
    indexes: Vec<InstructionPointer>,
    context: &Context,
    state: &State,
) -> LocationRule {
    use ClusterType::*;
    let database = &context.tmp_database;
    let cmp_type = context.cmp_to_type.get(cmp_name).unwrap();
    match cmp_type {
        Mixed { tmp_name } => {
            let signal_code = TemplateDB::get_signal_id(database, tmp_name, signal);
            let indexes = indexing_instructions_filter(indexes, state);
            LocationRule::Mapped { signal_code, indexes }
        }
        Uniform { instance_id, header, .. } => {
            let env = TemplateDB::get_instance_addresses(database, *instance_id);
            let location = env.get_variable(signal).unwrap().clone();
            let full_address = compute_full_address(state, location, indexes);
            LocationRule::Indexed { location: full_address, template_header: Some(header.clone()) }
        }
    }
}

struct SymbolDef {
    meta: Meta,
    symbol: String,
    acc: Vec<Access>,
}

struct ProcessedSymbol {
    line: usize,
    length: usize,
    message_id: usize,
    name: String,
    symbol: SymbolInfo,
    xtype: TypeReduction,
    signal: Option<LocationRule>,
    signal_type: Option<SignalType>,
    before_signal: Vec<InstructionPointer>,
}

impl ProcessedSymbol {
    fn new(definition: SymbolDef, state: &mut State, context: &Context) -> ProcessedSymbol {
        use Access::*;
        let symbol_name = definition.symbol;
        let meta = definition.meta;
        let symbol_info = state.environment.get_variable(&symbol_name).unwrap().clone();
        let mut lengths = symbol_info.dimensions.clone();
        lengths.reverse();
        let mut with_length = symbol_info.dimensions.iter().fold(1, |r, c| r * (*c));
        let mut signal = None;
        let mut signal_type = state.signal_to_type.get(&symbol_name).cloned();
        let mut bf_index = vec![];
        let mut af_index = vec![];
        let mut multiple_possible_lengths: Vec<Vec<usize>> = vec![];
        for acc in definition.acc {
            match acc {
                ArrayAccess(exp) if signal.is_none() => {
                    let length = lengths.pop().unwrap();
                    with_length /= length;
                    bf_index.push(translate_expression(exp, state, context));
                }
                ArrayAccess(exp) => {
                    for possible_length in &mut multiple_possible_lengths{
                        possible_length.pop();
                    }
                    af_index.push(translate_expression(exp, state, context));
                }
                ComponentAccess(name) => {
                    let possible_cmp_id = state.component_to_instance.get(&symbol_name).unwrap().clone();
                    for cmp_id in possible_cmp_id{
                        let aux = context.tmp_database.signal_info[cmp_id].get(&name).unwrap();
                        signal_type = Some(aux.signal_type);
                        let mut new_length = aux.lengths.clone();
                        new_length.reverse();
                        multiple_possible_lengths.push(new_length);
                    }
                    signal = Some(name);
                }
            }
        }
        if signal.is_some(){
            let mut is_first = true;
            for possible_length in multiple_possible_lengths{
                if is_first{
                    with_length = possible_length.iter().fold(1, |r, c| r * (*c));
                    is_first = false;
                }
                else{
                    if with_length != possible_length.iter().fold(1, |r, c| r * (*c)){
                        unreachable!("On development: Circom compiler does not accept for now the assignment of arrays of unknown sizes during the execution of loops");
                    }
                }
            } 
        }

        let signal_location = signal.map(|signal_name| {
            build_signal_location(
                &signal_name,
                &symbol_name,
                af_index,
                context,
                state,
            )
        });
        ProcessedSymbol {
            xtype: meta.get_type_knowledge().get_reduces_to(),
            line: context.files.get_line(meta.start, meta.get_file_id()).unwrap(),
            message_id: state.message_id,
            length: with_length,
            symbol: symbol_info,
            name: symbol_name,
            before_signal: bf_index,
            signal: signal_location,
            signal_type
        }
    }

    fn into_call_assign(
        self,
        id: String,
        args: ArgData,
        state: &State,
    ) -> InstructionPointer {
        let data = if let Option::Some(signal) = self.signal {
            let dest_type = AddressType::SubcmpSignal {
                cmp_address: compute_full_address(state, self.symbol, self.before_signal),
                is_output: self.signal_type.unwrap() == SignalType::Output,
                uniform_parallel_value: state.component_to_parallel.get(&self.name).unwrap().uniform_parallel_value,
                input_information : match self.signal_type.unwrap() {
                    SignalType::Input => InputInformation::Input { status: StatusInput:: Unknown},
                    _ => InputInformation::NoInput,
                },
            };
            FinalData {
                context: InstrContext { size: self.length },
                dest_is_output: false,
                dest_address_type: dest_type,
                dest: signal,
            }
        } else {
            let address = compute_full_address(state, self.symbol, self.before_signal);
            let xtype = match self.xtype {
                TypeReduction::Variable => AddressType::Variable,
                _ => AddressType::Signal,
            };
            FinalData {
                context: InstrContext { size: self.length },
                dest_is_output: self.signal_type.map_or(false, |t| t == SignalType::Output),
                dest_address_type: xtype,
                dest: LocationRule::Indexed { location: address, template_header: None },
            }
        };
        CallBucket {
            line: self.line,
            message_id: self.message_id,
            symbol: id,
            argument_types: args.argument_data,
            arguments: args.arguments,
            arena_size: 200,
            return_info: ReturnType::Final(data),
        }
        .allocate()
    }

    fn into_store(self, src: InstructionPointer, state: &State) -> InstructionPointer {
        if let Option::Some(signal) = self.signal {
            let dest_type = AddressType::SubcmpSignal {
                cmp_address: compute_full_address(state, self.symbol, self.before_signal),
                uniform_parallel_value: state.component_to_parallel.get(&self.name).unwrap().uniform_parallel_value,
                is_output: self.signal_type.unwrap() == SignalType::Output,
                input_information : match self.signal_type.unwrap() {
                    SignalType::Input => InputInformation::Input { status:StatusInput:: Unknown},
                    _ => InputInformation::NoInput,
                },
            };
            StoreBucket {
                src,
                dest: signal,
                line: self.line,
                message_id: self.message_id,
                context: InstrContext { size: self.length },
                dest_is_output: false,
                dest_address_type: dest_type,
            }
            .allocate()
        } else {
            let address = compute_full_address(state, self.symbol, self.before_signal);
            let xtype = match self.xtype {
                TypeReduction::Variable => AddressType::Variable,
                _ => AddressType::Signal,
            };
            StoreBucket {
                src,
                line: self.line,
                dest_address_type: xtype,
                message_id: self.message_id,
                dest_is_output: self.signal_type.map_or(false, |t| t == SignalType::Output),
                dest: LocationRule::Indexed { location: address, template_header: None },
                context: InstrContext { size: self.length },
            }
            .allocate()
        }
    }

    fn into_load(self, state: &State) -> InstructionPointer {
        if let Option::Some(signal) = self.signal {
            let dest_type = AddressType::SubcmpSignal {
                cmp_address: compute_full_address(state, self.symbol, self.before_signal),
                uniform_parallel_value: state.component_to_parallel.get(&self.name).unwrap().uniform_parallel_value,
                is_output: self.signal_type.unwrap() == SignalType::Output,
                input_information : match self.signal_type.unwrap() {
                    SignalType::Input => InputInformation::Input { status: StatusInput:: Unknown},
                    _ => InputInformation::NoInput,
                },
            };
            LoadBucket {
                src: signal,
                line: self.line,
                message_id: self.message_id,
                address_type: dest_type,
                context: InstrContext { size: self.length },
            }
            .allocate()
        } else {
            let address = compute_full_address(state, self.symbol, self.before_signal);
            let xtype = match self.xtype {
                TypeReduction::Variable => AddressType::Variable,
                _ => AddressType::Signal,
            };
            LoadBucket {
                line: self.line,
                address_type: xtype,
                message_id: self.message_id,
                src: LocationRule::Indexed { location: address, template_header: None },
                context: InstrContext { size: self.length },
            }
            .allocate()
        }
    }
}

fn compute_full_address(
    state: &State,
    symbol: SymbolInfo,
    indexed_with: Vec<InstructionPointer>,
) -> InstructionPointer {
    if symbol.dimensions.is_empty() {
        symbol.access_instruction
    } else {
        let at = symbol.access_instruction;
        let mut with_dimensions = symbol.dimensions;
        with_dimensions.reverse();
        let mut linear_length = with_dimensions.iter().fold(1, |p, c| p * (*c));
        let index_stack = indexing_instructions_filter(indexed_with, state);
        let mut stack = vec![];
        for instruction in index_stack {
            let dimension_length = with_dimensions.pop().unwrap();
            linear_length /= dimension_length;
            let inst = ValueBucket {
                line: at.get_line(),
                message_id: at.get_message_id(),
                parse_as: ValueType::U32,
                op_aux_no: 0,
                value: linear_length,
            }
            .allocate();
            let jump = ComputeBucket {
                line: at.get_line(),
                message_id: at.get_message_id(),
                op_aux_no: 0,
                op: OperatorType::MulAddress,
                stack: vec![inst, instruction],
            }
            .allocate();
            stack.push(jump);
        }
        stack.push(at);
        fold(OperatorType::AddAddress, stack, state)
    }
}

fn indexing_instructions_filter(
    indexing: Vec<InstructionPointer>,
    state: &State,
 ) -> Vec<InstructionPointer>{
    let mut index_stack = vec![];
    for i in indexing {

        let (possible_to_usize, _) = check_if_possible_to_usize_single(&i, state);

        if possible_to_usize{
            let new_index = convert_to_usize_single(i, state);
            index_stack.push(new_index);
        } else{

            let to_usize = ComputeBucket {
                line: i.get_line(),
                message_id: i.get_message_id(),
                op_aux_no: 0,
                op: OperatorType::ToAddress,
                stack: vec![i.allocate()],
            }.allocate();
            index_stack.push(to_usize);

        }
    }
    index_stack
}

fn check_if_possible_to_usize_single( // returns if it is possible to convert to usize and if it is a small usize
                                      // we consider that a usize is small if it is a number < 100
                                      // we consider that a multiplication is usize if at least one of its operands is usize 
                                      // and the other is usize
    index: &InstructionPointer,
    state: &State,
)-> (bool, bool){

    use Instruction::{Value, Compute};

    match &**index {
        Value(v) if v.parse_as == ValueType::U32 => {
            (true, v.value < 100)
        }
        Value(v) if v.parse_as == ValueType::BigInt => {
            let field = state.field_tracker.get_constant(v.value).unwrap();
            let new_value  = usize::from_str_radix(field, 10);

            match new_value{
                Ok(_) =>{
                    (true, new_value.unwrap() < 100)
                }
                _ =>{
                    (false, false)
                }
            }

        }
        Compute(v) if v.op == OperatorType::Add => {
            let (are_usize, _) = check_if_possible_to_usize_multiple(&v.stack, state);
            (are_usize, false)
        } 
        Compute(v) if v.op == OperatorType::Mul => {
            let (are_usize, are_small) = check_if_possible_to_usize_multiple(&v.stack, state);
            (are_usize && are_small, false)
        }
        Compute(_) =>{
            (false, false)
        }
        _ => {
            // Case variable
            (true, false)
        }
    }
}

fn check_if_possible_to_usize_multiple( // returns if all of them are usize and if the number of non small usizes is at most one
    indexing: &Vec<InstructionPointer>,
    state: &State,
) -> (bool, bool) { 
    let mut is_usize = true;
    let mut number_non_small = 0;
    for i in indexing {
        let (is_usize_i, is_small_i) = check_if_possible_to_usize_single(i, state);
        is_usize &= is_usize_i;
        if !is_small_i{
            number_non_small += 1;
        }
    }
    (is_usize, number_non_small <= 1)
}



fn convert_to_usize_single(
    index: InstructionPointer,
    state: &State,
)-> InstructionPointer{

    use Instruction::{Value, Compute};

    match *index {
        Value(v) if v.parse_as == ValueType::U32 => {
            v.allocate()
        }
        Value(mut v) if v.parse_as == ValueType::BigInt => {
            let field = state.field_tracker.get_constant(v.value).unwrap();
            let new_value  = usize::from_str_radix(field, 10);

            match new_value{
                Ok(value) =>{
                    v.parse_as = ValueType::U32;
                    v.value = value;
                    v.allocate()
                }
                _ =>{
                    unreachable!()
                }
            }

        }
        Compute(mut v) if v.op == OperatorType::Add => {
            v.stack = convert_to_usize_multiple(v.stack, state);
            v.op = OperatorType::AddAddress;
            v.into_instruction().allocate()
        }
        Compute(mut v) if v.op == OperatorType::Mul => {
            v.stack = convert_to_usize_multiple(v.stack, state);
            v.op = OperatorType::MulAddress;
            v.into_instruction().allocate()
        }
        Compute(_) =>{
            unreachable!()
        }
        _ => {
            // Case variable
            ComputeBucket {
                line: index.get_line(),
                message_id: index.get_message_id(),
                op_aux_no: 0,
                op: OperatorType::ToAddress,
                stack: vec![index.allocate()],
            }.allocate()
        }

    }
}

fn convert_to_usize_multiple(
    indexing: Vec<InstructionPointer>,
    state: &State,
) ->  Vec<InstructionPointer> { 
    let mut index_stack = vec![];
    for i in indexing {
        let new_index = convert_to_usize_single(i, state);
        index_stack.push(new_index);
    }
    index_stack
}


fn fold(using: OperatorType, mut stack: Vec<InstructionPointer>, state: &State) -> InstructionPointer {
    let instruction = stack.pop().unwrap();
    if stack.len() == 0 {
        instruction
    } else {
        ComputeBucket {
            line: instruction.get_line(),
            message_id: instruction.get_message_id(),
            op_aux_no: 0,
            op: using,
            stack: vec![fold(using, stack, state), instruction],
        }
        .allocate()
    }
}

struct ArgData {
    argument_data: Vec<InstrContext>,
    arguments: InstructionList,
}
fn translate_call_arguments(
    args: Vec<Expression>,
    state: &mut State,
    context: &Context,
) -> ArgData {
    let mut info = ArgData { argument_data: vec![], arguments: InstructionList::new() };
    for arg in args {
        let length = arg
            .get_meta()
            .get_memory_knowledge()
            .get_concrete_dimensions()
            .iter()
            .fold(1, |r, c| r * (*c));
        let instr = translate_expression(arg, state, context);
        info.argument_data.push(InstrContext { size: length });
        info.arguments.push(instr);
    }
    info
}

pub struct ParallelClusters{
    pub positions_to_parallel: BTreeMap<Vec<usize>, bool>,
    pub uniform_parallel_value: Option<bool>,
}

pub struct CodeInfo<'a> {
    pub header: String,
    pub message_id: usize,
    pub params: Vec<Param>,
    pub signals: Vec<Signal>,
    pub files: &'a FileLibrary,
    pub constants: Vec<Argument>,
    pub components: Vec<Component>,
    pub fresh_cmp_id: usize,
    pub template_database: &'a TemplateDB,
    pub triggers: Vec<Trigger>,
    pub clusters: Vec<TriggerCluster>,
    pub cmp_to_type: HashMap<String, ClusterType>,
    pub functions: &'a HashMap<String, Vec<Length>>,
    pub field_tracker: FieldTracker,
    pub component_to_parallel: HashMap<String, ParallelClusters>,
    pub string_table: HashMap<String, usize>,
    pub signals_to_tags: BTreeMap<String, TagInfo>,
}

pub struct CodeOutput {
    pub stack_depth: usize,
    pub signal_depth: usize,
    pub expression_depth: usize,
    pub next_cmp_id: usize,
    pub code: InstructionList,
    pub constant_tracker: FieldTracker,
    pub string_table: HashMap<String, usize>,
}

pub fn translate_code(body: Statement, code_info: CodeInfo) -> CodeOutput {
    use crate::ir_processing;
    let mut state = State::new(
        code_info.message_id,
        code_info.fresh_cmp_id,
        code_info.field_tracker,
        code_info.component_to_parallel,
        code_info.signals_to_tags,
    );
    state.string_table = code_info.string_table;
    initialize_components(&mut state, code_info.components);
    initialize_signals(&mut state, code_info.signals);
    initialize_constants(&mut state, code_info.constants);
    initialize_parameters(&mut state, code_info.params);

    let context = Context {
        files: code_info.files,
        translating: code_info.header,
        functions: code_info.functions,
        cmp_to_type: code_info.cmp_to_type,
        tmp_database: code_info.template_database,
    };

    create_components(&mut state, &code_info.triggers, code_info.clusters);
    translate_statement(body, &mut state, &context);

    ir_processing::build_inputs_info(&mut state.code);

    let mut code = ir_processing::reduce_intermediate_operations(state.code);
    let expression_depth = ir_processing::build_auxiliary_stack(&mut code);
    

    CodeOutput {
        code,
        expression_depth,
        next_cmp_id: state.fresh_cmp_id,
        stack_depth: state.max_stack_depth,
        signal_depth: state.signal_stack,
        constant_tracker: state.field_tracker,
        string_table : state.string_table
    }
}
