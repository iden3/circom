use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait VarInfo {}
pub trait SignalInfo {}
pub trait ComponentInfo {}

#[derive(Clone)]
pub struct OnlyVars;
impl VarInfo for OnlyVars {}
#[derive(Clone)]
pub struct OnlySignals;
impl SignalInfo for OnlySignals {}
#[derive(Clone)]
pub struct OnlyComponents;
impl ComponentInfo for OnlyComponents {}
#[derive(Clone)]
pub struct FullEnvironment;
impl VarInfo for FullEnvironment {}
impl SignalInfo for FullEnvironment {}
impl ComponentInfo for FullEnvironment {}

pub type VarEnvironment<VC> = RawEnvironment<OnlyVars, (), (), VC>;
pub type SignalEnvironment<SC> = RawEnvironment<OnlySignals, (), SC, ()>;
pub type ComponentEnvironment<CC> = RawEnvironment<OnlyComponents, CC, (), ()>;
pub type CircomEnvironment<CC, SC, VC> = RawEnvironment<FullEnvironment, CC, SC, VC>;

pub enum CircomEnvironmentError {
    NonExistentSymbol,
}

#[derive(Clone)]
pub struct RawEnvironment<T, CC, SC, VC> {
    components: HashMap<String, CC>,
    inputs: HashMap<String, SC>,
    outputs: HashMap<String, SC>,
    intermediates: HashMap<String, SC>,
    variables: Vec<VariableBlock<VC>>,
    behaviour: PhantomData<T>,
}
impl<T, CC, SC, VC> Default for RawEnvironment<T, CC, SC, VC> {
    fn default() -> Self {
        let variables = vec![VariableBlock::new()];
        RawEnvironment {
            components: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            intermediates: HashMap::new(),
            variables,
            behaviour: PhantomData,
        }
    }
}
impl<T, CC, SC, VC> RawEnvironment<T, CC, SC, VC>
where
    T: VarInfo + SignalInfo + ComponentInfo,
{
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.has_signal(symbol) || self.has_component(symbol) || self.has_variable(symbol)
    }
}
impl<T, CC, SC, VC> RawEnvironment<T, CC, SC, VC> {
    pub fn merge(
        left: RawEnvironment<T, CC, SC, VC>,
        right: RawEnvironment<T, CC, SC, VC>,
        using: fn(VC, VC) -> VC,
    ) -> RawEnvironment<T, CC, SC, VC> {
        let mut components = left.components;
        let mut inputs = left.inputs;
        let mut outputs = left.outputs;
        let mut intermediates = left.intermediates;
        components.extend(right.components);
        inputs.extend(right.inputs);
        outputs.extend(right.outputs);
        intermediates.extend(right.intermediates);
        let mut variables_left = left.variables;
        let mut variables_right = right.variables;
        let mut variables = Vec::new();
        while !variables_left.is_empty() && !variables_right.is_empty() {
            let left_block = variables_left.pop().unwrap();
            let right_block = variables_right.pop().unwrap();
            let merged_blocks = VariableBlock::merge(left_block, right_block, using);
            variables.push(merged_blocks);
        }
        variables.reverse();
        RawEnvironment {
            components,
            inputs,
            intermediates,
            outputs,
            variables,
            behaviour: PhantomData,
        }
    }
}
impl<T, CC, SC, VC> RawEnvironment<T, CC, SC, VC>
where
    T: VarInfo,
{
    fn block_with_variable_symbol(&self, symbol: &str) -> Option<&VariableBlock<VC>> {
        let variables = &self.variables;
        let mut act = variables.len();
        while act > 0 {
            if VariableBlock::contains_variable(&variables[act - 1], symbol) {
                return Option::Some(&variables[act - 1]);
            }
            act -= 1;
        }
        Option::None
    }
    fn mut_block_with_variable_symbol(&mut self, symbol: &str) -> Option<&mut VariableBlock<VC>> {
        let variables = &mut self.variables;
        let mut act = variables.len();
        while act > 0 {
            if VariableBlock::contains_variable(&variables[act - 1], symbol) {
                return Option::Some(&mut variables[act - 1]);
            }
            act -= 1;
        }
        Option::None
    }
    pub fn new() -> RawEnvironment<T, CC, SC, VC> {
        RawEnvironment::default()
    }
    pub fn add_variable_block(&mut self) {
        self.variables.push(VariableBlock::new());
    }
    pub fn remove_variable_block(&mut self) {
        assert!(!self.variables.is_empty());
        self.variables.pop();
    }
    pub fn add_variable(&mut self, variable_name: &str, content: VC) {
        assert!(!self.variables.is_empty());
        let last_block = self.variables.last_mut().unwrap();
        last_block.add_variable(variable_name, content);
    }
    pub fn has_variable(&self, symbol: &str) -> bool {
        self.block_with_variable_symbol(symbol).is_some()
    }

    pub fn get_variable(&self, symbol: &str) -> Option<&VC> {
        let possible_block = self.block_with_variable_symbol(symbol);
        if let Option::Some(block) = possible_block {
            Option::Some(block.get_variable(symbol))
        } else {
            Option::None
        }
    }
    pub fn get_mut_variable(&mut self, symbol: &str) -> Option<&mut VC> {
        let possible_block = self.mut_block_with_variable_symbol(symbol);
        if let Option::Some(block) = possible_block {
            Option::Some(block.get_mut_variable(symbol))
        } else {
            Option::None
        }
    }
    pub fn get_variable_res(&self, symbol: &str) -> Result<&VC, CircomEnvironmentError> {
        let possible_block = self.block_with_variable_symbol(symbol);
        if let Option::Some(block) = possible_block {
            Result::Ok(block.get_variable(symbol))
        } else {
            Result::Err(CircomEnvironmentError::NonExistentSymbol)
        }
    }
    pub fn remove_variable(&mut self, symbol: &str) {
        let possible_block = self.mut_block_with_variable_symbol(symbol);
        if let Option::Some(block) = possible_block {
            block.remove_variable(symbol)
        }
    }
    pub fn get_variable_or_break(&self, symbol: &str, file: &str, line: u32) -> &VC {
        assert!(self.has_variable(symbol), "Method call in file {} line {}", file, line);
        if let Result::Ok(v) = self.get_variable_res(symbol) {
            v
        } else {
            unreachable!();
        }
    }
    pub fn get_mut_variable_mut(
        &mut self,
        symbol: &str,
    ) -> Result<&mut VC, CircomEnvironmentError> {
        let possible_block = self.mut_block_with_variable_symbol(symbol);
        if let Option::Some(block) = possible_block {
            Result::Ok(block.get_mut_variable(symbol))
        } else {
            Result::Err(CircomEnvironmentError::NonExistentSymbol)
        }
    }
    pub fn get_mut_variable_or_break(&mut self, symbol: &str, file: &str, line: u32) -> &mut VC {
        assert!(self.has_variable(symbol), "Method call in file {} line {}", file, line);
        if let Result::Ok(v) = self.get_mut_variable_mut(symbol) {
            v
        } else {
            unreachable!();
        }
    }
}

