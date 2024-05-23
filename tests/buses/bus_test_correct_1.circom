pragma circom 2.0.0;

bus Point (n) {
    signal {binary} x[n], y[n];
}

template Main(n) {
    Point(n) output {babyedwards} pout;

    Point(n) {babyedwards} pin;
    for (var i=0; i<n; i++) {
        pin.x[i] <== 1;
        pin.y[i] <== 0;
    }

    component pipe = Pipe(n);
    pipe.pin <== pin;
    pout <== pipe.pout;
}

template Pipe (n) {
    Point(n) input {babyedwards} pin;
    Point(n) output {babyedwards} pout;
    Point(n) {babyedwards} p;

    for (var i=0; i<n; i++) {
        p.x[i] <== pin.x[i];
        p.y[i] <== pin.y[i];
    }
    
    pout.x <== p.x;
    pout.y <== p.y;
}

component main = Main(255);