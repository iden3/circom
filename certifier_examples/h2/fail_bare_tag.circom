pragma circom 2.0.0;

// (H2) No Bare Tags -- FAIL.
// `out` declares {binary} but its value comes from an arithmetic expression,
// not a bare copy from an already-tagged source. Circom itself compiles this
// with no complaint (nothing downstream ever consumes the tag) -- this is
// exactly the reactive-vs-proactive gap -cbc closes: CE02.
template BareTag() {
    signal input a;
    signal output {binary} out;
    out <== a * 1;
}

component main = BareTag();
