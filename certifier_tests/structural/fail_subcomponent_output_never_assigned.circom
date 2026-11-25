pragma circom 2.2.0;

// (Structural) FAIL: Sub leaves one of its outputs unassigned; Top never
// reads it. The failure belongs to Sub (CA01 on its own output); Top's
// CA02 (an unused subcomponent output) is not a failure by itself.
// @expect FAIL
// @certified 1
// @fail Sub() CA01

template Sub() {
    signal input a;
    signal output used;
    signal output forgotten;
    used <== a + 1;
}

template Top() {
    signal input a;
    signal output out;
    component s = Sub();
    s.a <== a;
    out <== s.used;
}

component main = Top();
