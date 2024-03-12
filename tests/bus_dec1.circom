pragma circom 2.0.0;

bus Point (n) {
    signal {binary} x[n];
    signal {maxvalue} y;
}

bus New (m,n) {
    Point(m) {babyedwards} a;
    signal {binary} select;
    Point(n) {babymontgomery} b;
}

template Multiplier () {
    signal input a,b;
    signal output c;
    Point(3) input {tag1} p1;
    New(2,4) output {tag2} n1;
    Point(3) {tag1} p2 <== p1, p3;

    p3.x[0] <-- p1.y;
    c <== a * b;

}

component main = Multiplier();