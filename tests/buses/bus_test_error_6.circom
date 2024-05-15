pragma circom 2.0.0;

/*

    This code should fail because there are template elements declared inside the bus.

*/

template T () {
    signal input in;
    signal output out;

    out <== in;
}

bus A () {
    signal a1, a2;
    a1 <== 1;
    a2 <-- a1;
    a2 === a1;
    signal array1[2], array2[2];
    array1[0] <== 1;
    array1[1] <-- 1;
    array1[0] === array1[1];
    array2 <== array1;
    array2 <-- array1;
    array2 === array1;
    component c = T();
    A bus1, bus2;
    bus1 <== 1;
    bus2 <-- bus1;
    bus2 === bus1;
    A busArray1[2], busArray2[2];
    busArray1[0] <== 1;
    busArray1[0] === busArray1[1];
    busArray2 <== busArray1;
    busArray2 <-- busArray1;
    busArray2 === busArray1;
}

template Main () {
    signal input in;
    A output out;
}

component main {public [in]} = Main();