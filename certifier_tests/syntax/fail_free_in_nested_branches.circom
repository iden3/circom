pragma circom 2.2.0;

// (Syntax + H1) FAIL: the `<--` is three parameter-dependent branches deep;
// only Deep(1,2) reaches it, Deep(0,2) and Deep(1,0) do not.
// @expect FAIL
// @certified 3
// @fail Deep(1,2) CE01

template Deep(a, b) {
    signal input x;
    signal output o;
    if (a == 1) {
        if (b > 1) {
            if (a + b == 3) {
                o <-- x;
                o === x;
            } else {
                o <== x;
            }
        } else {
            o <== x;
        }
    } else {
        o <== x;
    }
}

template Top() {
    signal input x;
    signal output o[3];
    o[0] <== Deep(0, 2)(x);
    o[1] <== Deep(1, 0)(x);
    o[2] <== Deep(1, 2)(x);
}

component main = Top();
