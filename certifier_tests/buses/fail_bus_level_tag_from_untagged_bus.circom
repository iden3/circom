pragma circom 2.2.0;

// (Buses) FAIL: a bus-level {babyedwards} tag copied (bare `<==`) from a
// bus that does NOT carry it. The copy is syntactically bare, but its
// source has no such tag: CE02 on `q`.
// @expect FAIL
// @certified 0
// @fail Top() CE02

bus Point() {
    signal x;
    signal y;
}

template Top() {
    input Point() p;
    output Point() {babyedwards} q;
    q <== p;
}

component main = Top();
