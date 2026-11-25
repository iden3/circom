pragma circom 2.2.0;

// (H3) FAIL: token-for-token circomlib IsZero under another name. Trust is
// keyed by (template name, hash), so MyIsZero is checked as own code.
// @expect FAIL
// @certified 1
// @fail MyIsZero() CE01 CE02

template MyIsZero() {
    input signal in;
    output signal {binary} out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    out <== -in*inv +1;
    in*out === 0;
}

template Top() {
    signal input a;
    signal output {binary} out;
    out <== MyIsZero()(a);
}

component main = Top();
