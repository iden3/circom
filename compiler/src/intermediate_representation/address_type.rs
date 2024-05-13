use super::ir_interface::*;

#[derive(Clone)]
pub enum StatusInput {
    Last,
    NoLast,
    Unknown,
}

impl ToString for StatusInput {
    fn to_string(&self) -> String {
        use  StatusInput::*;
        match self {
            Last => "\"Last\"".to_string(),
            NoLast => "\"NoLast\"".to_string(),
            Unknown => "\"Unknown\"".to_string(),
	}
    }
}

#[derive(Clone)]
pub enum InputInformation {
    NoInput,
    Input {status: StatusInput},
}

impl ToString for InputInformation {
    fn to_string(&self) -> String {
        use InputInformation::*;
        match self {
            NoInput => "\"NoInput\"".to_string(),
            Input { status } => format!("{{\"Input\":{} }}",status.to_string())
	}
    }
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
            Variable => "\"VARIABLE\"".to_string(),
            Signal => "\"SIGNAL\"".to_string(),
            SubcmpSignal { cmp_address, uniform_parallel_value, is_output, input_information } => {
		format!("{{\"SUBCOMPONENT\":{{\"Component_address\":{},\"Uniform_parallel_value\":{},\"Is_output\":{},\"Input_information\":{} }} }}",
			cmp_address.to_string(),
			if let Option::Some(is_uniform) = uniform_parallel_value {is_uniform.to_string()} else {"\"Nothing\"".to_string()},
			is_output.to_string(),
			input_information.to_string())
	    }
        }
    }
}
