pragma circom 2.2.0;

// (Syntax) PASS: a `template parallel` definition and a main component with
// public inputs.
// @expect PASS
// @certified 2

template parallel Sq() {
    signal input a;
    signal output o;
    o <== a * a;
}

template Top() {
    signal input a;
    signal input b;
    signal output o;
    signal s <== Sq()(a);
    o <== s + b;
}

component main {public [a]} = Top();
