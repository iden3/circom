pragma circom 2.0.0;

include "comparators.circom"; // from the real circomlib2 (circomlib_buses) -- see README

// Array of bus instances -- one element honest, one bare -- FAIL.
// Circom itself only confirms a tag "if it inherits the tag in all
// positions" (see assignment_utils.rs, perform_tag_propagation): a single
// dishonest element poisons the whole collapsed path `arr.x`, regardless of
// how many OTHER elements were honestly assigned. Reported exactly once, at
// the offending element, not once per element.
bus Inner {
    signal {binary} x;
}

template ArrOfBuses() {
    signal input a; // untagged main input -- IsZero() below manufactures the tag
    signal input b;
    component z = IsZero();
    z.in <== a;

    Inner() arr[2];
    arr[0].x <== z.out; // honest
    arr[1].x <== b * 2; // bare -- poisons arr.x for the whole array

    signal output out;
    out <== arr[0].x;
}

component main = ArrOfBuses();
