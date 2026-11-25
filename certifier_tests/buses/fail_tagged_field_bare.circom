pragma circom 2.2.0;

// (Buses) FAIL: the {binary} field comes from an expression. CE02 cites the
// exact field path `out.x`.
// @expect FAIL
// @certified 0
// @fail BareField() CE02

bus Flag() {
    signal {binary} x;
}

template BareField() {
    signal input a;
    output Flag() out;
    out.x <== a * a;
}

component main = BareField();
