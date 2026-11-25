pragma circom 2.2.0;

// (Nested buses) FAIL: `out.inner.x` is tagged in the Inner TYPE and
// written from an expression: CE02 on the exact nested field.
// @expect FAIL
// @certified 0
// @fail BareNested() CE02

bus Inner {
    signal {tagA} x;
}

bus Outer {
    Inner inner;
}

template BareNested() {
    signal input a;
    output Outer() out;
    out.inner.x <== a * 2;
}

component main = BareNested();
