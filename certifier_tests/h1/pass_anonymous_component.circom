pragma circom 2.2.0;

// (H1) PASS: anonymous component syntax; both instances certify.
// @expect PASS
// @certified 2

template Mul() {
    signal input a;
    signal input b;
    signal output out;
    out <== a * b;
}

template Square() {
    signal input x;
    signal output out;
    out <== Mul()(x, x);
}

component main = Square();
