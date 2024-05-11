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
include "mux3.circom";
include "babyjub.circom";

bus Point {
    signal {bn128} x,y;
}

template Window4() {
    signal input {binary} in[4];
    Point input {babymontgomery} base;
    Point output {babymontgomery} out;
    Point output {babymontgomery} out8;   // Returns 8*Base (To be linked)

    component mux = MultiMux3(2);

    mux.s[0] <== in[0];
    mux.s[1] <== in[1];
    mux.s[2] <== in[2];

    component dbl2 = MontgomeryDouble();
    component adr3 = MontgomeryAdd();
    component adr4 = MontgomeryAdd();
    component adr5 = MontgomeryAdd();
    component adr6 = MontgomeryAdd();
    component adr7 = MontgomeryAdd();
    component adr8 = MontgomeryAdd();

// in[0]  -> 1*BASE

    mux.c[0][0] <== base.x;
    mux.c[1][0] <== base.y;

// in[1] -> 2*BASE
    dbl2.in <== base;
    mux.c[0][1] <== dbl2.pout.x;
    mux.c[1][1] <== dbl2.pout.y;

// in[2] -> 3*BASE
    adr3.pin1 <== base;
    adr3.pin2 <== dbl2.pout;
    mux.c[0][2] <== adr3.pout.x;
    mux.c[1][2] <== adr3.pout.y;

// in[3] -> 4*BASE
    adr4.pin1 <== base;
    adr4.pin2 <== adr3.pout;
    mux.c[0][3] <== adr4.pout.x;
    mux.c[1][3] <== adr4.pout.y;

// in[4] -> 5*BASE
    adr5.pin1 <== base;
    adr5.pin2 <== adr4.pout;
    mux.c[0][4] <== adr5.pout.x;
    mux.c[1][4] <== adr5.pout.y;

// in[5] -> 6*BASE
    adr6.pin1 <== base;
    adr6.pin2 <== adr5.pout;
    mux.c[0][5] <== adr6.pout.x;
    mux.c[1][5] <== adr6.pout.y;

// in[6] -> 7*BASE
    adr7.pin1 <== base;
    adr7.pin2 <== adr6.pout;
    mux.c[0][6] <== adr7.pout.x;
    mux.c[1][6] <== adr7.pout.y;

// in[7] -> 8*BASE
    adr8.pin1 <== base;
    adr8.pin2 <== adr7.pout;
    mux.c[0][7] <== adr8.pout.x;
    mux.c[1][7] <== adr8.pout.y;

    out8 <== adr8.pout;

    out.x <== mux.out[0];
    out.y <== - mux.out[1]*2*in[3] + mux.out[1];  // Negate y if in[3] is one
}


template Segment(nWindows) {
    signal input {binary} in[nWindows*4];
    Point input {babyedwards} base;
    Point output {babyedwards} out;

    var i;
    var j;

    // Convert the base to montgomery

    component e2m = Edwards2Montgomery();
    e2m.pin <== base;

    component windows[nWindows];
    component doublers1[nWindows-1];
    component doublers2[nWindows-1];
    component adders[nWindows-1];
    for (i=0; i<nWindows; i++) {
        windows[i] = Window4();
        for (j=0; j<4; j++) {
            windows[i].in[j] <== in[4*i+j];
        }
        if (i==0) {
            windows[i].base <== e2m.pout;
        } else {
            doublers1[i-1] = MontgomeryDouble();
            doublers2[i-1] = MontgomeryDouble();
            doublers1[i-1].pin <== windows[i-1].out8;
            doublers2[i-1].pin <== doublers1[i-1].pout;

            windows[i].base <== doublers2[i-1].pout;

            adders[i-1] = MontgomeryAdd();
            if (i==1) {
                adders[i-1].pin1 <== windows[0].out;
            } else {
                adders[i-1].pin1 <== adders[i-2].pout;
            }
            adders[i-1].pin2 <== windows[i].pout;
        }
    }

    component m2e = Montgomery2Edwards();

    if (nWindows > 1) {
        m2e.pin <== adders[nWindows-2].pout;
    } else {
        m2e.pin <== windows[0].out;
    }

    out <== m2e.pout;
}

template Pedersen(n) {
    signal input {binary} in[n];
    Point output {babyedwards} pout;

    var BASE[10][2] = [
        [10457101036533406547632367118273992217979173478358440826365724437999023779287,19824078218392094440610104313265183977899662750282163392862422243483260492317],
        [2671756056509184035029146175565761955751135805354291559563293617232983272177,2663205510731142763556352975002641716101654201788071096152948830924149045094],
        [5802099305472655231388284418920769829666717045250560929368476121199858275951,5980429700218124965372158798884772646841287887664001482443826541541529227896],
        [7107336197374528537877327281242680114152313102022415488494307685842428166594,2857869773864086953506483169737724679646433914307247183624878062391496185654],
        [20265828622013100949498132415626198973119240347465898028410217039057588424236,1160461593266035632937973507065134938065359936056410650153315956301179689506],
        [1487999857809287756929114517587739322941449154962237464737694709326309567994,14017256862867289575056460215526364897734808720610101650676790868051368668003],
        [14618644331049802168996997831720384953259095788558646464435263343433563860015,13115243279999696210147231297848654998887864576952244320558158620692603342236],
        [6814338563135591367010655964669793483652536871717891893032616415581401894627,13660303521961041205824633772157003587453809761793065294055279768121314853695],
        [3571615583211663069428808372184817973703476260057504149923239576077102575715,11981351099832644138306422070127357074117642951423551606012551622164230222506],
        [18597552580465440374022635246985743886550544261632147935254624835147509493269,6753322320275422086923032033899357299485124665258735666995435957890214041481]

    ];

    var nSegments = ((n-1)\200)+1;

    component segments[nSegments];

    var i;
    var j;
    var nBits;
    var nWindows;
    for (i=0; i<nSegments; i++) {
        nBits = (i == (nSegments-1)) ? n - (nSegments-1)*200 : 200;
        nWindows = ((nBits - 1)\4)+1;
        segments[i] = Segment(nWindows);
        segments[i].base.x <== BASE[i][0];
        segments[i].base.y <== BASE[i][1];
        for (j = 0; j<nBits; j++) {
            segments[i].in[j] <== in[i*200+j];
        }
        // Fill padding bits
        for (j = nBits; j < nWindows*4; j++) {
            segments[i].in[j] <== 0;
        }
    }

    component adders[nSegments-1];

    for (i=0; i<nSegments-1; i++) {
        adders[i] = BabyAdd();
        if (i==0) {
            adders[i].p1 <== segments[0].out;
            adders[i].p2 <== segments[1].out;
        } else {
            adders[i].p1 <== adders[i-1].pout;
            adders[i].p2 <== segments[i+1].out;
        }
    }

/*
    coponent packPoint = PackPoint();

    if (nSegments>1) {
        packPoint.in[0] <== adders[nSegments-2].xout;
        packPoint.in[1] <== adders[nSegments-2].yout;
    } else {
        packPoint.in[0] <== segments[0].out[0];
        packPoint.in[1] <== segments[0].out[1];
    }

    out[0] <== packPoint.out[0];
    out[1] <== packPoint.out[1];
*/

    if (nSegments>1) {
        pout <== adders[nSegments-2].pout;
    } else {
        pout <== segments[0].out;
    }
}