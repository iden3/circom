// Uncomment lines 163, 165, 336 and 338 to print cluster information
use super::{ConstraintStorage, EncodingIterator, SEncoded, Simplifier, A, C, S};
use crate::SignalMap;
use circom_algebra::num_bigint::BigInt;
use circom_algebra::modular_arithmetic;
use constraint_writers::json_writer::SubstitutionJSON;
use std::collections::{HashMap, HashSet, LinkedList, BTreeSet, VecDeque, BTreeMap};
use std::sync::Arc;
use z3::ast::{Bool, Int};
use z3::{Config, Optimize, Solver, SatResult, with_z3_config};
use std::str::FromStr;
use std::fmt::Write as FmtWrite;
use std::fs;
use bitwuzla_sys::*;
use std::ffi::CString;
use cadical::{Solver as CadicalSolver, Timeout};

fn log_substitutions(substitutions: &LinkedList<S>, writer: &mut Option<SubstitutionJSON>) {
    use super::json_porting::port_substitution;
    if let Some(w) = writer {
        for s in substitutions {
            let (from, to) = port_substitution(s);
            w.write_substitution(&from, &to).unwrap();
        }
    }
}

#[derive(Default, Clone)]
struct Cluster {
    constraints: LinkedList<C>,
    num_signals: usize,
}
impl Cluster {
    pub fn new(constraint: C, num_signals: usize) -> Cluster {
        let mut new = Cluster::default();
        LinkedList::push_back(&mut new.constraints, constraint);
        new.num_signals = num_signals;
        new
    }

    pub fn merge(mut c0: Cluster, mut c1: Cluster) -> Cluster {
        let mut result = Cluster::default();
        LinkedList::append(&mut result.constraints, &mut c0.constraints);
        LinkedList::append(&mut result.constraints, &mut c1.constraints);
        result.num_signals = c0.num_signals + c1.num_signals - 1;
        result
    }

    pub fn size(&self) -> usize {
        LinkedList::len(&self.constraints)
    }
}

fn build_clusters(linear: LinkedList<C>, no_vars: usize) -> Vec<Cluster> {
    type ClusterArena = Vec<Option<Cluster>>;
    type ClusterPath = Vec<usize>;
    fn shrink_jumps_and_find(c_to_c: &mut ClusterPath, org: usize) -> usize {
        let mut current = org;
        let mut jumps = Vec::new();
        while current != c_to_c[current] {
            Vec::push(&mut jumps, current);
            current = c_to_c[current];
        }
        while let Some(redirect) = Vec::pop(&mut jumps) {
            c_to_c[redirect] = current;
        }
        current
    }

    fn arena_merge(arena: &mut ClusterArena, c_to_c: &mut ClusterPath, src: usize, dest: usize) {
        let current_dest = shrink_jumps_and_find(c_to_c, dest);
        let current_src = shrink_jumps_and_find(c_to_c, src);
        let c0 = std::mem::replace(&mut arena[current_dest], None).unwrap_or_default();
        let c1 = std::mem::replace(&mut arena[current_src], None).unwrap_or_default();
        let merged = Cluster::merge(c0, c1);
        arena[current_dest] = Some(merged);
        c_to_c[current_src] = current_dest;
    }

    let no_linear = LinkedList::len(&linear);
    let mut arena = ClusterArena::with_capacity(no_linear);
    let mut cluster_to_current = ClusterPath::with_capacity(no_linear);
    let mut signal_to_cluster = vec![no_linear; no_vars];
    for constraint in linear {
        if !constraint.is_empty() {
            let signals = C::take_cloned_signals(&constraint);
            let dest = ClusterArena::len(&arena);
            ClusterArena::push(&mut arena, Some(Cluster::new(constraint, signals.len())));
            Vec::push(&mut cluster_to_current, dest);
            for signal in signals {
                let prev = signal_to_cluster[signal];
                signal_to_cluster[signal] = dest;
                if prev < no_linear {
                    arena_merge(&mut arena, &mut cluster_to_current, prev, dest);
                }
            }
        }
    }
    let mut clusters = Vec::new();
    for cluster in arena {
        if let Some(cluster) = cluster {
            if Cluster::size(&cluster) != 0 {
                Vec::push(&mut clusters, cluster);
            }
        }
    }

    clusters
}

// fn build_expr_clusters(
//     expressions: &Vec<(usize, HashMap<usize, BigInt>, usize)>,
//     no_vars: usize,
// ) -> Vec<Vec<usize>> {
//     type ClusterPath = Vec<usize>;

//     fn shrink_jumps_and_find(c_to_c: &mut ClusterPath, org: usize) -> usize {
//         let mut current = org;
//         let mut jumps = Vec::new();
//         while current != c_to_c[current] {
//             Vec::push(&mut jumps, current);
//             current = c_to_c[current];
//         }
//         while let Some(redirect) = Vec::pop(&mut jumps) {
//             c_to_c[redirect] = current;
//         }
//         current
//     }

//     fn arena_merge(c_to_c: &mut ClusterPath, src: usize, dest: usize) {
//         let current_dest = shrink_jumps_and_find(c_to_c, dest);
//         let current_src = shrink_jumps_and_find(c_to_c, src);

//         c_to_c[current_src] = current_dest;
//     }

//     let no_expressions = expressions.len();
//     let mut cluster_to_current: ClusterPath = Vec::with_capacity(no_expressions);
//     // For each variable, which expressions contain it
//     let mut signal_to_expressions: Vec<Vec<usize>> = vec![Vec::new(); no_vars];

//     for expression_id in 0..no_expressions {
//         let (_, expr, _) = &expressions[expression_id];

//         let constant = C::constant_coefficient();
//         let signals: Vec<usize> =
//             expr.keys().filter(|signal| **signal != constant).cloned().collect();

//         Vec::push(&mut cluster_to_current, expression_id);

//         // How much variables are shared between the previous expression and the actual one
//         let mut shared_signals: HashMap<usize, usize> = HashMap::new();

//         for signal in signals {
//             for previous_expression in &signal_to_expressions[signal] {
//                 let count = shared_signals.entry(*previous_expression).or_insert(0);
//                 *count += 1;
//             }

//             signal_to_expressions[signal].push(expression_id);
//         }

//         // Two expressions belong to the same cluster if they share at least 2 variables
//         for (previous_expression, count) in shared_signals {
//             if count >= 2 {
//                 arena_merge(&mut cluster_to_current, previous_expression, expression_id);
//             }
//         }
//     }

//     let mut clusters: Vec<Vec<usize>> = Vec::new();
//     for expression_id in 0..no_expressions {
//         let root = shrink_jumps_and_find(&mut cluster_to_current, expression_id);

//         while clusters.len() <= root {
//             clusters.push(Vec::new());
//         }

//         clusters[root].push(expression_id);
//     }

//     clusters.retain(|cluster| !cluster.is_empty());
//     clusters
// }

fn build_expr_clusters(
    expressions: &[(usize, BTreeMap<usize, BigInt>, usize)],
    field: &BigInt,
) -> Vec<Vec<usize>> {
    type ClusterPath = Vec<usize>;

    fn shrink_jumps_and_find(c_to_c: &mut ClusterPath, org: usize) -> usize {
        let mut current = org;
        let mut jumps = Vec::new();

        while current != c_to_c[current] {
            Vec::push(&mut jumps, current);
            current = c_to_c[current];
        }

        while let Some(redirect) = Vec::pop(&mut jumps) {
            c_to_c[redirect] = current;
        }

        current
    }

    fn arena_merge(c_to_c: &mut ClusterPath, src: usize, dest: usize) {
        let current_dest = shrink_jumps_and_find(c_to_c, dest);
        let current_src = shrink_jumps_and_find(c_to_c, src);

        c_to_c[current_src] = current_dest;
    }

    let no_expressions = expressions.len();
    let mut cluster_to_current: ClusterPath = (0..no_expressions).collect();
    let mut pair_to_expression: HashMap<(usize, usize, BigInt), usize> = HashMap::new();

    for expression_id in 0..no_expressions {
        let (_, expr, _) = &expressions[expression_id];
        let constant = C::constant_coefficient();
        let terms: Vec<(usize, &BigInt)> = expr
            .iter()
            .filter(|(signal, coefficient)| {
                **signal != constant && **coefficient != BigInt::from(0)
            })
            .map(|(signal, coefficient)| (*signal, coefficient))
            .collect();

        for i in 0..terms.len() {
            for j in i + 1..terms.len() {
                let (signal_a, coeff_a) = terms[i];
                let (signal_b, coeff_b) = terms[j];
                let ratio = match modular_arithmetic::div(coeff_b, coeff_a, field) {
                    Ok(ratio) => ratio,
                    Err(_) => continue,
                };
                let pair = (signal_a, signal_b, ratio);

                if let Some(previous_expression) = pair_to_expression.get(&pair) {
                    arena_merge(&mut cluster_to_current, *previous_expression, expression_id);
                } else {
                    pair_to_expression.insert(pair, expression_id);
                }
            }
        }
    }

    let mut clusters: Vec<Vec<usize>> = Vec::new();

    for expression_id in 0..no_expressions {
        let root = shrink_jumps_and_find(
            &mut cluster_to_current,
            expression_id,
        );

        while clusters.len() <= root {
            clusters.push(Vec::new());
        }

        clusters[root].push(expression_id);
    }

    clusters.retain(|cluster| !cluster.is_empty());

    clusters
}

#[cfg(test)]
mod greedy_cluster_tests {
    use super::*;

    fn expression(id: usize, terms: &[(usize, i64)]) -> (usize, BTreeMap<usize, BigInt>, usize) {
        let coefficients = terms
            .iter()
            .map(|(signal, coefficient)| (*signal, BigInt::from(*coefficient)))
            .collect();
        (id, coefficients, 0)
    }

    #[test]
    fn clusters_expressions_with_the_same_pair_ratio() {
        let expressions = vec![
            expression(0, &[(1, 2), (2, 4)]),
            expression(1, &[(1, 3), (2, 6)]),
            expression(2, &[(1, 1), (2, 3)]),
        ];

        let clusters = build_expr_clusters(&expressions, &BigInt::from(101));

        assert!(clusters.iter().any(|cluster| cluster == &vec![0, 1]));
        assert!(clusters.iter().any(|cluster| cluster == &vec![2]));
    }
}

// fn count_variable_pairs(
//     cluster: &[usize],
//     expressions: &[(usize, HashMap<usize, BigInt>, usize)],
// ) -> Vec<((usize, usize, BigInt, BigInt), usize)> {
//     let mut frequencies: HashMap<(usize, usize, BigInt, BigInt), usize> = HashMap::new();

//     for &expression_id in cluster {
//         let (_, expression, _) = &expressions[expression_id];

//         let mut terms: Vec<(usize, BigInt)> = Vec::new();

//         for (signal, coeff) in expression {
//             if *signal != 0 && coeff != &BigInt::from(0) {
//                 terms.push((*signal, coeff.clone()));
//             }
//         }

//         terms.sort_by_key(|(signal, _)| *signal);

//         for i in 0..terms.len() {
//             for j in (i + 1)..terms.len() {
//                 let pair = (terms[i].0, terms[j].0, terms[i].1.clone(), terms[j].1.clone());

//                 *frequencies.entry(pair).or_insert(0) += 1;
//             }
//         }
//     }

//     let mut result: Vec<((usize, usize, BigInt, BigInt), usize)> =
//         frequencies.into_iter().collect();

//     result
// }

fn count_variable_pairs(
    cluster: &[usize], expressions: &[(usize, BTreeMap<usize, BigInt>, usize)], field: &BigInt) -> Vec<((usize, usize, BigInt, BigInt), usize)> {
    let mut frequencies: BTreeMap<(usize, usize, BigInt, BigInt), usize> = BTreeMap::new();

    for &expression_id in cluster {
        let (_, expression, _) = &expressions[expression_id];

        let mut terms: Vec<(usize, BigInt)> = Vec::new();

        for (signal, coeff) in expression {
            if *signal != 0 && coeff != &BigInt::from(0) {
                terms.push((*signal, coeff.clone()));
            }
        }

        terms.sort_by_key(|(signal, _)| *signal);

        for i in 0..terms.len() {
            for j in (i + 1)..terms.len() {
                let signal_a = terms[i].0;
                let signal_b = terms[j].0;

                let coeff_a = &terms[i].1;
                let coeff_b = &terms[j].1;

                // Normalize the pair as:
                // aux = signal_a + ratio * signal_b
                if let Ok(ratio) = modular_arithmetic::div(coeff_b, coeff_a, field) {
                    let pair = (signal_a, signal_b, BigInt::from(1), ratio);
    
                    *frequencies.entry(pair).or_insert(0) += 1;
                }

            }
        }
    }

    frequencies.into_iter().collect()
}

// fn select_most_frequent_pair(
//     cluster: &[usize],
//     expressions: &[(usize, HashMap<usize, BigInt>, usize)],
//     pairs: Vec<((usize, usize, BigInt, BigInt), usize)>,
// ) -> Option<((usize, usize, BigInt, BigInt), usize)> {
//     let mut valid_pairs = Vec::new();

//     for (pair, frequency) in pairs {
//         let (signal_a, signal_b, coeff_a, coeff_b) = &pair;
//         let mut can_transform = false;

//         for &expression_id in cluster {
//             let (_, expression, _) = &expressions[expression_id];

//             if expression.get(signal_a) == Some(coeff_a)
//                 && expression.get(signal_b) == Some(coeff_b)
//             {
//                 can_transform = true;
//                 break;
//             }
//         }

//         if can_transform {
//             valid_pairs.push((pair, frequency));
//         }
//     }

//     let mut best_pair = None;
//     let mut best_frec = 0;

//     for candidate in valid_pairs {
//         if best_pair.is_none()
//             || candidate.1 > best_frec
//             || (candidate.1 == best_frec && candidate.0 < *best_pair.as_ref().unwrap())
//         {
//             best_pair = Some(candidate.0);
//             best_frec = candidate.1;
//         }
//     }

//     if let Some(pair) = best_pair {
//         Some((pair, best_frec))
//     } else {
//         None
//     }
// }

fn select_most_frequent_pair(cluster: &[usize], expressions: &[(usize, BTreeMap<usize, BigInt>, usize)], pairs: Vec<((usize, usize, BigInt, BigInt), usize)>, field: &BigInt) -> Option<((usize, usize, BigInt, BigInt), usize)> {
    let mut best_pair = None;
    let mut best_frec = 0;

    for (pair, frequency) in pairs {

        // if frequency <= 1 { continue; }
        let (signal_a, signal_b, _, ratio) = &pair;

        let mut can_transform = false;

        for &expression_id in cluster {
            let (_, expression, _) = &expressions[expression_id];

            let coeff_a = expression.get(signal_a);
            let coeff_b = expression.get(signal_b);

            if let (Some(coeff_a), Some(coeff_b)) = (coeff_a, coeff_b) {
                if coeff_a != &BigInt::from(0) && coeff_b != &BigInt::from(0) {
                    if let Ok(current_ratio) = modular_arithmetic::div(coeff_b, coeff_a, field) {
                        if current_ratio == *ratio {
                            can_transform = true;
                            break;
                        }
                    }

                }
            }
        }

        if !can_transform {
            continue;
        }

        if best_pair.is_none() || frequency > best_frec || (frequency == best_frec && pair < *best_pair.as_ref().unwrap())  {
            best_pair = Some(pair);
            best_frec = frequency;
        }
    }

    if let Some(pair) = best_pair {
        Some((pair, best_frec))
    } else {
        None
    }
}

