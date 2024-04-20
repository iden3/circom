pragma circom 2.0.0;

include "bitify.circom";
include "escalarmulfix.circom";

bus Point () {
    signal x, y;
}

bus Parameters () {
    signal beta, gamma, delta, tau;
}

template BabyAdd() {
    Point input p1, p2;
    Point output pout;

    Parameters params;

    var a = 168700;
    var d = 168696;

    params.beta <== p1.x*p2.y;
    params.gamma <== p1.y*p2.x;
    params.delta <== (-a*p1.x+p1.y)*(p2.x + p2.y);
    params.tau <== params.beta * params.gamma;

    pout.x <-- (params.beta + params.gamma) / (1+ d*params.tau);
    (1 + d*params.tau) * pout.x === (params.beta + params.gamma);

    pout.y <-- (params.delta + a*params.beta - params.gamma) / (1-d*params.tau);
    (1-d*params.tau)*pout.y === (params.delta + a*params.beta - params.gamma);
}

template BabyDbl() {
    Point() input pin;
    Point output pout;

    component adder = BabyAdd();
    adder.p1 <== pin;
    adder.p2 <== pin;

    adder.pout ==> pout;
}


template BabyCheck() {
    Point input p;

    Point q;

    var a = 168700;
    var d = 168696;

    q.x <== p.x*p.x;
    q.y <== p.y*p.y;

    a*q.x + q.y === 1 + d*q.x*q.y;
}

// Extracts the public key from private key
template BabyPbk() {
    signal input in;
    Point output A;

    var BASE8[2] = [
        5299619240641551281634865583518297030282874472190772894086521144482721001553,
        16950150798460657717958625567821834550301663161624707787222815936182638968203
    ];

    component pvkBits = Num2Bits(253);
    pvkBits.in <== in;

    component mulFix = EscalarMulFix(253, BASE8);

    var i;
    for (i=0; i<253; i++) {
        mulFix.e[i] <== pvkBits.out[i];
    }
    A.x  <== mulFix.out[0];
    A.y  <== mulFix.out[1];
}