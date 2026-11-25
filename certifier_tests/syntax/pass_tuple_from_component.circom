pragma circom 2.2.0;

// (Syntax) PASS: tuple destructuring of a component with two outputs.
// @expect PASS
// @certified 2

template DivMod2() {
    signal input a;
    signal input b;
    signal output s;
    signal output p;
    s <== a + b;
    p <== a * b;
}

template Top() {
    signal input a;
    signal input b;
    signal output s;
    signal output p;
    (s, p) <== DivMod2()(a, b);
}

component main = Top();