// This function creates the constraint for the new variable introduced
// fn create_aux_constraint(
//     auxiliary: usize,
//     pair: &(usize, usize, BigInt, BigInt),
//     field: &BigInt,
// ) -> C {
//     let mut coefficients = HashMap::new();

//     coefficients.insert(0, BigInt::from(0));
//     coefficients.insert(pair.0, pair.2.clone());
//     coefficients.insert(pair.1, pair.3.clone());

//     let substitution = S::new(auxiliary, A::Linear { coefficients }).unwrap();

//     let mut constraint = S::substitution_into_constraint(substitution, field);

//     constraint
// }

fn create_aux_constraint(auxiliary: usize, pair: &(usize, usize, BigInt, BigInt), field: &BigInt) -> C {
    let mut coefficients = BTreeMap::new();

    coefficients.insert(0, BigInt::from(0));
    coefficients.insert(pair.0, pair.2.clone());
    coefficients.insert(pair.1, pair.3.clone());

    let coefficients: HashMap<usize, BigInt> = coefficients.into_iter().collect();

    let substitution = S::new(auxiliary, A::Linear { coefficients }).unwrap();

    S::substitution_into_constraint(substitution, field)
}

// fn apply_pair_to_cluster(
//     storage: &mut ConstraintStorage,
//     cluster: &[usize],
//     expressions: &[(usize, HashMap<usize, BigInt>, usize)],
//     pair: (usize, usize, BigInt, BigInt),
//     aux_signals: &mut HashMap<(usize, usize, BigInt, BigInt), usize>,
//     next_aux: &mut usize,
//     non_linear_map: &mut SignalToConstraints,
//     field: &BigInt,
// ) -> usize {
//     // If the aux var exists for those variables and coefficients, it is reused
//     let aux = if let Some(existing) = aux_signals.get(&pair) {
//         *existing
//     }
//     // If not, it creates a new variable and introduces the corresponding constraint
//     else {
//         let new_aux = *next_aux;
//         *next_aux += 1;

//         let constraint = create_aux_constraint(new_aux, &pair, field);
//         let constraint_id = storage.add_constraint(constraint);

//         non_linear_map.entry(new_aux).or_insert_with(LinkedList::new).push_back(constraint_id);
//         aux_signals.insert(pair.clone(), new_aux);

//         new_aux
//     };

//     let (s_a, s_b, coeff_a, coeff_b) = pair;
//     let mut processed_constraints = HashSet::new();

//     // Replaces each appearance of the pair in the cluster constraints
//     for &expression_id in cluster {
//         let (constraint_id, _, _) = &expressions[expression_id];

//         if !processed_constraints.insert(*constraint_id) {
//             continue;
//         }

//         let mut constraint = storage.read_constraint(*constraint_id).unwrap();
//         constraint.replace_pair_with_auxiliary(s_a, &coeff_a, s_b, &coeff_b, aux);
//         constraint.normalize_constraint(field);

//         storage.replace(*constraint_id, constraint);
//     }

//     aux
// }

fn apply_pair_to_cluster(
    storage: &mut ConstraintStorage,
    cluster: &[usize],
    expressions: &[(usize, BTreeMap<usize, BigInt>, usize)],
    pair: (usize, usize, BigInt, BigInt),
    aux_signals: &mut BTreeMap<(usize, usize, BigInt, BigInt), usize>,
    next_aux: &mut usize,
    non_linear_map: &mut SignalToConstraints,
    field: &BigInt,
) -> usize {
    let aux = aux_signals.get(&pair).copied().unwrap_or(*next_aux);
    let (signal_a, signal_b, coeff_a, coeff_b) = (&pair.0, &pair.1, &pair.2, &pair.3);
    let mut processed_constraints = BTreeSet::new();
    let mut modified_constraints = Vec::new();

    for &expression_id in cluster {
        let (constraint_id, _, _) = &expressions[expression_id];

        if !processed_constraints.insert(*constraint_id) {
            continue;
        }

        let mut constraint = storage.read_constraint(*constraint_id).unwrap();
        if constraint.replace_pair_with_auxiliary(
            *signal_a,
            coeff_a,
            *signal_b,
            coeff_b,
            aux,
            field,
        ) {
            modified_constraints.push((*constraint_id, constraint));
        }
    }

    if modified_constraints.is_empty() {
        return 0;
    }

    if !aux_signals.contains_key(&pair) {
        let constraint = create_aux_constraint(aux, &pair, field);
        let constraint_id = storage.add_constraint(constraint);
        non_linear_map
            .entry(aux)
            .or_insert_with(LinkedList::new)
            .push_back(constraint_id);
        aux_signals.insert(pair, aux);
        *next_aux += 1;
    }

    let modified = modified_constraints.len();
    for (constraint_id, mut constraint) in modified_constraints {
        constraint.normalize_constraint(field);
        storage.replace(constraint_id, constraint);
    }

    modified
}

fn restore_plonk_constraints(
    storage: &mut ConstraintStorage,
    aux_signals: &BTreeMap<(usize, usize, BigInt, BigInt), usize>,
    auxiliary_constraint_ids: &BTreeMap<usize, usize>,
    non_linear_map: &mut SignalToConstraints,
    deleted: &mut HashSet<usize>,
    field: &BigInt,
) {
    let mut substitutions = Vec::with_capacity(aux_signals.len());
    let mut auxiliary_sources: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (pair, auxiliary) in aux_signals {
        let mut coefficients = BTreeMap::new();
        coefficients.insert(C::constant_coefficient(), BigInt::from(0));
        coefficients.insert(pair.0, pair.2.clone());
        coefficients.insert(pair.1, pair.3.clone());
        let coefficients: HashMap<usize, BigInt> = coefficients.into_iter().collect();
        substitutions.push((*auxiliary, S::new(*auxiliary, A::Linear { coefficients }).unwrap()));
        auxiliary_sources.insert(*auxiliary, vec![pair.0, pair.1]);
    }
    substitutions.sort_by(|(left, _), (right, _)| right.cmp(left));

    let definition_ids: BTreeSet<usize> = auxiliary_constraint_ids.values().cloned().collect();
    for constraint_id in storage.get_ids() {
        if !definition_ids.contains(&constraint_id) {
            let mut constraint = storage.read_constraint(constraint_id).unwrap();
            let mut changed = true;
            // Repeats while the constraint is plonk after substituying
            while changed {
                changed = false;
                for (auxiliary, substitution) in &substitutions {
                    if !constraint.a().contains_key(auxiliary)
                        && !constraint.b().contains_key(auxiliary)
                        && !constraint.c().contains_key(auxiliary)
                    {
                        continue;
                    }

                    let mut candidate = constraint.clone();
                    C::apply_substitution(&mut candidate, substitution, field);
                    // candidate.normalize_constraint(field);
                    if candidate.is_plonk() {
                        constraint = candidate;
                        changed = true;
                    }
                }
            }
            storage.replace(constraint_id, constraint);
        }
    }

    // for constraint_id in storage.get_ids() {
    //     if definition_ids.contains(&constraint_id) {
    //         continue;
    //     }

    //     let original = storage.read_constraint(constraint_id).unwrap();
    //     let mut candidate = original.clone();

    //     for (auxiliary, substitution) in &substitutions {
    //         if !candidate.a().contains_key(auxiliary)
    //             && !candidate.b().contains_key(auxiliary)
    //             && !candidate.c().contains_key(auxiliary)
    //         {
    //             continue;
    //         }

    //         C::apply_substitution(&mut candidate, substitution, field);
    //     }

    //     if candidate.is_plonk() {
    //         storage.replace(constraint_id, candidate);
    //     } else {
    //         storage.replace(constraint_id, original);
    //     }
    // }

    // Checks which variables are not necessary and deletes them
    let mut used_auxiliaries = BTreeSet::new();
    let mut pending = Vec::new();
    for constraint_id in storage.get_ids() {
        if definition_ids.contains(&constraint_id) {
            continue;
        }
        let constraint = storage.read_constraint(constraint_id).unwrap();
        for signal in C::take_cloned_signals(&constraint) {
            if auxiliary_sources.contains_key(&signal) {
                pending.push(signal);
            }
        }
    }

    while let Some(auxiliary) = pending.pop() {
        if !used_auxiliaries.insert(auxiliary) {
            continue;
        }
        if let Some(sources) = auxiliary_sources.get(&auxiliary) {
            for source in sources {
                if auxiliary_sources.contains_key(source) {
                    pending.push(*source);
                }
            }
        }
    }

    for (auxiliary, constraint_id) in auxiliary_constraint_ids {
        if !used_auxiliaries.contains(auxiliary) {
            storage.replace(*constraint_id, C::empty());
            non_linear_map.remove(auxiliary);
            deleted.insert(*auxiliary);
        }
    }
    let _removed = storage.extract_with(&|constraint| C::is_empty(constraint));
}

