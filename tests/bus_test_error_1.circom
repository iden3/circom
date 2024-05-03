pragma circom 2.0.0;

/*

    This code should fail because buses cannot be defined with tags.
    Tags are asigned to bus signals when declared.

*/

bus {babyedwards} Point (n) {
    signal {binary} x[n], y[n];
}

template Pipe (n) {
    Point(n) input pin;
    Point(n) output pout;

    for (var i=0; i<n; i++) {
        pout.x[i] <== pin.x[i];
        pout.y[i] <== pin.y[i];
    }
}