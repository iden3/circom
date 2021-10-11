use super::ast::*;

impl Statement {
    pub fn get_meta(&self) -> &Meta {
        use Statement::*;
        match self {
            IfThenElse { meta, .. }
            | While { meta, .. }
            | Return { meta, .. }
            | Declaration { meta, .. }
            | Substitution { meta, .. }
            | LogCall { meta, .. }
            | Block { meta, .. }
            | Assert { meta, .. }
            | ConstraintEquality { meta, .. }
            | InitializationBlock { meta, .. } => meta,
        }
    }
    pub fn get_mut_meta(&mut self) -> &mut Meta {
        use Statement::*;
        match self {
            IfThenElse { meta, .. }
            | While { meta, .. }
            | Return { meta, .. }
            | Declaration { meta, .. }
            | Substitution { meta, .. }
            | LogCall { meta, .. }
            | Block { meta, .. }
            | Assert { meta, .. }
            | ConstraintEquality { meta, .. }
            | InitializationBlock { meta, .. } => meta,
        }
    }

    pub fn is_if_then_else(&self) -> bool {
        use Statement::IfThenElse;
        if let IfThenElse { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_while(&self) -> bool {
        use Statement::While;
        if let While { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_return(&self) -> bool {
        use Statement::Return;
        if let Return { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_initialization_block(&self) -> bool {
        use Statement::InitializationBlock;
        if let InitializationBlock { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_declaration(&self) -> bool {
        use Statement::Declaration;
        if let Declaration { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_substitution(&self) -> bool {
        use Statement::Substitution;
        if let Substitution { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_constraint_equality(&self) -> bool {
        use Statement::ConstraintEquality;
        if let ConstraintEquality { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_log_call(&self) -> bool {
        use Statement::LogCall;
        if let LogCall { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_block(&self) -> bool {
        use Statement::Block;
        if let Block { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_assert(&self) -> bool {
        use Statement::Assert;
        if let Assert { .. } = self {
            true
        } else {
            false
        }
    }
}

impl FillMeta for Statement {
    fn fill(&mut self, file_id: usize, element_id: &mut usize) {
        use Statement::*;
        self.get_mut_meta().elem_id = *element_id;
        *element_id += 1;
        match self {
            IfThenElse { meta, cond, if_case, else_case, .. } => {
                fill_conditional(meta, cond, if_case, else_case, file_id, element_id)
            }
            While { meta, cond, stmt } => fill_while(meta, cond, stmt, file_id, element_id),
            Return { meta, value } => fill_return(meta, value, file_id, element_id),
            InitializationBlock { meta, initializations, .. } => {
                fill_initialization(meta, initializations, file_id, element_id)
            }
            Declaration { meta, dimensions, .. } => {
                fill_declaration(meta, dimensions, file_id, element_id)
            }
            Substitution { meta, access, rhe, .. } => {
                fill_substitution(meta, access, rhe, file_id, element_id)
            }
            ConstraintEquality { meta, lhe, rhe } => {
                fill_constraint_equality(meta, lhe, rhe, file_id, element_id)
            }
            LogCall { meta, arg, .. } => fill_log_call(meta, arg, file_id, element_id),
            Block { meta, stmts, .. } => fill_block(meta, stmts, file_id, element_id),
            Assert { meta, arg, .. } => fill_assert(meta, arg, file_id, element_id),
        }
    }
}

fn fill_conditional(
    meta: &mut Meta,
    cond: &mut Expression,
    if_case: &mut Statement,
    else_case: &mut Option<Box<Statement>>,
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    cond.fill(file_id, element_id);
    if_case.fill(file_id, element_id);
    if let Option::Some(s) = else_case {
        s.fill(file_id, element_id);
    }
}

fn fill_while(
    meta: &mut Meta,
    cond: &mut Expression,
    stmt: &mut Statement,
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    cond.fill(file_id, element_id);
    stmt.fill(file_id, element_id);
}

fn fill_return(meta: &mut Meta, value: &mut Expression, file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    value.fill(file_id, element_id);
}

fn fill_initialization(
    meta: &mut Meta,
    initializations: &mut [Statement],
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    for init in initializations {
        init.fill(file_id, element_id);
    }
}

fn fill_declaration(
    meta: &mut Meta,
    dimensions: &mut [Expression],
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    for d in dimensions {
        d.fill(file_id, element_id);
    }
}

fn fill_substitution(
    meta: &mut Meta,
    access: &mut [Access],
    rhe: &mut Expression,
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    rhe.fill(file_id, element_id);
    for a in access {
        if let Access::ArrayAccess(e) = a {
            e.fill(file_id, element_id);
        }
    }
}

fn fill_constraint_equality(
    meta: &mut Meta,
    lhe: &mut Expression,
    rhe: &mut Expression,
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    lhe.fill(file_id, element_id);
    rhe.fill(file_id, element_id);
}

fn fill_log_call(meta: &mut Meta, arg: &mut Expression, file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    arg.fill(file_id, element_id);
}

fn fill_block(meta: &mut Meta, stmts: &mut [Statement], file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    for s in stmts {
        s.fill(file_id, element_id);
    }
}

fn fill_assert(meta: &mut Meta, arg: &mut Expression, file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    arg.fill(file_id, element_id);
}
