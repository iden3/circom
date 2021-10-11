use super::{ConstraintStorage, C, S};
use crate::SignalMap;
use circom_algebra::num_bigint::BigInt;
use constraint_writers::debug_writer::DebugWriter;
use json::JsonValue;
use std::collections::HashMap;

pub fn transform_constraint_to_json(constraint: &C) -> JsonValue {
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

#[allow(unused)]
pub fn port_substitution(sub: &S) -> (String, String) {
    let to = hashmap_as_json(sub.to()).to_string();
    let from = sub.from().to_string();
    (from, to)
}

pub fn port_constraints(
    storage: &ConstraintStorage,
    map: &SignalMap,
    debug: &DebugWriter,
) -> Result<(), ()> {
    let mut writer = debug.build_constraints_file()?;
    for c_id in storage.get_ids() {
        let constraint = storage.read_constraint(c_id).unwrap();
        let constraint = C::apply_correspondence(&constraint, map);
        let json_value = transform_constraint_to_json(&constraint);
        writer.write_constraint(&json_value.to_string())?;
    }
    writer.end()
}
