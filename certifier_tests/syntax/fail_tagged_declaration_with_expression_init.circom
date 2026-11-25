pragma circom 2.2.0;

// (Syntax + H2) FAIL: `signal {binary} t <== a * a;` claims the tag in the
// declaration from an expression.
// @expect FAIL
// @certified 0
// @fail T() CE02

template T() {
    signal input a;
    signal output o;
    signal {binary} t <== a * a;
    o <== t;
}

component main = T();
