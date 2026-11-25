pragma circom 2.2.0;

// (H1) FAIL: a hand-written bit decomposition with `<--` inside a loop
// (outputs are untagged, so H2 is not involved). One CE01 per `<--`
// statement in the code, not one per unrolled iteration.
// @expect FAIL
// @certified 0
// @fail MyBits(4) CE01

template MyBits(n) {
    signal input in;
    signal output out[n];
    var lc = 0;
    var e2 = 1;
    for (var i = 0; i < n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] - 1) === 0;
        lc += out[i] * e2;
        e2 = e2 + e2;
    }
    lc === in;
}

component main = MyBits(4);
