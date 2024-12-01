pragma circom 2.0.0;

include "circomlib/circuits/poseidon.circom";

template MixerCommitmentProof() {
    signal input commitment;   // Public commitment
    signal input s;            // Private secret value
    signal input r;            // Private random value
    signal output isValid;     // Output signal to indicate validity

    // Instantiate Poseidon for 2 inputs
    component hash = Poseidon(2);

    // Connect inputs to Poseidon
    hash.inputs[0] <== s;
    hash.inputs[1] <== r;

    // Define the constraint for matching commitment
    signal diff;             // Difference between hash and commitment
    diff <== hash.out - commitment;

    // Set isValid to 1 if diff is 0, otherwise 0
    isValid <== 1 - diff * diff;
}

component main = MixerCommitmentProof();
