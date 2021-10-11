use super::ast::AssignOp;

impl AssignOp {
    pub fn is_signal_operator(self) -> bool {
        use AssignOp::*;
        match self {
            AssignConstraintSignal | AssignSignal => true,
            _ => false,
        }
    }
}
