use circom_algebra::num_bigint::BigInt;
use circom_algebra::constraint_storage::ConstraintStorage;
use std::collections::{HashSet, LinkedList};


pub fn simplify(
    _storage: &mut ConstraintStorage,
    _forbidden: &HashSet<usize>,
    _field: &BigInt
) -> LinkedList<usize> {
    LinkedList::new()
}
