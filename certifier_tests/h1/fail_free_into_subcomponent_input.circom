pragma circom 2.2.0;

// (H1) FAIL: feeding a subcomponent input with `<--`. The violation belongs
// to the template that writes it (Top), not to the callee (Mul certifies).
// @expect FAIL
// @certified 1
// @fail Top() CE01

template Mul() {
    signal input a;
    signal input b;
    signal output out;
    out <== a * b;
}

template Top() {
    signal input x;
    signal output out;
    component m = Mul();
    m.a <-- x;
    m.b <== x;
    out <== m.out;
}

component main = Top();
