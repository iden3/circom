use std::collections::HashMap;
use std::fs::File;
use compiler::hir::very_concrete_program::{Component, VCP};
use compiler::intermediate_representation::translate::{SignalInfo, TemplateDB};
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
    visibility: String,
    idx: usize,
    public: bool
}

#[derive(Serialize)]
struct SubcmpSummary {
    name: String,
    idx: usize
}

#[derive(Serialize)]
struct TemplateSummary {
    name: String,
    main: bool,
    signals: Vec<SignalSummary>,
    subcmps: Vec<SubcmpSummary>,
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

fn index_names(lengths: &[usize]) -> Vec<String> {
    if lengths.is_empty() {
        return vec!["".to_string()]
    }
    let hd = lengths[0];
    let tl = &lengths[1..lengths.len()];
    let mut res = vec![];

    for i in 0..hd {
        for acc in index_names(tl) {
            res.push(format!("[{i}]{acc}"));
        }
    }

    res
}

fn unroll_signal(name: &String, info: &SignalInfo, idx: usize) -> Vec<SignalSummary> {
    if info.lengths.is_empty() {
        return vec![SignalSummary {
            name: name.to_string(),
            visibility: match info.signal_type {
                SignalType::Output => "output",
                SignalType::Input => "input",
                SignalType::Intermediate => "intermediate"
            }.to_string(),
            public: false,
            idx
        }]
    }
    let mut signals = vec![];

    for (offset, indices) in index_names(&info.lengths).iter().enumerate() {
        signals.push(SignalSummary {
            name: format!("{name}{indices}"),
            visibility: match info.signal_type {
                SignalType::Output => "output",
                SignalType::Input => "input",
                SignalType::Intermediate => "intermediate"
            }.to_string(),
            idx: idx + offset,
            public: false
        })
    }

    signals
}

fn unroll_subcmp(name: &String, lengths: &[usize], idx: usize) -> Vec<SubcmpSummary> {
    if lengths.is_empty() {
        return vec![SubcmpSummary {
            name: name.to_string(),
            idx
        }]
    }

    let mut subcmps = vec![];

    for (offset, indices)  in index_names(lengths).iter().enumerate() {
        subcmps.push(SubcmpSummary {
            name: format!("{name}{indices}"),
            idx: idx + offset
        })
    }

    subcmps
}

impl SummaryRoot {
    pub fn new(vcp: &VCP) -> SummaryRoot {
        let meta = Meta { is_ir_ssa: false };
        let mut templates = vec![];

        let mut subcmps_data = HashMap::<String, &Vec<Component>>::new();
        for i in &vcp.templates {
            subcmps_data.insert(i.template_name.clone(), &i.components);
        }

        let template_database = TemplateDB::build(&vcp.templates);
        for (template_name, template_id) in template_database.indexes {
            let mut signals = vec![];

            let mut signals_data: Vec<(String, usize)> = template_database.signals_id[template_id].clone().into_iter().collect();
            signals_data.sort_by_key(|(_, x)| *x);
            for (signal_name, _signal_idx) in &signals_data {
                let signal_info = &template_database.signal_info[template_id][signal_name];

                for signal_summary in unroll_signal(signal_name, signal_info, signals.len()) {
                    signals.push(signal_summary);
                }
            }

            let mut subcmps = vec![];
            for subcmp in subcmps_data[&template_name] {
                for subcmp_summary in unroll_subcmp(&subcmp.name, &subcmp.lengths, subcmps.len()) {
                    subcmps.push(subcmp_summary);
                }
            }

            let template = TemplateSummary {
                name: template_name.clone(),
                main: template_id == vcp.main_id,
                subcmps,
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
            functions: vec![],
        }
    }

    pub fn write_to_file(self, summary_file: &str) -> Result<(), serde_json::Error> {
        let writer = File::create(summary_file).unwrap();
        serde_json::to_writer(&writer, &self)
    }
}