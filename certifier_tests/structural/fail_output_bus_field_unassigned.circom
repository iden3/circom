pragma circom 2.2.0;

// (Structural) FAIL: an output bus with one field never assigned: CA01 on
// `p.y`, promoted because `p` is an output.
// @expect FAIL
// @certified 0
// @fail T() CA01

bus Pair() {
    signal x;
    signal y;
}

template T() {
    signal input a;
    output Pair() p;
    p.x <== a;
}

component main = T();
