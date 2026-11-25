pragma circom 2.2.0;

// (H2, inherited tags) FAIL: t copies `b`, which DECLARES {binary} but gets
// it dishonestly. A dishonest source carries nothing, so t inherits
// nothing: CE02 on b and CE02 on o.
// @expect FAIL
// @certified 0
// @fail Top() CE02 CE02

template Top() {
    signal input a;
    signal output {binary} o;
    signal {binary} b <== a * a;
    signal t;
    t <== b;
    o <== t;
}

component main = Top();
