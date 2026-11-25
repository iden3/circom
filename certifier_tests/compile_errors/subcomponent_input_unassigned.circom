pragma circom 2.2.0;

// COMPILE_ERROR: reading an output of a component with an unassigned input
// is rejected by circom itself (T3001).
// @expect COMPILE_ERROR
// @error T3001

template M() {
    signal input a;
    signal input b;
    signal output o;
    o <== a * b;
}

template T() {
    signal input a;
    signal output o;
    component m = M();
    m.a <== a;
    o <== m.o;
}

component main = T();
