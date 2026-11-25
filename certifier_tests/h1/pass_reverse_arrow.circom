pragma circom 2.2.0;

// (H1) PASS: `==>` is the constrained operator written right-to-left.
// @expect PASS
// @certified 1

template RevMul() {
    signal input a;
    signal input b;
    signal output out;
    a * b ==> out;
}

component main = RevMul();