// Optimization of the number of new variables introduced, using an SMT solver
fn variables_optimization(
    storage: &mut ConstraintStorage,
    expression: &[(usize, BTreeMap<usize, BigInt>, usize)],
    cluster: &[usize],
    aux_signals: &mut BTreeMap<(usize, usize, BigInt, BigInt), usize>,
    next_aux: &mut usize,
    non_linear_map: &mut SignalToConstraints,
    field: &BigInt,
) {
    let mut constraints_ids = Vec::new();
    let mut coeffs_and_vars: Vec<(usize, BigInt)> = Vec::new();

    for &expression_id in cluster {
        let constraint_id = expression[expression_id].0;
        if !constraints_ids.contains(&constraint_id) {
            constraints_ids.push(constraint_id);
        }

        for (var, coeff) in &expression[expression_id].1 {
            if *var == 0 || coeff == &BigInt::from(0) {
                continue;
            }

            let pair = (*var, coeff.clone());
            if !coeffs_and_vars.contains(&pair) {
                coeffs_and_vars.push(pair);
            }
        }
    }

    let mut vars_per_expression: Vec<Vec<bool>> = Vec::new();

    for &expression_id in cluster {
        let expression_coeffs = &expression[expression_id].1;
        let mut v = Vec::new();

        for (var, coeff) in &coeffs_and_vars {
            let present = expression_coeffs.get(var) == Some(coeff);
            v.push(present);
        }

        vars_per_expression.push(v);
    }

    let mut bound = cluster.len(); // Maximum number of variables that are needed to transform all the expressions of the cluster into plonk
    for &expression_id in cluster {
        bound += expression[expression_id].2;
    }

    let config = Config::new();
    with_z3_config(&config, || {
        let mut solver = Solver::new();
        let zero = Int::from_i64(0);
        let one = Int::from_i64(1);

        let mut expr_use_vars = Vec::new();
        let rows_expr = cluster.len();
        // Because we are impossing that each expression at the end just has 1 variable, it cannot be an original expression because if not, it will plonk originally
        let cols_expr = bound;

        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let var = Bool::new_const(format!("expr_{}_uses_var_{}", e, c));

                expr_use_vars.push(var);
            }
        }

        let get_expr_uses = |r: usize, c: usize| expr_use_vars[r * cols_expr + c].clone();

        let mut vars_composition = Vec::new();
        let rows_vars = bound;
        let cols_vars = coeffs_and_vars.len() + bound;

        for v in 0..rows_vars {
            for c in 0..cols_vars {
                let v = Bool::new_const(format!("var_{}_uses_{}", v, c));
                vars_composition.push(v);
            }
        }

        let get_v_composition = |r: usize, c: usize| vars_composition[r * cols_vars + c].clone();

        // Each new variable can be empty or formed with exactly 2 original variables or previous declared new variables
        for v in 0..rows_vars {
            // At most 2 positions are used for each variable
            for c in 0..cols_vars {
                let c_used = get_v_composition(v, c);
                for c_next in c + 1..cols_vars {
                    let both_used = Bool::and(&[&get_v_composition(v, c_next), &c_used]);
                    for c_not in c_next + 1..cols_vars {
                        let c_not_used = get_v_composition(v, c_not);

                        // If both variables are used yet, any other variable can be used
                        solver.assert(Bool::or(&[&both_used.not(), &c_not_used.not()]));
                    }
                }
            }

            // At least 2 positions or 0 are used for each variable
            let mut is_empty = Bool::from_bool(true);
            for c in 0..cols_vars {
                is_empty = Bool::and(&[&is_empty, &&get_v_composition(v, c).not()]);
            }

            for c in 0..cols_vars {
                let current = get_v_composition(v, c);
                let mut another_used = Bool::from_bool(false);

                for other_c in 0..cols_vars {
                    if other_c != c {
                        another_used = Bool::or(&[&another_used, &get_v_composition(v, other_c)]);
                    }
                }

                solver.assert(Bool::or(&[&is_empty, &current.not(), &another_used]));
            }

            // If a variable is not empty it is because it is used by someone.
            let mut used_by_another_var = Bool::from_bool(false);
            for next_var in (v + 1)..rows_vars {
                let uses_v = get_v_composition(next_var, coeffs_and_vars.len() + v);
                used_by_another_var = Bool::or(&[&used_by_another_var, &uses_v]);
            }

            let mut used_by_final_expr = Bool::from_bool(false);
            for e in 0..rows_expr {
                let uses_v = get_expr_uses(e, v);
                used_by_final_expr = Bool::or(&[&used_by_final_expr, &uses_v]);
            }

            solver.assert(Bool::or(&[
                &is_empty,
                &Bool::or(&[&used_by_another_var, &used_by_final_expr]),
            ]));

            // Cannot exist two variables with the same composition except for the empty variables
            for next in (v + 1)..rows_vars {
                let mut different = Bool::from_bool(false);
                for c in 0..cols_vars {
                    let comp_v = get_v_composition(v, c);
                    let comp_next = get_v_composition(next, c);
                    different = Bool::or(&[&different, &comp_v._eq(&comp_next).not()]);
                }
                solver.assert(Bool::or(&[&is_empty, &different]));
            }

            // If a variable uses another variable created before, then the next variables must hold the same. First appear the variables that only use factors
            let mut uses_another_var = Bool::from_bool(false);
            for c in (coeffs_and_vars.len())..v {
                let comp_v = get_v_composition(v, c);
                uses_another_var = Bool::or(&[&uses_another_var, &comp_v]);
            }

            for next in (v + 1)..rows_vars {
                let mut next_var_uses_another_var = Bool::from_bool(false);
                let mut next_is_empty = Bool::from_bool(true);
                for c in coeffs_and_vars.len()..next {
                    // because it can only use previous vars ..cols_vars
                    let comp_next = get_v_composition(next, c);
                    next_var_uses_another_var = Bool::or(&[&next_var_uses_another_var, &comp_next]);
                    next_is_empty = Bool::and(&[&next_is_empty, &comp_next.not()]);
                }
                solver.assert(Bool::or(&[
                    &uses_another_var.not(),
                    &next_is_empty,
                    &next_var_uses_another_var,
                ]));
            }

            let empty: Vec<Bool> = (0..rows_vars)
                .map(|v| {
                    let nots: Vec<Bool> =
                        (0..cols_vars).map(|c| get_v_composition(v, c).not()).collect();
                    Bool::and(&nots.iter().collect::<Vec<_>>())
                })
                .collect();

            // Ordered by its maximum column
            for var in 0..rows_vars.saturating_sub(1) {
                for c in 0..cols_vars {
                    let mut lits: Vec<Bool> =
                        (c..cols_vars).map(|k| get_v_composition(var + 1, k)).collect();
                    lits.push(empty[var + 1].clone());
                    solver.assert(
                        &get_v_composition(var, c)
                            .implies(&Bool::or(&lits.iter().collect::<Vec<_>>())),
                    );
                }
            }

            // Two terms with the same variables cannot be used in the same variable
            for first in 0..coeffs_and_vars.len() {
                for second in (first + 1)..coeffs_and_vars.len() {
                    if coeffs_and_vars[first].0 == coeffs_and_vars[second].0 {
                        solver.assert(Bool::or(&[
                            &get_v_composition(v, first).not(),
                            &get_v_composition(v, second).not(),
                        ]));
                    }
                }
            }
        }

        // Each new variable can only use previous variables and cannot use itself to create a new variable
        for v in 0..rows_vars {
            for c in (coeffs_and_vars.len() + v)..cols_vars {
                solver.assert(&get_v_composition(v, c).not());
            }
        }

        // Empty variables must be at the end of the matrix
        for v in 0..rows_vars.saturating_sub(1) {
            let mut empty_var = Bool::from_bool(true);
            let mut empty_next = Bool::from_bool(true);

            for c in 0..cols_vars {
                let selected_v = get_v_composition(v, c);
                empty_var = Bool::and(&[&empty_var, &selected_v.not()]);

                let selected_next = get_v_composition(v + 1, c);
                empty_next = Bool::and(&[&empty_next, &selected_next.not()]);
            }

            solver.assert(Bool::or(&[&empty_var.not(), &empty_next]));
        }

        // We must check that every subexpression contained in the original expression, is also contained in the transformed one
        // Does each new variable contain or not a subexpression: just itself or another variable that is contained on it
        let mut subexpressions_var: Vec<Vec<Bool>> = Vec::new();
        for v in 0..rows_vars {
            let mut sub_vars: Vec<Bool> = Vec::new();
            for subexpr in 0..coeffs_and_vars.len() {
                let mut contains_subexpr = get_v_composition(v, subexpr);

                for prev_v in 0..v {
                    let uses_prev = get_v_composition(v, coeffs_and_vars.len() + prev_v);

                    let prev_contains = &subexpressions_var[prev_v][subexpr];
                    let uses_subexpr_through_prev = Bool::and(&[&uses_prev, prev_contains]);

                    contains_subexpr = Bool::or(&[&contains_subexpr, &uses_subexpr_through_prev]);
                }
                sub_vars.push(contains_subexpr);
            }
            subexpressions_var.push(sub_vars);
        }

        for e in 0..rows_expr {
            for subexpr in 0..coeffs_and_vars.len() {
                let mut used = Bool::from_bool(false);

                for v in 0..rows_vars {
                    used = Bool::or(&[
                        &used,
                        &Bool::and(&[&get_expr_uses(e, v), &subexpressions_var[v][subexpr]]),
                    ]);
                }

                // If the initial expression contains this subexpression, it must be used by the transformed expression directly or by a variable that it uses
                if vars_per_expression[e][subexpr] {
                    solver.assert(used);
                }
                // Otherwise it must not be used by the transformed expression or any variable that it uses
                else {
                    solver.assert(used.not());
                }
            }
        }

        // Substituted by: each expression at the end just has 1 variable. If it uses one variable, it cannot use any of the following variables
        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let used = get_expr_uses(e, c);

                for next in c + 1..cols_expr {
                    solver.assert(Bool::or(&[&used.not(), &get_expr_uses(e, next).not()]));
                }
            }
        }

        // An expression just uses non empty variables
        for e in 0..rows_expr {
            for v in 0..rows_vars {
                let mut empty_var = Bool::from_bool(true);

                for c in 0..cols_vars {
                    let selected = get_v_composition(v, c);
                    empty_var = Bool::and(&[&empty_var, &selected.not()]);
                }

                let expr_uses_aux = get_expr_uses(e, v);

                // Or it does not use it, or if it uses is because the variable is not empty
                solver.assert(Bool::or(&[&expr_uses_aux.not(), &empty_var.not()]));
            }
        }

        match solver.check() {
            SatResult::Sat => {
                let mut model = solver.get_model().expect("SAT result must have a model");
                let mut created_variables = (0..rows_vars)
                    .filter(|v| {
                        (0..cols_vars).any(|c| {
                            model
                                .eval(&vars_composition[v * cols_vars + c], true)
                                .and_then(|value| value.as_bool())
                                .unwrap_or(false)
                        })
                    })
                    .count();
                println!("\nSolution found with {} variables", created_variables);

                // Active variables form a prefix, so disabling the last one tests whether
                // the same constraints can be represented with fewer auxiliaries --> Incremental solving
                loop {
                    if created_variables == 0 {
                        break;
                    }
                    for col in 0..cols_vars {
                        solver.assert(&get_v_composition(created_variables - 1, col).not());
                    }

                    for e in 0..rows_expr {
                        solver.assert(&get_expr_uses(e, created_variables - 1).not());
                    }

                    match solver.check() {
                        SatResult::Sat => {
                            model = solver.get_model().expect("SAT result must have a model");
                            created_variables = (0..rows_vars)
                                .filter(|v| {
                                    (0..cols_vars).any(|c| {
                                        model
                                            .eval(&vars_composition[v * cols_vars + c], true)
                                            .and_then(|value| value.as_bool())
                                            .unwrap_or(false)
                                    })
                                })
                                .count();
                            println!("\nSolution found with {} variables", created_variables);
                        }
                        SatResult::Unsat => break,
                        SatResult::Unknown => {
                            println!(
                                "\nVariable minimization: UNKNOWN; keeping the last SAT solution"
                            );
                            break;
                        }
                    }
                }

                println!("\nFinal solution found with {} variables", created_variables);

                let is_selected = |variable: &Bool| {
                    model.eval(variable, true).and_then(|value| value.as_bool()).unwrap_or(false)
                };

                let mut model_aux_signals = vec![None; rows_vars];
                for v in 0..created_variables {
                    let selected_sources: Vec<usize> = (0..cols_vars)
                        .filter(|c| is_selected(&vars_composition[v * cols_vars + c]))
                        .collect();
                    if selected_sources.is_empty() {
                        continue;
                    }

                    let source = |column: usize, model_aux_signals: &[Option<usize>]| {
                        if column < coeffs_and_vars.len() {
                            (coeffs_and_vars[column].0, coeffs_and_vars[column].1.clone())
                        } else {
                            let previous_aux = column - coeffs_and_vars.len();
                            (model_aux_signals[previous_aux].unwrap(), BigInt::from(1))
                        }
                    };
                    let (signal_a, coeff_a) = source(selected_sources[0], &model_aux_signals);
                    let (signal_b, coeff_b) = source(selected_sources[1], &model_aux_signals);
                    let pair = (signal_a, signal_b, coeff_a, coeff_b);
                    let auxiliary = apply_pair_to_cluster(
                        storage,
                        cluster,
                        expression,
                        pair,
                        aux_signals,
                        next_aux,
                        non_linear_map,
                        field,
                    );
                    model_aux_signals[v] = Some(auxiliary);
                }

                if created_variables > 0 {
                    println!("New constraints:");
                    let mut introduced = 0;
                    for v in 0..created_variables {
                        let mut terms = Vec::new();
                        for c in 0..cols_vars {
                            if !is_selected(&vars_composition[v * cols_vars + c]) {
                                continue;
                            }

                            if c < coeffs_and_vars.len() {
                                let (signal, coefficient) = &coeffs_and_vars[c];
                                terms.push(format!("{} * x{}", coefficient, signal));
                            } else {
                                let previous_aux = c - coeffs_and_vars.len();
                                terms
                                    .push(format!("x{}", model_aux_signals[previous_aux].unwrap()));
                            }
                        }

                        if !terms.is_empty() {
                            let auxiliary = model_aux_signals[v].unwrap();
                            println!("  x{} = {}", auxiliary, terms.join(" + "));
                            introduced += 1;
                        }
                    }

                    println!("Introduced variables ({}):", introduced);
                    for v in 0..created_variables {
                        println!("  x{}", model_aux_signals[v].unwrap());
                    }

                    println!("Transformed expressions:");
                    for e in 0..rows_expr {
                        let expression_id = cluster[e];
                        let mut terms = Vec::new();
                        if let Some(constant) = expression[expression_id].1.get(&0) {
                            if constant != &BigInt::from(0) {
                                terms.push(constant.to_string());
                            }
                        }

                        for v in 0..created_variables {
                            if is_selected(&expr_use_vars[e * cols_expr + v]) {
                                terms.push(format!("x{}", model_aux_signals[v].unwrap()));
                            }
                        }

                        println!("  expression {}: {}", expression_id, terms.join(" + "));
                    }
                }
            }
            SatResult::Unsat => {
                println!("\nVariable optimization: UNSAT");
            }
            SatResult::Unknown => {
                println!("\nVariable optimization: UNKNOWN");
            }
        }
    })
}

