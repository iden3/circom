pragma circom 2.0.0;

/*

    This code should fail because field accesses and array accesses do not match
    with the types and dimensions of the expresions.

*/

bus A () {
    signal {tag1} a;
}

bus B {
    A b1, b2;
}

bus C (n) {
    A () a;
    B () {tag2} b[n];
}

bus D (array,m,n) {
    A a1;
    A {tag3} a2;
    B b1[array[0]], b2[array[1]];
    C(m) {tag4} c1[n], c2[n];
}

template Main (m,n) {
    C(m) input c[n];
    A output {tag1} a;

    var array[2] = [2,4];
    var s = 0;

    D(array,m,n) {tag5} d;

    d.c1[0] <== c;

    for (var i=0; i<m; i++) {
        d.c2.a.a.tag1 = 0;
        d.c2[i].b.b1.a <== c[i].b[0].b2.a;
    }
}

component main {public [c]} = Main(2,3);