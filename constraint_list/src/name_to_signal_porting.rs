use super::{ConstraintList, EncodingIterator, IteratorSignal, SignalMap};
use circom_algebra::num_traits::AsPrimitive;
use constraint_writers::{name_to_signal_writer::Name2SignalFile, name_to_signal_writer::Name2Signal};

pub fn port_name_to_signal(list: &ConstraintList, file_name: &str) -> Result<(), ()> {
    let iter = EncodingIterator::new(&list.dag_encoding);
    let mut dot_sym = Name2SignalFile::new(file_name)?;
    signal_iteration(iter, &list.signal_map, &mut dot_sym)?;
    Name2SignalFile::finish_writing(dot_sym)?;
    //SymFile::close(dot_sym);
    Ok(())
}

pub fn signal_iteration(
    mut iter: EncodingIterator,
    map: &SignalMap,
    dot_sym: &mut Name2SignalFile,
) -> Result<(), ()> {
    let (signals, _) = EncodingIterator::take(&mut iter);

    for signal in signals {
        let signal = IteratorSignal::new(signal, map);
        let sym_elem = Name2Signal {
            witness: if signal.witness == map.len() { -1 } else { signal.witness.as_() },
            symbol: signal.name.clone(),
        };
        Name2SignalFile::write_sym_elem(dot_sym, sym_elem)?;
    }

    for edge in EncodingIterator::edges(&iter) {
        let next = EncodingIterator::next(&iter, edge);
        signal_iteration(next, map, dot_sym)?;
    }
    Ok(())
}
