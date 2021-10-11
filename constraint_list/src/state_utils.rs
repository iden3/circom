use super::*;

pub fn empty_encoding_constraints(encoding: &mut DAGEncoding) {
    for node in &mut encoding.nodes {
        node.non_linear.clear();
    }
}

pub fn clear_encoding_iterator(iterator: &mut EncodingIterator) {
    iterator.signals = Vec::with_capacity(0);
    iterator.non_linear.clear();
}

pub fn build_encoding_iterator(mut iterator: EncodingIterator) -> EncodingIterator {
    let encoding = iterator.encoding;
    let offset = iterator.offset;
    let node_id = iterator.node_id;
    let path = iterator.path;
    let mut non_linear = LinkedList::new();
    let mut signals = Vec::new();
    for signal in &encoding.nodes[node_id].signals {
        let new_signal =
            SignalInfo { id: signal.id + offset, name: format!("{}.{}", path, signal.name) };
        Vec::push(&mut signals, new_signal);
    }

    for constraint in &encoding.nodes[node_id].non_linear {
        let constraint = C::apply_offset(constraint, offset);
        LinkedList::push_back(&mut non_linear, constraint);
    }
    iterator.path = path;
    iterator.non_linear = non_linear;
    iterator.signals = signals;
    iterator
}
