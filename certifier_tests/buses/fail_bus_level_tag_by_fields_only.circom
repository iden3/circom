pragma circom 2.2.0;

// (Buses) FAIL: writing every field of a bus never establishes its
// bus-level tag (it is a property of the whole bus): `q` stays without an
// honest source -> CE02.
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
    q.x <== p.x;
    q.y <== p.y;
}

component main = Top();
