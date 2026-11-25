pragma circom 2.2.0;

// (Syntax) PASS: a subcomponent with no outputs (constraints only), plus
// `log` and `assert`, which add nothing to the constraint system.
// @expect PASS
// @certified 2

template CheckInv() {
    signal input a;
    signal input b;
    a * b === 1;
}

template Top() {
    signal input a;
    signal input b;
    signal output o;
    component c = CheckInv();
    c.a <== a;
    c.b <== b;
    log("a =", a);
    assert(1 == 1);
    o <== a + b;
}

component main = Top();
