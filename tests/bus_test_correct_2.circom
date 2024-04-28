pragma circom 2.0.0;

bus Point (dim) {
    signal {maxbit} x[dim];
}

bus Figure (N,dim) {
    Point(dim) list[N];
}

template Create (N,bitmax) {
    Figure(N) output fig(N,3);

    var c0 = 0;
    var c1 = 0;
    var c2 = 0;
    for (var i=0; i<N; i++) {
        fig.list[i].x.maxbit = bitmax;
        fig.list[i].x[0] <== c0;
        fig.list[i].x[1] <== c1;
        fig.list[i].x[2] <== c2;

        c0 = (c0+1)%(2**bitmax);
        c1 = (c1*c1+1)%(2**bitmax);
        c2 = (c0*c0+c1*c1)%(2**bitmax);
    }
}

component main = Create(10,5);