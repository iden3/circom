pragma circom 2.0.0;

include "comparators.circom"; // from the real circomlib2 (circomlib_buses) -- see README

// Array of bus instances, all elements honest -- PASS.
bus Inner {
    signal {binary} x;
}

template ArrOfBuses() {
    signal input a;
    component z = IsZero();
    z.in <== a;

    Inner() arr[2];
    arr[0].x <== z.out;
    arr[1].x <== z.out;

    signal output out;
    out <== arr[1].x;
}

component main = ArrOfBuses();
