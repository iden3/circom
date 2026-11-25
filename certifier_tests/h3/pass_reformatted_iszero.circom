pragma circom 2.2.0;

// (H3) PASS: a hand copy of circomlib's IsZero, re-indented, with lines
// joined and new comments, but the SAME tokens with the same spacing
// between them. Normalisation drops comments and collapses whitespace runs,
// so the hash still matches and IsZero is trusted.
// @expect PASS
// @certified 1
// @trusted IsZero()

template IsZero()
{
  // re-indented and re-commented: still the audited circomlib2 IsZero
  input signal in;   output signal {binary} out;
  signal inv; /* the witness */
  inv <-- in!=0 ? 1/in : 0;
  out <== -in*inv +1;   // same tokens, same intra-line spacing
  in*out === 0;
}

template Top() {
    signal input a;
    signal output {binary} out;
    out <== IsZero()(a);
}

component main = Top();
