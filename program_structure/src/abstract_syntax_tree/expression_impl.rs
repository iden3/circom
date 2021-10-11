use super::ast::*;

impl Expression {
    pub fn get_meta(&self) -> &Meta {
        use Expression::*;
        match self {
            InfixOp { meta, .. }
            | PrefixOp { meta, .. }
            | InlineSwitchOp { meta, .. }
            | Variable { meta, .. }
            | Number(meta, ..)
            | Call { meta, .. }
            | ArrayInLine { meta, .. } => meta,
        }
    }
    pub fn get_mut_meta(&mut self) -> &mut Meta {
        use Expression::*;
        match self {
            InfixOp { meta, .. }
            | PrefixOp { meta, .. }
            | InlineSwitchOp { meta, .. }
            | Variable { meta, .. }
            | Number(meta, ..)
            | Call { meta, .. }
            | ArrayInLine { meta, .. } => meta,
        }
    }

    pub fn is_array(&self) -> bool {
        use Expression::*;
        if let ArrayInLine { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_infix(&self) -> bool {
        use Expression::*;
        if let InfixOp { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_prefix(&self) -> bool {
        use Expression::*;
        if let PrefixOp { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_switch(&self) -> bool {
        use Expression::*;
        if let InlineSwitchOp { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_variable(&self) -> bool {
        use Expression::*;
        if let Variable { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_number(&self) -> bool {
        use Expression::*;
        if let Number(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_call(&self) -> bool {
        use Expression::*;
        if let Call { .. } = self {
            true
        } else {
            false
        }
    }
}

impl FillMeta for Expression {
    fn fill(&mut self, file_id: usize, element_id: &mut usize) {
        use Expression::*;
        self.get_mut_meta().elem_id = *element_id;
        *element_id += 1;
        match self {
            Number(meta, _) => fill_number(meta, file_id, element_id),
            Variable { meta, access, .. } => fill_variable(meta, access, file_id, element_id),
            InfixOp { meta, lhe, rhe, .. } => fill_infix(meta, lhe, rhe, file_id, element_id),
            PrefixOp { meta, rhe, .. } => fill_prefix(meta, rhe, file_id, element_id),
            InlineSwitchOp { meta, cond, if_false, if_true, .. } => {
                fill_inline_switch_op(meta, cond, if_true, if_false, file_id, element_id)
            }
            Call { meta, args, .. } => fill_call(meta, args, file_id, element_id),
            ArrayInLine { meta, values, .. } => {
                fill_array_inline(meta, values, file_id, element_id)
            }
        }
    }
}

fn fill_number(meta: &mut Meta, file_id: usize, _element_id: &mut usize) {
    meta.set_file_id(file_id);
}

fn fill_variable(meta: &mut Meta, access: &mut [Access], file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    for acc in access {
        if let Access::ArrayAccess(e) = acc {
            e.fill(file_id, element_id)
        }
    }
}

fn fill_infix(
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

fn fill_prefix(meta: &mut Meta, rhe: &mut Expression, file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    rhe.fill(file_id, element_id);
}

fn fill_inline_switch_op(
    meta: &mut Meta,
    cond: &mut Expression,
    if_true: &mut Expression,
    if_false: &mut Expression,
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    cond.fill(file_id, element_id);
    if_true.fill(file_id, element_id);
    if_false.fill(file_id, element_id);
}

fn fill_call(meta: &mut Meta, args: &mut [Expression], file_id: usize, element_id: &mut usize) {
    meta.set_file_id(file_id);
    for a in args {
        a.fill(file_id, element_id);
    }
}

fn fill_array_inline(
    meta: &mut Meta,
    values: &mut [Expression],
    file_id: usize,
    element_id: &mut usize,
) {
    meta.set_file_id(file_id);
    for v in values {
        v.fill(file_id, element_id);
    }
}
