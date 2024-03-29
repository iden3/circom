use super::{ConstraintStorage, EncodingIterator, SFrames, C};
use circom_algebra::num_bigint::BigInt;
use circom_algebra::simplification_utils::fast_encoded_constraint_substitution;
use std::collections::LinkedList;

pub fn obtain_and_simplify_non_linear(
    mut iter: EncodingIterator,
    storage: &mut ConstraintStorage,
    frames: &SFrames,
    field: &BigInt,
) -> LinkedList<C> {
    let mut linear = LinkedList::new();
    let (_, non_linear) = EncodingIterator::take(&mut iter);
    for mut constraint in non_linear {
        for frame in frames {
            fast_encoded_constraint_substitution(&mut constraint, frame, field);
        }
        C::fix_constraint(&mut constraint, field);
        if C::is_linear(&constraint) {
            linear.push_back(constraint);
        } else {
            storage.add_constraint(constraint);
        }
    }
    for edge in EncodingIterator::edges(&iter) {
        let next = EncodingIterator::next(&iter, edge);
        let mut linear_in_next = obtain_and_simplify_non_linear(next, storage, frames, field);
        linear.append(&mut linear_in_next);
    }
    linear
}
