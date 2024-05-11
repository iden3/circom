pragma circom 2.0.0;

include "compconstant.circom";

bus BinaryNumber {
    signal {binary} bits[254];
}

template AliasCheck() {
    signal input {binary} in[254];
    BinaryNumber output {unique, maxvalue} out;

    component compConstant = CompConstant(-1);

    for (var i=0; i<254; i++) {
        in[i] ==> compConstant.in[i];
        in[i] ==> out.bits[i];
    }

    out.maxvalue = (1 << 254) - 1;

    compConstant.out === 0;
}