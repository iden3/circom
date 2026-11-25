pragma circom 2.2.0;

// (H2) FAIL: a {binary} output computed from an arithmetic expression over
// an untagged input. Circom accepts it (nobody downstream consumes the
// tag); -cbc reports CE02.
// @expect FAIL
// @certified 0
// @fail BareTag() CE02

template BareTag() {
    signal input a;
    signal output {binary} out;
    out <== a * 1;
}

component main = BareTag();
