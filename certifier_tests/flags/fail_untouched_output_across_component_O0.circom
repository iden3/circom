pragma circom 2.2.0;

// (Flags) FAIL with --O0: same as
// arrays/fail_untouched_output_element_across_component.circom.
// @expect FAIL
// @certified 1
// @fail TagA() CE01 CE02
// @fail ArrOfBuses() CA01
// @args --O0

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
}

template Consumer() {
    signal input a;
    component c = ArrOfBuses();
    c.a <== a;
    signal output {tagA} out;
    out <== c.arr[1].x;
}

component main = Consumer();
