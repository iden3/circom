pragma circom 2.0.0;


// FIXED (was a known limitation) -- scalar variant of
// known_limitation_untouched_output_across_component.circom.
//
// Despite the file name, `arr` is here a plain OUTPUT signal of `ArrOfBuses`
// (no array, no bus; `Inner` is unused), NEVER touched anywhere. Reading an
// uninitialized signal from WITHIN the template that declares it is caught
// by circom itself, but `Consumer` reads it from OUTSIDE, across the
// component boundary (`c.arr`), where circom's own initialization check does
// not fire. The DAG-level "unconstrained signal" analysis does see it, and
// since `arr` is a declared output, `absorb_structural_warnings` promotes its
// CA01 into a certification failure.
//
// Run:
//   target/debug/circom certifier_examples/arrays/known_limitation_untouched_output_across_component_array.circom \
//     -cbc -o /tmp
//
// Expected: FAIL.
//   - `TagA()`: CE01 (`<--`) and CE02 (bare `tagA` tag) -- hand-written,
//     unrelated to this file's point.
//   - `ArrOfBuses()`: `[CA01] ... Local signal arr does not appear in any
//     constraint`.
//   - `Consumer()`: CE02 -- its output `out` declares `{tagA}` but is copied
//     from `c.arr`, which carries no tag.
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

    output signal arr;
    //arr;
    // arr[1] intentionally left untouched.
}

template Consumer() {
    signal input a;
    component c = ArrOfBuses();
    c.a <== a;
    signal output {tagA} out;
    out <== c.arr; // reads the never-touched element through the component
}

component main = Consumer();
