pragma circom 2.0.0;

/*

    This code should fail because bus fields are unknown at compilation time.
    They cannot be used to declare other buses or templates.

*/

bus Vector (n) {
    signal {integer} dim;
    signal {floating} x[n];
}

template FirstRowSum(n) {
    signal output sum;

    Vector(n) v;
    v.dim <== 6;
    var length = n/v.dim;
    
    var s = 0;
    for (var i=0; i<length; i++) {
        v.x[i] <== i;
        s += v.x[i];
    }

    sum <== s;
}

component main = FirstRowSum(24);