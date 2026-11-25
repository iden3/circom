pragma circom 2.2.0;

// (Syntax) PASS: a function computes constants used inside `<==`. Functions
// cannot touch signals, so they are irrelevant to H1/H2.
// @expect PASS
// @certified 1

function coef(i) {
    var c = 1;
    for (var k = 0; k < i; k++) {
        c = c * 3;
    }
    return c;
}

template Poly(n) {
    signal input x[n];
    signal output out;
    var lc = 0;
    for (var i = 0; i < n; i++) {
        lc += coef(i) * x[i];
    }
    out <== lc;
}

component main = Poly(4);
