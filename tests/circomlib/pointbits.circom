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
include "aliascheck.circom";
include "compconstant.circom";
include "babyjub.circom";


function sqrt(n) {

    if (n == 0) {
        return 0;
    }

    // Test that have solution
    var res = n ** ((-1) >> 1);
//        if (res!=1) assert(false, "SQRT does not exists");
    if (res!=1) return 0;

    var m = 28;
    var c = 19103219067921713944291392827692070036145651957329286315305642004821462161904;
    var t = n ** 81540058820840996586704275553141814055101440848469862132140264610111;
    var r = n ** ((81540058820840996586704275553141814055101440848469862132140264610111+1)>>1);
    var sq;
    var i;
    var b;
    var j;

    while ((r != 0)&&(t != 1)) {
        sq = t*t;
        i = 1;
        while (sq!=1) {
            i++;
            sq = sq*sq;
        }

        // b = c ^ m-i-1
        b = c;
        for (j=0; j< m-i-1; j ++) b = b*b;

        m = i;
        c = b*b;
        t = t*c;
        r = r*b;
    }

    if (r < 0 ) {
        r = -r;
    }

    return r;
}

bus BinaryPoint(n) {
    BinaryNumber(n) binY;
    signal {binary} signX;
}


template Bits2Point(n) {
    BinaryPoint(n) {babyedwardsbin} input in;
    Point output {babyedwards} pout;

    var i;

    // Check aliasing
    component b2nY = Bits2Num(n);
    b2nY.in <== in.binY;
    pout.y <== b2nY.out;

    var a = 168700;
    var d = 168696;

    var y2 = pout.y * pout.y;

    var x = sqrt( (1-y2)/(a - d*y2) );

    if (in.signX == 1) x = -x;

    pout.x <-- x;

    component babyCheck = BabyCheck();
    babyCheck.pin <== pout;

    component n2bX = Num2Bits(n);
    n2bX.in <== pout.x;

    component signCalc = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);
    signCalc.in <== n2bX.out;

    signCalc.out === in.signX;
}

template Bits2Point_Strict() {
    BinaryPoint(254) {babyedwardsbin} input in;
    Point output {babyedwards} pout;

    var i;

    // Check aliasing
    component b2nsY = Bits2Num_strict();
    b2nsY.in <== in.binY;
    pout.y <== b2nsY.out;

    var a = 168700;
    var d = 168696;

    var y2 = pout.y * pout.y;

    var x = sqrt( (1-y2)/(a - d*y2) );

    if (in.signX == 1) x = -x;

    pout.x <-- x;

    component babyCheck = BabyCheck();
    babyCheck.pin <== pout;

    component n2bsX = Num2Bits_strict();
    n2bsX.in <== pout.x;

    component signCalc = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);
    signCalc.in <== n2bsX.out;

    signCalc.out === in.signX;
}


template Point2Bits(n) {
    Point input {babyedwards} pin;
    BinaryPoint(n) {babyedwardsbin} output out;

    var i;

    component n2bX = Num2Bits(n);
    n2bX.in <== pin.x;
    component n2bY = Num2Bits(n);
    n2bY.in <== pin.y;

    component signCalc = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);
    signCalc.in <== n2bX.out;

    out.binY <== n2bY.out;
    out.signX <== signCalc.out;
}

template Point2Bits_Strict() {
    Point input {babyedwards} pin;
    BinaryPoint(254) {babyedwardsbin} output out;

    var i;

    component n2bsX = Num2Bits_strict();
    n2bsX.in <== pin.x;
    component n2bsY = Num2Bits_strict();
    n2bsY.in <== pin.y;

    component signCalc = CompConstant(10944121435919637611123202872628637544274182200208017171849102093287904247808);
    signCalc.in <== n2bsX.out;

    out.binY <== n2bsY.out;
    out.signX <== signCalc.out;
}