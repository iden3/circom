use super::DAG;
use circom_algebra::num_bigint::BigInt;
use program_structure::constants::UsefulConstants;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Number of constraints, split by shape. `linear` is the number of constraints
/// whose A and B parts are empty, and `signal_equalities`/`constant_equalities`
/// are the two degenerate subcases of a linear constraint (s1 === s2 and
/// s === k), so they are already counted inside `linear`.
#[derive(Default, Clone, Copy, Deserialize, Serialize, Debug)]
pub struct ConstraintStats {
    pub total: usize,
    pub non_linear: usize,
    pub linear: usize,
    pub signal_equalities: usize,
    pub constant_equalities: usize,
}

impl ConstraintStats {
    fn add(&mut self, other: &ConstraintStats) {
        self.total = self.total.saturating_add(other.total);
        self.non_linear = self.non_linear.saturating_add(other.non_linear);
        self.linear = self.linear.saturating_add(other.linear);
        self.signal_equalities = self.signal_equalities.saturating_add(other.signal_equalities);
        self.constant_equalities =
            self.constant_equalities.saturating_add(other.constant_equalities);
    }

    fn scale(&self, times: usize) -> ConstraintStats {
        ConstraintStats {
            total: self.total.saturating_mul(times),
            non_linear: self.non_linear.saturating_mul(times),
            linear: self.linear.saturating_mul(times),
            signal_equalities: self.signal_equalities.saturating_mul(times),
            constant_equalities: self.constant_equalities.saturating_mul(times),
        }
    }
}

/// Number of signals, split by kind. `public_inputs` is a subset of `inputs`.
#[derive(Default, Clone, Copy, Deserialize, Serialize, Debug)]
pub struct SignalStats {
    pub total: usize,
    pub inputs: usize,
    pub public_inputs: usize,
    pub outputs: usize,
    pub intermediates: usize,
}

impl SignalStats {
    fn add(&mut self, other: &SignalStats) {
        self.total = self.total.saturating_add(other.total);
        self.inputs = self.inputs.saturating_add(other.inputs);
        self.public_inputs = self.public_inputs.saturating_add(other.public_inputs);
        self.outputs = self.outputs.saturating_add(other.outputs);
        self.intermediates = self.intermediates.saturating_add(other.intermediates);
    }

    fn scale(&self, times: usize) -> SignalStats {
        SignalStats {
            total: self.total.saturating_mul(times),
            inputs: self.inputs.saturating_mul(times),
            public_inputs: self.public_inputs.saturating_mul(times),
            outputs: self.outputs.saturating_mul(times),
            intermediates: self.intermediates.saturating_mul(times),
        }
    }
}

/// What a piece of the circuit costs: its constraints, its signals and how many
/// components it contains.
#[derive(Default, Clone, Copy, Deserialize, Serialize, Debug)]
pub struct Cost {
    pub constraints: ConstraintStats,
    pub signals: SignalStats,
    pub components: usize,
}

impl Cost {
    fn add(&mut self, other: &Cost) {
        self.constraints.add(&other.constraints);
        self.signals.add(&other.signals);
        self.components = self.components.saturating_add(other.components);
    }

    fn scale(&self, times: usize) -> Cost {
        Cost {
            constraints: self.constraints.scale(times),
            signals: self.signals.scale(times),
            components: self.components.saturating_mul(times),
        }
    }
}

/// Stats of a single template instance, that is, of one concrete instantiation
/// of a template such as `Num2Bits(254)`. There is exactly one of these per
/// node of the DAG.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InstanceInfo {
    pub node_id: usize,
    /// Instantiation name, parameters included: `Num2Bits(254)`.
    pub name: String,
    /// Name of the template alone: `Num2Bits`.
    pub template: String,
    pub parameters: Vec<String>,
    pub is_custom_gate: bool,
    pub is_parallel: bool,
    pub is_deterministic: bool,
    /// Times this instance appears as a component in the expanded circuit.
    pub components: usize,
    /// Cost of one component, counting only what the template body declares.
    pub own: Cost,
    /// Cost of one component, subcomponents included.
    pub tree: Cost,
    /// `own` times `components`: what this instance adds to the circuit.
    pub total_own: Cost,
    /// `tree` times `components`. Subtrees shared with other templates are
    /// counted once per component, so this can exceed the circuit total.
    pub total_tree: Cost,
}

