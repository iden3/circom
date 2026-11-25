pragma circom 2.0.0;

include "comparators.circom"; // from the real circomlib2 (circomlib_buses) -- see README

// PASSES ON PURPOSE: `arr[1].x` is NEVER touched anywhere in this file -- no
// `<==`, no `<--`, nothing. It is a completely free, unconstrained field.
// `arr[0].x` and `arr[1].x` collapse into the same tracked path `"arr.x"` in
// H2's own bookkeeping, so H2 alone cannot see that `arr[1].x` was never
// touched at all (only that whatever WAS touched was honest).
//
// The DAG-level "unconstrained signal" analysis that `--inspect` uses DOES
// see it (`[CA01] ... Local signal arr[1].x does not appear in any
// constraint`), and `CertificationResult::absorb_structural_warnings` re-runs
// it under -cbc -- but only promotes hits on OUTPUT signals. `arr` is an
// intermediate signal of `ArrOfBuses`: nothing reads `arr[1].x`, so it cannot
// make any output non-deterministic, and is deliberately allowed. Compare
// known_limitation_untouched_output_across_component.circom, where the same
// untouched field is part of an OUTPUT and is rejected.
//
// Run:
//   target/debug/circom certifier_examples/arrays/known_limitation_untouched_element.circom \
//     -cbc -l <path-to-circomlib>/circuits -o /tmp
//
// Expected: PASS -- `ArrOfBuses()` certified, `IsZero()` trusted. With
// `--inspect` the CA01 warning on `arr[1].x` is still printed.
bus Inner {
    signal {binary} x;
}

template ArrOfBuses() {
    signal input a;
    component z = IsZero();
    z.in <== a;

    Inner() arr[2];
    arr[0].x <== z.out;
    // arr[1].x intentionally left untouched.

    signal output out;
    out <== arr[0].x;
}

component main = ArrOfBuses();
