use program_structure::ast::Statement;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{self, FileID};
use program_structure::template_data::TemplateData;

pub fn free_of_returns(template_data: &TemplateData) -> Result<(), ReportCollection> {
    let file_id = template_data.get_file_id();
    let template_body = template_data.get_body();
    let mut reports = ReportCollection::new();
    look_for_return(&template_body, file_id, &mut reports);
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn look_for_return(stmt: &Statement, file_id: FileID, reports: &mut ReportCollection) {
    use Statement::*;
    match stmt {
        IfThenElse { if_case, else_case, .. } => {
            look_for_return(if_case, file_id, reports);
            if let Option::Some(else_block) = else_case {
                look_for_return(else_block, file_id, reports);
            }
        }
        While { stmt, .. } => {
            look_for_return(stmt, file_id, reports);
        }
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                look_for_return(stmt, file_id, reports);
            }
        }
        Return { meta, .. } => {
            let mut report = Report::error(
                "Return found in template".to_string(),
                ReportCode::TemplateWithReturnStatement,
            );
            report.add_primary(
                file_definition::generate_file_location(meta.get_start(), meta.get_end()),
                file_id,
                "This return statement is inside a template".to_string(),
            );
            reports.push(report);
        }
        _ => {}
    };
}
