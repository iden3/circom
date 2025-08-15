use super::ast::*;
use super::expression_builders::*;
use super::statement_builders::*;
use crate::ast::{Access, Expression, VariableType};
use num_bigint::BigInt;

#[derive(Clone)]
pub struct Symbol {
    pub name: String,
    pub is_array: Vec<Expression>,
    pub init: Option<Expression>,
}

pub struct TupleInit{
    pub tuple_init : (AssignOp,Expression)
}

pub fn assign_with_op_shortcut(
    op: ExpressionInfixOpcode,
    meta: Meta,
    variable: (String, Vec<Access>),
    rhe: Expression,
) -> Statement {
    let (var, access) = variable;
    let variable = build_variable(meta.clone(), var.clone(), access.clone());
    let infix = build_infix(meta.clone(), variable, op, rhe);
    build_substitution(meta, var, access, AssignOp::AssignVar, infix)
}

pub fn plusplus(meta: Meta, variable: (String, Vec<Access>), field: &BigInt) -> Statement {
    let one = build_number(meta.clone(), BigInt::from(1), field);
    assign_with_op_shortcut(ExpressionInfixOpcode::Add, meta, variable, one)
}

pub fn subsub(meta: Meta, variable: (String, Vec<Access>), field: &BigInt) -> Statement {
    let one = build_number(meta.clone(), BigInt::from(1), field);
    assign_with_op_shortcut(ExpressionInfixOpcode::Sub, meta, variable, one)
}

pub fn for_into_while(
    meta: Meta,
    init: Statement,
    cond: Expression,
    step: Statement,
    body: Statement,
) -> Statement {
    let while_body = build_block(body.get_meta().clone(), vec![body, step]);
    let while_statement = build_while_block(meta.clone(), cond, while_body);
    build_block(meta, vec![init, while_statement])
}

pub fn split_declaration_into_single_nodes(
    meta: Meta,
    xtype: VariableType,
    symbols: Vec<Symbol>,
    op: AssignOp,
    flag_no_init: bool
) -> Statement {
    use crate::ast_shortcuts::VariableType::Var;

    let mut initializations = Vec::new();

    for symbol in symbols {
        let with_meta = meta.clone();
        let has_type = xtype.clone();
        let name = symbol.name.clone();
        let dimensions = symbol.is_array;
        let possible_init = symbol.init;
        let single_declaration = build_declaration(with_meta, has_type, name, dimensions.clone(), false);
        initializations.push(single_declaration);
        // For the variables, we need to initialize them to 0 in case:
        //     - They are not initialized to other value
        //     - They are arrays (and maybe not all positions are initialized)
        // in case flag no_init we do not perform these changes

        if xtype == Var && (possible_init.is_none() || dimensions.len() > 0) && !flag_no_init{
            let mut value = Expression:: Number(meta.clone(), BigInt::from(0));
            for dim_expr in dimensions.iter().rev(){
                value = build_uniform_array(meta.clone(), value, dim_expr.clone());
            }

            let substitution = 
                build_substitution(meta.clone(), symbol.name.clone(), vec![], op, value);
            initializations.push(substitution);
        }
        
        if let Option::Some(init) = possible_init {
            let substitution =
                build_substitution(meta.clone(), symbol.name, vec![], op, init);
            initializations.push(substitution);
        }

    }
    build_initialization_block(meta, xtype, initializations)
}

