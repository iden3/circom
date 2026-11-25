pragma circom 2.0.0;

// (H1) No Free Assignments -- PASS.
// `out` is fixed with `<==`, so value and constraint come in the same step.
template SafeMul() {
    signal input a;
    signal input b;
    signal output out;
    out <== a * b;
}

component main = SafeMul();
