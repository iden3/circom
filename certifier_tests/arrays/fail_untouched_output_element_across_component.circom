pragma circom 2.2.0;

// (Arrays of buses) FAIL: arr is an OUTPUT of ArrOfBuses and arr[1].x is
// never touched; Consumer reads it through the component. Circom's own
// "not initialized" check does not fire across the component boundary and
// H2 cannot see it (collapsed path), but the structural analysis reports
// CA01 on a declared output, which is promoted. TagA fails on its own
// (`<--`, unrelated). Consumer's CA02 is never a failure by itself.
// @expect FAIL
// @certified 1
// @fail TagA() CE01 CE02
// @fail ArrOfBuses() CA01

bus Inner {
    signal {tagA} x;
}

template TagA() {
    signal input a;
    signal output {tagA} out;
    out <-- a;
    out === a;
}

template ArrOfBuses() {
    signal input a;
    component tg = TagA();
    tg.a <== a;

    output Inner() arr[2];
    arr[0].x <== tg.out;
    // arr[1].x intentionally left untouched.
}

template Consumer() {
    signal input a;
    component c = ArrOfBuses();
    c.a <== a;
    signal output {tagA} out;
    out <== c.arr[1].x;
}

component main = Consumer();
