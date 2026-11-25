pragma circom 2.2.0;

// (Syntax + H1) FAIL: `<--` inside a `while` loop.
// @expect FAIL
// @certified 0
// @fail W(3) CE01

template W(n) {
    signal input a;
    signal output out[n];
    var i = 0;
    while (i < n) {
        out[i] <-- a + i;
        out[i] === a + i;
        i++;
    }
}

component main = W(3);
