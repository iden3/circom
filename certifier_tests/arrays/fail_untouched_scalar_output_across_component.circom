pragma circom 2.2.0;

// (Structural) FAIL, scalar variant of the previous test: `arr` is a plain
// output of ArrOfBuses that is never touched, read by Consumer through the
// component. ArrOfBuses: CA01 (promoted). Consumer: CE02, since its
// {tagA} output is copied from `c.arr`, which carries no tag. TagA fails
// on its own.
// @expect FAIL
// @certified 0
// @fail TagA() CE01 CE02
// @fail ArrOfBuses() CA01
// @fail Consumer() CE02

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

    output signal arr;
}

template Consumer() {
    signal input a;
    component c = ArrOfBuses();
    c.a <== a;
    signal output {tagA} out;
    out <== c.arr;
}

component main = Consumer();
