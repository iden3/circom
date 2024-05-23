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

include "montgomery.circom";
include "babyjub.circom";
include "comparators.circom";

template Multiplexor2() {
    signal input sel;
    signal input in[2][2];
    signal output out[2];

    out[0] <== (in[1][0] - in[0][0])*sel + in[0][0];
    out[1] <== (in[1][1] - in[0][1])*sel + in[0][1];
}

template BitElementMulAny() {
    signal input sel;
    Point input {babymontgomery} dblIn;
    Point input {babymontgomery} addIn;
    Point output {babymontgomery} dblOut;
    Point output {babymontgomery} addOut;

    component doubler = MontgomeryDouble();
    component adder = MontgomeryAdd();
    component selector = Multiplexor2();


    sel ==> selector.sel;

    dblIn ==> doubler.pin;
    doubler.pout ==> adder.pin1;
    addIn ==> adder.pin2;
    addIn.x ==> selector.in[0][0];
    addIn.y ==> selector.in[0][1];
    adder.pout.x ==> selector.in[1][0];
    adder.pout.y ==> selector.in[1][1];

    doubler.pout ==> dblOut;
    selector.out[0] ==> addOut.x;
    selector.out[1] ==> addOut.y;
}

// pin is edwards point
// n must be <= 248
// returns pout in twisted edwards
// dbl is in montgomery to be linked;

template SegmentMulAny(n) {
    BinaryNumber(n) input e;
    Point input {babyedwards} pin;
    Point output {babyedwards} pout;
    Point output {babymontgomery} dbl;

    component bits[n-1];

    component e2m = Edwards2Montgomery();

    pin ==> e2m.pin;

    var i;

    bits[0] = BitElementMulAny();
    e2m.pout ==> bits[0].dblIn;
    e2m.pout ==> bits[0].addIn;
    e.bits[1] ==> bits[0].sel;

    for (i=1; i<n-1; i++) {
        bits[i] = BitElementMulAny();

        bits[i-1].dblOut ==> bits[i].dblIn;
        bits[i-1].addOut ==> bits[i].addIn;
        e.bits[i+1] ==> bits[i].sel;
    }

    bits[n-2].dblOut ==> dbl;

    component m2e = Montgomery2Edwards();

    bits[n-2].addOut ==> m2e.pin;

    component eadder = BabyAdd();

    m2e.pout ==> eadder.p1;
    -pin.x ==> eadder.p2.x;
    pin.y ==> eadder.p2.y;

    component lastSel = Multiplexor2();

    e.bits[0] ==> lastSel.sel;
    eadder.pout.x ==> lastSel.in[0][0];
    eadder.pout.y ==> lastSel.in[0][1];
    m2e.pout.x ==> lastSel.in[1][0];
    m2e.pout.y ==> lastSel.in[1][1];

    lastSel.out[0] ==> pout.x;
    lastSel.out[1] ==> pout.y;
}

// This function assumes that p is in the subgroup and it is different to 0

template EscalarMulAny(n) {
    BinaryNumber(n) input e;              // Input in binary format
    Point input {babyedwards} pin;        // Point (Twisted format)
    Point output {babyedwards} pout;      // Point (Twisted format)

    var nsegments = (n-1)\148 +1;
    var nlastsegment = n - (nsegments-1)*148;

    component segments[nsegments];
    component doublers[nsegments-1];
    component m2e[nsegments-1];
    component adders[nsegments-1];
    component zeropoint = IsZero();
    zeropoint.in <== pin.x;

    var s;
    var i;
    var nseg;

    for (s=0; s<nsegments; s++) {

        nseg = (s < nsegments-1) ? 148 : nlastsegment;

        segments[s] = SegmentMulAny(nseg);

        for (i=0; i<nseg; i++) {
            e.bits[s*148+i] ==> segments[s].e.bits[i];
        }

        if (s==0) {
            // force G8 point if input point is zero
            segments[s].pin.x <== pin.x + (5299619240641551281634865583518297030282874472190772894086521144482721001553 - pin.x)*zeropoint.out;
            segments[s].pin.y <== pin.y + (16950150798460657717958625567821834550301663161624707787222815936182638968203 - pin.y)*zeropoint.out;
        } else {
            doublers[s-1] = MontgomeryDouble();
            m2e[s-1] = Montgomery2Edwards();
            adders[s-1] = BabyAdd();

            segments[s-1].dbl ==> doublers[s-1].pin;
            doublers[s-1].pout ==> m2e[s-1].pin;
            m2e[s-1].pout ==> segments[s].pin;

            if (s==1) {
                segments[s-1].pout ==> adders[s-1].p1;
            } else {
                adders[s-2].pout ==> adders[s-1].p1;
            }
            segments[s].pout ==> adders[s-1].p2;
        }
    }

    if (nsegments == 1) {
        segments[0].pout.x*(1-zeropoint.out) ==> pout.x;
        segments[0].pout.y+(1-segments[0].pout.y)*zeropoint.out ==> pout.y;
    } else {
        adders[nsegments-2].pout.x*(1-zeropoint.out) ==> pout.x;
        adders[nsegments-2].pout.y+(1-adders[nsegments-2].pout.y)*zeropoint.out ==> pout.y;
    }
}