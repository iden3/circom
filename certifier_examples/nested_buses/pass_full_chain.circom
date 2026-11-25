pragma circom 2.0.0;

include "comparators.circom"; // from the real circomlib2 (circomlib_buses) -- see README

// Nested bus field -- PASS, exercising all three honest patterns at once:
//   - HonestNested:      tag flows through a SUBCOMPONENT's nested bus output
//                         (out.inner <== m.out;), trusting the callee's own
//                         declared structure (Assumption 1), recursively.
//   - DirectFieldWrite:  tag flows straight into a nested field
//                         (out.inner.x <== a;).
//   - LocalBusToBusCopy: tag flows through a LOCAL whole-bus-to-bus copy
//                         (dst <== src;), propagating src's own PROVEN
//                         sub-tree into dst's fields.
// All three need `IsZero()` (real circomlib2) to actually be recognized as
// trusted -- it is, automatically, because the hash of its own definition
// matches `KNOWN_CIRCOMLIB2_TEMPLATE_HASHES`, no flag needed. Point `-l` at a
// copy where `IsZero()`'s definition was modified instead and it stops
// matching, falls through to ordinary H1/H2 checking, and fails H1 on its own
// `<--` -- try it.
bus Inner {
    signal {binary} x;
}

bus Outer {
    Inner inner;
}

template MakesInner() {
    signal input {binary} a;
    output Inner() out;
    out.x <== a;
}

template HonestNested() {
    signal input {binary} a;
    output Outer() out;
    component m = MakesInner();
    m.a <== a;
    out.inner <== m.out;
}

template DirectFieldWrite() {
    signal input {binary} a;
    output Outer() out;
    out.inner.x <== a;
}

template LocalBusToBusCopy() {
    signal input {binary} a;
    Outer() src;
    src.inner.x <== a;
    output Outer() dst;
    dst <== src;
}

template Top() {
    signal input a; // untagged main input -- IsZero() below manufactures the tag
    signal output ok1;
    signal output ok2;
    signal output ok3;

    component z = IsZero();
    z.in <== a;

    component h = HonestNested();
    h.a <== z.out;
    ok1 <== h.out.inner.x;

    component d = DirectFieldWrite();
    d.a <== z.out;
    ok2 <== d.out.inner.x;

    component l = LocalBusToBusCopy();
    l.a <== z.out;
    ok3 <== l.dst.inner.x;
}

component main = Top();
