pragma circom 2.0.0;

bus Point (n) {
    signal {binary} x[n];
    signal {maxvalue} y;
}

template Multiplier () {
    signal input a,b;
    signal output c;

    c <== a * b;
}

component main = Multiplier();