pragma circom 2.2.0;

// (Buses + H1) FAIL: `<--` into an (untagged) bus field.
// @expect FAIL
// @certified 0
// @fail FreeField() CE01

bus Pair() {
    signal x;
    signal y;
}

template FreeField() {
    signal input a;
    output Pair() out;
    out.x <-- a * 3;
    out.x === a * 3;
    out.y <== a;
}

component main = FreeField();
