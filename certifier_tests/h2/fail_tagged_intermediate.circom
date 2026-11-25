pragma circom 2.2.0;

// (H2) FAIL: tags are checked on every signal, not only outputs. The
// intermediate `t` claims {binary} from an expression.
// @expect FAIL
// @certified 0
// @fail TaggedTmp() CE02

template TaggedTmp() {
    signal input a;
    signal output out;
    signal {binary} t;
    t <== a * a;
    out <== t;
}

component main = TaggedTmp();
