pragma circom 2.0.0;

bus Point (n) {
    signal {binary} x[n], y[n];
}

template Main {
    Point(2) input pin;
    Point(3) output pout;

    pout <== pin;
}

component main {public [pin]} = Main();