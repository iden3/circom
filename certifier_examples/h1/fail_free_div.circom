pragma circom 2.0.0;

// (H1) No Free Assignments -- FAIL.
// `out` is fixed with the unconstrained `<--`: -cbc must flag it
// with CE01, citing the exact line, even though this compiles fine normally.
// It also reports CA01: `<--` adds no constraint, so `out` appears in none.
template FreeDiv() {
    signal input a;
    signal input b;
    signal output out;
    out <-- a / b;
}

component main = FreeDiv();
