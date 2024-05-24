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
 
// The templates and functions of this file only work for finite field F_p = bn128,
// with the prime number p = 21888242871839275222246405745257275088548364400416034343698204186575808495617. 

/*
    Source: https://en.wikipedia.org/wiki/Montgomery_curve

*/

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
    Here circuits are provided to transform a point of the Baby-Jubjub curve in twisted Edwards to its Montgomery form and vice versa.
    Circuits to add and double points of the Baby-Jubjub Montgomery curve are provided as well.
*/

/*
    spec tag babyedwards: 168700*(p.x)^2 + (p.y)^2 = 1 + 168696*(p.x)^2*(p.y)^2
    spec tag babymontgomery: (p.y)^2 = (p.x)^3 + 168698*(p.x)^2 + p.x
*/

bus Point {
    signal x,y;
}

/*
*** Edwards2Montgomery(): template that receives a point of the Baby-Jubjub curve in twisted Edwards form
                          and returns the equivalent point in Montgomery form.
        - Inputs: pin -> bus representing a point of the Baby-Jubjub curve in twisted Edwards form
        - Outputs: pout -> bus representing a point of the Baby-Jubjub curve in Montgomery form
         
    Map from twisted Edwards elliptic curve to its birationally equivalent Montgomery curve:
    
                          1 + y        1 + y
    (x, y) -> (u, v) = [ -------  , ----------- ]
                          1 - y      (1 - y)*x
    
*/

template Edwards2Montgomery() {
    Point input {babyedwards} pin;
    Point output {babymontgomery} pout;

    pout.x <-- (1 + pin.y) / (1 - pin.y);
    pout.y <-- pout.x / pin.x;

    pout.x * (1 - pin.y) === (1 + pin.y);
    pout.y * pin.x === pout.x;
}

/*
*** Montgomery2Edwards(): template that receives an input pin representing a point of the Baby-Jubjub curve in Montgomery form
                          and returns the equivalent point in twisted Edwards form.
        - Inputs: pin -> bus representing a point of the Baby-Jubjub curve in Montgomery form
        - Outputs: pout -> bus representing a point of the curve Baby-Jubjub in twisted Edwards form
         
    Map from Montgomery elliptic curve to its birationally equivalent twisted Edwards curve:
    
                          u    u - 1
    (u, v) -> (x, y) = [ ---, ------- ]
                          v    u + 1

 */

template Montgomery2Edwards() {
    Point input {babymontgomery} pin;
    Point output {babyedwards} pout;

    pout.x <-- pin.x / pin.y;
    pout.y <-- (pin.x - 1) / (pin.x + 1);

    pout.x * pin.y === pin.x;
    pout.y * (pin.x + 1) === pin.x - 1;
}


/*
*** MontgomeryAdd(): template that receives two inputs pin1, pin2 representing points of the Baby-Jubjub curve in Montgomery form
                     and returns the addition of the points.
        - Inputs: pin1 -> bus representing a point of the Baby-Jubjub curve in Montgomery form
                  pin2 -> bus representing a point of the Baby-Jubjub curve in Montgomery form
        - Outputs: pout -> bus representing the point pin1 + pin2 of the Baby-Jubjub curve in Montgomery form
         
    Montgomery Addition Law:

                                            y2 - y1                        y2 - y1                           y2 - y1
    [x3, y3] = [x1, y1] + [x2, y2] = [ B*( --------- )^2 - A - x1 - x2, ( --------- )*(A + 2*x1 + x2) - B*( --------- )^3 - y1 ]
                                            x2 - x1                        x2 - x1                           x2 - x1

             y2 - y1
    lamda = ---------
             x2 - x1

    x3 = B*lamda^2 - A - x1 -x2

    y3 = lamda*( x1 - x3 ) - y1
*/

template MontgomeryAdd() {
    Point input {babymontgomery} pin1, pin2;
    Point output {babymontgomery} pout;

    var A = 168698;
    var B = 1;

    signal lamda;

    lamda <-- (pin2.y - pin1.y) / (pin2.x - pin1.x);
    lamda * (pin2.x - pin1.x) === pin2.y - pin1.y;

    pout.x <== B*lamda*lamda - A - pin1.x - pin2.x;
    pout.y <== lamda * (pin1.x - pout.x) - pin1.y;
}

/*
*** MontgomeryDouble(): template that receives an input pin representing a point of the Baby-Jubjub curve in Montgomery form
                        and returns the point 2 * pin.
        - Inputs: pin -> bus representing a point of the Baby-Jubjub curve in Montgomery form
        - Outputs: pout -> bus representing the point 2*pin of the Baby-Jubjub curve in Montgomery form
         
         
    Montgomery Doubling Law:

                                   3*x1^2 + 2*A*x1 + 1                        3*x1^2 + 2*A*x1 + 1                           3*x1^2 + 2*A*x1 + 1
    [x2, y2] = 2*[x1, y1] = [ B*( --------------------- )^2 - A - x1 - x2, ( --------------------- )*(A + 2*x1 + x2) - B*( --------------------- )^3 - y1 ]
                                         2*B*y1                                     2*B*y1                                        2*B*y1

    x1_2 = x1*x1

             3*x1_2 + 2*A*x1 + 1
    lamda = ---------------------
                   2*B*y1

    x2 = B*lamda^2 - A - x1 -x1

    y2 = lamda*( x1 - x2 ) - y1

 */
 
template MontgomeryDouble() {
    Point input {babymontgomery} pin;
    Point output {babymontgomery} pout;

    var A = 168698;
    var B = 1;

    signal lamda;
    signal x1_2;

    x1_2 <== pin.x * pin.x;

    lamda <-- (3*x1_2 + 2*A*pin.x + 1) / (2*B*pin.y);
    lamda * (2*B*pin.y) === (3*x1_2 + 2*A*pin.x + 1);

    pout.x <== B*lamda*lamda - A - 2*pin.x;
    pout.y <== lamda * (pin.x - pout.x) - pin.y;
}