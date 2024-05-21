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
pragma circom 2.1.5;

include "bitify.circom";
include "montgomery.circom";
include "escalarmulfix.circom";

// The templates and functions of this file only work for finite field F_p = bn128,
// with the prime number p = 21888242871839275222246405745257275088548364400416034343698204186575808495617.

/*
    The Baby-Jubjub Montgomery elliptic curve defined over the finite field bn128 is given by the equation

    B*v^2 = u^3 + A*u^2 + u, A = 168698, B = 1

    This curve is birationally equivalent to the twisted Edwards elliptic curve

    a*x^2 + y^2 = 1 + d*x^2*y^2, a = 168700 = (A+2)/B, d = 168696 = (A-2)/B

                                    u     u-1                                     1+y       1+y
    via the map (u,v) -> (x,y) = [ --- , ----- ] with inverse (x,y) -> (u,v) = [ ----- , --------- ]
                                    v     u+1                                     1-y     (1-y)*x

    Since a is not a square in bn128, the twisted Edwards curve is a quadratic twist of the Edwards curve
    
    x'^2 + y'^2 = 1 + d'*x'^2*y'^2

    via the map (x,y) -> (x',y') = [ sqrt(a)*x , y ]

    where d' = 9706598848417545097372247223557719406784115219466060233080913168975159366771.
    We will be working with the twisted Edwards form of the Baby-Jubjub curve because the algorithms for adding, doubling
    and multiplying points by a scalar are faster this way. 
*/

/*
*** BabyAdd(): template that receives two points of the Baby-Jubjub curve in twisted Edwards form and returns the addition of the points.
        - Inputs: p1 = (x1, y1) -> bus representing a point of the curve in twisted Edwards form
                  p2 = (x2, y2) -> bus representing a point of the curve in twisted Edwards form
        - Outputs: pout = (xout, yout) -> bus representing a point of the curve in twisted Edwards form, pout = p1 + p2

    Twisted Edwards Addition Law:
                                               x1*y2 + y1*x2         y1*y2 - a * x1*x2
    [xout, yout] = [x1, y1] + [x2, y2] = [ --------------------- , --------------------- ]
                                            1 + d * x1*x2*y1*y2     1 - d * x1*x2*y1*y2     
    
*/

template BabyAdd() {
    Point input {babyedwards} p1,p2;
    Point output {babyedwards} pout;

    signal beta;
    signal gamma;
    signal delta;
    signal tau;

    var a = 168700;
    var d = 168696;

    beta <== p1.x*p2.y;
    gamma <== p1.y*p2.x;
    delta <== (-a*p1.x + p1.y)*(p2.x + p2.y);
    tau <== beta * gamma;

    pout.x <-- (beta + gamma) / (1 + d*tau);
    (1 + d*tau) * pout.x === (beta + gamma);

    pout.y <-- (delta + a*beta - gamma) / (1 - d*tau);
    (1 - d*tau)*pout.y === (delta + a*beta - gamma);
}



/*
*** BabyDouble(): template that receives a point pin of the Baby-Jubjub curve in twisted Edwards form and returns the point 2 * pin.
        - Inputs: pin = (x1, y1) -> bus representing a point of the curve in twisted Edwards form
        - Outputs: pout = (x2, y2) -> bus representing a point of the curve in twisted Edwards form, 2 * pin = pout
         
    Twisted Edwards Doubling Law: 2 * [x, y] = [x, y] + [x, y]
    
*/

template BabyDbl() {
    Point input {babyedwards} pin;
    Point output {babyedwards} pout;

    component adder = BabyAdd();

    adder.p1 <== pin;
    adder.p2 <== pin;
    adder.pout ==> pout;
}


/*
*** BabyCheck(): template that receives an input point pin and checks if it belongs to the Baby-Jubjub curve in twisted Edwards form.
        - Inputs: pin = (x1, y1) -> bus representing the point that we want to check
        - Outputs: pout = (x2, y2) -> two field values representing the same point as the input but with the babyedwards tag
                                      attached to point out it is a point of the Baby-Jubjub curve in twisted Edwards form
        
    The set of solutions of BabyCheck()(p) are the points of the Baby-Jubjub curve in twisted Edwards form.
    They must fulfil the equation a*x^2 + y^2 = 1 + d*x^2*y^2, a = 168700, d = 168696.
    
*/


template BabyCheck() {
    Point input pin;
    Point output {babyedwards} pout;

    // Point p2;
    signal x2;
    signal y2;

    var a = 168700;
    var d = 168696;

    x2 <== pin.x*pin.x; //x2 = pin.x^2
    y2 <== pin.y*pin.y; //y2 = pin.y^2
    
    a*x2 + y2 === 1 + d*x2*y2;
    
    pout <== pin; 
}


/*
*** BabyPbk(): template that receives an input in representing a value in the prime subgroup with order
               r = 2736030358979909402780800718157159386076813972158567259200215660948447373041,
               and returns the point of the Baby-Jubjub curve in twisted Edwards form in * P, with P being the point
               P = (5299619240641551281634865583518297030282874472190772894086521144482721001553, 16950150798460657717958625567821834550301663161624707787222815936182638968203)

This template is used to extract the public key from the private key.
        - Inputs: in -> field value in [1,r-1]
        - Outputs: A = (x, y) -> two field values representing a point of the curve in Edwards form, in * P = A
    
*/

template BabyPbk() {
    signal input {minvalue, maxvalue} in;
    Point output {babyedwards} A;


    var r = 2736030358979909402780800718157159386076813972158567259200215660948447373041;
    assert(in.minvalue > 0 && in.maxvalue < r);
    var BASE8[2] = [
        5299619240641551281634865583518297030282874472190772894086521144482721001553,
        16950150798460657717958625567821834550301663161624707787222815936182638968203
    ];

    component pvkBits = Num2Bits(253);
    pvkBits.in <== in;

    component mulFix = EscalarMulFix(253, BASE8);

    mulFix.e <== pvkBits.out;
    A <== mulFix.out;
}