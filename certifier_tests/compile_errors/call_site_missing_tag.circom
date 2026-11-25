pragma circom 2.2.0;

// COMPILE_ERROR: circom itself (T3001, AssignmentMissingTags) rejects
// feeding a tagged subcomponent input with an untagged value, before -cbc
// runs. This is why -cbc does not re-check subcomponent call sites.
// @expect COMPILE_ERROR
// @error T3001

template Relay() {
    signal input {binary} a;
    signal output {binary} out;
    out <== a;
}

template Top() {
    signal input a;
    signal output out;
    component r = Relay();
    r.a <== a;
    out <== r.out;
}

component main = Top();