fn variables_optimization_unsat_core(
    storage: &mut ConstraintStorage,
    expression: &[(usize, BTreeMap<usize, BigInt>, usize)],
    cluster: &[usize],
    aux_signals: &mut BTreeMap<(usize, usize, BigInt, BigInt), usize>,
    next_aux: &mut usize,
    non_linear_map: &mut SignalToConstraints,
    field: &BigInt,
    cluster_id: usize,
    auxiliary_constraint_ids: &mut BTreeMap<usize, usize>,
) {
    let mut constraints_ids = Vec::new();
    let mut coeffs_and_vars: Vec<(usize, BigInt)> = Vec::new();

    for &expression_id in cluster {
        let constraint_id = expression[expression_id].0;
        if !constraints_ids.contains(&constraint_id) {
            constraints_ids.push(constraint_id);
        }

        for (var, coeff) in &expression[expression_id].1 {
            if *var == 0 || coeff == &BigInt::from(0) {
                continue;
            }

            let pair = (*var, coeff.clone());
            if !coeffs_and_vars.contains(&pair) {
                coeffs_and_vars.push(pair);
            }
        }
    }

    let mut vars_per_expression: Vec<Vec<bool>> = Vec::new();

    for &expression_id in cluster {
        let expression_coeffs = &expression[expression_id].1;
        let mut v = Vec::new();

        for (var, coeff) in &coeffs_and_vars {
            let present = expression_coeffs.get(var) == Some(coeff);
            v.push(present);
        }

        vars_per_expression.push(v);
    }

    let mut bound = cluster.len(); // Maximum number of variables that are needed to transform all the expressions of the cluster into plonk
    for &expression_id in cluster {
        bound += expression[expression_id].2;
    }

    let config = Config::new();

    // Unsatisfiable core
    let mut tracked_assertions: Vec<(String, String)> = Vec::new();
    let mut assertion_id = 0usize;

    macro_rules! assert_tracked {
        ($solver:expr, $constraint:expr, $description:expr) => {{
            let tracking_name = format!("track_{}", assertion_id);
            let tracking = Bool::new_const(tracking_name.as_str());

            $solver.assert_and_track($constraint, &tracking);

            tracked_assertions.push((tracking_name, $description.to_string()));

            assertion_id += 1;
        }};
    }

    with_z3_config(&config, || {
        let mut solver = Solver::new();
        let zero = Int::from_i64(0);
        let one = Int::from_i64(1);

        let mut expr_use_vars = Vec::new();
        let rows_expr = cluster.len();
        // Because we are impossing that each expression at the end just has 1 variable, it cannot be an original expression because if not, it will plonk originally
        let cols_expr = bound;

        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let var = Bool::new_const(format!("expr_{}_uses_var_{}", e, c));

                expr_use_vars.push(var);
            }
        }

        let get_expr_uses = |r: usize, c: usize| expr_use_vars[r * cols_expr + c].clone();

        let mut vars_composition = Vec::new();
        let rows_vars = bound;
        let cols_vars = coeffs_and_vars.len() + bound;

        for v in 0..rows_vars {
            for c in 0..cols_vars {
                let v = Bool::new_const(format!("var_{}_uses_{}", v, c));
                vars_composition.push(v);
            }
        }

        let get_v_composition = |r: usize, c: usize| vars_composition[r * cols_vars + c].clone();

        // Each new variable can be empty or formed with exactly 2 original variables or previous declared new variables
        for v in 0..rows_vars {
            // At most 2 positions are used for each variable
            for c in 0..cols_vars {
                let c_used = get_v_composition(v, c);
                for c_next in c + 1..cols_vars {
                    let both_used = Bool::and(&[&get_v_composition(v, c_next), &c_used]);
                    for c_not in c_next + 1..cols_vars {
                        let c_not_used = get_v_composition(v, c_not);

                        // If both variables are used yet, any other variable can be used
                        assert_tracked!(
                            solver,
                            Bool::or(&[&both_used.not(), &c_not_used.not()]),
                            format!("Variable composition at most 2: v{} c{} c_not{}", v, c, c_not)
                        );
                    }
                }
            }

            // At least 2 positions or 0 are used for each variable because it can always be rebuild with 2 elements per variable because if not the expression would be plonk initially (1 variable)
            let mut is_empty = Bool::from_bool(true);
            for c in 0..cols_vars {
                is_empty = Bool::and(&[&is_empty, &&get_v_composition(v, c).not()]);
            }

            for c in 0..cols_vars {
                let current = get_v_composition(v, c);
                let mut another_used = Bool::from_bool(false);

                for other_c in 0..cols_vars {
                    if other_c != c {
                        another_used = Bool::or(&[&another_used, &get_v_composition(v, other_c)]);
                    }
                }

                assert_tracked!(
                    solver,
                    Bool::or(&[&is_empty, &current.not(), &another_used]),
                    format!("Variable composition at least 2: v{} c{}", v, c)
                );
            }

            // If a variable is not empty it is because it is used by someone.
            let mut used_by_another_var = Bool::from_bool(false);
            for next_var in (v + 1)..rows_vars {
                let uses_v = get_v_composition(next_var, coeffs_and_vars.len() + v);
                used_by_another_var = Bool::or(&[&used_by_another_var, &uses_v]);
            }

            let mut used_by_final_expr = Bool::from_bool(false);
            for e in 0..rows_expr {
                let uses_v = get_expr_uses(e, v);
                used_by_final_expr = Bool::or(&[&used_by_final_expr, &uses_v]);
            }

            assert_tracked!(
                solver,
                Bool::or(&[&is_empty, &Bool::or(&[&used_by_another_var, &used_by_final_expr])]),
                format!("Used by someone or empty: v{}", v)
            );

            // Cannot exist two variables with the same composition except for the empty variables
            for next in (v + 1)..rows_vars {
                let mut different = Bool::from_bool(false);
                for c in 0..cols_vars {
                    let comp_v = get_v_composition(v, c);
                    let comp_next = get_v_composition(next, c);
                    different = Bool::or(&[&different, &comp_v._eq(&comp_next).not()]);
                }
                assert_tracked!(
                    solver,
                    Bool::or(&[&is_empty, &different]),
                    format!("Two vars same composition: v{} next{}", v, next)
                );
            }

            // If a variable uses another variable created before, then the next variables must hold the same. First appear the variables that only use factors
            let mut uses_another_var = Bool::from_bool(false);
            for c in (coeffs_and_vars.len())..v {
                let comp_v = get_v_composition(v, c);
                uses_another_var = Bool::or(&[&uses_another_var, &comp_v]);
            }

            for next in (v + 1)..rows_vars {
                let mut next_var_uses_another_var = Bool::from_bool(false);
                let mut next_is_empty = Bool::from_bool(true);
                for c in coeffs_and_vars.len()..next {
                    // because it can only use previous
                    let comp_next = get_v_composition(next, c);
                    next_var_uses_another_var = Bool::or(&[&next_var_uses_another_var, &comp_next]);
                    next_is_empty = Bool::and(&[&next_is_empty, &comp_next.not()]);
                }
                assert_tracked!(
                    solver,
                    Bool::or(&[
                        &uses_another_var.not(),
                        &next_is_empty,
                        &next_var_uses_another_var
                    ]),
                    format!("First variables with only factors: v{} next{}", v, next)
                );
            }

            // Two terms with the same variables cannot be used in the same variable
            for first in 0..coeffs_and_vars.len() {
                for second in (first + 1)..coeffs_and_vars.len() {
                    if coeffs_and_vars[first].0 == coeffs_and_vars[second].0 {
                        assert_tracked!(
                            solver,
                            Bool::or(&[
                                &get_v_composition(v, first).not(),
                                &get_v_composition(v, second).not()
                            ]),
                            format!(
                                "Two terms with same variables: v{} first{} second{}",
                                v, first, second
                            )
                        );
                    }
                }
            }
        }

        let empty: Vec<Bool> = (0..rows_vars)
            .map(|v| {
                let nots: Vec<Bool> =
                    (0..cols_vars).map(|c| get_v_composition(v, c).not()).collect();
                Bool::and(&nots.iter().collect::<Vec<_>>())
            })
            .collect();

        // Ordered by its maximum column
        for var in 0..rows_vars.saturating_sub(1) {
            for c in 0..cols_vars {
                let mut lits: Vec<Bool> =
                    (c..cols_vars).map(|k| get_v_composition(var + 1, k)).collect();
                lits.push(empty[var + 1].clone());
                assert_tracked!(
                    solver,
                    &get_v_composition(var, c).implies(&Bool::or(&lits.iter().collect::<Vec<_>>())),
                    format!("Order column var{} c{}", var, c)
                );
            }
        }

        // Each new variable can only use previous variables and cannot use itself to create a new variable
        for v in 0..rows_vars {
            for c in (coeffs_and_vars.len() + v)..cols_vars {
                assert_tracked!(
                    solver,
                    &get_v_composition(v, c).not(),
                    format!("Only use previous vars: v{} c{}", v, c)
                );
            }
        }

        // Empty variables must be at the end of the matrix
        for v in 0..rows_vars.saturating_sub(1) {
            let mut empty_var = Bool::from_bool(true);
            let mut empty_next = Bool::from_bool(true);

            for c in 0..cols_vars {
                let selected_v = get_v_composition(v, c);
                empty_var = Bool::and(&[&empty_var, &selected_v.not()]);

                let selected_next = get_v_composition(v + 1, c);
                empty_next = Bool::and(&[&empty_next, &selected_next.not()]);
            }

            assert_tracked!(
                solver,
                Bool::or(&[&empty_var.not(), &empty_next]),
                format!("Empty vars at the end: v{}", v)
            );
        }

        // We must check that every subexpression contained in the original expression, is also contained in the transformed one
        // Does each new variable contain or not a subexpression: just itself or another variable that is contained on it
        let mut subexpressions_var: Vec<Vec<Bool>> = Vec::new();
        for v in 0..rows_vars {
            let mut sub_vars: Vec<Bool> = Vec::new();
            for subexpr in 0..coeffs_and_vars.len() {
                let mut contains_subexpr = get_v_composition(v, subexpr);

                for prev_v in 0..v {
                    let uses_prev = get_v_composition(v, coeffs_and_vars.len() + prev_v);

                    let prev_contains = &subexpressions_var[prev_v][subexpr];
                    let uses_subexpr_through_prev = Bool::and(&[&uses_prev, prev_contains]);

                    contains_subexpr = Bool::or(&[&contains_subexpr, &uses_subexpr_through_prev]);
                }
                sub_vars.push(contains_subexpr);
            }
            subexpressions_var.push(sub_vars);
        }

        for e in 0..rows_expr {
            for subexpr in 0..coeffs_and_vars.len() {
                let mut used = Bool::from_bool(false);

                for v in 0..rows_vars {
                    used = Bool::or(&[
                        &used,
                        &Bool::and(&[&get_expr_uses(e, v), &subexpressions_var[v][subexpr]]),
                    ]);
                }

                // If the initial expression contains this subexpression, it must be used by the transformed expression directly or by a variable that it uses
                if vars_per_expression[e][subexpr] {
                    assert_tracked!(
                        solver,
                        used,
                        format!("Subexpression contained in expression: e{} subexpr{}", e, subexpr)
                    );
                }
                // Otherwise it must not be used by the transformed expression or any variable that it uses
                else {
                    assert_tracked!(
                        solver,
                        used.not(),
                        format!(
                            "Subexpression not contained in expression: e{} subexpr{}",
                            e, subexpr
                        )
                    );
                }
            }
        }

        // Substituted by: each expression at the end just has 1 variable. If it uses one variable, it cannot use any of the following variables
        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let used = get_expr_uses(e, c);

                for next in c + 1..cols_expr {
                    assert_tracked!(
                        solver,
                        Bool::or(&[&used.not(), &get_expr_uses(e, next).not()]),
                        format!("If c used in e, next no: e{} c{} next{}", e, c, next)
                    );
                }
            }
        }

        // An expression just uses non empty variables
        for e in 0..rows_expr {
            for v in 0..rows_vars {
                let mut empty_var = Bool::from_bool(true);

                for c in 0..cols_vars {
                    let selected = get_v_composition(v, c);
                    empty_var = Bool::and(&[&empty_var, &selected.not()]);
                }

                let expr_uses_aux = get_expr_uses(e, v);

                // Or it does not use it, or if it uses is because the variable is not empty
                assert_tracked!(
                    solver,
                    Bool::or(&[&expr_uses_aux.not(), &empty_var.not()]),
                    format!("Expressions just use non empty vars: e{} v{}", e, v)
                );
            }
        }

        // Impose a saved solution for poseidon3
        // let json_path = r"C:\Users\paula\OneDrive\Documentos\Personal\MASTER METODOS FORMALES\PLI\back_circom\variables_simplification.json";
        // let json_contents = fs::read_to_string(json_path)
        //     .expect("failed to read variables_simplification.json");
        // let cluster_counts =
        //     json::parse(&json_contents).expect("failed to parse variables_simplification.json");
        // let max_vars = cluster_counts
        //     .members()
        //     .find(|entry| entry["cluster_id"].as_usize() == Some(cluster_id))
        //     .and_then(|entry| entry["variables_introduced"].as_usize())
        //     .expect("cluster variable count is missing from variables_simplification.json") + cluster.len();

        // for v in max_vars..rows_vars {
        //     for c in 0..cols_vars {
        //         assert_tracked!(solver, &get_v_composition(v, c).not(), format!("Do not use var v{} col c{}", v, c));
        //     }
        // }

        for k in 1..rows_vars + 1 {
            solver.push();

            for v in 0..k {
                let mut used = Bool::from_bool(false);
                for col in 0..cols_vars {
                    used = Bool::or(&[&used, &get_v_composition(v, col)]);
                }
                solver.assert(&used);
            }

            for v in k..rows_vars {
                // solver.assert(&row_nonempty[v].not());
                for col in 0..cols_vars {
                    solver.assert(&get_v_composition(v, col).not());
                }
            }

            match solver.check() {
                SatResult::Sat => {
                    let mut model = solver.get_model().expect("SAT result must have a model");
                    let is_selected = |variable: &Bool| {
                        model
                            .eval(variable, true)
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false)
                    };
                    let mut created_variables = (0..rows_vars)
                        .filter(|v| {
                            (0..cols_vars).any(|c| {
                                model
                                    .eval(&vars_composition[v * cols_vars + c], true)
                                    .and_then(|value| value.as_bool())
                                    .unwrap_or(false)
                            })
                        })
                        .count();

                    let mut model_aux_signals = vec![None; rows_vars];
                    for v in 0..rows_vars {
                        let selected_sources: Vec<usize> = (0..cols_vars)
                            .filter(|c| is_selected(&vars_composition[v * cols_vars + c]))
                            .collect();
                        if selected_sources.is_empty() {
                            continue;
                        }

                        let source = |column: usize, model_aux_signals: &[Option<usize>]| {
                            if column < coeffs_and_vars.len() {
                                (coeffs_and_vars[column].0, coeffs_and_vars[column].1.clone())
                            } else {
                                let previous_aux = column - coeffs_and_vars.len();
                                (model_aux_signals[previous_aux].unwrap(), BigInt::from(1))
                            }
                        };
                        let (signal_a, coeff_a) = source(selected_sources[0], &model_aux_signals);
                        let (signal_b, coeff_b) = source(selected_sources[1], &model_aux_signals);
                        let pair = (signal_a, signal_b, coeff_a, coeff_b);
                        let is_new_auxiliary = !aux_signals.contains_key(&pair);
                        let auxiliary = apply_pair_to_cluster(
                            storage,
                            cluster,
                            expression,
                            pair,
                            aux_signals,
                            next_aux,
                            non_linear_map,
                            field,
                        );
                        if is_new_auxiliary {
                            let definition_id = storage
                                .get_ids()
                                .last()
                                .copied()
                                .expect("new auxiliary must have a defining constraint");
                            auxiliary_constraint_ids.insert(auxiliary, definition_id);
                        }
                        model_aux_signals[v] = Some(auxiliary);
                    }

                    if created_variables > 0 {
                        println!("New constraints:");
                        let mut introduced = 0;
                        for v in 0..created_variables {
                            let mut terms = Vec::new();
                            for c in 0..cols_vars {
                                if !is_selected(&vars_composition[v * cols_vars + c]) {
                                    continue;
                                }

                                if c < coeffs_and_vars.len() {
                                    let (signal, coefficient) = &coeffs_and_vars[c];
                                    terms.push(format!("{} * x{}", coefficient, signal));
                                } else {
                                    let previous_aux = c - coeffs_and_vars.len();
                                    terms.push(format!(
                                        "x{}",
                                        model_aux_signals[previous_aux].unwrap()
                                    ));
                                }
                            }

                            if !terms.is_empty() {
                                let auxiliary = model_aux_signals[v].unwrap();
                                println!("  x{} = {}", auxiliary, terms.join(" + "));
                                introduced += 1;
                            }
                        }

                        println!("Introduced variables ({}):", introduced);
                        for v in 0..created_variables {
                            println!("  x{}", model_aux_signals[v].unwrap());
                        }

                        println!("Transformed expressions:");
                        for e in 0..rows_expr {
                            let expression_id = cluster[e];
                            let mut terms = Vec::new();
                            if let Some(constant) = expression[expression_id].1.get(&0) {
                                if constant != &BigInt::from(0) {
                                    terms.push(constant.to_string());
                                }
                            }

                            for v in 0..rows_vars {
                                if is_selected(&expr_use_vars[e * cols_expr + v]) {
                                    terms.push(format!("x{}", model_aux_signals[v].unwrap()));
                                }
                            }

                            println!("  expression {}: {}", expression_id, terms.join(" + "));
                        }
                    }

                    break;
                }
                SatResult::Unsat => {
                    println!("\nVariable optimization: UNSAT with k = {}", k);
                    // println!("\nVariable optimization: UNSAT");

                    // let unsat_core = solver.get_unsat_core();

                    // println!(
                    //     "\nUnsatisfiable Core ({} constraints):",
                    //     unsat_core.len()
                    // );

                    // for core_constraint in &unsat_core {
                    //     let core_name = core_constraint.to_string();

                    //     for (tracking_name, description) in &tracked_assertions {
                    //         if tracking_name == &core_name {
                    //             println!("  - {}", description);
                    //             break;
                    //         }
                    //     }
                    // }

                    solver.pop(1);
                    continue;
                }
                SatResult::Unknown => {
                    println!("\nVariable optimization: UNKNOWN with k = {}", k);
                    // println!("\nVariable optimization: UNKNOWN");
                    solver.pop(1);
                    break;
                }
            }
        }
    })
}

