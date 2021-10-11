use super::num_bigint::BigInt;
use crate::num_traits::ToPrimitive;
use std::collections::{BTreeMap, HashMap, HashSet, LinkedList};

type C = crate::algebra::Constraint<usize>;
type S = crate::algebra::Substitution<usize>;
type A = crate::algebra::ArithmeticExpression<usize>;
type SH = BTreeMap<usize, S>;

#[allow(dead_code)]
fn debug_check_keys_in_order(tree: &BTreeMap<usize, S>) -> bool {
    let mut prev: i32 = -1;
    let mut ret = true;
    for i in tree.keys() {
        ret = ret && (i.to_i32().unwrap() > prev);
        prev = i.to_i32().unwrap();
    }
    ret
}

struct SignalDefinition<'a> {
    deleted_symbols: HashSet<usize>,
    forbidden: &'a HashSet<usize>,
}

impl<'a> SignalDefinition<'a> {
    pub fn can_be_taken(&self, k: usize) -> bool {
        !self.forbidden.contains(&k)
    }
    pub fn delete(&mut self, k: usize) {
        self.deleted_symbols.insert(k);
    }
    pub fn is_deleted(&self, k: usize) -> bool {
        self.deleted_symbols.contains(&k)
    }
}

fn substitution_process(
    signals: &mut SignalDefinition,
    constraints: &mut LinkedList<C>,
    substitutions: &mut SH,
    field: &BigInt,
) {
    let mut lconst = LinkedList::new();
    while let Option::Some(actual_constraint) = LinkedList::pop_back(constraints) {
        treat_constraint(signals, substitutions, &mut lconst, actual_constraint, field);
    }
    *constraints = lconst;
}

fn treat_constraint(
    signals: &mut SignalDefinition,
    substitutions: &mut SH,
    lconst: &mut LinkedList<C>,
    mut work: C,
    field: &BigInt,
) {
    loop {
        if C::is_empty(&work) {
            break;
        }
        let out = take_signal(signals, &work);
        if out.is_none() {
            LinkedList::push_back(lconst, work);
            break;
        }
        let out = out.unwrap();
        signals.delete(out);
        let substitution = C::clear_signal_from_linear(work, &out, field);
        let in_conflict = substitutions.get(&substitution.from()).cloned();
        if in_conflict.is_none() {
            substitutions.insert(*substitution.from(), substitution);
            break;
        }
        let in_conflict = in_conflict.unwrap();
        let right = S::decompose(in_conflict).1;
        let left = S::decompose(substitution).1;
        let merge = A::sub(&left, &right, field);
        work = A::transform_expression_to_constraint_form(merge, field).unwrap();
        C::remove_zero_value_coefficients(&mut work);
    }
}

fn take_signal(signals: &SignalDefinition, constraint: &C) -> Option<usize> {
    let mut ret = Option::None;
    for k in constraint.c().keys() {
        if signals.can_be_taken(*k) {
            if signals.is_deleted(*k) {
                ret = Some(*k);
                break;
            } else {
                let new_v = ret.map_or(*k, |v| std::cmp::max(*k, v));
                ret = Some(new_v);
            }
        }
    }
    ret
}

fn take_substitutions_to_be_applied<'a>(sh: &'a HashMap<usize, S>, subs: &S) -> Vec<&'a S> {
    let mut to_be_applied = vec![];
    for s in subs.to().keys() {
        if let Option::Some(s) = sh.get(s) {
            to_be_applied.push(s);
        }
    }
    to_be_applied.shrink_to_fit();
    to_be_applied
}

fn create_nonoverlapping_substitutions(possible_overlap: SH, field: &BigInt) -> HashMap<usize, S> {
    debug_assert!(debug_check_keys_in_order(&possible_overlap));
    let mut no_overlap = HashMap::with_capacity(possible_overlap.len());
    for (s, mut substitution) in possible_overlap {
        let to_be_applied = take_substitutions_to_be_applied(&no_overlap, &substitution);
        for sub in to_be_applied {
            S::apply_substitution(&mut substitution, sub, field);
        }
        no_overlap.insert(s, substitution);
    }
    no_overlap.shrink_to_fit();
    no_overlap
}

pub fn debug_substitution_check(substitutions: &HashMap<usize, S>) -> bool {
    let mut result = true;
    let mut left_hand = HashSet::new();
    for k in substitutions.keys() {
        left_hand.insert(*k);
    }
    for s in substitutions.values() {
        for signal in s.to().keys() {
            result = result && !left_hand.contains(signal);
        }
    }
    result
}

pub fn fast_encoded_constraint_substitution(c: &mut C, enc: &HashMap<usize, A>, field: &BigInt) {
    let signals = C::take_cloned_signals(c);
    for signal in signals {
        if let Some(expr) = HashMap::get(enc, &signal) {
            let sub = S::new(signal, expr.clone()).unwrap();
            C::apply_substitution(c, &sub, field);
        }
    }
}

pub fn fast_encoded_substitution_substitution(s: &mut S, enc: &HashMap<usize, A>, field: &BigInt) {
    let signals = S::take_cloned_signals(s);
    for signal in signals {
        if let Some(expr) = HashMap::get(enc, &signal) {
            let sub = S::new(signal, expr.clone()).unwrap();
            S::apply_substitution(s, &sub, field);
        }
    }
    S::rmv_zero_coefficients(s)
}

pub fn build_encoded_fast_substitutions(fast_sub: LinkedList<S>) -> HashMap<usize, A> {
    let mut encoded = HashMap::with_capacity(LinkedList::len(&fast_sub));
    for sub in fast_sub {
        let (from, to) = S::decompose(sub);
        HashMap::insert(&mut encoded, from, to);
    }
    encoded
}

pub struct Config<T> {
    pub field: BigInt,
    pub constraints: LinkedList<C>,
    pub forbidden: T,
}

pub struct Simplified {
    pub constraints: LinkedList<C>,
    pub substitutions: LinkedList<S>,
    pub removed: LinkedList<usize>,
}

pub fn full_simplification<T>(config: Config<T>) -> Simplified
where
    T: AsRef<HashSet<usize>>,
{
    let field = config.field;
    let mut signals =
        SignalDefinition { forbidden: config.forbidden.as_ref(), deleted_symbols: HashSet::new() };
    let mut constraints = config.constraints;
    let mut holder = SH::new();
    substitution_process(&mut signals, &mut constraints, &mut holder, &field);
    let non_overlapping = create_nonoverlapping_substitutions(holder, &field);
    let mut substitutions = LinkedList::new();
    let mut removed = LinkedList::new();
    for (s, v) in non_overlapping {
        LinkedList::push_back(&mut removed, s);
        LinkedList::push_back(&mut substitutions, v);
    }
    Simplified { constraints, substitutions, removed }
}
