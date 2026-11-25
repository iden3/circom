pragma circom 2.2.0;

// (H1) FAIL, locality: Inner uses `<--`; Middle and Top only use `<==`.
// H1 is local to each template, so only Inner is reported; its parents are
// still certified (their own code is clean).
// @expect FAIL
// @certified 2
// @fail Inner() CE01

template Inner() {
    signal input a;
    signal output out;
    out <-- a * a;
    out === a * a;
}

template Middle() {
    signal input a;
    signal output out;
    out <== Inner()(a);
}

template Top() {
    signal input a;
    signal output out;
    signal m <== Middle()(a);
    out <== m + 1;
}

component main = Top();
