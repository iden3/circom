use program_structure::ast::produce_report_with_message;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::Report;
use virtual_fs::{FileSystem, VPath};
use std::collections::{HashMap, HashSet};

pub struct FileStack {
    current_location: VPath,
    black_paths: HashSet<VPath>,
    stack: Vec<VPath>,
}

impl FileStack {
    pub fn new(src: VPath) -> FileStack {
        let mut location = src.clone();
        location.pop();
        FileStack { current_location: location, black_paths: HashSet::new(), stack: vec![src] }
    }

    pub fn add_include(
        fs: &mut dyn FileSystem,
        f_stack: &mut FileStack,
        name: String,
        libraries: &Vec<VPath>,
    ) -> Result<String, Report> {
        let mut libraries2 = Vec::new();
        libraries2.push(f_stack.current_location.clone());
        libraries2.append(&mut libraries.clone());
        for lib in libraries2 {
            let mut path = lib.clone();
            path.push(&name);

            if fs.exists(&path).map_err(|_| produce_report_with_message(ReportCode::FileOs, "io failed".into()))? {
                if !f_stack.black_paths.contains(&path) {
                    f_stack.stack.push(path.clone());
                }
                return Ok(path.to_string());
            }
        }
        Err(produce_report_with_message(ReportCode::IncludeNotFound, name))
    }

    pub fn take_next(f_stack: &mut FileStack) -> Option<VPath> {
        loop {
            match f_stack.stack.pop() {
                None => {
                    break None;
                }
                Some(file) if !f_stack.black_paths.contains(&file) => {
                    f_stack.current_location = file.clone();
                    f_stack.current_location.pop();
                    f_stack.black_paths.insert(file.clone());
                    break Some(file);
                }
                _ => {}
            }
        }
    }
}

pub struct IncludesNode {
    pub path: VPath,
    pub custom_gates_pragma: bool,
}

#[derive(Default)]
pub struct IncludesGraph {
    nodes: Vec<IncludesNode>,
    adjacency: HashMap<VPath, Vec<usize>>,
    custom_gates_nodes: Vec<usize>,
}

impl IncludesGraph {
    pub fn new() -> IncludesGraph {
        IncludesGraph::default()
    }

    pub fn add_node(&mut self, path: VPath, custom_gates_pragma: bool, custom_gates_usage: bool) {
        self.nodes.push(IncludesNode { path, custom_gates_pragma });
        if custom_gates_usage {
            self.custom_gates_nodes.push(self.nodes.len() - 1);
        }
    }

    pub fn add_edge(&mut self, old_path: String) -> Result<(), Report> {
        let edges = self.adjacency.entry(old_path.into()).or_insert(vec![]);
        edges.push(self.nodes.len() - 1);
        Ok(())
    }

    pub fn get_problematic_paths(&self) -> Vec<Vec<VPath>> {
        let mut problematic_paths = Vec::new();
        for from in &self.custom_gates_nodes {
            problematic_paths.append(&mut self.traverse(*from, Vec::new(), HashSet::new()));
        }
        problematic_paths
    }

    fn traverse(
        &self,
        from: usize,
        path: Vec<VPath>,
        traversed_edges: HashSet<(usize, usize)>,
    ) -> Vec<Vec<VPath>> {
        let mut problematic_paths = Vec::new();
        let (from_path, using_pragma) = {
            let node = &self.nodes[from];
            (&node.path, node.custom_gates_pragma)
        };
        let new_path = {
            let mut new_path = path.clone();
            new_path.push(from_path.clone());
            new_path
        };
        if !using_pragma {
            problematic_paths.push(new_path.clone());
        }
        if let Some(edges) = self.adjacency.get(from_path) {
            for to in edges {
                let edge = (from, *to);
                if !traversed_edges.contains(&edge) {
                    let new_traversed_edges = {
                        let mut new_traversed_edges = traversed_edges.clone();
                        new_traversed_edges.insert(edge);
                        new_traversed_edges
                    };
                    problematic_paths.append(&mut self.traverse(
                        *to,
                        new_path.clone(),
                        new_traversed_edges,
                    ));
                }
            }
        }
        problematic_paths
    }

    pub fn display_path(path: &Vec<VPath>) -> String {
        let mut res = String::new();
        let mut sep = "";
        for file in path.iter().map(|file| file.to_string()) {
            res.push_str(sep);
            let result_split = file.rsplit_once("/");
            if result_split.is_some(){
                res.push_str(result_split.unwrap().1);
            } else{
                res.push_str(&file);
            }
            sep = " -> ";
        }
        res
    }
}
