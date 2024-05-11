pragma circom 2.0.0;

/*

    This code should fail because field accesses and array accesses do not match
    with the types and dimensions of the expresions.

*/

bus A () {
    signal {tag1} a1;
    signal {tag2} a2;
}

bus B {
    signal {tag1} b1;
    signal {tag2} b2;
}

bus C {
    B b;
}

template Main () {
    signal input in1, in2;
    B output busB;

    A busA;
    C busC;

    busA.a1 <== in1;
    busA.a2 <== busA.a1 + in2 * busA.a1;

    busB <== busA;

    busB.b1 <== busA.a1;
    busB.b2 <== busA.a2;
    busB.b1 <== busA.a2;
    busB.b2 <== busA.a1;

    busC <== busA;
    busC.b <== busA;
    busB <== busC;
    busB <== busC.b;

    busC.b.b1 <== busA.a2 + busC.b.b2;
    busB.b2 + busC.b.b2 === busB.b1 + busA.a1;
}

component main {public [in1, in2]} = Main();