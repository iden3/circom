use std::collections::{HashMap, HashSet, LinkedList};

use circom_algebra::constraint_storage::ConstraintStorage;
use circom_algebra::num_bigint::BigInt;
use constraint_writers::debug_writer::DebugWriter;
use constraint_writers::ConstraintExporter;

mod constraint_simplification;
mod json_porting;
mod non_linear_utils;
mod r1cs_porting;
mod state_utils;
mod sym_porting;
mod non_linear_simplification;

type C = circom_algebra::algebra::Constraint<usize>;
type S = circom_algebra::algebra::Substitution<usize>;
type A = circom_algebra::algebra::ArithmeticExpression<usize>;
type SignalMap = HashMap<usize, usize>;
type SEncoded = HashMap<usize, A>;
type SFrames = LinkedList<SEncoded>;

pub struct SignalInfo {
    pub name: String,
    pub id: usize,
}
pub struct EncodingNode {
    pub id: usize,
    pub name: String,
    pub parameters: Vec<BigInt>,
    pub signals: Vec<SignalInfo>,
    pub ordered_signals: Vec<usize>,
    pub non_linear: LinkedList<C>,
    pub is_custom_gate: bool,
}

pub struct EncodingEdge {
    pub goes_to: usize,
    pub path: String,
    pub offset: usize,
}

pub struct DAGEncoding {
    pub init: usize,
    pub no_constraints: usize,
    pub nodes: Vec<EncodingNode>,
    pub adjacency: Vec<Vec<EncodingEdge>>,
}

pub struct IteratorSignal {
    pub name: String,
    pub original: usize,
    pub witness: usize,
}

impl IteratorSignal {
    pub fn new(signal: SignalInfo, map: &SignalMap) -> IteratorSignal {
        let original = signal.id;
        let name = signal.name;
        let witness = HashMap::get(map, &original).map_or(map.len(), |s| *s);
        IteratorSignal { original, name, witness }
    }
}

pub struct EncodingIterator<'a> {
    encoding: &'a DAGEncoding,
    pub node_id: usize,
    pub path: String,
    pub offset: usize,
    pub signals: Vec<SignalInfo>,
    pub non_linear: LinkedList<C>,
}

impl<'a> EncodingIterator<'a> {
    pub fn new(encoding: &'a DAGEncoding) -> EncodingIterator<'a> {
        let iter = EncodingIterator {
            encoding,
            offset: 0,
            non_linear: LinkedList::new(),
            path: "main".to_string(),
            signals: Vec::new(),
            node_id: encoding.init,
        };
        state_utils::build_encoding_iterator(iter)
    }

    pub fn next(iterator: &'a EncodingIterator, edge: &EncodingEdge) -> EncodingIterator<'a> {
        let iter = EncodingIterator {
            encoding: iterator.encoding,
            offset: iterator.offset + edge.offset,
            node_id: edge.goes_to,
            path: format!("{}.{}", iterator.path, edge.path),
            non_linear: LinkedList::new(),
            signals: Vec::new(),
        };
        state_utils::build_encoding_iterator(iter)
    }

    pub fn edges(iterator: &'a EncodingIterator) -> &'a Vec<EncodingEdge> {
        &iterator.encoding.adjacency[iterator.node_id]
    }

    pub fn take(iter: &mut EncodingIterator) -> (Vec<SignalInfo>, LinkedList<C>) {
        let ret = (std::mem::take(&mut iter.signals), std::mem::take(&mut iter.non_linear));
        state_utils::clear_encoding_iterator(iter);
        ret
    }
}

pub struct Simplifier {
    pub field: BigInt,
    pub dag_encoding: DAGEncoding,
    pub no_public_inputs: usize,
    pub no_public_outputs: usize,
    pub no_private_inputs: usize,
    pub forbidden: HashSet<usize>,
    pub cons_equalities: LinkedList<C>,
    pub equalities: LinkedList<C>,
    pub linear: LinkedList<C>,
    //  Signals in [witness_len, Vec::len(&signal_map)) are the ones deleted
    pub max_signal: usize,
    // Flags
    pub no_rounds: usize,
    pub parallel_flag: bool,
    pub flag_s: bool,
    pub flag_old_heuristics: bool,
    pub port_substitution: bool,
}
impl Simplifier {
    pub fn simplify_constraints(mut self) -> ConstraintList {
        let (portable, map) = constraint_simplification::simplification(&mut self);
        ConstraintList {
            field: self.field,
            dag_encoding: self.dag_encoding,
            no_public_outputs: self.no_public_outputs,
            no_public_inputs: self.no_public_inputs,
            no_private_inputs: self.no_private_inputs,
            no_labels: self.max_signal,
            constraints: portable,
            signal_map: map,
        }
    }

    pub fn no_labels(&self) -> usize {
        self.max_signal
    }

    pub fn no_wires(&self) -> usize {
        self.max_signal
    }
}

pub struct ConstraintList {
    pub field: BigInt,
    pub dag_encoding: DAGEncoding,
    pub no_public_inputs: usize,
    pub no_public_outputs: usize,
    pub no_private_inputs: usize,
    pub constraints: ConstraintStorage,
    pub no_labels: usize,
    //  Signals in [witness_len, Vec::len(&signal_map)) are the ones deleted
    pub signal_map: SignalMap,
}

impl ConstraintExporter for ConstraintList {
    fn r1cs(&self, out: &str, custom_gates: bool) -> Result<(), ()> {
        r1cs_porting::port_r1cs(self, out, custom_gates)
    }

    fn json_constraints(&self, writer: &DebugWriter) -> Result<(), ()> {
        json_porting::port_constraints(&self.constraints, &self.signal_map, writer)
    }

    fn sym(&self, out: &str) -> Result<(), ()> {
        sym_porting::port_sym(self, out)
    }
}

impl ConstraintList {
    pub fn get_witness(&self) -> &SignalMap {
        &self.signal_map
    }

    pub fn get_witness_as_vec(&self) -> Vec<usize> {
        let mut witness = vec![0; self.no_wires()];
        for (key, value) in &self.signal_map {
            witness[*value] = *key;
        }
        witness
    }

    pub fn no_labels(&self) -> usize {
        self.no_labels
    }

    pub fn no_wires(&self) -> usize {
        self.signal_map.len()
    }
}
