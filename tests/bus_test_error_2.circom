pragma circom 2.0.0;

/*

    This code should fail because bus fields are unknown at compilation time.
    They cannot be used to declare other buses or templates.

*/

bus Vector (n) {
    signal {integer} dim;
    signal {real} x[n];
}

template FirstRowSum(n) {
    Vector(n) input v;
    signal output sum;

    var length = n/v.dim;
    Vector(length) w;
    w.dim <== 1;
    var s = 0;

    for (var i=0; i<length; i++) {
        w.x[i] <== v.x[i];
        s += w.x[i];
    }

    sum <== s;
}