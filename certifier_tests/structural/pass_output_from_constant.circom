pragma circom 2.2.0;

// (Structural) PASS: an output fixed to a constant is fully determined.
// @expect PASS
// @certified 1

template Const() {
    signal input a;
    signal output out;
    signal output copy;
    out <== 5;
    copy <== a;
}

component main = Const();
