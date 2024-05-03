pragma circom 2.0.0;

bus Point (n) {
    signal {binary} x[n], y[n];
}

template Pipe (n) {
    Point(n) input {babyedwards} pin;
    Point(n) output {babyedwards} pout;

    for (var i=0; i<n; i++) {
        pout.x[i] <== pin.x[i];
        pout.y[i] <== pin.y[i];
    }
}