fn variables_optimization_bitwuzla(storage: &mut ConstraintStorage, expression: &[(usize, BTreeMap<usize, BigInt>, usize)], cluster: &[usize], aux_signals: &mut BTreeMap<(usize, usize, BigInt, BigInt), usize>,
next_aux: &mut usize, non_linear_map: &mut SignalToConstraints, field: &BigInt, cluster_id: usize, auxiliary_constraint_ids: &mut BTreeMap<usize, usize>) {

    let mut coeffs_and_vars: Vec<(usize, BigInt)> = Vec::new();

    for &expression_id in cluster {
        for (var, coeff) in &expression[expression_id].1 {
            if *var == 0 || coeff == &BigInt::from(0) {
                continue;
            }

            let pair = (*var, coeff.clone());
            if !coeffs_and_vars.contains(&pair) {
                coeffs_and_vars.push(pair);
            }
        }
    }

    let mut vars_per_expression: Vec<Vec<bool>> = Vec::new();

    for &expression_id in cluster {
        let expression_coeffs = &expression[expression_id].1;
        let mut v = Vec::new();

        for (var, coeff) in &coeffs_and_vars {
            v.push(expression_coeffs.get(var) == Some(coeff));
        }

        vars_per_expression.push(v);
    }

    let mut bound = cluster.len();

    for &expression_id in cluster {
        bound += expression[expression_id].2;
    }

    unsafe {
        let tm = bitwuzla_term_manager_new();
        let options = bitwuzla_options_new();
        bitwuzla_set_option(options, BITWUZLA_OPT_PRODUCE_MODELS, 1);

        let solver = bitwuzla_new(tm, options);
        bitwuzla_options_delete(options);

        let bool_sort = bitwuzla_mk_bool_sort(tm);
        let true_term = bitwuzla_mk_true(tm);
        let false_term = bitwuzla_mk_false(tm);

        let mk_nary = |kind: BitwuzlaKind, mut args: Vec<BitwuzlaTerm>| -> BitwuzlaTerm {
            if args.is_empty() {
                return if kind == BITWUZLA_KIND_AND {
                    true_term
                }
                else {
                    false_term
                };
            }
            if args.len() == 1 && kind != BITWUZLA_KIND_NOT {
                return args[0];
            }
            bitwuzla_mk_term(tm, kind, args.len() as u32, args.as_mut_ptr())
        };

        let mk_not = |term: BitwuzlaTerm| -> BitwuzlaTerm {
            mk_nary(BITWUZLA_KIND_NOT, vec![term])
        };

        let mk_and = |terms: Vec<BitwuzlaTerm>| -> BitwuzlaTerm {
            mk_nary(BITWUZLA_KIND_AND, terms)
        };

        let mk_or = |terms: Vec<BitwuzlaTerm>| -> BitwuzlaTerm {
            mk_nary(BITWUZLA_KIND_OR, terms)
        };

        let mk_implies = |a: BitwuzlaTerm, b: BitwuzlaTerm| -> BitwuzlaTerm {
            mk_nary(BITWUZLA_KIND_IMPLIES, vec![a, b])
        };

        let mk_equal = |a: BitwuzlaTerm, b: BitwuzlaTerm| -> BitwuzlaTerm {
            mk_nary(BITWUZLA_KIND_EQUAL, vec![a, b])
        };

        let mut assertion_id = 0usize;

        let mut new_bool = |name: String| -> BitwuzlaTerm {
            let symbol = CString::new(name).unwrap();
            let term = bitwuzla_mk_const(tm, bool_sort, symbol.as_ptr());
            assertion_id += 1;
            term
        };

        let assert_formula = |formula: BitwuzlaTerm| {
            bitwuzla_assert(solver, formula);
        };

        let is_selected = |variable: BitwuzlaTerm| -> bool {
            let value = bitwuzla_get_value(solver, variable);
            bitwuzla_term_is_true(value)
        };

        let rows_expr = cluster.len();
        let cols_expr = bound;

        let mut expr_use_vars: Vec<BitwuzlaTerm> = Vec::new();
        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let var = new_bool(format!("expr_{}_uses_var{}", e, c));

                expr_use_vars.push(var);
            }
        }

        let get_expr_uses = |r: usize, c: usize| {
            expr_use_vars[r * cols_expr + c]
        };

        let rows_vars = bound;
        let cols_vars = coeffs_and_vars.len() + bound;

        let mut vars_composition: Vec<BitwuzlaTerm> = Vec::new();

        for v in 0..rows_vars {
            for c in 0..cols_vars {
                let var = new_bool(format!("var_{}_uses_{}", v, c));
                vars_composition.push(var);
            }
        }

        let get_v_composition = |r: usize, c: usize| {
            vars_composition[r * cols_vars + c]
        };

        // Each new variable can be empty or formed with exactly two original variables or previous declared variables.
        for v in 0..rows_vars {
            // At most 2 positions are used for each variable
            for c in 0..cols_vars {
                let c_used = get_v_composition(v, c);
                for c_next in c+1..cols_vars {
                    let both_used = mk_and(vec![get_v_composition(v, c_next), c_used]);
                    for c_not in c_next + 1..cols_vars {
                        let c_not_used = get_v_composition(v, c_not);

                        assert_formula(mk_or(vec![mk_not(both_used), mk_not(c_not_used)]));
                    }
                }
            }

            // At least 2 positions or 0 are used
            let mut is_empty = true_term;

            for c in 0..cols_vars {
                is_empty = mk_and(vec![is_empty, mk_not(get_v_composition(v, c))]);
            }

            for c in 0..cols_vars {
                let current = get_v_composition(v, c);
                let mut another_used = false_term;
                for other_c in 0..cols_vars {
                    if other_c != c {
                        another_used = mk_or(vec![another_used, get_v_composition(v, other_c)]);
                    }
                }

                assert_formula(mk_or(vec![is_empty, mk_not(current), another_used]));
            }

            // If a variable is not empty, it must be used by another variable or a final expression.
            let mut used_by_another_var = false_term;

            for next_var in v + 1..rows_vars {
                let uses_v = get_v_composition(next_var, coeffs_and_vars.len() + v);
                used_by_another_var = mk_or(vec![used_by_another_var, uses_v]);
            }

            let mut used_by_final_expr = false_term;

            for e in 0..rows_expr {
                used_by_final_expr = mk_or(vec![used_by_final_expr, get_expr_uses(e, v)]);
            }

            assert_formula(mk_or(vec![is_empty, used_by_another_var, used_by_final_expr]));

            for next in v + 1..rows_vars {
                let mut different = false_term;

                for c in 0..cols_vars {
                    let comp_v = get_v_composition(v, c);
                    let comp_next = get_v_composition(next, c);

                    different = mk_or(vec![different, mk_not(mk_equal(comp_v, comp_next))]);
                }

                assert_formula(mk_or(vec![is_empty, different]));
            }

            let mut uses_another_var = false_term;

            for c in coeffs_and_vars.len()..v {
                uses_another_var = mk_or(vec![uses_another_var, get_v_composition(v, c)]);
            }

            for next in v + 1..rows_vars {
                let mut next_var_uses_another_var = false_term;
                let mut next_is_empty = true_term;

                for c in coeffs_and_vars.len()..next {
                    let comp_next = get_v_composition(next, c);

                    next_var_uses_another_var = mk_or(vec![next_var_uses_another_var, comp_next]);
                    next_is_empty = mk_and(vec![next_is_empty, mk_not(comp_next)]);
                }

                assert_formula(mk_or(vec![mk_not(uses_another_var), next_is_empty, next_var_uses_another_var]));
            }

            // Two terms with the same variables cannot be used in the same auxiliary variable.
            for first in 0..coeffs_and_vars.len() {
                for second in first + 1..coeffs_and_vars.len() {
                    if coeffs_and_vars[first].0 == coeffs_and_vars[second].0 {
                        assert_formula(mk_or(vec![mk_not(get_v_composition(v, first)), mk_not(get_v_composition(v, second))]));
                    }
                }
            }
        }

        // Empty variables
        let mut empty: Vec<BitwuzlaTerm> = Vec::new();

        for v in 0..rows_vars {
            let mut nots = Vec::new();

            for c in 0..cols_vars {
                nots.push(mk_not(get_v_composition(v, c)));
            }

            empty.push(mk_and(nots));
        }

        // Ordered by its maximum column
        for var in 0..rows_vars.saturating_sub(1) {
            for c in 0..cols_vars {
                let mut lits = Vec::new();
                for k in c..cols_vars {
                    lits.push(get_v_composition(var + 1, k));
                }

                lits.push(empty[var + 1]);

                assert_formula(mk_implies(get_v_composition(var, c), mk_or(lits)));
            }
        }

        // Each new variable can only use previous variables
        for v in 0..rows_vars {
            for c in coeffs_and_vars.len() + v..cols_vars {
                assert_formula(mk_not(get_v_composition(v, c)));
            }
        }

        // Empty variables must be at the end
        for v in 0..rows_vars.saturating_sub(1) {
            let mut empty_var = true_term;
            let mut empty_next = true_term;

            for c in 0..cols_vars {
                let selected_v = get_v_composition(v, c);
                empty_var = mk_and(vec![empty_var, mk_not(selected_v)]);

                let selected_next = get_v_composition(v + 1, c);
                empty_next = mk_and(vec![empty_next, mk_not(selected_next)]);
            }

            assert_formula(mk_or(vec![mk_not(empty_var), empty_next]));
        }

        let mut subexpressions_var: Vec<Vec<BitwuzlaTerm>> = Vec::new();

        for v in 0..rows_vars {
            let mut sub_vars = Vec::new();
            for subexpr in 0..coeffs_and_vars.len() {
                let mut contains_subexpr = get_v_composition(v, subexpr);

                for prev_v in 0..v {
                    let uses_prev = get_v_composition(v, coeffs_and_vars.len() + prev_v);
                    let prev_contains = subexpressions_var[prev_v][subexpr];
                    let uses_subexpr_through_prev = mk_and(vec![uses_prev, prev_contains]);
                    contains_subexpr = mk_or(vec![contains_subexpr, uses_subexpr_through_prev]);
                }

                sub_vars.push(contains_subexpr);
            }

            subexpressions_var.push(sub_vars);
        }

        for e in 0..rows_expr {
            for subexpr in 0..coeffs_and_vars.len() {
                let mut used = false_term;

                for v in 0..rows_vars {
                    used = mk_or(vec![used, mk_and(vec![get_expr_uses(e, v), subexpressions_var[v][subexpr]])]);
                }

                if vars_per_expression[e][subexpr] {
                    assert_formula(used);
                }
                else {
                    assert_formula(mk_not(used));
                }
            }
        }

        // Each final expression uses at most one auxiliary variable
        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let used = get_expr_uses(e, c);

                for next in c + 1..cols_expr {
                    assert_formula(mk_or(vec![mk_not(used), mk_not(get_expr_uses(e, next))]));
                }
            }
        }

        // An expression only uses non-empty variables
        for e in 0..rows_expr {
            for v in 0..rows_vars {
                let mut empty_var = true_term;

                for c in 0..cols_vars {
                    empty_var = mk_and(vec![empty_var, mk_not(get_v_composition(v, c))]);
                }

                let expr_uses_aux = get_expr_uses(e, v);
                assert_formula(mk_or(vec![mk_not(expr_uses_aux), mk_not(empty_var)]));
            }
        }

        // Incremental minimization of auxiliary variables
        for k in 1..rows_vars + 1 {
            bitwuzla_push(solver, 1);

            for v in 0..k {
                let mut used = false_term;

                for col in 0..cols_vars {
                    used = mk_or(vec![used, get_v_composition(v, col)]);
                }

                bitwuzla_assert(solver, used);
            }

            for v in k..rows_vars {
                for col in 0..cols_vars {
                    bitwuzla_assert(solver, mk_not(get_v_composition(v, col)));
                }
            }

            match bitwuzla_check_sat(solver) {
                BITWUZLA_SAT => {

                    println!("[Bitwuzla] Cluster {} -> SAT with k = {}", cluster_id, k);
                    let mut created_variables = 0usize;

                    for v in 0..rows_vars {
                        let mut used = false;

                        for c in 0..cols_vars {
                            if is_selected(vars_composition[v * cols_vars + c]) {
                                used = true;
                                break;
                            }
                        }

                        if used {
                            created_variables += 1;
                        }
                    }

                    let mut model_aux_signals = vec![None; rows_vars];

                    for v in 0..rows_vars {
                        let mut selected_sources = Vec::new();

                        for c in 0..cols_vars {
                            if is_selected(vars_composition[v * cols_vars + c]) {
                                selected_sources.push(c);
                            }
                        }

                        if selected_sources.is_empty() {continue;}

                        let source = |column: usize, model_aux_signals: &[Option<usize>]| -> (usize, BigInt) {
                            if column < coeffs_and_vars.len() {
                                (
                                    coeffs_and_vars[column].0,
                                    coeffs_and_vars[column].1.clone(),
                                )
                            } else {
                                let previous_aux = column - coeffs_and_vars.len();
                                (
                                    model_aux_signals[previous_aux]
                                        .unwrap(),
                                    BigInt::from(1),
                                )
                            }
                        };

                        let (signal_a, coeff_a) = source(
                            selected_sources[0],
                            &model_aux_signals,
                        );

                        let (signal_b, coeff_b) = source(
                            selected_sources[1],
                            &model_aux_signals,
                        );

                        let pair = (
                            signal_a,
                            signal_b,
                            coeff_a,
                            coeff_b,
                        );

                        let is_new_auxiliary =
                            !aux_signals.contains_key(&pair);

                        let auxiliary = apply_pair_to_cluster(
                            storage,
                            cluster,
                            expression,
                            pair,
                            aux_signals,
                            next_aux,
                            non_linear_map,
                            field,
                        );

                        if is_new_auxiliary {
                            let definition_id = storage
                                .get_ids()
                                .last()
                                .copied()
                                .expect(
                                    "new auxiliary must have a defining constraint",
                                );

                            auxiliary_constraint_ids.insert(
                                auxiliary,
                                definition_id,
                            );
                        }

                        model_aux_signals[v] = Some(auxiliary);
                    }

                    if created_variables > 0 {
                        // println!("New constraints:");

                        let mut introduced = 0;

                        for v in 0..created_variables {
                            let mut terms = Vec::new();

                            for c in 0..cols_vars {
                                if !is_selected(
                                    vars_composition[v * cols_vars + c],
                                ) {
                                    continue;
                                }

                                if c < coeffs_and_vars.len() {
                                    let (signal, coefficient) =
                                        &coeffs_and_vars[c];

                                    terms.push(format!(
                                        "{} * x{}",
                                        coefficient,
                                        signal
                                    ));
                                } else {
                                    let previous_aux =
                                        c - coeffs_and_vars.len();

                                    terms.push(format!(
                                        "x{}",
                                        model_aux_signals[previous_aux]
                                            .unwrap()
                                    ));
                                }
                            }

                            if !terms.is_empty() {
                                let auxiliary =
                                    model_aux_signals[v].unwrap();

                                // println!(
                                //     "  x{} = {}",
                                //     auxiliary,
                                //     terms.join(" + ")
                                // );

                                introduced += 1;
                            }
                        }

                        // println!(
                        //     "Introduced variables ({}):",
                        //     introduced
                        // );

                        for v in 0..created_variables {
                            // println!(
                            //     "  x{}",
                            //     model_aux_signals[v].unwrap()
                            // );
                        }

                        // println!("Transformed expressions:");

                        for e in 0..rows_expr {
                            let expression_id = cluster[e];
                            let mut terms = Vec::new();

                            if let Some(constant) =
                                expression[expression_id].1.get(&0)
                            {
                                if constant != &BigInt::from(0) {
                                    terms.push(constant.to_string());
                                }
                            }

                            for v in 0..rows_vars {
                                if is_selected(get_expr_uses(e, v)) {
                                    terms.push(format!(
                                        "x{}",
                                        model_aux_signals[v].unwrap()
                                    ));
                                }
                            }

                            // println!(
                            //     "  expression {}: {}",
                            //     expression_id,
                            //     terms.join(" + ")
                            // );
                        }
                    }

                    break;
                }

                BITWUZLA_UNSAT => {
                    println!(
                        "\nVariable optimization: UNSAT with k = {}",
                        k
                    );

                    bitwuzla_pop(solver, 1);
                    continue;
                }

                BITWUZLA_UNKNOWN => {
                    println!(
                        "\nVariable optimization: UNKNOWN with k = {}",
                        k
                    );

                    bitwuzla_pop(solver, 1);
                    break;
                }

                _ => {
                    bitwuzla_pop(solver, 1);
                    break;
                }
            }
        }

        // Release Bitwuzla resources
        bitwuzla_delete(solver);
        bitwuzla_term_manager_delete(tm);
    }
}

