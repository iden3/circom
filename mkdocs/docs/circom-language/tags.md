#Signal Tags
circom 2.1.0 introduces a new feature called __signal tags__. Tags can be defined during the declaration of any signal in a template. The tag list is indicated between brackets right before the signal name.

```
signal (input/output) {tag_1,...,tag_n} signalname;
```

Let us see a first well-known example in the circomlib, where the tag is declared in an input signal:

```
template Bits2Num(n) {
    signal input {binary} in[n];
    signal output out;
    var lc1=0;

    var e2 = 1;
    for (var i = 0; i<n; i++) {
        lc1 += in[i] * e2;
        e2 = e2 + e2;
    }

    lc1 ==> out;
}

template A(){
    ...
    component b = Bits2Num(10);
    b.in <== a;
    ...
}
```
The input array `in` is declared with the tag `binary`. This tag means that each signal in the array is always expected to be `0`or `1`, in order to compute the corresponding number correctly. 

Then, whenever the previous template is instantiated, the compiler checks if the array  `a` assigned to the input array has the tag binary, since `in` has the tag `binary` in its declaration. If it does not, an error is reported. Notice that the compiler also checks if both arrays have the same size. 

It is important to highlight that the compiler does never make any check about the validity of the tags. It is the programmer's responsability to include the constraints and executable code to guarantee that the inteded meaning of each signal is always true.

Let us consider another well-known template that the programmer can use to guarantee that the output signal is always binary. 

```
template IsZero() {
    signal input in;
    signal output {binary} out;
    signal inv;
    inv <-- in!=0 ? 1/in : 0;
    out <== -in*inv +1;
    in*out === 0;
}
```

To the light of this example, when using tags in intermediate or output signals, the programmer must use components like the previous one or explicitly include the constraints to guarantee the validity of the tags.

### Tags with value
Notice that in the previous template `Bits2Num`, we can add more information about the output signal `out`: the maximum number of bits needed to represent it is `n`. To express this fact is necessary that tags can have a value.

The value of the tag can be accessed using the notation `.` at any moment as a part of an arithmetic expression. However, if the tag has not been previously initialized, then the compiler reports an error. 

The value of the tag can be also modified using the notation `.`, as long as the corresponding signal has not received any value. Valued tags behave like parameters which means that they can only be assigned to values known at compilation time.

Let us modify the previous example to include this tag in the template.

```
template Bits2Num(n) {
    signal input {binary} in[n];
    signal output {maxbit} out;
    var lc1=0;

    var e2 = 1;
    for (var i = 0; i<n; i++) {
        lc1 += in[i] * e2;
        e2 = e2 + e2;
    }
    out.maxbit = n;
    lc1 ==> out;
}
```

On the other hand, the next code is erroneous since the tag value is modified after the output receives its value.
```
template Bits2Num(n) {
    ...
    lc1 ==> out;
    out.maxbit = n;
}
```

###Tags in signal arrays
Every signal in an array has exactly the same tag value. Then, the tag is accessed directly from the array name instead of accessing from a particular signal in the array.  Similarly to the previous erroneous example: if a particular position of the array is modified, then the tag value of the whole array cannot be modified at all. 