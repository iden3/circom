pragma circom 2.2.0;

// (Structural) FAIL: `out2` is an output that is never assigned. CA01 on
// an output is promoted to a certification failure.
// @expect FAIL
// @certified 0
// @fail Half() CA01

template Half() {
    signal input a;
    signal output out1;
    signal output out2;
    out1 <== a * a;
}

component main = Half();
