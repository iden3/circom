pragma circom 2.2.0;

// (H1 + H2) FAIL: `<--` into a tagged output violates both properties at
// once: CE01 for the free assignment, CE02 because `<--` is not a bare
// tagged copy.
// @expect FAIL
// @certified 0
// @fail TagA() CE01 CE02

template TagA() {
    signal input a;
    signal output {tagA} out;
    out <-- a;
    out === a;
}

component main = TagA();
