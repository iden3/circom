pragma circom 2.2.0;

// (Syntax + H1) FAIL: `signal t <-- expr;` in the declaration itself.
// @expect FAIL
// @certified 0
// @fail T() CE01

template T() {
    signal input a;
    signal input b;
    signal output o;
    signal t <-- a * b;
    t === a * b;
    o <== t;
}

component main = T();
