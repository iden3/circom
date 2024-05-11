pragma circom 2.0.0;

/*

    This code should fail because buses cannot be defined with input or output fields.
    Input or output signals can only be defined inside templates.

*/

bus InputPoint (n) {
    signal input {binary} x[n], y[n];
}

bus OutputPoint (n) {
    signal output {binary} x[n], y[n];
}

template Pipe (n) {
    InputPoint(n) input {babyedwards} pin;
    OutputPoint(n) output {babyedwards} pout;

    for (var i=0; i<n; i++) {
        pout.x[i] <== pin.x[i];
        pout.y[i] <== pin.y[i];
    }
}

component main {public [pin]} = Pipe(3);