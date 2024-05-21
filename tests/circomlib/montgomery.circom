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
 
 // The templates and functions of this file only work for prime field bn128 (21888242871839275222246405745257275088548364400416034343698204186575808495617)
 

/*
    Source: https://en.wikipedia.org/wiki/Montgomery_curve

*/

bus Point {
    signal x,y;
}

/*
*** Edwards2Montgomery(): template that receives an input pin representing a point of an elliptic curve in Edwards form
                          and returns the equivalent point in Montgomery form.
        - Inputs: pin -> bus representing a point of the curve in Edwards form
        - Outputs: pout -> bus representing a point of the curve in Montgomery form
         
    Example: if we consider the input pin = (x, y), then the circuit produces the following output pout = (u, v).
    
                1 + y       1 + y
    (u, v) = [ -------  , ---------- ]
                1 - y      (1 - y)x
    
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
*** Montgomery2Edwards(): template that receives an input pin representing a point of an elliptic curve in Montgomery form
                          and returns the equivalent point in Edwards form.
        - Inputs: pin -> bus representing a point of the curve in Montgomery form
        - Outputs: pout -> bus representing a point of the curve in Edwards form
         
    Example: if we consider the input pin = (u, v), then the circuit produces the following output pout = (x, y)
    
                u    u - 1
    (x, y) = [ ---, ------- ]
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
*** MontgomeryAdd(): template that receives two inputs pin1, pin2 representing points of the Baby Jubjub curve in Montgomery form
                     and returns the addition of the points.
        - Inputs: pin1 -> bus representing a point of the curve in Montgomery form
                  pin2 -> bus representing a point of the curve in Montgomery form
        - Outputs: pout -> bus representing the point pin1 + pin2 in Montgomery form
         
    Example: if we consider the inputs pin1 = (x1, y1) and pin2 = (x2, y2), then the circuit produces the following output pout = (x3, y3):

             y2 - y1
    lamda = ---------
             x2 - x1

    x3 = B * lamda^2 - A - x1 -x2

    y3 = lamda * ( x1 - x3 ) - y1
    
    where A and B are two constants defined below. 
 */

template MontgomeryAdd() {
    Point input {babymontgomery} pin1, pin2;
    Point output {babymontgomery} pout;

    var a = 168700;
    var d = 168696;

    var A = (2 * (a + d)) / (a - d);
    var B = 4 / (a - d);

    signal lamda;

    lamda <-- (pin2.y - pin1.y) / (pin2.x - pin1.x);
    lamda * (pin2.x - pin1.x) === pin2.y - pin1.y;

    pout.x <== B*lamda*lamda - A - pin1.x - pin2.x;
    pout.y <== lamda * (pin1.x - pout.x) - pin1.y;
}

/*
*** MontgomeryDouble(): template that receives an input pin representing a point of the Baby Jubjub curve in Montgomery form
                        and returns the point 2 * pin.
        - Inputs: pin -> bus representing a point of the curve in Montgomery form
        - Outputs: pout -> bus representing the point 2*pin in Montgomery form
         
         
    Example: if we consider the input pin = (x1, y1), then the circuit produces the following output pout = (x2, y2):

    x1_2 = x1*x1

             3*x1_2 + 2*A*x1 + 1
    lamda = ---------------------
                   2*B*y1

    x2 = B * lamda^2 - A - x1 -x1

    y2 = lamda * ( x1 - x2 ) - y1

 */
 
template MontgomeryDouble() {
    Point input {babymontgomery} pin;
    Point output {babymontgomery} pout;

    var a = 168700;
    var d = 168696;

    var A = (2 * (a + d)) / (a - d);
    var B = 4 / (a - d);

    signal lamda;
    signal x1_2;

    x1_2 <== pin.x * pin.x;

    lamda <-- (3*x1_2 + 2*A*pin.x + 1) / (2*B*pin.y);
    lamda * (2*B*pin.y) === (3*x1_2 + 2*A*pin.x + 1);

    pout.x <== B*lamda*lamda - A - 2*pin.x;
    pout.y <== lamda * (pin.x - pout.x) - pin.y;
}