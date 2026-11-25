pragma circom 2.2.0;

// (H1) FAIL: same template as pass_if_branch_not_taken.circom, but here the
// branch holding the `<--` is the one taken (flag = 1).
// @expect FAIL
// @certified 0
// @fail Pick(1) CE01

template Pick(flag) {
    signal input a;
    signal output out;
    if (flag == 1) {
        out <-- a;
        out === a;
    } else {
        out <== a;
    }
}

component main = Pick(1);
