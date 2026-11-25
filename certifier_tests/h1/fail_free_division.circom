pragma circom 2.2.0;

// (H1) FAIL: `out <-- a / b` adds no constraint. CE01 for the `<--`, and
// CA01 because `out` then appears in no constraint at all (promoted to a
// failure because `out` is an output).
// @expect FAIL
// @certified 0
// @fail FreeDiv() CE01 CA01

template FreeDiv() {
    signal input a;
    signal input b;
    signal output out;
    out <-- a / b;
}

component main = FreeDiv();
