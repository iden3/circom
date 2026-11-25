pragma circom 2.2.0;

// (H2) FAIL: a constant is not a tagged source either.
// @expect FAIL
// @certified 0
// @fail ConstTag() CE02

template ConstTag() {
    signal input a;
    signal output {binary} out;
    signal output copy;
    out <== 1;
    copy <== a;
}

component main = ConstTag();
