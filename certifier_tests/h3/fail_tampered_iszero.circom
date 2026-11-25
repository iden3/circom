pragma circom 2.2.0;

// (H3) FAIL: IsZero without its `in*out === 0;` line (an actual soundness
// bug: `inv` is then free). The hash no longer matches, IsZero is checked
// as own code and fails CE01 + CE02. Top itself certifies: it only trusts
// IsZero's DECLARED {binary} output (Assumption 1); IsZero's honesty is
// judged separately, and here it is rejected.
// @expect FAIL
// @certified 1
// @fail IsZero() CE01 CE02

template IsZero() {
    input signal in;
    output signal {binary} out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    out <== -in*inv +1;
}

template Top() {
    signal input a;
    signal output {binary} out;
    out <== IsZero()(a);
}

component main = Top();