impl<T, CC, SC, VC> RawEnvironment<T, CC, SC, VC>
where
    T: ComponentInfo,
{
    pub fn add_component(&mut self, component_name: &str, content: CC) {
        self.components.insert(component_name.to_string(), content);
    }
    pub fn remove_component(&mut self, component_name: &str) {
        self.components.remove(component_name);
    }
    pub fn has_component(&self, symbol: &str) -> bool {
        self.components.contains_key(symbol)
    }
    pub fn get_component(&self, symbol: &str) -> Option<&CC> {
        self.components.get(symbol)
    }
    pub fn get_mut_component(&mut self, symbol: &str) -> Option<&mut CC> {
        self.components.get_mut(symbol)
    }
    pub fn get_component_res(&self, symbol: &str) -> Result<&CC, CircomEnvironmentError> {
        self.components.get(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_component_or_break(&self, symbol: &str, file: &str, line: u32) -> &CC {
        assert!(self.has_component(symbol), "Method call in file {} line {}", file, line);
        self.components.get(symbol).unwrap()
    }
    pub fn get_mut_component_res(
        &mut self,
        symbol: &str,
    ) -> Result<&mut CC, CircomEnvironmentError> {
        self.components.get_mut(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_mut_component_or_break(&mut self, symbol: &str, file: &str, line: u32) -> &mut CC {
        assert!(self.has_component(symbol), "Method call in file {} line {}", file, line);
        self.components.get_mut(symbol).unwrap()
    }
}

impl<T, CC, SC, VC> RawEnvironment<T, CC, SC, VC>
where
    T: SignalInfo,
{
    pub fn add_input(&mut self, input_name: &str, content: SC) {
        self.inputs.insert(input_name.to_string(), content);
    }
    pub fn remove_input(&mut self, input_name: &str) {
        self.inputs.remove(input_name);
    }
    pub fn add_output(&mut self, output_name: &str, content: SC) {
        self.outputs.insert(output_name.to_string(), content);
    }
    pub fn remove_output(&mut self, output_name: &str) {
        self.outputs.remove(output_name);
    }
    pub fn add_intermediate(&mut self, intermediate_name: &str, content: SC) {
        self.intermediates.insert(intermediate_name.to_string(), content);
    }
    pub fn remove_intermediate(&mut self, intermediate_name: &str) {
        self.intermediates.remove(intermediate_name);
    }
    pub fn has_input(&self, symbol: &str) -> bool {
        self.inputs.contains_key(symbol)
    }
    pub fn has_output(&self, symbol: &str) -> bool {
        self.outputs.contains_key(symbol)
    }
    pub fn has_intermediate(&self, symbol: &str) -> bool {
        self.intermediates.contains_key(symbol)
    }
    pub fn has_signal(&self, symbol: &str) -> bool {
        self.has_input(symbol) || self.has_output(symbol) || self.has_intermediate(symbol)
    }
    pub fn get_input(&self, symbol: &str) -> Option<&SC> {
        self.inputs.get(symbol)
    }
    pub fn get_mut_input(&mut self, symbol: &str) -> Option<&mut SC> {
        self.inputs.get_mut(symbol)
    }
    pub fn get_input_res(&self, symbol: &str) -> Result<&SC, CircomEnvironmentError> {
        self.inputs.get(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_input_or_break(&self, symbol: &str, file: &str, line: u32) -> &SC {
        assert!(self.has_input(symbol), "Method call in file {} line {}", file, line);
        self.inputs.get(symbol).unwrap()
    }
    pub fn get_mut_input_res(&mut self, symbol: &str) -> Result<&mut SC, CircomEnvironmentError> {
        self.inputs.get_mut(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_mut_input_or_break(&mut self, symbol: &str, file: &str, line: u32) -> &mut SC {
        assert!(self.has_input(symbol), "Method call in file {} line {}", file, line);
        self.inputs.get_mut(symbol).unwrap()
    }

    pub fn get_output(&self, symbol: &str) -> Option<&SC> {
        self.outputs.get(symbol)
    }
    pub fn get_mut_output(&mut self, symbol: &str) -> Option<&mut SC> {
        self.outputs.get_mut(symbol)
    }
    pub fn get_output_res(&self, symbol: &str) -> Result<&SC, CircomEnvironmentError> {
        self.outputs.get(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_output_or_break(&self, symbol: &str, file: &str, line: u32) -> &SC {
        assert!(self.has_output(symbol), "Method call in file {} line {}", file, line);
        self.outputs.get(symbol).unwrap()
    }
    pub fn get_mut_output_res(&mut self, symbol: &str) -> Result<&mut SC, CircomEnvironmentError> {
        self.outputs.get_mut(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_mut_output_or_break(&mut self, symbol: &str, file: &str, line: u32) -> &mut SC {
        assert!(self.has_output(symbol), "Method call in file {} line {}", file, line);
        self.outputs.get_mut(symbol).unwrap()
    }

    pub fn get_intermediate(&self, symbol: &str) -> Option<&SC> {
        self.intermediates.get(symbol)
    }
    pub fn get_mut_intermediate(&mut self, symbol: &str) -> Option<&mut SC> {
        self.intermediates.get_mut(symbol)
    }
    pub fn get_intermediate_res(&self, symbol: &str) -> Result<&SC, CircomEnvironmentError> {
        self.intermediates.get(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_intermediate_or_break(&self, symbol: &str, file: &str, line: u32) -> &SC {
        assert!(self.has_intermediate(symbol), "Method call in file {} line {}", file, line);
        self.intermediates.get(symbol).unwrap()
    }
    pub fn get_mut_intermediate_res(
        &mut self,
        symbol: &str,
    ) -> Result<&mut SC, CircomEnvironmentError> {
        self.intermediates.get_mut(symbol).ok_or_else(|| CircomEnvironmentError::NonExistentSymbol)
    }
    pub fn get_mut_intermediate_or_break(
        &mut self,
        symbol: &str,
        file: &str,
        line: u32,
    ) -> &mut SC {
        assert!(self.has_intermediate(symbol), "Method call in file {} line {}", file, line);
        self.intermediates.get_mut(symbol).unwrap()
    }

    pub fn get_signal(&self, symbol: &str) -> Option<&SC> {
        if self.has_input(symbol) {
            self.get_input(symbol)
        } else if self.has_output(symbol) {
            self.get_output(symbol)
        } else if self.has_intermediate(symbol) {
            self.get_intermediate(symbol)
        } else {
            Option::None
        }
    }
    pub fn get_mut_signal(&mut self, symbol: &str) -> Option<&mut SC> {
        if self.has_input(symbol) {
            self.get_mut_input(symbol)
        } else if self.has_output(symbol) {
            self.get_mut_output(symbol)
        } else if self.has_intermediate(symbol) {
            self.get_mut_intermediate(symbol)
        } else {
            Option::None
        }
    }
    pub fn get_signal_res(&self, symbol: &str) -> Result<&SC, CircomEnvironmentError> {
        if self.has_input(symbol) {
            self.get_input_res(symbol)
        } else if self.has_output(symbol) {
            self.get_output_res(symbol)
        } else if self.has_intermediate(symbol) {
            self.get_intermediate_res(symbol)
        } else {
            Result::Err(CircomEnvironmentError::NonExistentSymbol)
        }
    }
    pub fn get_signal_or_break(&self, symbol: &str, file: &str, line: u32) -> &SC {
        assert!(self.has_signal(symbol), "Method call in file {} line {}", file, line);
        if let Result::Ok(v) = self.get_signal_res(symbol) {
            v
        } else {
            unreachable!();
        }
    }
    pub fn get_mut_signal_res(&mut self, symbol: &str) -> Result<&mut SC, CircomEnvironmentError> {
        if self.has_input(symbol) {
            self.get_mut_input_res(symbol)
        } else if self.has_output(symbol) {
            self.get_mut_output_res(symbol)
        } else if self.has_intermediate(symbol) {
            self.get_mut_intermediate_res(symbol)
        } else {
            Result::Err(CircomEnvironmentError::NonExistentSymbol)
        }
    }
    pub fn get_mut_signal_or_break(&mut self, symbol: &str, file: &str, line: u32) -> &mut SC {
        assert!(self.has_signal(symbol), "Method call in file {} line {}", file, line);
        if let Result::Ok(v) = self.get_mut_signal_res(symbol) {
            v
        } else {
            unreachable!();
        }
    }
}

#[derive(Clone)]
struct VariableBlock<VC> {
    variables: HashMap<String, VC>,
}
impl<VC> Default for VariableBlock<VC> {
    fn default() -> Self {
        VariableBlock { variables: HashMap::new() }
    }
}
impl<VC> VariableBlock<VC> {
    pub fn new() -> VariableBlock<VC> {
        VariableBlock::default()
    }
    pub fn add_variable(&mut self, symbol: &str, content: VC) {
        self.variables.insert(symbol.to_string(), content);
    }
    pub fn remove_variable(&mut self, symbol: &str) {
        self.variables.remove(symbol);
    }
    pub fn contains_variable(&self, symbol: &str) -> bool {
        self.variables.contains_key(symbol)
    }
    pub fn get_variable(&self, symbol: &str) -> &VC {
        assert!(self.contains_variable(symbol));
        self.variables.get(symbol).unwrap()
    }
    pub fn get_mut_variable(&mut self, symbol: &str) -> &mut VC {
        assert!(self.contains_variable(symbol));
        self.variables.get_mut(symbol).unwrap()
    }
    pub fn merge(
        left: VariableBlock<VC>,
        right: VariableBlock<VC>,
        using: fn(VC, VC) -> VC,
    ) -> VariableBlock<VC> {
        let left_block = left.variables;
        let right_block = right.variables;
        let result_block = hashmap_union(left_block, right_block, using);
        VariableBlock { variables: result_block }
    }
}

fn hashmap_union<K, V>(
    l: HashMap<K, V>,
    mut r: HashMap<K, V>,
    merge_function: fn(V, V) -> V,
) -> HashMap<K, V>
where
    K: Hash + Eq,
{
    let mut result = HashMap::new();
    for (k, v) in l {
        if let Option::Some(r_v) = r.remove(&k) {
            result.insert(k, merge_function(v, r_v));
        } else {
            result.insert(k, v);
        }
    }
    for (k, v) in r {
        result.entry(k).or_insert(v);
    }
    result
}