pub fn split_declaration_into_single_nodes_and_multisubstitution(
    meta: Meta,
    xtype: VariableType,
    symbols: Vec<Symbol>,
    init: Option<TupleInit>,
    flag_no_init: bool
) -> Statement {
    use crate::ast_shortcuts::VariableType::Var;

    let mut initializations = Vec::new();
    let mut values = Vec::new();
    for symbol in symbols {
        let with_meta = meta.clone();
        let has_type = xtype.clone();
        let name = symbol.name.clone();
        let dimensions = symbol.is_array;
        debug_assert!(symbol.init.is_none());
        let single_declaration = build_declaration(with_meta.clone(), has_type, name.clone(), dimensions.clone(), false);
        initializations.push(single_declaration);
        
        if xtype == Var && (init.is_none() || dimensions.len() > 0) &&!flag_no_init {
            let mut value = Expression:: Number(meta.clone(), BigInt::from(0));
            for dim_expr in dimensions.iter().rev(){
                value = build_uniform_array(meta.clone(), value, dim_expr.clone());
            }

            let substitution = 
                build_substitution(meta.clone(), symbol.name, vec![], AssignOp::AssignVar, value);
            initializations.push(substitution);
        }
        values.push(Expression::Variable { meta: with_meta.clone(), name: name, access: Vec::new() })
    }
    if let Some( tuple) = init {
        let (op,expression) = tuple.tuple_init;
        let multi_sub = if values.len() == 1 {
            if let Expression::Variable { name, .. } = values.get(0).unwrap() {
                build_substitution(meta.clone(), name.clone(), Vec::new(), op, expression)
            } else { unreachable!();}
        } else{
            build_mult_substitution(meta.clone(), build_tuple(meta.clone(),values), op, expression)
        };
        initializations.push(multi_sub);
    }
    build_initialization_block(meta, xtype, initializations)
}



pub fn split_bus_declaration_into_single_nodes(
    meta: Meta,
    bustype: Expression,
    xtype: VariableType,
    symbols: Vec<Symbol>,
    op: AssignOp,
) -> Statement {
    use crate::ast_shortcuts::AssignOp::AssignVar;

    let mut initializations = Vec::new();

    for symbol in symbols {
        let with_meta = meta.clone();
        let has_type = xtype.clone();
        let name = symbol.name.clone();
        let dimensions = symbol.is_array;
        let possible_init = symbol.init;
        let single_declaration = build_declaration(with_meta, has_type, name, dimensions.clone(), false);
        
        let mut value = bustype.clone();
        for dim_expr in dimensions.iter().rev(){
            value = build_uniform_array(meta.clone(), value, dim_expr.clone());
        }
        
        let bus_declaration = build_substitution(meta.clone(), symbol.name.clone(), vec![], AssignVar, value);
        initializations.push(single_declaration);
        initializations.push(bus_declaration);

        if let Option::Some(init) = possible_init {
            let substitution =
                build_substitution(meta.clone(), symbol.name, vec![], op, init);
            initializations.push(substitution);
        }
    }
    build_initialization_block(meta, xtype, initializations)
}

pub fn split_bus_declaration_into_single_nodes_and_multisubstitution(
    meta: Meta,
    bustype: Expression,
    xtype: VariableType,
    symbols: Vec<Symbol>,
    init: Option<TupleInit>,
) -> Statement {
    use crate::ast_shortcuts::AssignOp::AssignVar;

    let mut initializations = Vec::new();
    let mut values = Vec::new();
    for symbol in symbols {
        let with_meta = meta.clone();
        let has_type = xtype.clone();
        let name = symbol.name.clone();
        let dimensions = symbol.is_array;
        debug_assert!(symbol.init.is_none());
        let single_declaration = build_declaration(with_meta.clone(), has_type, name.clone(), dimensions.clone(), false);
        let bus_declaration = build_substitution(meta.clone(), symbol.name.clone(), vec![], AssignVar, bustype.clone());
        initializations.push(single_declaration);
        initializations.push(bus_declaration);
        values.push(Expression::Variable { meta: with_meta.clone(), name: name, access: Vec::new() })
    }
    if let Some(tuple) = init {
        let (op,expression) = tuple.tuple_init;
        let multi_sub = build_mult_substitution(meta.clone(), build_tuple(meta.clone(), values), op, expression);
        initializations.push(multi_sub);
    }
    build_initialization_block(meta, xtype, initializations)
}
