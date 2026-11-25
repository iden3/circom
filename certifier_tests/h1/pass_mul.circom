pragma circom 2.2.0;

// (H1) PASS: the output is fixed with `<==`, value and constraint at once.
// @expect PASS
// @certified 1

template Mul() {
    signal input a;
    signal input b;
    signal output out;
    out <== a * b;
}

component main = Mul();
