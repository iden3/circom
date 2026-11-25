pragma circom 2.2.0;

// (H1) FAIL: `-->` is the unconstrained operator written right-to-left.
// @expect FAIL
// @certified 0
// @fail RevFree() CE01

template RevFree() {
    signal input a;
    signal output out;
    a * 2 --> out;
    out === a * 2;
}

component main = RevFree();
