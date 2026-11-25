pragma circom 2.0.0;

// Nested bus field -- FAIL.
// `out.inner.x` declares {tagA} (in the bus TYPE definition, not here), but
// is written from an arithmetic expression. Before the nested-bus fix, this
// went completely unchecked in both directions (reading it was also
// silently accepted); now it correctly reports CE02 on the exact field.
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