fn variables_optimization_cadical(
    storage: &mut ConstraintStorage,
    expression: &[(usize, BTreeMap<usize, BigInt>, usize)],
    cluster: &[usize],
    aux_signals: &mut BTreeMap<(usize, usize, BigInt, BigInt), usize>,
    next_aux: &mut usize,
    non_linear_map: &mut SignalToConstraints,
    field: &BigInt,
    cluster_id: usize,
    auxiliary_constraint_ids: &mut BTreeMap<usize, usize>,
) {
    let mut constraints_ids = Vec::new();
    let mut coeffs_and_vars: Vec<(usize, BigInt)> = Vec::new();

    for &expression_id in cluster {
        let constraint_id = expression[expression_id].0;

        if !constraints_ids.contains(&constraint_id) {
            constraints_ids.push(constraint_id);
        }

        for (var, coeff) in &expression[expression_id].1 {
            if *var == 0 || coeff == &BigInt::from(0) {
                continue;
            }

            let pair = (*var, coeff.clone());

            if !coeffs_and_vars.contains(&pair) {
                coeffs_and_vars.push(pair);
            }
        }
    }

    let mut vars_per_expression: Vec<Vec<bool>> = Vec::new();

    for &expression_id in cluster {
        let expression_coeffs = &expression[expression_id].1;
        let mut v = Vec::new();

        for (var, coeff) in &coeffs_and_vars {
            let present = expression_coeffs.get(var) == Some(coeff);
            v.push(present);
        }

        vars_per_expression.push(v);
    }

    let mut bound = cluster.len();

    for &expression_id in cluster {
        bound += expression[expression_id].2;
    }

    let mut solver: CadicalSolver<Timeout> = CadicalSolver::default();

    // SAT variable 1 == TRUE and -1 == FALSE
    solver.add_clause([1]);
    let mut next_sat_var: i32 = 2;

    // Create a new SAT var
    macro_rules! new_sat_var {
        () => {{
            let var = next_sat_var;
            next_sat_var += 1;
            var
        }};
    }

    // Create a SAT var equivalent to the conjunction
    macro_rules! sat_and {
        ($lits:expr) => {{
            let lits: Vec<i32> = $lits;
            if lits.is_empty() {
                1
            } else if lits.len() == 1 {
                lits[0]
            } else {
                let result = new_sat_var!();
                for lit in &lits {
                    solver.add_clause([-result, *lit]);
                }
                let mut clause = vec![result];

                for lit in &lits {
                    clause.push(-*lit);
                }
                solver.add_clause(clause);
                result
            }
        }};
    }

    macro_rules! sat_or {
        ($lits:expr) => {{
            let lits: Vec<i32> = $lits;

            if lits.is_empty() {
                -1
            } else if lits.len() == 1 {
                lits[0]
            } else {
                let result = new_sat_var!();

                for lit in &lits {
                    solver.add_clause([-*lit, result]);
                }

                let mut clause = vec![-result];

                for lit in &lits {
                    clause.push(*lit);
                }

                solver.add_clause(clause);

                result
            }
        }};
    }

    macro_rules! sat_xor {
        ($a:expr, $b:expr) => {{
            let a = $a;
            let b = $b;

            let result = new_sat_var!();

            solver.add_clause([a, b, -result]);
            solver.add_clause([-a, -b, -result]);
            solver.add_clause([a, -b, result]);
            solver.add_clause([-a, b, result]);

            result
        }};
    }

    let rows_expr = cluster.len();
    let cols_expr = bound;
    let mut expr_use_vars = Vec::new();

    for e in 0..rows_expr {
        for c in 0..cols_expr {
            let var = new_sat_var!();
            expr_use_vars.push(var);
        }
    }

    let get_expr_uses = |r: usize, c: usize| expr_use_vars[r * cols_expr + c];

    let rows_vars = bound;
    let cols_vars = coeffs_and_vars.len() + bound;

    let mut vars_composition = Vec::new();

    for v in 0..rows_vars {
        for c in 0..cols_vars {
            let var = new_sat_var!();
            vars_composition.push(var);
        }
    }

    let get_v_composition = |r: usize, c: usize| vars_composition[r * cols_vars + c];

    // non_empty
    let mut row_nonempty = Vec::new();

    for v in 0..rows_vars {
        let mut lits = Vec::new();

        for c in 0..cols_vars {
            lits.push(get_v_composition(v, c));
        }

        row_nonempty.push(sat_or!(lits));
    }

    // Empty or formed with exactly 2 variables
    for v in 0..rows_vars {
        // At most 2 variables
        for c in 0..cols_vars {
            let c_used = get_v_composition(v, c);

            for c_next in c + 1..cols_vars {
                let c_next_used = get_v_composition(v, c_next);
                for c_not in c_next + 1..cols_vars {
                    let c_not_used = get_v_composition(v, c_not);
                    solver.add_clause([-c_used, -c_next_used, -c_not_used]);
                }
            }
        }

        // At least 2
        for c in 0..cols_vars {
            let current = get_v_composition(v, c);
            let mut another_used = Vec::new();

            for other_c in 0..cols_vars {
                if other_c != c {
                    another_used.push(get_v_composition(v, other_c));
                }
            }

            let another_used = sat_or!(another_used);
            solver.add_clause([-row_nonempty[v], -current, another_used]);
        }

        // If non empty, it is used by someone
        let mut used_by_another_var = Vec::new();

        for next_var in (v + 1)..rows_vars {
            let uses_v = get_v_composition(next_var, coeffs_and_vars.len() + v);

            used_by_another_var.push(uses_v);
        }

        let mut used_by_final_expr = Vec::new();

        for e in 0..rows_expr {
            let uses_v = get_expr_uses(e, v);
            used_by_final_expr.push(uses_v);
        }

        let mut used_by_someone = used_by_another_var;
        used_by_someone.extend(used_by_final_expr);

        let used_by_someone = sat_or!(used_by_someone);

        solver.add_clause([-row_nonempty[v], used_by_someone]);

        // Cannot exist 2 variables with the same composition
        for next in (v + 1)..rows_vars {
            let mut different = Vec::new();

            for c in 0..cols_vars {
                let comp_v = get_v_composition(v, c);
                let comp_next = get_v_composition(next, c);

                different.push(sat_xor!(comp_v, comp_next));
            }

            let different = sat_or!(different);
            solver.add_clause([-row_nonempty[v], different]);
        }

        // First appear the variables that only use factors.
        let mut uses_another_var = Vec::new();

        for c in coeffs_and_vars.len()..v {
            let comp_v = get_v_composition(v, c);
            uses_another_var.push(comp_v);
        }

        let uses_another_var = sat_or!(uses_another_var);

        for next in (v + 1)..rows_vars {
            let mut next_var_uses_another_var = Vec::new();
            let mut next_is_empty = Vec::new();

            for c in coeffs_and_vars.len()..next {
                let comp_next = get_v_composition(next, c);
                next_var_uses_another_var.push(comp_next);
                next_is_empty.push(-comp_next);
            }

            let next_var_uses_another_var = sat_or!(next_var_uses_another_var);
            let next_is_empty = sat_and!(next_is_empty);

            solver.add_clause([-uses_another_var, next_is_empty, next_var_uses_another_var]);
        }

        // Two terms with the same variables cannot be used in the same variable.
        for first in 0..coeffs_and_vars.len() {
            for second in (first + 1)..coeffs_and_vars.len() {
                if coeffs_and_vars[first].0 == coeffs_and_vars[second].0 {
                    solver
                        .add_clause([-get_v_composition(v, first), -get_v_composition(v, second)]);
                }
            }
        }
    }

    // Empty vars
    let mut empty = Vec::new();

    for v in 0..rows_vars {
        empty.push(-row_nonempty[v]);
    }

    // Ordered by maximum column
    for var in 0..rows_vars.saturating_sub(1) {
        for c in 0..cols_vars {
            let mut clause = Vec::new();
            clause.push(-get_v_composition(var, c));

            for k in c..cols_vars {
                clause.push(get_v_composition(var + 1, k));
            }

            clause.push(empty[var + 1]);
            solver.add_clause(clause);
        }
    }

    // Each new variable can only use previous variables and cannot use itself to create a new variable.
    for v in 0..rows_vars {
        for c in (coeffs_and_vars.len() + v)..cols_vars {
            solver.add_clause([-get_v_composition(v, c)]);
        }
    }

    // Empty vars at the end of the matrix
    for v in 0..rows_vars.saturating_sub(1) {
        solver.add_clause([row_nonempty[v], -row_nonempty[v + 1]]);
    }

    let mut subexpressions_var: Vec<Vec<i32>> = Vec::new();

    for v in 0..rows_vars {
        let mut sub_vars: Vec<i32> = Vec::new();

        for subexpr in 0..coeffs_and_vars.len() {
            let mut contains_terms = Vec::new();

            contains_terms.push(get_v_composition(v, subexpr));

            for prev_v in 0..v {
                let uses_prev = get_v_composition(v, coeffs_and_vars.len() + prev_v);
                let prev_contains = subexpressions_var[prev_v][subexpr];
                let uses_subexpr_through_prev = sat_and!(vec![uses_prev, prev_contains]);

                contains_terms.push(uses_subexpr_through_prev);
            }

            let contains_subexpr = sat_or!(contains_terms);

            sub_vars.push(contains_subexpr);
        }

        subexpressions_var.push(sub_vars);
    }

    for e in 0..rows_expr {
        for subexpr in 0..coeffs_and_vars.len() {
            let mut used_terms = Vec::new();

            for v in 0..rows_vars {
                let used_by_expr = get_expr_uses(e, v);
                let contains_subexpr = subexpressions_var[v][subexpr];

                let used = sat_and!(vec![used_by_expr, contains_subexpr]);

                used_terms.push(used);
            }

            let used = sat_or!(used_terms);

            if vars_per_expression[e][subexpr] {
                solver.add_clause([used]);
            } else {
                solver.add_clause([-used]);
            }
        }
    }

    // Each expression at the end just has 1 variable
    for e in 0..rows_expr {
        for c in 0..cols_expr {
            let used = get_expr_uses(e, c);

            for next in c + 1..cols_expr {
                solver.add_clause([-used, -get_expr_uses(e, next)]);
            }
        }
    }

    // An expression just uses non-empty variables.
    for e in 0..rows_expr {
        for v in 0..rows_vars {
            let expr_uses_aux = get_expr_uses(e, v);
            solver.add_clause([-expr_uses_aux, row_nonempty[v]]);
        }
    }

    let _ = cluster_id;
    let _ = constraints_ids;

    // Incremental minimization of auxiliary variables.
    for k in 1..rows_vars + 1 {
        let mut assumptions = Vec::new();

        for v in 0..k {
            assumptions.push(row_nonempty[v]);
        }

        for v in k..rows_vars {
            assumptions.push(-row_nonempty[v]);
        }

        match solver.solve_with(assumptions) {
            Some(true) => {
                println!("[Cadical] Cluster {} -> SAT with k = {}", cluster_id, k);

                let is_selected = |variable: i32| solver.value(variable).unwrap_or(false);

                let created_variables = (0..rows_vars)
                    .filter(|v| {
                        (0..cols_vars).any(|c| is_selected(vars_composition[*v * cols_vars + c]))
                    })
                    .count();

                let mut model_aux_signals = vec![None; rows_vars];

                for v in 0..rows_vars {
                    let selected_sources: Vec<usize> = (0..cols_vars)
                        .filter(|c| is_selected(vars_composition[v * cols_vars + *c]))
                        .collect();

                    if selected_sources.is_empty() {
                        continue;
                    }

                    assert_eq!(
                        selected_sources.len(),
                        2,
                        "An auxiliary variable must use exactly two sources"
                    );

                    let source = |column: usize, model_aux_signals: &[Option<usize>]| {
                        if column < coeffs_and_vars.len() {
                            (coeffs_and_vars[column].0, coeffs_and_vars[column].1.clone())
                        } else {
                            let previous_aux = column - coeffs_and_vars.len();

                            (model_aux_signals[previous_aux].unwrap(), BigInt::from(1))
                        }
                    };

                    let (signal_a, coeff_a) = source(selected_sources[0], &model_aux_signals);
                    let (signal_b, coeff_b) = source(selected_sources[1], &model_aux_signals);
                    let pair = (signal_a, signal_b, coeff_a, coeff_b);

                    let is_new_auxiliary = !aux_signals.contains_key(&pair);

                    let auxiliary = apply_pair_to_cluster(
                        storage,
                        cluster,
                        expression,
                        pair,
                        aux_signals,
                        next_aux,
                        non_linear_map,
                        field,
                    );

                    if is_new_auxiliary {
                        let definition_id = storage
                            .get_ids()
                            .last()
                            .copied()
                            .expect("new auxiliary must have a defining constraint");

                        auxiliary_constraint_ids.insert(auxiliary, definition_id);
                    }

                    model_aux_signals[v] = Some(auxiliary);
                }

                if created_variables > 0 {
                    println!("New constraints:");

                    let mut introduced = 0;

                    for v in 0..created_variables {
                        let mut terms = Vec::new();

                        for c in 0..cols_vars {
                            if !is_selected(vars_composition[v * cols_vars + c]) {
                                continue;
                            }

                            if c < coeffs_and_vars.len() {
                                let (signal, coefficient) = &coeffs_and_vars[c];
                                terms.push(format!("{} * x{}", coefficient, signal));
                            } else {
                                let previous_aux = c - coeffs_and_vars.len();
                                terms.push(format!("x{}", model_aux_signals[previous_aux].unwrap()));
                            }
                        }

                        if !terms.is_empty() {
                            let auxiliary = model_aux_signals[v].unwrap();
                            println!("  x{} = {}", auxiliary, terms.join(" + "));
                            introduced += 1;
                        }
                    }

                    println!("Introduced variables ({}):", introduced);

                    for v in 0..created_variables {
                        println!("  x{}", model_aux_signals[v].unwrap());
                    }

                    println!("Transformed expressions:");

                    for e in 0..rows_expr {
                        let expression_id = cluster[e];
                        let mut terms = Vec::new();

                        if let Some(constant) = expression[expression_id].1.get(&0) {
                            if constant != &BigInt::from(0) {
                                terms.push(constant.to_string());
                            }
                        }

                        for v in 0..rows_vars {
                            if is_selected(expr_use_vars[e * cols_expr + v]) {
                                terms.push(format!("x{}", model_aux_signals[v].unwrap()));
                            }
                        }

                        println!("  expression {}: {}", expression_id, terms.join(" + "));
                    }
                }

                break;
            }

            Some(false) => {
                println!("Variable optimization: UNSAT with k = {}", k);
            }

            None => {
                println!("Variable optimization: UNKNOWN with k = {}", k);
                break;
            }
        }
    }
}

fn export_expr_clusters_json(
    path: &str,
    expressions: &[(usize, BTreeMap<usize, BigInt>, usize)],
    clusters: &[Vec<usize>],
) -> std::io::Result<()> {
    let constant = C::constant_coefficient();

    let mut all_variables = BTreeSet::new();
    let mut all_constraints = BTreeSet::new();

    let mut total_terms = 0usize;
    let mut max_cluster_size = 0usize;
    let mut min_cluster_size = usize::MAX;

    // First pass: global statistics
    for cluster in clusters {
        max_cluster_size = max_cluster_size.max(cluster.len());
        min_cluster_size = min_cluster_size.min(cluster.len());

        for &expression_id in cluster {
            let (constraint_id, expression, _) = &expressions[expression_id];

            all_constraints.insert(*constraint_id);

            for (signal, coeff) in expression {
                if *signal != constant && coeff != &BigInt::from(0) {
                    all_variables.insert(*signal);
                    total_terms += 1;
                }
            }
        }
    }

    let num_expressions = expressions.len();

    let mut json = String::new();

    // Global metadata
    writeln!(&mut json, "{{").unwrap();
    writeln!(&mut json, "  \"metadata\": {{").unwrap();
    writeln!(&mut json, "    \"number_of_clusters\": {},", clusters.len()).unwrap();
    writeln!(&mut json, "    \"number_of_expressions\": {},", num_expressions).unwrap();
    writeln!(&mut json, "    \"number_of_constraints\": {},", all_constraints.len()).unwrap();
    writeln!(&mut json, "    \"number_of_distinct_variables\": {},", all_variables.len()).unwrap();
    writeln!(&mut json, "    \"total_terms\": {},", total_terms).unwrap();
    writeln!(
        &mut json,
        "    \"min_cluster_size\": {},",
        if clusters.is_empty() { 0 } else { min_cluster_size }
    )
    .unwrap();
    writeln!(&mut json, "    \"max_cluster_size\": {}", max_cluster_size).unwrap();
    writeln!(&mut json, "  }},").unwrap();

    writeln!(&mut json, "  \"clusters\": [").unwrap();

    // Each cluster
    for (cluster_id, cluster) in clusters.iter().enumerate() {
        let mut cluster_variables = BTreeSet::new();
        let mut cluster_constraints = BTreeSet::new();
        let mut cluster_total_terms = 0usize;

        for &expression_id in cluster {
            let (constraint_id, expression, _) = &expressions[expression_id];

            cluster_constraints.insert(*constraint_id);

            for (signal, coeff) in expression {
                if *signal != constant && coeff != &BigInt::from(0) {
                    cluster_variables.insert(*signal);
                    cluster_total_terms += 1;
                }
            }
        }

        writeln!(&mut json, "    {{").unwrap();
        writeln!(&mut json, "      \"cluster_id\": {},", cluster_id).unwrap();
        writeln!(&mut json, "      \"number_of_expressions\": {},", cluster.len()).unwrap();
        writeln!(&mut json, "      \"number_of_constraints\": {},", cluster_constraints.len())
            .unwrap();
        writeln!(&mut json, "      \"number_of_distinct_variables\": {},", cluster_variables.len())
            .unwrap();
        writeln!(&mut json, "      \"total_terms\": {},", cluster_total_terms).unwrap();

        // Constraint identifiers
        write!(&mut json, "      \"constraint_ids\": [").unwrap();

        for (i, id) in cluster_constraints.iter().enumerate() {
            if i > 0 {
                write!(&mut json, ", ").unwrap();
            }
            write!(&mut json, "{}", id).unwrap();
        }

        writeln!(&mut json, "],").unwrap();

        // Variables used by this cluster
        write!(&mut json, "      \"variables\": [").unwrap();

        for (i, signal) in cluster_variables.iter().enumerate() {
            if i > 0 {
                write!(&mut json, ", ").unwrap();
            }
            write!(&mut json, "{}", signal).unwrap();
        }

        writeln!(&mut json, "],").unwrap();

        // Expressions
        writeln!(&mut json, "      \"expressions\": [").unwrap();

        for (position, &expression_id) in cluster.iter().enumerate() {
            let (constraint_id, expression, required_aux) = &expressions[expression_id];

            let mut terms: Vec<(usize, BigInt)> = expression
                .iter()
                .filter(|(signal, coeff)| **signal != constant && **coeff != BigInt::from(0))
                .map(|(signal, coeff)| (*signal, coeff.clone()))
                .collect();

            terms.sort_by_key(|(signal, _)| *signal);

            writeln!(&mut json, "        {{").unwrap();
            writeln!(&mut json, "          \"expression_id\": {},", expression_id).unwrap();
            writeln!(&mut json, "          \"constraint_id\": {},", constraint_id).unwrap();
            writeln!(&mut json, "          \"number_of_variables\": {},", terms.len()).unwrap();
            writeln!(&mut json, "          \"required_auxiliary_variables\": {},", required_aux)
                .unwrap();

            writeln!(&mut json, "          \"terms\": [").unwrap();

            for (term_position, (signal, coeff)) in terms.iter().enumerate() {
                writeln!(&mut json, "            {{").unwrap();
                writeln!(&mut json, "              \"signal\": {},", signal).unwrap();
                writeln!(&mut json, "              \"coefficient\": \"{}\"", coeff).unwrap();

                if term_position + 1 == terms.len() {
                    writeln!(&mut json, "            }}").unwrap();
                } else {
                    writeln!(&mut json, "            }},").unwrap();
                }
            }

            writeln!(&mut json, "          ]").unwrap();

            if position + 1 == cluster.len() {
                writeln!(&mut json, "        }}").unwrap();
            } else {
                writeln!(&mut json, "        }},").unwrap();
            }
        }

        writeln!(&mut json, "      ]").unwrap();

        if cluster_id + 1 == clusters.len() {
            writeln!(&mut json, "    }}").unwrap();
        } else {
            writeln!(&mut json, "    }},").unwrap();
        }
    }

    writeln!(&mut json, "  ]").unwrap();
    writeln!(&mut json, "}}").unwrap();

    fs::write(path, json)
}

