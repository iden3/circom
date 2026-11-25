pragma circom 2.0.0;

include "comparators.circom"; // from the real circomlib2 (circomlib_buses) -- see README

// Demonstrates the three possible outcomes side by side:
//   - Adder: ordinary code, checked normally, certifies clean.
//   - IsZero: a REAL circomlib2 template (comparators.circom), uses `<--`
//     internally -- automatically treated as a black box because the hash
//     of its own definition matches KNOWN_CIRCOMLIB2_TEMPLATE_HASHES, no
//     flag needed. If you change IsZero's tokens (or point `-l` at a copy
//     where it was modified), the hash stops matching and it falls through
//     to ordinary H1/H2 checking, where it fails H1 on its own -- exactly
//     like un-vetted code. Editing OTHER templates of comparators.circom, or
//     only IsZero's comments/whitespace, leaves IsZero trusted (it has no
//     subcomponents whose trust it could inherit).
//   - Broken: ordinary code with its own genuine H1 violation -- always
//     fails, trusted or not: CE01 for its `<--`, plus CA01 because its
//     output `out` ends up in no constraint.
template Adder() {
    signal input a;
    signal input b;
    signal output out;
    out <== a + b;
}

template Broken() {
    signal input a;
    signal output out;
    out <-- a * 2;
}

template Top() {
    signal input a;
    signal input b;
    signal output sum;
    signal output isz;
    signal output broke;

    component add = Adder();
    add.a <== a;
    add.b <== b;
    sum <== add.out;

    component z = IsZero();
    z.in <== a;
    isz <== z.out;

    component brk = Broken();
    brk.a <== a;
    broke <== brk.out;
}

component main = Top();
