pragma circom 2.2.0;

// (H2) FAIL: setting the tag VALUE by hand (`out.maxbit = 8`) and then
// copying an untagged input. Nothing proves `a` fits in 8 bits.
// @expect FAIL
// @certified 0
// @fail ClaimMaxbit() CE02

template ClaimMaxbit() {
    signal input a;
    signal output {maxbit} out;
    out.maxbit = 8;
    out <== a;
}

component main = ClaimMaxbit();
