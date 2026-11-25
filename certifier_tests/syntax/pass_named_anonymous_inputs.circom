pragma circom 2.2.0;

// (Syntax) PASS: anonymous component with named inputs, in a different
// order than declared.
// @expect PASS
// @certified 2

template Mul() {
    signal input a;
    signal input b;
    signal output out;
    out <== a * b;
}

template Top() {
    signal input x;
    signal input y;
    signal output o;
    o <== Mul()(b <== y, a <== x);
}

component main = Top();
