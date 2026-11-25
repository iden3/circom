pragma circom 2.2.0;

// (H1) FAIL: the classic inverse witness on an INTERMEDIATE signal. Even
// though `inv * a === 1` constrains it, it is fixed with `<--`.
// @expect FAIL
// @certified 0
// @fail Inverse() CE01

template Inverse() {
    signal input a;
    signal output out;
    signal inv;
    inv <-- 1 / a;
    inv * a === 1;
    out <== inv;
}

component main = Inverse();
