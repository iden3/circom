pragma circom 2.2.0;

// (H1) PASS: a linear combination accumulated in a `var` inside a loop and
// then fixed with a single `<==`. Vars are not signals: H1 only looks at
// signal assignments.
// @expect PASS
// @certified 1

template WeightedSum(n) {
    signal input in[n];
    signal output out;
    var lc = 0;
    for (var i = 0; i < n; i++) {
        lc += in[i] * (i + 1);
    }
    out <== lc;
}

component main = WeightedSum(4);
