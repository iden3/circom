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
    Input {status: StatusInput, needs_decrement: bool},
}

impl ToString for InputInformation {
    fn to_string(&self) -> String {
        use InputInformation::*;
        match self {
            NoInput => "NO_INPUT".to_string(),
            Input { status , needs_decrement} => {
                format!("status {} needs decrement {}",  
                    match status {
                        StatusInput::Last => "LAST",
                        StatusInput::NoLast => "NO_LAST",
                        StatusInput::Unknown => "UNKNOWN",
                    },
                    needs_decrement
                )
            }
        }
    }
}

#[derive(Clone)]
pub enum AddressType {
    Variable,
    Signal,
    SubcmpSignal { 
        cmp_address: InstructionPointer, 
        uniform_parallel_value: Option<bool>, 
        is_output: bool, 
        input_information: InputInformation,
        is_anonymous: bool,
        cmp_name: String,
    },
}

impl ToString for AddressType {
    fn to_string(&self) -> String {
        use AddressType::*;
        match self {
            Variable => "VARIABLE".to_string(),
            Signal => "SIGNAL".to_string(),
            SubcmpSignal { cmp_address, input_information, .. } => format!("SUBCOMPONENT:{}:{}", cmp_address.to_string(), input_information.to_string()),
        }
    }
}
