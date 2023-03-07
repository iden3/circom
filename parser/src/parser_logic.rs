use super::lang;
use program_structure::ast::{AST};
use program_structure::ast::produce_report;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{ReportCollection, Report};
use program_structure::file_definition::FileID;

pub fn preprocess(expr: &str, file_id: FileID) -> Result<String, ReportCollection> {
    let mut pp = String::new();
    let mut state = 0;
    let mut loc = 0;
    let mut block_start = 0;

    let mut it = expr.chars();
    while let Some(c0) = it.next() {
        loc += 1;
        match (state, c0) {
            (0, '/') => {
                loc += 1;
                match it.next() {
                    Some('/') => {
                        state = 1;
                        pp.push(' ');
                        pp.push(' ');
                    }
                    Some('*') => {
                        block_start = loc;
                        state = 2;
                        pp.push(' ');
                        pp.push(' ');
                    }
                    Some(c1) => {
                        pp.push(c0);
                        pp.push(c1);
                    }
                    None => {
                        pp.push(c0);
                        break;
                    }
                }
            }
            (0, _) => pp.push(c0),
            (1, '\n') => {
                pp.push(c0);
                state = 0;
            }
            (2, '*') => {
                loc += 1;
                let mut next = it.next();
                while next == Some('*') {
                    pp.push(' ');
                    loc += 1;
                    next = it.next();
                }
                match next {
                    Some('/') => {
                        pp.push(' ');
                        pp.push(' ');
                        state = 0;
                    }
                    Some(c) => {
                        pp.push(' ');
                        for _i in 0..c.len_utf8() {
                            pp.push(' ');
                        }
                    }
                    None => {}
                }
            }
            (_, c) => {
                for _i in 0..c.len_utf8() {
                    pp.push(' ');
                }
            }
        }
    }
    if state == 2 {
        Err(vec![
            produce_report(ReportCode::UnclosedComment,  block_start..block_start, file_id)
        ])
    } else {
        Ok(pp)
    }
}

pub fn parse_file(src: &str, file_id: FileID) -> Result<AST, ReportCollection> {
    use lalrpop_util::ParseError::*;

    let mut errors = Vec::new();
    let preprocess = preprocess(src, file_id)?;

    let ast = lang::ParseAstParser::new()
        .parse(file_id, &mut errors, &preprocess)
        // TODO: is this always fatal?
        .map_err(|parse_error| match parse_error {
            InvalidToken { location } => 
                produce_generic_report(
                format!("{:?}", parse_error),
                 location..location, file_id
                ),
            UnrecognizedToken { ref token, .. } => 
            produce_generic_report(
                format!("{:?}", parse_error),
                 token.0..token.2, file_id
                ),
            ExtraToken { ref token } => produce_generic_report(
                format!("{:?}", parse_error),
                 token.0..token.2, file_id
                ),
            _ => produce_generic_report(
                format!("{:?}", parse_error),
                 0..0, file_id
                )
        })
        .map_err(|e| vec![e])?;

    if !errors.is_empty() {
        return Err(errors.into_iter().collect());
    }

    Ok(ast)
}

fn produce_generic_report(format: String, token: std::ops::Range<usize>, file_id: usize) -> Report {
    let mut report = Report::error(format, ReportCode::IllegalExpression);
    report.add_primary(token, file_id, "here".to_string());
    report
}

