pragma circom 2.2.0;

// (Syntax + H1) FAIL: a component array whose elements get different
// parameters; only the Pick(1) instance takes the `<--` branch.
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
    signal output o[2];
    component c[2];
    for (var i = 0; i < 2; i++) {
        c[i] = Pick(i);
        c[i].a <== a;
        o[i] <== c[i].out;
    }
}

component main = Top();
