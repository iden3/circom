pragma circom 2.2.0;

// (H1) FAIL, per instance: the same template instantiated with two
// parameters. Only Pick(1) takes the `<--` branch; Pick(0) and Top certify.
// @expect FAIL
// @certified 2
// @fail Pick(1) CE01

template Pick(flag) {
    signal input a;
    signal output out;
    if (flag == 1) {
        out <-- a;
        out === a;
    } else {
        out <== a;
    }
}

template Top() {
    signal input a;
    signal output o0;
    signal output o1;
    o0 <== Pick(0)(a);
    o1 <== Pick(1)(a);
}

component main = Top();
