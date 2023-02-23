use crate::hir::very_concrete_program::VCP;
use program_structure::ast::*;

pub fn rm_component_ci(vcp: &mut VCP) {
    for template in &mut vcp.templates {
        rm_statement(&mut template.code);
    }
}

fn rm_statement(stmt: &mut Statement) {
    if stmt.is_while() {
        rm_while(stmt);
    } else if stmt.is_if_then_else() {
        rm_if_then_else(stmt);
    } else if stmt.is_block() {
        rm_block(stmt);
    } else if stmt.is_initialization_block() {
        rm_init(stmt);
    } else if stmt.is_substitution(){ 
        rm_substitution(stmt);
    } else if stmt.is_underscore_substitution(){ 
        rm_underscore_substitution(stmt);
    }
}

fn rm_underscore_substitution(stmt: &mut Statement){
    use Statement::{Block, UnderscoreSubstitution};
    if let UnderscoreSubstitution { meta, .. } = stmt{
        *stmt = Block{ meta: meta.clone(), stmts: Vec::new() };
    }
}

fn rm_block(stmt: &mut Statement) {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        let filter = std::mem::take(stmts);
        for mut s in filter {
            rm_statement(&mut s);
            if !should_be_removed(&s) {
                stmts.push(s);
            }
        }
    } else {
        unreachable!()
    }
}

fn rm_if_then_else(stmt: &mut Statement) {
    use Statement::IfThenElse;
    if let IfThenElse { if_case, else_case, .. } = stmt {
        rm_statement(if_case);
        if let Option::Some(s) = else_case {
            rm_statement(s);
        }
    } else {
        unreachable!()
    }
}

fn rm_while(stmt: &mut Statement) {
    use Statement::While;
    if let While { stmt, .. } = stmt {
        rm_statement(stmt);
    } else {
        unreachable!()
    }
}

fn rm_init(stmt: &mut Statement) {
    use Statement::InitializationBlock;
    use VariableType::*;
    if let InitializationBlock { initializations, xtype, .. } = stmt {
        if let Signal(..) = xtype {
            let work = std::mem::take(initializations);
            for mut i in work {
                if i.is_substitution() {
                    initializations.push(i);
                }
                else if i.is_block(){
                    rm_block(&mut i);
                    initializations.push(i);
                }
            }
        } else {
            let filter = std::mem::take(initializations);
            for mut s in filter {
                rm_statement(&mut s);
                if !should_be_removed(&s) {
                    initializations.push(s);
                }
            }
        }
    } else {
        unreachable!()
    }
}

fn rm_substitution(stmt: &mut Statement){
    use Statement::{Block, Substitution};
    if should_be_removed(stmt){
        if let Substitution { meta, .. } = stmt{
            *stmt = Block{ meta: meta.clone(), stmts: Vec::new() };
        }
    }
}

fn should_be_removed(stmt: &Statement) -> bool {
    use Statement::{InitializationBlock, Substitution};
    use VariableType::*;
    if let InitializationBlock { xtype, .. } = stmt {
        Component == *xtype || AnonymousComponent == *xtype
    } else if let Substitution { meta, .. } = stmt {
        meta.get_type_knowledge().is_component() || meta.get_type_knowledge().is_tag()
    } else {
        false
    }
}
