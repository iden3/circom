pragma circom 2.0.0;

// Not a -cbc example: this circuit does not even reach the
// certifier. Circom's OWN runtime tag propagation (execute.rs:1734,
// `AssignmentMissingTags`) already refuses to compile a value into a
// tag-requiring input unless that value itself already carries the tag --
// try it:
//
//   circom note_circom_already_checks_call_sites.circom -cbc
//   -> error[T3001] Invalid assignment: missing tags required by input signal
//
// This is exactly why `check_subcomponent_input` was removed from
// certifier.rs: whatever it would have caught, circom already catches first,
// unconditionally, for every compilation (not just -cbc ones).
template Relay() {
    signal input {binary} a;
    signal output {binary} out;
    out <== a;
}

template Top() {
    signal input a; // untagged on purpose
    signal output out;
    component r = Relay();
    r.a <== a; // <-- circom itself rejects this line before -cbc runs
    out <== r.out;
}

component main = Top();
