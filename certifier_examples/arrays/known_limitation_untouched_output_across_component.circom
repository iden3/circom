pragma circom 2.0.0;


// FIXED (was a more serious variant of known_limitation_untouched_element.circom).
//
// `arr[1].x` is declared as part of an OUTPUT bus array of `ArrOfBuses` and
// is NEVER touched anywhere. Reading it from WITHIN the same template that
// declares it (see the other file) is caught by circom itself as a hard
// error ("trying to access to a signal that is not initialized"), before
// -cbc even runs. But here `Consumer` reads it from OUTSIDE, across
// the component boundary (`c.arr[1].x`) -- that goes through a different
// code path (reading the subcomponent's already-"exported" declared output
// storage, not the mid-execution environment) and circom's own
// initialization check does NOT fire there. H2 alone cannot catch this
// either, for the same collapsed-path reason as the other file. This is now
// caught by `CertificationResult::absorb_structural_warnings`, which re-runs
// the same DAG-level "unconstrained signal" analysis `--inspect` uses
// (regardless of whether `--inspect` was passed) after `export()` builds the
// full constraint graph, where every concrete wire (including
// `c.arr[...].x`) is visible regardless of which side of a component
// boundary declared it.
//
// Run:
//   target/debug/circom certifier_examples/arrays/known_limitation_untouched_output_across_component.circom \
//     -cbc -l certifier_examples/arrays -o /tmp
//
// Expected: `TagA()` fails on its own (hand-written, uses `<--` -- unrelated
// to this file's point). `ArrOfBuses()` fails with `[CA01] ... Local signal
// arr[1].x does not appear in any constraint` (`arr` is one of its declared
// outputs). `Consumer()` is NOT reported: the DAG also emits a `CA02` for it
// (`c.arr[0].x` -- simplification may alias the wire name), but a father not
// using some subcomponent output is never a certification failure by itself;
// the gap is owned, and rejected, at `ArrOfBuses()`.
bus Inner {
    signal {tagA} x;
}

template TagA() {
    signal input a;
    signal output {tagA} out;
    out <-- a;
    out === a;
}


template ArrOfBuses() {
    signal input a;
    component tg = TagA();
    tg.a <== a;

    output Inner() arr[2];
    arr[0].x <== tg.out;
    // arr[1].x intentionally left untouched.
}

template Consumer() {
    signal input a;
    component c = ArrOfBuses();
    c.a <== a;
    signal output {tagA} out;
    out <== c.arr[1].x; // reads the never-touched element through the component
}

component main = Consumer();
