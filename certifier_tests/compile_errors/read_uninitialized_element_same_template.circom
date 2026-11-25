pragma circom 2.2.0;

// COMPILE_ERROR: reading a never-initialised bus array element from WITHIN
// the template that declares it is caught by circom itself ("trying to
// access to a signal that is not initialized"). Contrast with
// arrays/fail_untouched_output_element_across_component.circom.
// @expect COMPILE_ERROR
// @error T3001

bus Inner {
    signal x;
}

template Reader() {
    signal input a;
    Inner() arr[2];
    arr[0].x <== a;
    signal output out;
    out <== arr[1].x;
}

component main = Reader();
