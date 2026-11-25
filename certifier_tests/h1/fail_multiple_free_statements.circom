pragma circom 2.2.0;

// (H1) FAIL: two separate `<--` statements in the same template, each
// reported with its own CE01.
// @expect FAIL
// @certified 0
// @fail TwoFree() CE01 CE01

template TwoFree() {
    signal input a;
    signal output x;
    signal output y;
    x <-- a + 1;
    x === a + 1;
    y <-- a + 2;
    y === a + 2;
}

component main = TwoFree();
