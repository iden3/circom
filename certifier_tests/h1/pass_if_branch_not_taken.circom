pragma circom 2.2.0;

// (H1) PASS: the `<--` lives in a branch that this concrete instantiation
// (flag = 0) never takes. H1 walks the code already pruned to the branches
// actually taken, so the dead `<--` is not reported.
// See fail_if_branch_taken.circom for flag = 1.
// @expect PASS
// @certified 1

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

component main = Pick(0);
