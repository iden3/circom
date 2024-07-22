use super::{ConstraintList, EncodingIterator, IteratorSignal, SignalMap};
use circom_algebra::num_traits::AsPrimitive;
use constraint_writers::sym_writer::*;
use virtual_fs::{FileSystem, FsResult};

pub fn port_sym(fs: &mut dyn FileSystem, list: &ConstraintList, file_name: &str) -> FsResult<()> {
    let iter = EncodingIterator::new(&list.dag_encoding);
    let mut dot_sym = SymFile::new();
    signal_iteration(iter, &list.signal_map, &mut dot_sym);
    fs.write(&file_name.into(), &dot_sym.data)?;
    Ok(())
}

pub fn signal_iteration(
    mut iter: EncodingIterator,
    map: &SignalMap,
    dot_sym: &mut SymFile,
) {
    let (signals, _) = EncodingIterator::take(&mut iter);

    for signal in signals {
        let signal = IteratorSignal::new(signal, map);
        let sym_elem = SymElem {
            original: signal.original.as_(),
            witness: if signal.witness == map.len() { -1 } else { signal.witness.as_() },
            node_id: iter.node_id.as_(),
            symbol: signal.name.clone(),
        };
        SymFile::write_sym_elem(dot_sym, sym_elem);
    }

    for edge in EncodingIterator::edges(&iter) {
        let next = EncodingIterator::next(&iter, edge);
        signal_iteration(next, map, dot_sym);
    }
}