fn rebuild_witness(
    max_signal: usize,
    deleted: &mut HashSet<usize>,
    forbidden: &HashSet<usize>,
    non_linear_map: SignalToConstraints,
    remove_unused: bool,
) -> SignalMap {
    let mut map = SignalMap::with_capacity(max_signal);
    let mut free = LinkedList::new();
    for signal in 0..max_signal {
        if deleted.contains(&signal) {
            free.push_back(signal);
        } else if remove_unused
            && !forbidden.contains(&signal)
            && !non_linear_map.contains_key(&signal)
        {
            deleted.insert(signal);
            free.push_back(signal);
        } else if let Some(new_pos) = free.pop_front() {
            map.insert(signal, new_pos);
            free.push_back(signal);
        } else {
            map.insert(signal, signal);
        }
    }
    map
}

fn eq_cluster_simplification(
    mut cluster: Cluster,
    forbidden: &HashSet<usize>,
    field: &BigInt,
) -> (LinkedList<S>, LinkedList<C>) {
    if Cluster::size(&cluster) == 1 {
        let mut substitutions = LinkedList::new();
        let mut constraints = LinkedList::new();
        let constraint = LinkedList::pop_back(&mut cluster.constraints).unwrap();
        let signals: Vec<_> = C::take_cloned_signals_ordered(&constraint).iter().cloned().collect();
        let s_0 = signals[0];
        let s_1 = signals[1];
        if HashSet::contains(forbidden, &s_0) && HashSet::contains(forbidden, &s_1) {
            LinkedList::push_back(&mut constraints, constraint);
        } else if HashSet::contains(forbidden, &s_0) {
            LinkedList::push_back(
                &mut substitutions,
                S::new(s_1, A::Signal { symbol: s_0 }).unwrap(),
            );
        } else if HashSet::contains(forbidden, &s_1) {
            LinkedList::push_back(
                &mut substitutions,
                S::new(s_0, A::Signal { symbol: s_1 }).unwrap(),
            );
        } else {
            let (l, r) = if s_0 > s_1 { (s_0, s_1) } else { (s_1, s_0) };
            LinkedList::push_back(&mut substitutions, S::new(l, A::Signal { symbol: r }).unwrap());
        }
        (substitutions, constraints)
    } else {
        let mut cons = LinkedList::new();
        let mut subs = LinkedList::new();
        let (mut remains, mut min_remains) = (BTreeSet::new(), None);
        let (mut remove, mut min_remove) = (HashSet::new(), None);
        for c in cluster.constraints {
            for signal in C::take_cloned_signals_ordered(&c) {
                if HashSet::contains(&forbidden, &signal) {
                    BTreeSet::insert(&mut remains, signal);
                    min_remains = Some(min_remains.map_or(signal, |s| std::cmp::min(s, signal)));
                } else {
                    min_remove = Some(min_remove.map_or(signal, |s| std::cmp::min(s, signal)));
                    HashSet::insert(&mut remove, signal);
                }
            }
        }

        let rh_signal = if let Some(signal) = min_remains {
            BTreeSet::remove(&mut remains, &signal);
            signal
        } else {
            let signal = min_remove.unwrap();
            HashSet::remove(&mut remove, &signal);
            signal
        };

        for signal in remains {
            let l = A::Signal { symbol: signal };
            let r = A::Signal { symbol: rh_signal };
            let expr = A::sub(&l, &r, field);
            let c = A::transform_expression_to_constraint_form(expr, field).unwrap();
            LinkedList::push_back(&mut cons, c);
        }

        for signal in remove {
            let sub = S::new(signal, A::Signal { symbol: rh_signal }).unwrap();
            LinkedList::push_back(&mut subs, sub);
        }

        (subs, cons)
    }
}

fn eq_simplification(
    equalities: LinkedList<C>,
    forbidden: Arc<HashSet<usize>>,
    no_vars: usize,
    field: &BigInt,
    substitution_log: &mut Option<SubstitutionJSON>,
) -> (LinkedList<S>, LinkedList<C>) {
    use std::sync::mpsc;
    use threadpool::ThreadPool;
    let field = Arc::new(field.clone());
    let mut constraints = LinkedList::new();
    let mut substitutions = LinkedList::new();
    let clusters = build_clusters(equalities, no_vars);
    let (cluster_tx, simplified_rx) = mpsc::channel();
    let pool = ThreadPool::new(num_cpus::get());
    let no_clusters = Vec::len(&clusters);
    // println!("Clusters: {}", no_clusters);
    let mut single_clusters = 0;
    let mut id = 0;
    let mut aux_constraints = vec![LinkedList::new(); clusters.len()];
    for cluster in clusters {
        if Cluster::size(&cluster) == 1 {
            let (mut subs, cons) = eq_cluster_simplification(cluster, &forbidden, &field);
            aux_constraints[id] = cons;
            LinkedList::append(&mut substitutions, &mut subs);
            single_clusters += 1;
        } else {
            let cluster_tx = cluster_tx.clone();
            let forbidden = Arc::clone(&forbidden);
            let field = Arc::clone(&field);
            let job = move || {
                //println!("Cluster: {}", id);
                let result = eq_cluster_simplification(cluster, &forbidden, &field);
                //println!("End of cluster: {}", id);
                cluster_tx.send((id, result)).unwrap();
            };
            ThreadPool::execute(&pool, job);
        }
        let _ = id;
        id += 1;
    }
    // println!("{} clusters were of size 1", single_clusters);
    ThreadPool::join(&pool);
    for _ in 0..(no_clusters - single_clusters) {
        let (id, (mut subs, cons)) = simplified_rx.recv().unwrap();
        aux_constraints[id] = cons;
        LinkedList::append(&mut substitutions, &mut subs);
    }
    for id in 0..no_clusters {
        LinkedList::append(&mut constraints, &mut aux_constraints[id]);
    }
    log_substitutions(&substitutions, substitution_log);
    (substitutions, constraints)
}

fn constant_eq_simplification(
    c_eq: LinkedList<C>,
    forbidden: &HashSet<usize>,
    field: &BigInt,
    substitution_log: &mut Option<SubstitutionJSON>,
) -> (LinkedList<S>, LinkedList<C>) {
    let mut cons = LinkedList::new();
    let mut subs = LinkedList::new();
    for constraint in c_eq {
        let mut signals: Vec<_> =
            C::take_cloned_signals_ordered(&constraint).iter().cloned().collect();
        let signal = signals.pop().unwrap();
        if HashSet::contains(&forbidden, &signal) {
            LinkedList::push_back(&mut cons, constraint);
        } else {
            let sub = C::clear_signal_from_linear(constraint, &signal, field);
            LinkedList::push_back(&mut subs, sub);
        }
    }
    log_substitutions(&subs, substitution_log);
    (subs, cons)
}

fn linear_simplification(
    log: &mut Option<SubstitutionJSON>,
    linear: LinkedList<C>,
    forbidden: Arc<HashSet<usize>>,
    no_labels: usize,
    field: &BigInt,
    use_old_heuristics: bool,
) -> (LinkedList<S>, LinkedList<C>) {
    use circom_algebra::simplification_utils::full_simplification;
    use circom_algebra::simplification_utils::Config;
    use std::sync::mpsc;
    use threadpool::ThreadPool;

    // println!("Cluster simplification");
    let mut cons = LinkedList::new();
    let mut substitutions = LinkedList::new();
    let clusters = build_clusters(linear, no_labels);
    let (cluster_tx, simplified_rx) = mpsc::channel();
    let pool = ThreadPool::new(num_cpus::get());
    let no_clusters = Vec::len(&clusters);
    // println!("Clusters: {}", no_clusters);
    let mut id = 0;
    for cluster in clusters {
        let cluster_tx = cluster_tx.clone();
        let config = Config {
            field: field.clone(),
            constraints: cluster.constraints,
            forbidden: Arc::clone(&forbidden),
            num_signals: cluster.num_signals,
            use_old_heuristics,
        };
        let job = move || {
            // println!("cluster: {}", id);
            let result = full_simplification(config);
            // println!("End of cluster: {}", id);
            cluster_tx.send(result).unwrap();
        };
        ThreadPool::execute(&pool, job);
        let _ = id;
        id += 1;
    }
    ThreadPool::join(&pool);

    for _ in 0..no_clusters {
        let mut result = simplified_rx.recv().unwrap();
        log_substitutions(&result.substitutions, log);
        LinkedList::append(&mut cons, &mut result.constraints);
        LinkedList::append(&mut substitutions, &mut result.substitutions);
    }
    (substitutions, cons)
}

type SignalToConstraints = HashMap<usize, LinkedList<usize>>;
fn build_non_linear_signal_map(non_linear: &ConstraintStorage) -> SignalToConstraints {
    let mut map = SignalToConstraints::new();
    for c_id in non_linear.get_ids() {
        let constraint = non_linear.read_constraint(c_id).unwrap();
        for signal in C::take_cloned_signals(&constraint) {
            if let Some(list) = map.get_mut(&signal) {
                list.push_back(c_id);
            } else {
                let mut new = LinkedList::new();
                new.push_back(c_id);
                map.insert(signal, new);
            }
        }
    }
    map
}

fn apply_substitution_to_map(
    storage: &mut ConstraintStorage,
    map: &mut SignalToConstraints,
    substitutions: &LinkedList<S>,
    field: &BigInt,
) -> LinkedList<C> {
    fn constraint_processing(
        storage: &mut ConstraintStorage,
        map: &mut SignalToConstraints,
        c_ids: &LinkedList<usize>,
        substitution: &S,
        field: &BigInt,
    ) -> LinkedList<usize> {
        let mut linear = LinkedList::new();
        let signals: LinkedList<_> = substitution.to().keys().cloned().collect();
        for c_id in c_ids {
            let c_id = *c_id;
            let mut constraint = storage.read_constraint(c_id).unwrap();
            C::apply_substitution(&mut constraint, substitution, field);
            C::fix_constraint(&mut constraint, field);
            if C::is_linear(&constraint) {
                linear.push_back(c_id);
            }
            storage.replace(c_id, constraint);
            for signal in &signals {
                if let Some(list) = map.get_mut(&signal) {
                    list.push_back(c_id);
                } else {
                    let mut new = LinkedList::new();
                    new.push_back(c_id);
                    map.insert(*signal, new);
                }
            }
        }
        linear
    }

    let mut linear_id = LinkedList::new();
    for substitution in substitutions {
        if let Some(c_ids) = map.get(substitution.from()).cloned() {
            let mut new_linear = constraint_processing(storage, map, &c_ids, substitution, field);
            linear_id.append(&mut new_linear);
        }
    }
    let mut linear = LinkedList::new();
    for c_id in linear_id {
        let constraint = storage.read_constraint(c_id).unwrap();
        linear.push_back(constraint);
        storage.replace(c_id, C::empty());
    }
    linear
}

fn build_relevant_set(
    mut iter: EncodingIterator,
    relevant: &mut HashSet<usize>,
    renames: &SEncoded,
    deletes: &SEncoded,
) {
    fn unwrapped_signal(map: &SEncoded, signal: usize) -> Option<usize> {
        let f = |e: &A| {
            if let A::Signal { symbol } = e {
                Some(*symbol)
            } else {
                None
            }
        };
        SEncoded::get(map, &signal).map_or(None, f)
    }

    let (_, non_linear) = EncodingIterator::take(&mut iter);
    for c in non_linear {
        for signal in C::take_cloned_signals(&c) {
            let signal = unwrapped_signal(renames, signal).unwrap_or(signal);
            if !SEncoded::contains_key(deletes, &signal) {
                HashSet::insert(relevant, signal);
            }
        }
    }

    for edge in EncodingIterator::edges(&iter) {
        let next = EncodingIterator::next(&iter, edge);
        build_relevant_set(next, relevant, renames, deletes)
    }
}

fn remove_not_relevant(substitutions: &mut SEncoded, relevant: &HashSet<usize>) {
    let signals: Vec<_> = substitutions.keys().cloned().collect();
    for signal in signals {
        if !HashSet::contains(&relevant, &signal) {
            SEncoded::remove(substitutions, &signal);
        }
    }
}

