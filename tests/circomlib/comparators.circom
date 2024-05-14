/*
    Copyright 2018 0KIMS association.

    This file is part of circom (Zero Knowledge Circuit Compiler).

    circom is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    circom is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with circom. If not, see <https://www.gnu.org/licenses/>.
*/
pragma circom 2.0.0;

include "bitify.circom";

bus Pair {
    signal s[2];
}

template IsZero() {
    signal input in;
    signal output out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    out <== -in*inv + 1;
    in*out === 0;
}

template ForceZero() {
    signal input in;
    signal output {zero} out;

    component isz = IsZero();

    in ==> isz.in;
    isz.out === 1;
    in ==> out;
}


template IsEqual() {
    signal input in[2];
    signal output out;

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;
    isz.out ==> out;
}

template ForceEqual() {
    signal input in[2];
    Pair output {equal} out;

    component ise = IsEqual();

    in ==> ise.in;
    ise.out === 1;
    in ==> out.s;
}

template ForceEqualIfEnabled() {
    signal input enabled;
    signal input in[2];

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;

    (1 - isz.out)*enabled === 0;
}


// N is the number of bits the input have.
// The MSF is the sign bit.
template LessThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;

    component n2b = Num2Bits(n+1);

    n2b.in <== in[0] + (1<<n) - in[1];

    out <== 1-n2b.out.bits[n];
}

template ForceLessThan(n) {
    signal input in[2];
    Pair output {lt} out;

    component lessThan = LessThan(n);
    lessThan.in <== in;
    lessThan.out === 1;
    out.s <== in;
}



// N is the number of bits the input  have.
// The MSF is the sign bit.
template LessEqThan(n) {
    signal input in[2];
    signal output out;

    component lt = LessThan(n);

    lt.in[0] <== in[0];
    lt.in[1] <== in[1]+1;
    lt.out ==> out;
}

template ForceLessEqThan(n) {
    signal input in[2];
    Pair output {le} out;

    component leq = LessEqThan(n);
    leq.in <== in;
    leq.out === 1;
    out.s <== in;
}

// N is the number of bits the input  have.
// The MSF is the sign bit.
template GreaterThan(n) {
    signal input in[2];
    signal output out;

    component lt = LessThan(n);

    lt.in[0] <== in[1];
    lt.in[1] <== in[0];
    lt.out ==> out;
}

template ForceGreaterThan(n) {
    signal input in[2];
    Pair output {gt} out;

    component greaterThan = GreaterThan(n);
    greaterThan.in <== in;
    greaterThan.out === 1;
    out.s <== in;
}

// N is the number of bits the input  have.
// The MSF is the sign bit.
template GreaterEqThan(n) {
    signal input in[2];
    signal output out;

    component lt = LessThan(n);

    lt.in[0] <== in[1];
    lt.in[1] <== in[0]+1;
    lt.out ==> out;
}

template ForceGreaterEqThan(n) {
    signal input in[2];
    Pair output {ge} out;

    component geq = GreaterEqThan(n);
    geq.in <== in;
    geq.out === 1;
    out.s <== in;
}