pragma circom 2.2.0;

// (Flags) FAIL with --O0: the structural CA01 promotion does not depend on
// the simplification level (see structural/fail_output_never_assigned).
// @expect FAIL
// @certified 0
// @fail Half() CA01
// @args --O0

template Half() {
    signal input a;
    signal output out1;
    signal output out2;
    out1 <== a * a;
}

component main = Half();
