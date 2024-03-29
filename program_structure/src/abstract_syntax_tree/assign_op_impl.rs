use super::ast::AssignOp;

impl AssignOp {
    pub fn is_signal_operator(self) -> bool {
        use AssignOp::*;
        matches!(self, AssignConstraintSignal | AssignSignal)
    }
}
