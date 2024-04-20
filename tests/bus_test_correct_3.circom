pragma circom 2.0.0;

bus Correct {
    signal {binary} correct;
}

bus RecursiveArray (N) {
    Correct array[N];
    RecursiveArray(N-1) rec;
}

template Create (N) {
    RecursiveArray(N) output out;

    component create_rec = Create(N-1);

    for (var i=0; i<n; i++) {
        out.array[i].correct <== 1;
    }
    out.rec <== create_rec.out;
}

component main = Create(2);