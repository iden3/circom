pragma circom 2.2.0;

// (Syntax + H1) FAIL: a tuple substitution with `<--`. The compiler splits
// it into one substitution per signal: one CE01 for `x`, one for `y`.
// @expect FAIL
// @certified 0
// @fail T() CE01 CE01

template T() {
    signal input a;
    signal input b;
    signal output x;
    signal output y;
    (x, y) <-- (a, b);
    x === a;
    y === b;
}

component main = T();
