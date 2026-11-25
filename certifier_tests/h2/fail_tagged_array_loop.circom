pragma circom 2.2.0;

// (H2) FAIL: a tagged output array filled element by element from
// expressions. All elements collapse into the same tracked path `out`, so
// it is reported once.
// @expect FAIL
// @certified 0
// @fail TaggedArr(3) CE02

template TaggedArr(n) {
    signal input in[n];
    signal output {binary} out[n];
    for (var i = 0; i < n; i++) {
        out[i] <== in[i] * 1;
    }
}

component main = TaggedArr(3);
