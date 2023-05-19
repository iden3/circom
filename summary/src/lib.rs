use std::fs::File;
use compiler::hir::very_concrete_program::VCP;
use compiler::intermediate_representation::translate::TemplateDB;
use program_structure::ast::SignalType;
use code_producers::llvm_elements::run_fn_name;
use serde::Serialize;


#[derive(Serialize)]
struct Meta {
    is_ir_ssa: bool
}

#[derive(Serialize)]
struct SignalSummary {
    name: String,
    dimensions: Vec<usize>,
    visibility: String,
    idx: usize
}

#[derive(Serialize)]
struct TemplateSummary {
    name: String,
    main: bool,
    params: Vec<String>,
    signals: Vec<SignalSummary>,
    logic_fn_name: String
}

#[derive(Serialize)]
struct FunctionSummary {
    name: String,
    params: Vec<String>,
    logic_fn_name: String,
}

#[derive(Serialize)]
pub struct SummaryRoot {
    version: String,
    compiler: String,
    framework: Option<String>,
    meta: Meta,
    components: Vec<TemplateSummary>,
    functions: Vec<FunctionSummary>
}

impl SummaryRoot {
    pub fn new(vcp: &VCP) -> SummaryRoot {
        let meta = Meta { is_ir_ssa: false };
        let funcs = vec![];
        let mut templates = vec![];

        let template_database = TemplateDB::build(&vcp.templates);
        for (template_name, template_id) in template_database.indexes {
            let signal_names = template_database.signal_info[template_id].keys();
            let mut signals = vec![];
            for signal in signal_names {
                let signal_info = &template_database.signal_info[template_id][signal];
                let mut dims = vec![];
                for d in &signal_info.lengths {
                    dims.push(*d)
                }
                let signal_idx = template_database.signals_id[template_id][signal];

                let signal_summary = SignalSummary {
                    name: signal.to_string(),
                    dimensions: dims,
                    visibility: match signal_info.signal_type {
                        SignalType::Output => "output",
                        SignalType::Input => "input",
                        SignalType::Intermediate => "intermediate"
                    }.to_string(),
                    idx: signal_idx
                };
                signals.push(signal_summary);
            }

            let template = TemplateSummary {
                name: template_name.clone(),
                main: template_id == vcp.main_id,
                params: vec![],
                signals,
                logic_fn_name: run_fn_name(format!("{}_{}", template_name, template_id)),
            };
            templates.push(template);
        }

        SummaryRoot {
            version: env!("CARGO_PKG_VERSION").to_string(),
            compiler: "circom".to_string(),
            framework: None,
            meta,
            components: templates,
            functions: funcs,
        }
    }

    pub fn write_to_file(self, summary_file: &str) -> Result<(), serde_json::Error> {
        let writer = File::create(summary_file).unwrap();
        serde_json::to_writer(&writer, &self)
    }
}