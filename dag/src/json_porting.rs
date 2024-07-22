use super::{Tree, DAG};
use circom_algebra::algebra::Constraint;
use circom_algebra::num_bigint::BigInt;
use constraint_writers::json_writer::ConstraintJSON;
use json::JsonValue;
use virtual_fs::{FileSystem, FsResult, VPath};
use std::collections::HashMap;

type C = Constraint<usize>;

fn transform_constraint_to_json(constraint: &C) -> JsonValue {
    JsonValue::Array(vec![
        hashmap_as_json(constraint.a()),
        hashmap_as_json(constraint.b()),
        hashmap_as_json(constraint.c()),
    ])
}
fn hashmap_as_json(values: &HashMap<usize, BigInt>) -> JsonValue {
    let mut order: Vec<&usize> = values.keys().collect();
    order.sort();
    let mut correspondence = json::object! {};
    for i in order {
        let (key, value) = values.get_key_value(i).unwrap();
        let value = value.to_str_radix(10);
        correspondence[format!("{}", key)] = value.as_str().into();
    }
    correspondence
}

fn visit_tree(tree: &Tree, writer: &mut ConstraintJSON) {
    for constraint in &tree.constraints {
        let json_value = transform_constraint_to_json(&constraint);
        writer.write_constraint(&json_value.to_string());
    }
    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        visit_tree(&subtree, writer);
    }
}

pub fn port_constraints(fs: &mut dyn FileSystem, dag: &DAG, json_constraints_file: &VPath) -> FsResult<()> {
    let mut writer = ConstraintJSON::new();
    visit_tree(&Tree::new(dag), &mut writer);

    fs.write(&json_constraints_file, &writer.data)
}