/// Stats of a template, aggregating all of its instances (`Num2Bits(254)` and
/// `Num2Bits(8)` are two instances of the template `Num2Bits`).
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TemplateInfo {
    pub template: String,
    /// Distinct instantiations of the template, one per parameter combination.
    pub instances: usize,
    /// Times the template appears as a component in the expanded circuit.
    pub components: usize,
    pub total_own: Cost,
    pub total_tree: Cost,
    pub instance_names: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CircuitTemplateInfo {
    pub main: String,
    pub prime: String,
    pub note: String,
    /// Totals of the whole circuit, equal to the sum of the `total_own` of every
    /// template. It is the `tree` cost of the main component, except that
    /// `components` counts the main component too.
    pub circuit: Cost,
    pub number_of_instances: usize,
    pub number_of_components: usize,
    /// One entry per template instance, that is, per distinct combination of
    /// arguments. Sorted by the constraints they add to the circuit.
    pub instances: Vec<InstanceInfo>,
    /// The same information rolled up by template name.
    pub templates: Vec<TemplateInfo>,
}

fn template_of(name: &str) -> String {
    match name.find('(') {
        Some(pos) => name[..pos].to_string(),
        None => name.to_string(),
    }
}

/// Nodes reachable from main, parents always before their children.
fn topological_order(dag: &DAG) -> Vec<usize> {
    let main = dag.main_id();
    let mut visited = vec![false; dag.nodes.len()];
    let mut order = Vec::new();
    let mut stack = vec![(main, 0)];
    visited[main] = true;
    while let Some((node, edge_index)) = stack.pop() {
        if edge_index < dag.adjacency[node].len() {
            stack.push((node, edge_index + 1));
            let next = dag.adjacency[node][edge_index].goes_to;
            if !visited[next] {
                visited[next] = true;
                stack.push((next, 0));
            }
        } else {
            order.push(node);
        }
    }
    order.reverse();
    order
}

/// Cost of a node counting only the constraints and signals declared in its own
/// template body.
fn own_cost(dag: &DAG, node_id: usize, field: &BigInt) -> Cost {
    let node = &dag.nodes[node_id];

    let mut constraints = ConstraintStats::default();
    for constraint in &node.constraints {
        if constraint.is_empty() {
            continue;
        }
        constraints.total += 1;
        if constraint.is_constant_equality() {
            constraints.linear += 1;
            constraints.constant_equalities += 1;
        } else if constraint.is_equality(field) {
            constraints.linear += 1;
            constraints.signal_equalities += 1;
        } else if super::Constraint::is_linear(constraint) {
            constraints.linear += 1;
        } else {
            constraints.non_linear += 1;
        }
    }

    // `number_of_signals` accumulates the signals of the subcomponents as well,
    // the ones declared by the template itself are exactly its local signals.
    let signals = SignalStats {
        total: node.locals.len(),
        inputs: node.inputs_length,
        public_inputs: node.public_inputs_length,
        outputs: node.outputs_length,
        intermediates: node.intermediates_length,
    };

    Cost { constraints, signals, components: dag.adjacency[node_id].len() }
}

pub fn map_to_template_info(dag: &DAG) -> CircuitTemplateInfo {
    let order = topological_order(dag);
    let main = dag.main_id();

    // How many components of each instance the expanded circuit contains.
    let mut components = vec![0usize; dag.nodes.len()];
    components[main] = 1;
    for node_id in &order {
        let reached = components[*node_id];
        for edge in &dag.adjacency[*node_id] {
            components[edge.goes_to] = components[edge.goes_to].saturating_add(reached);
        }
    }

    // Cost of one component, first without and then with its subcomponents.
    let constants = UsefulConstants::new(&dag.prime);
    let field = constants.get_p().clone();
    let own: Vec<Cost> =
        (0..dag.nodes.len()).map(|id| own_cost(dag, id, &field)).collect();
    let mut tree = vec![Cost::default(); dag.nodes.len()];
    for node_id in order.iter().rev() {
        let mut cost = Cost { components: 0, ..own[*node_id] };
        for edge in &dag.adjacency[*node_id] {
            let subtree = tree[edge.goes_to];
            cost.add(&subtree);
            cost.components = cost.components.saturating_add(1);
        }
        tree[*node_id] = cost;
    }

    let mut instances = Vec::new();
    for node_id in &order {
        let node = &dag.nodes[*node_id];
        let times = components[*node_id];
        instances.push(InstanceInfo {
            node_id: *node_id,
            name: node.template_name.clone(),
            template: template_of(&node.template_name),
            parameters: node.parameters.iter().map(|p| p.to_string()).collect(),
            is_custom_gate: node.is_custom_gate,
            is_parallel: node.is_parallel,
            is_deterministic: node.is_deterministic,
            components: times,
            own: own[*node_id],
            tree: tree[*node_id],
            total_own: own[*node_id].scale(times),
            total_tree: tree[*node_id].scale(times),
        });
    }
    instances.sort_by(|a, b| {
        b.total_own
            .constraints
            .total
            .cmp(&a.total_own.constraints.total)
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut by_template: HashMap<String, TemplateInfo> = HashMap::new();
    for instance in &instances {
        let entry = by_template.entry(instance.template.clone()).or_insert(TemplateInfo {
            template: instance.template.clone(),
            instances: 0,
            components: 0,
            total_own: Cost::default(),
            total_tree: Cost::default(),
            instance_names: Vec::new(),
        });
        entry.instances += 1;
        entry.components = entry.components.saturating_add(instance.components);
        entry.total_own.add(&instance.total_own);
        entry.total_tree.add(&instance.total_tree);
        entry.instance_names.push(instance.name.clone());
    }
    let mut templates: Vec<TemplateInfo> = by_template.into_iter().map(|(_, t)| t).collect();
    for template in &mut templates {
        template.instance_names.sort();
    }
    templates.sort_by(|a, b| {
        b.total_own
            .constraints
            .total
            .cmp(&a.total_own.constraints.total)
            .then_with(|| a.template.cmp(&b.template))
    });

    // The main component is not inside any subtree, so it is added apart.
    let number_of_components = tree[main].components.saturating_add(1);
    let circuit = Cost { components: number_of_components, ..tree[main] };
    CircuitTemplateInfo {
        main: dag.nodes[main].template_name.clone(),
        prime: dag.prime.clone(),
        note: "Constraints and signals are counted before any simplification round. \
               A template instance is one instantiation of a template with concrete \
               parameters; a component is one occurrence of an instance in the \
               expanded circuit. The 'own' figures cover only what the template body \
               declares, the 'tree' figures include its subcomponents, and the \
               'total_' ones multiply them by the number of components."
            .to_string(),
        circuit,
        number_of_instances: order.len(),
        number_of_components,
        instances,
        templates,
    }
}
