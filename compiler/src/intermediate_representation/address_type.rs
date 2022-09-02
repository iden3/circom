use super::ir_interface::*;

#[derive(Clone)]
pub enum StatusInput {
    Last,
    NoLast,
    Unknown,
}

#[derive(Clone)]
pub enum InputInformation {
    NoInput,
    Input {status: StatusInput},
}

#[derive(Clone)]
pub enum AddressType {
    Variable,
    Signal,
    SubcmpSignal { cmp_address: InstructionPointer, uniform_parallel_value: Option<bool>, is_output: bool, input_information: InputInformation },
}

impl ToString for AddressType {
    fn to_string(&self) -> String {
        use AddressType::*;
        match self {
            Variable => "VARIABLE".to_string(),
            Signal => "SIGNAL".to_string(),
            SubcmpSignal { cmp_address, .. } => format!("SUBCOMPONENT:{}", cmp_address.to_string()),
        }
    }
}