// returns the constraints, the assignment of the witness and the number of inputs in the witness
pub fn simplification(smp: &mut Simplifier) -> (ConstraintStorage, SignalMap, usize) {
    use super::non_linear_utils::obtain_and_simplify_non_linear;
    use circom_algebra::simplification_utils::build_encoded_fast_substitutions;
    use circom_algebra::simplification_utils::fast_encoded_constraint_substitution;
    use std::time::SystemTime;

    let mut substitution_log = if smp.port_substitution {
        Some(SubstitutionJSON::new(&smp.json_substitutions).unwrap())
    } else {
        None
    };
    let apply_linear = !smp.flag_s && !smp.plonk_simplification;
    let use_old_heuristics = smp.flag_old_heuristics;
    let field = smp.field.clone();
    let forbidden = Arc::new(std::mem::replace(&mut smp.forbidden, HashSet::with_capacity(0)));
    let no_labels = Simplifier::no_labels(smp);
    let equalities = std::mem::replace(&mut smp.equalities, LinkedList::new());
    let mut max_signal = smp.max_signal;
    let mut cons_equalities = std::mem::replace(&mut smp.cons_equalities, LinkedList::new());
    let mut linear = std::mem::replace(&mut smp.linear, LinkedList::new());
    let mut deleted = HashSet::new();
    let mut lconst = LinkedList::new();
    let mut no_rounds = smp.no_rounds;
    let remove_unused = true;

    let relevant_signals = {
        // println!("Creating first relevant set");
        let now = SystemTime::now();
        let mut relevant = HashSet::new();
        let iter = EncodingIterator::new(&smp.dag_encoding);
        let s_sub = HashMap::with_capacity(0);
        let c_sub = HashMap::with_capacity(0);
        build_relevant_set(iter, &mut relevant, &s_sub, &c_sub);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("First relevant set created: {} ms", dur);
        relevant
    };

    let single_substitutions = {
        // println!("Start of single assignment simplification");
        let now = SystemTime::now();
        let (subs, mut cons) = eq_simplification(
            equalities,
            Arc::clone(&forbidden),
            no_labels,
            &field,
            &mut substitution_log,
        );

        LinkedList::append(&mut lconst, &mut cons);
        let mut substitutions = build_encoded_fast_substitutions(subs);
        for constraint in &mut linear {
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field) {
                C::fix_constraint(constraint, &field);
            }
        }
        for constraint in &mut cons_equalities {
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field) {
                C::fix_constraint(constraint, &field);
            }
        }
        for signal in substitutions.keys().cloned() {
            deleted.insert(signal);
        }
        remove_not_relevant(&mut substitutions, &relevant_signals);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("End of single assignment simplification: {} ms", dur);
        substitutions
    };

    let cons_substitutions = {
        // println!("Start of constant assignment simplification");
        let now = SystemTime::now();
        let (subs, mut cons) =
            constant_eq_simplification(cons_equalities, &forbidden, &field, &mut substitution_log);
        LinkedList::append(&mut lconst, &mut cons);
        let substitutions = build_encoded_fast_substitutions(subs);
        for constraint in &mut linear {
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field) {
                C::fix_constraint(constraint, &field);
            }
        }
        for signal in substitutions.keys().cloned() {
            deleted.insert(signal);
        }
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("End of constant assignment simplification: {} ms", dur);
        substitutions
    };

    let relevant_signals = {
        // println!("Start building relevant");
        let now = SystemTime::now();
        let mut relevant = HashSet::new();
        let iter = EncodingIterator::new(&smp.dag_encoding);
        build_relevant_set(iter, &mut relevant, &single_substitutions, &cons_substitutions);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("Relevant built: {} ms", dur);
        relevant
    };

    let linear_substitutions = if apply_linear {
        let now = SystemTime::now();
        let (subs, mut cons) = linear_simplification(
            &mut substitution_log,
            linear,
            Arc::clone(&forbidden),
            no_labels,
            &field,
            use_old_heuristics,
        );
        // println!("Building substitution map");
        let now0 = SystemTime::now();
        let mut only_relevant = LinkedList::new();
        for substitution in subs {
            deleted.insert(*substitution.from());
            if relevant_signals.contains(substitution.from()) {
                only_relevant.push_back(substitution);
            }
        }
        let substitutions = build_encoded_fast_substitutions(only_relevant);
        let _dur0 = now0.elapsed().unwrap().as_millis();
        // println!("End of substitution map: {} ms", dur0);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("End of cluster simplification: {} ms", dur);
        LinkedList::append(&mut lconst, &mut cons);
        for constraint in &mut lconst {
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field) {
                C::fix_constraint(constraint, &field);
            }
        }
        substitutions
    } else {
        LinkedList::append(&mut lconst, &mut linear);
        HashMap::with_capacity(0)
    };

    let (with_linear, mut constraint_storage) = {
        // println!("Building constraint storage");
        let now = SystemTime::now();
        let mut frames = LinkedList::new();
        LinkedList::push_back(&mut frames, single_substitutions);
        LinkedList::push_back(&mut frames, cons_substitutions);
        LinkedList::push_back(&mut frames, linear_substitutions);
        let iter = EncodingIterator::new(&smp.dag_encoding);
        let mut storage = ConstraintStorage::new();
        let with_linear = obtain_and_simplify_non_linear(iter, &mut storage, &frames, &field);
        crate::state_utils::empty_encoding_constraints(&mut smp.dag_encoding);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("Storages built in {} ms", dur);
        if no_rounds > 0 {
            no_rounds -= 1;
        }
        (with_linear, storage)
    };

    let mut round_id = 0;
    let _ = round_id;
    let mut linear = with_linear;
    let mut apply_round = apply_linear && no_rounds > 0 && !linear.is_empty();
    let mut non_linear_map = if apply_round || remove_unused {
        // println!("Building non-linear map");
        let now = SystemTime::now();
        let non_linear_map = build_non_linear_signal_map(&constraint_storage);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("Non-linear was built in {} ms", dur);
        non_linear_map
    } else {
        SignalToConstraints::with_capacity(0)
    };
    while apply_round {
        let now = SystemTime::now();
        // println!("Number of linear constraints: {}", linear.len());
        let (substitutions, mut constants) = linear_simplification(
            &mut substitution_log,
            linear,
            Arc::clone(&forbidden),
            no_labels,
            &field,
            use_old_heuristics,
        );

        for sub in &substitutions {
            deleted.insert(*sub.from());
        }
        lconst.append(&mut constants);
        for constraint in &mut lconst {
            for substitution in &substitutions {
                C::apply_substitution(constraint, substitution, &field);
            }
            C::fix_constraint(constraint, &field);
        }
        linear = apply_substitution_to_map(
            &mut constraint_storage,
            &mut non_linear_map,
            &substitutions,
            &field,
        );
        round_id += 1;
        no_rounds -= 1;
        apply_round = !linear.is_empty() && no_rounds > 0;
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("Iteration no {} took {} ms", round_id, dur);
    }

    for constraint in linear {
        if remove_unused {
            let signals = C::take_cloned_signals(&constraint);
            let c_id = constraint_storage.add_constraint(constraint);
            for signal in signals {
                if let Some(list) = non_linear_map.get_mut(&signal) {
                    list.push_back(c_id);
                } else {
                    let mut new = LinkedList::new();
                    new.push_back(c_id);
                    non_linear_map.insert(signal, new);
                }
            }
        } else {
            constraint_storage.add_constraint(constraint);
        }
    }
    for mut constraint in lconst {
        if remove_unused {
            C::fix_constraint(&mut constraint, &field);
            let signals = C::take_cloned_signals(&constraint);
            let c_id = constraint_storage.add_constraint(constraint);
            for signal in signals {
                if let Some(list) = non_linear_map.get_mut(&signal) {
                    list.push_back(c_id);
                } else {
                    let mut new = LinkedList::new();
                    new.push_back(c_id);
                    non_linear_map.insert(signal, new);
                }
            }
        } else {
            C::fix_constraint(&mut constraint, &field);
            constraint_storage.add_constraint(constraint);
        }
    }

    let erased =
        crate::non_linear_simplification::simplify(&mut constraint_storage, &forbidden, &field);

    for signal in erased {
        deleted.insert(signal);
    }

    let _trash = constraint_storage.extract_with(&|c| C::is_empty(c));

    if smp.plonk_simplification {
        // At this point call to normalize all constraints
        // // if we are in PLONK mode

        let mut auxiliary_signals: BTreeMap<(usize, usize, BigInt, BigInt), usize> = BTreeMap::new();

        let mut expressions_to_fix: Vec<(usize, BTreeMap<usize, BigInt>, usize)> = Vec::new();
        let mut estimated_auxiliary_variables = 0;
        for i in constraint_storage.get_ids() {
            let mut constraint = constraint_storage.read_constraint(i).unwrap();
            constraint.normalize_constraint(&field);

            if !constraint.is_plonk() {
                let (expressions, total_auxiliaries) = constraint.vars_to_solve_plonk_with_total();
                estimated_auxiliary_variables += total_auxiliaries;
                for (expr, num_vars) in expressions {
                    let expr_btree: BTreeMap<usize, BigInt> = expr.iter().map(|(signal, coeff)| (*signal, coeff.clone())).collect();
                    expressions_to_fix.push((i, expr_btree, num_vars));
                }
            }

            constraint_storage.replace(i, constraint);
        }

        // Export the initial clusters before applying any optimization.
        let initial_clusters = build_expr_clusters(&expressions_to_fix, &field);

        if let Err(error) =
            export_expr_clusters_json("clusters.json", &expressions_to_fix, &initial_clusters)
        {
            eprintln!("Error exporting clusters.json: {}", error);
        }

        // Greedy optimization retained for comparison:
        if smp.apply_eager_plonk {
            loop {
                let mut expressions_to_fix: Vec<(usize, BTreeMap<usize, BigInt>, usize)> = Vec::new();
                for i in constraint_storage.get_ids() {
                    let constraint = constraint_storage.read_constraint(i).unwrap();
                    if !constraint.is_plonk() {
                        let (expressions, _) = constraint.vars_to_solve_plonk_with_total();
                        for (expr, num_vars) in expressions {
                            let expr_btree: BTreeMap<usize, BigInt> = expr.iter()
                                .map(|(signal, coeff)| (*signal, coeff.clone()))
                                .collect();

                            expressions_to_fix.push((i, expr_btree, num_vars));
                        }
                    }
                }

                if expressions_to_fix.is_empty() {
                    break; // all is PLONK
                }

                // Rebuild the clusters
                let clusters = build_expr_clusters(&expressions_to_fix, &field);

                let mut best_candidate: Option<(
                    (usize, usize, BigInt, BigInt),
                    usize,
                    Vec<usize>,
                )> = None;

                for cluster in &clusters {
                    let pairs = count_variable_pairs(cluster, &expressions_to_fix, &field);
                    if let Some((pair, frec)) = select_most_frequent_pair(cluster, &expressions_to_fix, pairs, &field)
                    {
                        let replace = match &best_candidate {
                            None => true,
                            Some((best_pair, best_frec, _)) => {
                                frec > *best_frec || (frec == *best_frec && pair < *best_pair)
                            }
                        };

                        if replace {
                            best_candidate = Some((pair, frec, cluster.clone()));
                        }
                    }
                }

                let (pair, _frec, cluster) = match best_candidate {
                    Some(candidate) => candidate,
                    None => break,
                };

                let modified = apply_pair_to_cluster(
                    &mut constraint_storage,
                    &cluster,
                    &expressions_to_fix,
                    pair,
                    &mut auxiliary_signals,
                    &mut max_signal,
                    &mut non_linear_map,
                    &field,
                );

                if modified == 0 {
                    eprintln!(
                        "Greedy PLONK simplification stopped: selected pair matched no constraints"
                    );
                    break;
                }
            }
        } else if smp.apply_sat_plonk {
            let mut expressions_to_fix: Vec<(usize, BTreeMap<usize, BigInt>, usize)> = Vec::new();
            for i in constraint_storage.get_ids() {
                let constraint = constraint_storage.read_constraint(i).unwrap();
                if !constraint.is_plonk() {
                    let (expressions, _) = constraint.vars_to_solve_plonk_with_total();
                    for (expr, num_vars) in expressions {
                        let expr_btree: BTreeMap<usize, BigInt> = expr.iter()
                            .map(|(signal, coeff)| (*signal, coeff.clone()))
                            .collect();

                        expressions_to_fix.push((i, expr_btree, num_vars));
                    }
                }
            }

            let clusters = build_expr_clusters(&expressions_to_fix, &field);
            let mut auxiliary_constraint_ids : BTreeMap<usize, usize> = BTreeMap::new();
            let mut i = 0;
            for cluster in &clusters {
                // variables_optimization(&mut constraint_storage, &expressions_to_fix, cluster, &mut auxiliary_signals, &mut max_signal, &mut non_linear_map, &field);
                // variables_optimization_unsat_core(&mut constraint_storage, &expressions_to_fix, cluster, &mut auxiliary_signals, &mut max_signal, &mut non_linear_map, &field, i, &mut auxiliary_constraint_ids);
                // variables_optimization_bitwuzla(&mut constraint_storage, &expressions_to_fix, cluster, &mut auxiliary_signals, &mut max_signal, &mut non_linear_map, &field, i, &mut auxiliary_constraint_ids);
                variables_optimization_cadical(&mut constraint_storage, &expressions_to_fix, cluster, &mut auxiliary_signals, &mut max_signal, &mut non_linear_map, &field, i, &mut auxiliary_constraint_ids);
                i += 1;
            }
            restore_plonk_constraints(
                &mut constraint_storage,
                &auxiliary_signals,
                &auxiliary_constraint_ids,
                &mut non_linear_map,
                &mut deleted,
                &field,
            );
        }

        // println!("\n========================================");
        // println!("       FINAL CONSTRAINTS");
        // println!("========================================");
        // println!("Estimated auxiliary-variable upper bound: {}", estimated_auxiliary_variables);
        // println!("Auxiliary variables introduced by Z3: {}", auxiliary_signals.len());

        let mut final_plonk = 0;
        let mut final_non_plonk = 0;

        for constraint_id in constraint_storage.get_ids() {
            let constraint = constraint_storage.read_constraint(constraint_id).unwrap();

            // println!("\nConstraint {}:", constraint_id);
            // fn print_expression(expr: &HashMap<usize, BigInt>) {
            //     print!("{{");

            //     for (signal, coefficient) in expr {
            //         print!("{} * x{}, ", coefficient, signal);
            //     }

            //     println!("}}");
            // }

            // print!("  A = ");
            // print_expression(&constraint.a());

            // print!("  B = ");
            // print_expression(&constraint.b());

            // print!("  C = ");
            // print_expression(&constraint.c());

            let constant = C::constant_coefficient();
            let count_variables = |expression: &HashMap<usize, BigInt>| {
                expression
                    .iter()
                    .filter(|(signal, coefficient)| {
                        **signal != constant && **coefficient != BigInt::from(0)
                    })
                    .count()
            };
            let mut distinct_variables = HashSet::new();
            for expression in [&constraint.a(), &constraint.b(), &constraint.c()] {
                distinct_variables.extend(
                    expression
                        .iter()
                        .filter(|(signal, coefficient)| {
                            **signal != constant && **coefficient != BigInt::from(0)
                        })
                        .map(|(signal, _)| *signal),
                );
            }
            // println!(
            //     "  Remaining variables after reconstruction: A={}, B={}, C={}, distinct in constraint={}",
            //     count_variables(constraint.a()),
            //     count_variables(constraint.b()),
            //     count_variables(constraint.c()),
            //     distinct_variables.len()
            // );

            if constraint.is_plonk() {
                // println!("  State: PLONK");
                final_plonk += 1;
            } else {
                // println!("  State: NO PLONK");
                final_non_plonk += 1;
            }
        }

        println!("\n========================================");
        println!("Constraints PLONK: {}", final_plonk);
        println!("Constraints NO PLONK: {}", final_non_plonk);
        println!("Total: {}", final_plonk + final_non_plonk);
        println!("========================================\n");
    }

    let signal_map = {
        // println!("Rebuild witness");
        let now = SystemTime::now();
        let signal_map =
            rebuild_witness(max_signal, &mut deleted, &forbidden, non_linear_map, remove_unused);
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("End of rebuild witness: {} ms", dur);
        signal_map
    };

    // count the number of deleted inputs
    let max_value_input = smp.no_public_outputs + smp.no_public_inputs + smp.no_private_inputs;
    let mut deleted_inputs = 0;
    for signal in &deleted {
        if signal >= &(smp.no_public_outputs + 1) && signal <= &max_value_input {
            deleted_inputs += 1;
        }
    }

    if let Some(w) = substitution_log {
        w.end().unwrap();
    }
    // println!("NO CONSTANTS: {}", constraint_storage.no_constants());
    (constraint_storage, signal_map, smp.no_private_inputs - deleted_inputs)
}
