pragma circom 2.2.0;

// (Structural) PASS: an intermediate signal declared and never used. The
// CA01 warning is not promoted (it cannot affect any output).
// @expect PASS
// @certified 1

template Spare() {
    signal input a;
    signal output out;
    signal unused;
    out <== a * a;
}

component main = Spare();
