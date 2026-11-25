pragma circom 2.1.5;

include "comparators.circom";
include "tags-managing.circom";

// H3 trust is per template and recursive. `LessThan` (comparators.circom)
// instantiates `Num2Bits` (bitify.circom); an instance is trusted only if the
// hash of its OWN definition matches KNOWN_CIRCOMLIB2_TEMPLATE_HASHES AND all
// of its descendants are trusted too.
//
// Run:
//   target/debug/circom certifier_examples/h3_trusted/per_template_lessthan.circom \
//     -cbc -l <path-to-circomlib>/circuits -o /tmp
//
// Expected with pristine circomlib2: PASS, with LessThan, Num2Bits and
// MaxbitCheck* all trusted. With a copy of circuits/ where Num2Bits lost its
// `out[i] * (out[i] -1 ) === 0;` line: Num2Bits(8) and Num2Bits(9) fail with
// CE01 (`<--`) and CE02 (bare `binary` tag), and LessThan and MaxbitCheck --
// untouched themselves, but instantiating Num2Bits -- lose trust as well and
// are checked like any other code (both fail CE02).

template Top() {
    signal input a;
    signal input b;
    signal output out;

    component mc = MaxbitCheckArray(8, 2);
    mc.in[0] <== a;
    mc.in[1] <== b;

    component lt = LessThan(8);
    lt.in <== mc.out;
    out <== lt.out;
}

component main = Top();
