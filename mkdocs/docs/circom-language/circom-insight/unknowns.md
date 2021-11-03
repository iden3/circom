# Unknowns

The concept of unknown is very important in the constructive part of circom, where constraints are generated. In order to understand the compiler behavior we need to define what is considered unknown at compile time.

As already said, the content of a signal is always considered unknown, and only constant values or template parameters are considered known. A var whose value depends on unknowns is unknown. 

```text
template A(n1, n2){
   signal input in;
   signal input in2;
   var x;
   while(n1 > 0){
      x += in;
   }
}
```

Parameters `n1` and `n2in` and `in2` are unknown. Consequently, the value of var `x` is also considered unknown since it depends on the value of the unknown signal `in`. 

Similarly, any expression that depends on unknowns is considered unknown. Additionally, if an array is modified with an unknown expression in an unknown position then all positions of the array become unknown. 

```text
pragma circom 2.0.0;

template A(n1, n2){
   signal input in;
   signal output out;
   var array[n2];
   array[in] = in * n1;
   out <== array[in];
}

component main = A(1,2);
```

The previous code generates the next error message: _"Non-quadratic constraint was detected statically, using unknown index will cause the constraint to be non-quadratic"_.

Finally, the result of a function call with unknown parameters is unknown.

```text
pragma circom 2.0.0;

function F(n){
   return n*n;
}
template A(n1, n2){
   signal input in;
   signal output out;
   var end = F(in);
   var j = 0;
   for(var i = 0; i < end; i++){
   	 j += 2;
   }
   out <== j;
}
component main = A(1,2);
```

Var `end` is considered unknown since it is the result of a function `F` called with the unknown parameter `in`. The compilation of the previous code produces the error _"Non quadratic constraints are not allowed!_", since the value of `j` depends on the value of `end`. 

The key point for the compiler in the constructive phase is that the generation of constraints cannot depend on conditions (expressions) that are unknown.This is imposed on [all statements](../../control-flow). This can be seen both in the previous example and the next one:

```text
pragma circom 2.0.0;

template wrong(){
    signal input in;
    var y = 0;
    var i = 0;
    while(i < in){
        i++;
        y += y;
    }
    out <== y;
}

component main = wrong();
```

