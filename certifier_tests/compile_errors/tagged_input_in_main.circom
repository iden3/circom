pragma circom 2.2.0;

// COMPILE_ERROR: the main component cannot have tagged inputs (T2051). This
// is why every test that needs a tagged input wraps it in a Top() that
// obtains the tag from a trusted circomlib template.
// @expect COMPILE_ERROR
// @error T2051

template Relay() {
    signal input {binary} in;
    signal output {binary} out;
    out <== in;
}

component main = Relay();
