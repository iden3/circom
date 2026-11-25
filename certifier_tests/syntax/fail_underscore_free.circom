pragma circom 2.2.0;

// (Syntax + H1) FAIL: `_ <-- expr` is still a free assignment (reported on
// the signal name `_`).
// @expect FAIL
// @certified 0
// @fail T() CE01

template T() {
    signal input a;
    signal output x;
    x <== a;
    _ <-- a * a;
}

component main = T();
