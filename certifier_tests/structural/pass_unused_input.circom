pragma circom 2.2.0;

// (Structural) PASS: an input that appears in no constraint. Inputs are
// not promoted either: every output is still uniquely determined.
// @expect PASS
// @certified 1

template IgnoresB() {
    signal input a;
    signal input b;
    signal output out;
    out <== a * a;
}

component main = IgnoresB();
