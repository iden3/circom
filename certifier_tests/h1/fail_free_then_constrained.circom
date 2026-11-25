pragma circom 2.2.0;

// (H1) FAIL: `<--` followed by a `===` that pins it down. The circuit is
// actually sound, but H1 is syntactic: every signal must be fixed with
// `<==`/`==>`. Only CE01 (`out` does appear in a constraint, so no CA01).
// @expect FAIL
// @certified 0
// @fail FreeEq() CE01

template FreeEq() {
    signal input a;
    signal output out;
    out <-- a;
    out === a;
}

component main = FreeEq();
