use super::ir_interface::*;

#[derive(Clone)]
pub enum LocationRule {
    Indexed { location: InstructionPointer, template_header: Option<String> },
    Mapped { signal_code: usize, indexes: Vec<InstructionPointer>, offset: usize },
}

impl ToString for LocationRule {
    fn to_string(&self) -> String {
        use LocationRule::*;
        match self {
            Indexed { location, template_header } => {
                let location_msg = location.to_string();
                let header_msg = template_header.as_ref().map_or("NONE".to_string(), |v| v.clone());
                format!("INDEXED: ({}, {})", location_msg, header_msg)
            }
            Mapped { signal_code, indexes, offset } => {
                let code_msg = signal_code.to_string();
                let index_mgs: Vec<String> = indexes.iter().map(|i| i.to_string()).collect();
                format!("MAPPED: ({}, {:?} {})", code_msg, index_mgs, offset)
            }
        }
    }
}
