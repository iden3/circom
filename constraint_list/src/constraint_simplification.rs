// Uncomment lines 163, 165, 336 and 338 to print cluster information
use super::{ConstraintStorage, EncodingIterator, SEncoded, Simplifier, A, C, S};
use crate::SignalMap;
use circom_algebra::num_bigint::BigInt;
use constraint_writers::json_writer::SubstitutionJSON;
use std::collections::{HashMap, HashSet, LinkedList, BTreeSet};
use std::sync::Arc;
use z3::ast::{Bool, Int};
use z3::{Config, Optimize, SatResult, with_z3_config};
use std::str::FromStr;

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
    num_signals: usize
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
        if !constraint.is_empty(){
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

fn build_expr_clusters(expressions: &Vec<(usize, HashMap<usize, BigInt>, usize)>, no_vars: usize) -> Vec<Vec<usize>> {

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
    let mut cluster_to_current : ClusterPath = Vec::with_capacity(no_expressions);
    let mut signal_to_cluster = vec![no_expressions; no_vars];

    for expression_id in 0..no_expressions {
        let (_, expr, _) = &expressions[expression_id];

        let constant = C::constant_coefficient();
        let signals: Vec<usize> = expr.keys().filter(|signal| **signal != constant).cloned().collect();

        let dest = expression_id;

        Vec::push(&mut cluster_to_current, dest);

        for signal in signals {
            let prev = signal_to_cluster[signal];
            signal_to_cluster[signal] = dest;

            if prev < no_expressions {
                arena_merge(&mut cluster_to_current, prev, dest);
            }
        }
    }

    let mut clusters: Vec<Vec<usize>> = Vec::new();
    for expression_id in 0..no_expressions {
        let root = shrink_jumps_and_find(&mut cluster_to_current, expression_id);

        while clusters.len() <= root {
            clusters.push(Vec::new());
        }

        clusters[root].push(expression_id);
    }

    clusters
}

fn count_variable_pairs(cluster: &[usize], expressions: &[(usize, HashMap<usize, BigInt>, usize)]) -> Vec<((usize, usize, BigInt, BigInt), usize)> {

    let mut frequencies : HashMap<(usize, usize, BigInt, BigInt), usize> = HashMap::new();

    for &expression_id in cluster {
        let (_,expression,_) = &expressions[expression_id];


        let mut terms: Vec<(usize, BigInt)> = Vec::new();
        
        for (signal, coeff) in expression {
            if *signal != 0 && coeff != &BigInt::from(0) {
                terms.push((*signal, coeff.clone()));
            }
        }

        terms.sort_by_key(|(signal, _)| *signal);

        for i in 0..terms.len() {
            for j in (i + 1)..terms.len() {
                let pair = (terms[i].0, terms[j].0, terms[i].1.clone(), terms[j].1.clone());

                *frequencies.entry(pair).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<((usize, usize, BigInt, BigInt), usize)> = frequencies.into_iter().collect();

    result
}

fn select_most_frequent_pair(cluster: &[usize], expressions: &[(usize, HashMap<usize, BigInt>, usize)], pairs: Vec<((usize, usize, BigInt, BigInt), usize)>)
-> Option<((usize, usize, BigInt, BigInt), usize)> {

    let mut valid_pairs = Vec::new();

    for (pair, frequency) in pairs {
        let (signal_a, signal_b, coeff_a, coeff_b) = &pair;
        let mut can_transform = false;

        for &expression_id in cluster {
            let (_, expression, _) = &expressions[expression_id];

            if expression.get(signal_a) == Some(coeff_a) && expression.get(signal_b) == Some(coeff_b) {
                can_transform = true;
                break;
            }
        }

        if can_transform {
            valid_pairs.push((pair, frequency));
        }
    }

    let mut best_pair = None;
    let mut best_frec = 0;

    for candidate in valid_pairs {
        if best_pair.is_none() || candidate.1 > best_frec || (candidate.1 == best_frec && candidate.0 < *best_pair.as_ref().unwrap()) {
            best_pair = Some(candidate.0);
            best_frec = candidate.1;
        }
    }

    if let Some(pair) = best_pair {
        Some((pair, best_frec))
    }
    else {
        None
    }
}

// This function creates the constraint for the new variable introduced
fn create_aux_constraint(auxiliary:usize, pair: &(usize, usize, BigInt, BigInt), field: &BigInt) -> C {
    let mut coefficients = HashMap::new();

    coefficients.insert(0, BigInt::from(0));
    coefficients.insert(pair.0, pair.2.clone());
    coefficients.insert(pair.1, pair.3.clone());

    let substitution = S::new(auxiliary, A::Linear{ coefficients }).unwrap();

    let mut constraint = S::substitution_into_constraint(substitution, field);

    constraint
}

fn apply_pair_to_cluster(storage: &mut ConstraintStorage, cluster: &[usize], expressions: &[(usize, HashMap<usize, BigInt>, usize)], pair: (usize, usize, BigInt, BigInt),
aux_signals: &mut HashMap<(usize, usize, BigInt, BigInt), usize>, next_aux: &mut usize, non_linear_map: &mut SignalToConstraints, field: &BigInt) -> usize {

    // If the aux var exists for those variables and coefficients, it is reused
    let aux = if let Some(existing) = aux_signals.get(&pair) {
        *existing
    }
    // If not, it creates a new variable and introduces the corresponding constraint
    else { 
        let new_aux = *next_aux;
        *next_aux += 1;

        let constraint = create_aux_constraint(new_aux, &pair, field);
        let constraint_id = storage.add_constraint(constraint);

        non_linear_map.entry(new_aux).or_insert_with(LinkedList::new).push_back(constraint_id);
        aux_signals.insert(pair.clone(), new_aux);

        new_aux
    };

    let (s_a, s_b, coeff_a, coeff_b) = pair;
    let mut processed_constraints = HashSet::new();

    // Replaces each appearance of the pair in the cluster constraints
    for &expression_id in cluster {
        let (constraint_id, _, _) = &expressions[expression_id];

        if !processed_constraints.insert(*constraint_id) {
            continue;
        }

        let mut constraint = storage.read_constraint(*constraint_id).unwrap();
        constraint.replace_pair_with_auxiliary(s_a, &coeff_a, s_b, &coeff_b, aux);
        constraint.normalize_constraint(field);

        storage.replace(*constraint_id, constraint);
    }

    aux
}

// Optimization of the number of new variables introduced, using an SMT solver
fn variables_optimization(storage: &mut ConstraintStorage, expression: &[(usize, HashMap<usize, BigInt>, usize)], cluster: &[usize], aux_signals: &mut HashMap<(usize, usize, BigInt, BigInt), usize>,
next_aux: &mut usize, non_linear_map: &mut SignalToConstraints, field: &BigInt) { // -> bool {

    let mut constraints_ids = Vec::new();
    let mut coeffs_and_vars: Vec<(usize, BigInt)> = Vec::new();

    for &expression_id in cluster {
        let constraint_id = expression[expression_id].0;
        if !constraints_ids.contains(&constraint_id) { constraints_ids.push(constraint_id); }
        
        for (var, coeff) in &expression[expression_id].1 {
            if *var == 0 || coeff == &BigInt::from(0) { continue; }

            let pair = (*var, coeff.clone());
            if !coeffs_and_vars.contains(&pair) {
                coeffs_and_vars.push(pair);
            }
        }
    }
    
    // if constraints_ids.is_empty() { return false; }
    
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

    let mut bound = 0; // Maximum number of variables that are needed to transform all the expressions of the cluster into plonk
    for &expression_id in cluster {
        bound += expression[expression_id].2;
    }

    let config = Config::new();
    with_z3_config(&config, || {
        let mut solver = Optimize::new();
        let zero = Int::from_i64(0);
        let one = Int::from_i64(1);

        let mut expr_use_vars = Vec::new();
        let rows_expr = cluster.len();
        let cols_expr = coeffs_and_vars.len() + bound;
        
        for e in 0..rows_expr {
            for c in 0..cols_expr {
                let var = Bool::new_const(format!("expr_{}_uses_var_{}", e, c));

                expr_use_vars.push(var);
            }
        }

        let get_expr_uses = |r:usize, c: usize| { expr_use_vars[r * cols_expr + c].clone() };

        let mut vars_composition = Vec::new();
        let rows_vars = bound;
        let cols_vars = coeffs_and_vars.len() + bound;

        for v in 0..rows_vars {
            for c in 0..cols_vars {
                let v = Bool::new_const(format!("var_{}_uses_{}", v, c));
                vars_composition.push(v);
            }
        }

        let get_v_composition = |r:usize, c: usize| { vars_composition[r * cols_vars + c].clone() };

        // Each new variable can be empty or formed with exactly 2 original variables or previous declared new variables
        for v in 0..rows_vars {
            let mut num_vars_compose = Int::from_i64(0);
            for c in 0..cols_vars {
                let selected = get_v_composition(v, c);
                let contribution = selected.ite(&one, &zero);

                num_vars_compose = &num_vars_compose + &contribution;
            }

            let two = Int::from_i64(2);
            let is_empty = num_vars_compose._eq(&zero);
            let has_two_terms = num_vars_compose._eq(&two);
            solver.assert(Bool::or(&[&is_empty, &has_two_terms]));

            // Two terms with the same variables cannot be used in the same variable
            // for first in 0..coeffs_and_vars.len() {
            //     for second in (first + 1)..coeffs_and_vars.len() {
            //         if coeffs_and_vars[first].0 == coeffs_and_vars[second].0 {
            //             let both_selected = Bool::and(&[&get_v_composition(v, first), &get_v_composition(v, second)]);
            //             solver.assert(&both_selected.not());
            //         }
            //     }
            // }
        }

        // Each new variable can only use previous variables and cannot use itself to create a new variable
        for v in 0..rows_vars {
            for c in (coeffs_and_vars.len()+ v)..cols_vars {
                solver.assert(&get_v_composition(v, c).not());
            }
        } 
        
        // Empty variables must be at the end of the matrix
        for v in 0..rows_vars.saturating_sub(1) {
            let mut num_vars_v = Int::from_i64(0);
            let mut num_vars_next = Int::from_i64(0);

            for c in 0..cols_vars {
                let selected_v = get_v_composition(v, c);
                let contribution_v = selected_v.ite(&one, &zero);
                num_vars_v = &num_vars_v + &contribution_v;

                let selected_next = get_v_composition(v + 1, c);
                let contribution_next = selected_next.ite(&one, &zero);
                num_vars_next = &num_vars_next + &contribution_next;
            }

            let current_is_empty = num_vars_v._eq(&zero);
            let next_is_empty = num_vars_next._eq(&zero);

            solver.assert(&current_is_empty.implies(&next_is_empty));
        }
        
        // Each new variable, how much original variables of each type does it have
        let mut vars_var = Vec::new();
        for v in 0.. rows_vars {
            let mut v_vars = Vec::new();
            for v_orig_id in 0..coeffs_and_vars.len() {
                let var = Int::new_const(format!("var_{}_contains_x_vars_{}", v, v_orig_id));

                v_vars.push(var);
            }

            vars_var.push(v_vars);
        }

        for v in 0..rows_vars {
            for v_orig_id in 0..coeffs_and_vars.len() {
                let original_signal = coeffs_and_vars[v_orig_id].0;
                let mut num_v = Int::from_i64(0);

                for c in 0..cols_vars {
                    let selected = get_v_composition(v, c);

                    if c < coeffs_and_vars.len() {
                        if coeffs_and_vars[c].0 == original_signal {
                            let coeff_str = coeffs_and_vars[c].1.to_string();
                            let coeff = Int::from_str(&coeff_str).unwrap();

                            let contribution = selected.ite(&coeff, &zero);
                            num_v = &num_v + &contribution;
                        }
                    } else {
                        let previous_aux_id = c - coeffs_and_vars.len();

                        if previous_aux_id < v {
                            let previous_count = &vars_var[previous_aux_id][v_orig_id];

                            let contribution = selected.ite(previous_count, &zero);
                            num_v = &num_v + &contribution;
                        }
                    }
                }

                solver.assert(&vars_var[v][v_orig_id]._eq(&num_v));
            }
        }

        // Al sustituir las variables nuevas se obtiene un resultado equivalente
        for e in 0..rows_expr {
            let expression_id = cluster[e];
            let expression_coeffs = &expression[expression_id].1;

            for c in 0..coeffs_and_vars.len() {
                // If the terms is not in the original expression, it cannot be used.
                if !vars_per_expression[e][c] {
                    solver.assert(&get_expr_uses(e, c).not());
                }
            }

            for v_orig_id in 0..coeffs_and_vars.len() {
                let original_signal = coeffs_and_vars[v_orig_id].0;

                let original_coeff = expression_coeffs.get(&original_signal).cloned().unwrap_or_else(|| BigInt::from(0));

                let original_coeff = Int::from_str(&original_coeff.to_string()).unwrap();

                let mut reconstructed_coeff = Int::from_i64(0);

                // Original terms that appear in the final expression
                for c in 0..coeffs_and_vars.len() {
                    if coeffs_and_vars[c].0 == original_signal {
                        let selected = get_expr_uses(e, c).clone();

                        let coeff = Int::from_str(&coeffs_and_vars[c].1.to_string(),).unwrap();

                        let contribution = selected.ite(&coeff, &zero);

                        reconstructed_coeff = &reconstructed_coeff + &contribution;
                    }
                }

                // The original variable is used in the new variables introduced
                for v in 0..rows_vars {
                    let selected = get_expr_uses(e, coeffs_and_vars.len() + v).clone();
                    let auxiliary_coeff = &vars_var[v][v_orig_id];

                    let contribution = selected.ite(auxiliary_coeff, &zero);

                    reconstructed_coeff = &reconstructed_coeff + &contribution;
                }

                solver.assert(&reconstructed_coeff._eq(&original_coeff));
            }
        }

        // Each transformed expression must use the required number of variables.
        // This can check if the expression in plonk???

        let mut vars_used_by_var = Vec::new();

        for v in 0..rows_vars {
            let mut aux = Int::from_i64(0);
            for prev_v in coeffs_and_vars.len()..(coeffs_and_vars.len() + v) {
                let selected = get_v_composition(v, prev_v);
                let contribution = selected.ite(&(&one + &vars_used_by_var[prev_v - coeffs_and_vars.len()]), &zero);

                aux = &aux + &contribution;
            }
            vars_used_by_var.push(aux);
        }

        for e in 0..rows_expr {
            let mut aux = Int::from_i64(0);
            for c in coeffs_and_vars.len()..cols_expr {
                let selected = get_expr_uses(e, c).clone();
                let contribution = selected.ite(&(&one + &vars_used_by_var[c - coeffs_and_vars.len()]), &zero);

                aux = &aux + &contribution;
            }

            let expression_id = cluster[e];
            let required_vars = Int::from_str(&expression[expression_id].2.to_string()).unwrap();
            solver.assert(&aux._eq(&required_vars));
        }

        // An expression just uses non empty variables
        for e in 0..rows_expr {
            for v in 0..rows_vars {
                let mut num_vars_compose = Int::from_i64(0);

                for c in 0..cols_vars {
                    let selected = get_v_composition(v, c);
                    let contribution = selected.ite(&one, &zero);
                    num_vars_compose = &num_vars_compose + &contribution;
                }

                let is_active = num_vars_compose._eq(&Int::from_i64(2));
                let expr_uses_aux = get_expr_uses(e, coeffs_and_vars.len() + v);

                solver.assert(&expr_uses_aux.implies(&is_active));
            }
        }

        // Check that each expression, after replacing the variables, is plonk
        // Order constraints

        // Minimize the number of variables introduced
        let mut active_vars = Int::from_i64(0);
        
        for v in 0..rows_vars {
            let active_positions: Vec<&Bool> = (0..cols_vars).map(|c| &vars_composition[v * cols_vars + c]).collect();
            let is_active = Bool::or(&active_positions);
            let contribution = is_active.ite(&one, &zero);
            active_vars = &active_vars + &contribution;
        }
        solver.minimize(&active_vars);

        match solver.check(&[]) {
            SatResult::Sat => {
                println!("\nVariable optimization: SAT");
                let model = solver.get_model().expect("SAT result must have a model");
                let is_selected = |variable: &Bool| {
                    model
                        .eval(variable, true)
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false)
                };

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

                println!("New constraints:");
                let mut introduced = 0;
                for v in 0..rows_vars {
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
                for v in 0..rows_vars {
                    let is_active = (0..cols_vars)
                        .any(|c| is_selected(&vars_composition[v * cols_vars + c]));
                    if is_active {
                        println!("  x{}", model_aux_signals[v].unwrap());
                    }
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

                    for c in 0..coeffs_and_vars.len() {
                        if is_selected(&expr_use_vars[e * cols_expr + c]) {
                            let (signal, coefficient) = &coeffs_and_vars[c];
                            terms.push(format!("{} * x{}", coefficient, signal));
                        }
                    }

                    for v in 0..rows_vars {
                        if is_selected(&expr_use_vars[e * cols_expr + coeffs_and_vars.len() + v]) {
                            terms.push(format!("x{}", model_aux_signals[v].unwrap()));
                        }
                    }

                    println!("  expression {}: {}", expression_id, terms.join(" + "));
                }
            }
            SatResult::Unsat => println!("\nVariable optimization: UNSAT"),
            SatResult::Unknown => println!("\nVariable optimization: UNKNOWN"),
        }
    })

    // Alguna optimizacion de que solo sean parejas de variables validas
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
        } else if remove_unused && !forbidden.contains(&signal) && !non_linear_map.contains_key(&signal){
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
        let mut signals: Vec<_> = C::take_cloned_signals_ordered(&constraint).iter().cloned().collect();
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

    let mut substitution_log =
        if smp.port_substitution { 
            Some(SubstitutionJSON::new(&smp.json_substitutions).unwrap()) 
        } else {
             None 
        };
    let apply_linear = !smp.flag_s;
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
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field){
                C::fix_constraint(constraint, &field);
            }
        }
        for constraint in &mut cons_equalities {
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field){
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
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field){
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
            if fast_encoded_constraint_substitution(constraint, &substitutions, &field){
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
        if no_rounds > 0{
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
            let signals =  C::take_cloned_signals(&constraint);
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
        }
        else{
            constraint_storage.add_constraint(constraint);
        }
    }
    for mut constraint in lconst {
        if remove_unused{
            C::fix_constraint(&mut constraint, &field);
            let signals =  C::take_cloned_signals(&constraint);
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
        }
        else{
            C::fix_constraint(&mut constraint, &field);
            constraint_storage.add_constraint(constraint);
        }
    }

    let erased = crate::non_linear_simplification::simplify(
        &mut constraint_storage,
        &forbidden,
        &field
    );

    for signal in erased {
        deleted.insert(signal);
    }

    let _trash = constraint_storage.extract_with(&|c| C::is_empty(c));

    if smp.plonk_simplification{
        // At this point call to normalize all constraints 
        // // if we are in PLONK mode

        let mut auxiliary_signals = HashMap::new();
        // let mut aux_compositions: HashMap<usize, Vec<(usize, BigInt)>> = HashMap::new();

        let mut expressions_to_fix = Vec::new();
        let mut estimated_auxiliary_variables = 0;
        for i in constraint_storage.get_ids() {
            let mut constraint = constraint_storage.read_constraint(i).unwrap();
            constraint.normalize_constraint(&field);

            if !constraint.is_plonk() {
                let (expressions, total_auxiliaries) = constraint.vars_to_solve_plonk_with_total();
                estimated_auxiliary_variables += total_auxiliaries;
                for (expr, num_vars) in expressions {
                    expressions_to_fix.push((i, expr.clone(), num_vars));
                }
            }

            constraint_storage.replace(i, constraint);
        }

        let clusters = build_expr_clusters(&expressions_to_fix, max_signal);
        for cluster in &clusters {
            // Greedy optimization retained for comparison:
            if smp.apply_eager_plonk{
                let pairs = count_variable_pairs(cluster, &expressions_to_fix);
                if let Some((pair, frequency)) = select_most_frequent_pair(cluster, &expressions_to_fix, pairs) {
                    println!("Chosen pair: {} * x{}, {} * x{} (appears {} times)", pair.2, pair.0, pair.3, pair.1, frequency);
                    apply_pair_to_cluster(&mut constraint_storage, cluster, &expressions_to_fix, pair, &mut auxiliary_signals, &mut max_signal, &mut non_linear_map, &field);
                }
            }
            if smp.apply_sat_plonk{

                variables_optimization(
                    &mut constraint_storage,
                    &expressions_to_fix,
                    cluster,
                    &mut auxiliary_signals,
                    &mut max_signal,
                    &mut non_linear_map,
                    &field,
                );
            }
        }

        println!("\n========================================");
        println!("       FINAL CONSTRAINTS");
        println!("========================================");
        println!("Estimated auxiliary-variable upper bound: {}", estimated_auxiliary_variables);
        println!("Auxiliary variables introduced by Z3: {}", auxiliary_signals.len());

        let mut final_plonk = 0;
        let mut final_non_plonk = 0;

        for constraint_id in constraint_storage.get_ids() {
            let constraint = constraint_storage
                .read_constraint(constraint_id)
                .unwrap();

            println!("\nConstraint {}:", constraint_id);
            fn print_expression(expr: &HashMap<usize, BigInt>) {
                print!("{{");

                for (signal, coefficient) in expr {
                    print!("{} * x{}, ", coefficient, signal);
                }

                println!("}}");
            }

            print!("  A = ");
            print_expression(&constraint.a());

            print!("  B = ");
            print_expression(&constraint.b());

            print!("  C = ");
            print_expression(&constraint.c());

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
            println!(
                "  Remaining variables after reconstruction: A={}, B={}, C={}, distinct in constraint={}",
                count_variables(constraint.a()),
                count_variables(constraint.b()),
                count_variables(constraint.c()),
                distinct_variables.len()
            );

            if constraint.is_plonk() {
                println!("  State: PLONK");
                final_plonk += 1;
            } else {
                println!("  State: NO PLONK");
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
        let signal_map= rebuild_witness(
            max_signal, 
            &mut deleted, 
            &forbidden, 
            non_linear_map, 
            remove_unused
        );
        let _dur = now.elapsed().unwrap().as_millis();
        // println!("End of rebuild witness: {} ms", dur);
       signal_map
    };

    // count the number of deleted inputs
    let max_value_input = smp.no_public_outputs + smp.no_public_inputs + smp.no_private_inputs;
    let mut deleted_inputs = 0;
    for signal in &deleted{
        if signal >= &(smp.no_public_outputs + 1) && signal <= &max_value_input{
            deleted_inputs += 1;
        }
    }


    if let Some(w) = substitution_log {
        w.end().unwrap();
    }
    // println!("NO CONSTANTS: {}", constraint_storage.no_constants());
    (constraint_storage, signal_map, smp.no_private_inputs - deleted_inputs)
}
