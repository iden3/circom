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

include "comparators.circom";
include "aliascheck.circom";

bus BinaryNumber(n) {
    signal {binary} bits[n];
}

template Num2Bits(n) {
    signal input in;
    BinaryNumber(n) output out;
    var lc1=0;

    var e2=1;
    for (var i=0; i<n; i++) {
        out.bits[i] <-- (in >> i) & 1;
        out.bits[i] * (out.bits[i] - 1) === 0;
        lc1 += out.bits[i] * e2;
        e2 = e2 + e2;
    }

    lc1 === in;
}

template Num2Bits_strict() {
    signal input in;
    BinaryNumber(254) output {unique} out;

    component aliasCheck = AliasCheck();
    component n2b = Num2Bits(254);

    in ==> n2b.in;
    n2b.out ==> aliasCheck.in;
    aliasCheck.out ==> out;
}

template Bits2Num(n) {
    BinaryNumber(n) input in;
    signal output out;
    var lc1=0;

    var e2 = 1;
    for (var i=0; i<n; i++) {
        lc1 += in.bits[i] * e2;
        e2 = e2 + e2;
    }

    lc1 ==> out;
}

template Bits2Num_strict() {
    BinaryNumber(254) input in;
    signal output out;

    component aliasCheck = AliasCheck();
    component b2n = Bits2Num(254);

    in ==> aliasCheck.in;
    aliasCheck.out ==> b2n.in;
    b2n.out ==> out;
}

template Num2BitsNeg(n) {
    signal input in;
    BinaryNumber(n) output out;

    component isZero = IsZero();

    var lc1 = 0;
    var pot = 1;
    var maxpot = 2**n;
    var neg = n == 0 ? 0 : maxpot - in;

    for (var i=0; i<n; i++) {
        out.bits[i] <-- (neg >> i) & 1;
        out.bits[i] * (out.bits[i] - 1) === 0;
        lc1 += out.bits[i] * pot;
        pot *= 2;
    }

    in ==> isZero.in;

    lc1 + isZero.out * maxpot === maxpot - in;
}

template Num2BitsNeg_strict() {
    signal input in;
    BinaryNumber(254) output {unique} out;

    component aliasCheck = AliasCheck();
    component n2bn = Num2BitsNeg(254);

    in ==> n2bn.in;
    n2bn.out ==> aliasCheck.in;
    aliasCheck.out ==> out;
}