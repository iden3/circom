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
include "pointbits.circom";
include "pedersen.circom";
include "escalarmulany.circom";
include "escalarmulfix.circom";

template EdDSAVerifier(n) {
    BinaryNumber(n) input msg;
    BinaryNumber(256) input A;
    BinaryNumber(256) input R8;
    BinaryNumber(256) input S;

    Point pA;
    Point pR8;

    var i;

// Ensure S<Subgroup Order

    component  compConstant = CompConstant(2736030358979909402780800718157159386076813972158567259200215660948447373040);

    for (i=0; i<254; i++) {
        S.bits[i] ==> compConstant.in.bits[i];
    }
    compConstant.out === 0;
    S.bits[254] === 0;
    S.bits[255] === 0;

// Convert A to Field elements (And verify A)

    component bits2pointA = Bits2Point_Strict();

    bits2pointA.in <== A;
    pA <== bits2pointA.pout;

// Convert R8 to Field elements (And verify R8)

    component bits2pointR8 = Bits2Point_Strict();

    bits2pointR8.in <== R8;
    pR8 <== bits2pointR8.pout;

// Calculate the h = H(R,A, msg)

    component hash = Pedersen(512+n);

    for (i=0; i<256; i++) {
        hash.in.bits[i] <== R8.bits[i];
        hash.in.bits[256+i] <== A.bits[i];
    }
    for (i=0; i<n; i++) {
        hash.in.bits[512+i] <== msg.bits[i];
    }

    component point2bitsH = Point2Bits_Strict();
    point2bitsH.pin <== hash.pout;

// Calculate second part of the right side:  right2 = h*8*A

    // Multiply by 8 by adding it 3 times.  This also ensure that the result is in
    // the subgroup.
    component dbl1 = BabyDbl();
    dbl1.pin <== pA;
    component dbl2 = BabyDbl();
    dbl2.pin <== dbl1.pout;
    component dbl3 = BabyDbl();
    dbl3.pin <== dbl2.pout;

    // We check that A is not zero.
    component isZero = IsZero();
    isZero.in <== dbl3.pin.x;
    isZero.out === 0;

    component mulAny = EscalarMulAny(256);
    mulAny.e <== point2bitsH.out;
    mulAny.pin <== dbl3.pout;


// Compute the right side: right =  R8 + right2

    component addRight = BabyAdd();
    addRight.p1 <== pR8;
    addRight.p2 <== mulAny.pout;

// Calculate left side of equation left = S*B8

    var BASE8[2] = [
        5299619240641551281634865583518297030282874472190772894086521144482721001553,
        16950150798460657717958625567821834550301663161624707787222815936182638968203
    ];
    component mulFix = EscalarMulFix(256, BASE8);
    mulFix.e <== S;

// Do the comparation left == right

    mulFix.out === addRight.pout;
}