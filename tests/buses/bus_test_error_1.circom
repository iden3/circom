pragma circom 2.0.0;

/*

    This code should fail because buses cannot be defined with tags.
    Tags are asigned to bus signals when declared.

*/

bus {babyedwards} Point (n) {
    signal {binary} x[n], y[n];
}

template Main(n) {
    Point(n) output pout;

    Point(n) pin;
    for (var i=0; i<n; i++) {
        pin.x[i] <== 1;
        pin.y[i] <== 0;
    }

    component pipe = Pipe(n);
    pipe.pin <== pin;
    pout <== pipe.pout;
}

template Pipe (n) {
    Point(n) input pin;
    Point(n) output pout;
    Point(n) p;

    for (var i=0; i<n; i++) {
        p.x[i] <== pin.x[i];
        p.y[i] <== pin.y[i];
    }
    
    pout.x <== p.x;
    pout.y <== p.y;
}

component main = Main(255);