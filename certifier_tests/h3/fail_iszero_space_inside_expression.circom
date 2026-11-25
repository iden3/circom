pragma circom 2.2.0;

// (H3) FAIL: the only change w.r.t. circomlib's IsZero is `+1` -> `+ 1`.
// Normalisation collapses each whitespace RUN into one space but does not
// remove whitespace between tokens, so inserting a space where there was
// none changes the hash: IsZero is no longer trusted and is checked as own
// code (CE01 for its `<--`; its {binary} out comes from an expression ->
// CE02). Fail-closed by design; documented here so it is not a surprise.
// @expect FAIL
// @certified 1
// @fail IsZero() CE01 CE02

template IsZero() {
    input signal in;
    output signal {binary} out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    out <== -in*inv + 1;
    in*out === 0;
}

template Top() {
    signal input a;
    signal output {binary} out;
    out <== IsZero()(a);
}

component main = Top();
