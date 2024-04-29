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

include "compconstant.circom";

bus BinaryNumber {
    signal {binary} bits[254];
}

template AliasCheck() {
    signal input {binary} in[254];
    BinaryNumber output {unique} out;

    component compConstant = CompConstant(-1);

/*    for (var i=0; i<254; i++) {
        in[i] ==> compConstant.in[i];
        in[i] ==> out.bits[i];
    }

    compConstant.out === 0;
*/
    signal out_aux <== CompConstant(-1)(in);
    out_aux === 0;
    in ==> out.bits;
    
}
