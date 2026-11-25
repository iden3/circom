pragma circom 2.2.0;

// (H2) FAIL: the source is a bare copy, but of a subcomponent output that
// carries no tag. Square certifies; Top fails CE02.
// @expect FAIL
// @certified 1
// @fail Top() CE02

template Square() {
    signal input a;
    signal output out;
    out <== a * a;
}

template Top() {
    signal input a;
    signal output {binary} out;
    component s = Square();
    s.a <== a;
    out <== s.out;
}

component main = Top();
