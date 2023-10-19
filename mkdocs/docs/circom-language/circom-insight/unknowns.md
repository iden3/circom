# Unknowns

As expressions accepted during [constraint generation](../constraint-generation) can at most be quadratic only, certain checks and conditions are imposed on the use of unknown values at compile time.

In circom, **constant values** and **template parameters** are always considered known, while **signals** are always considered unknown.

Expressions depending only on knowns are considered knowns, while those depending on some unknowns are considered unknowns.

```text
pragma circom 2.0.0;

template A(n1, n2){ // known
   signal input in1; // unknown
   signal input in2; // unknown
   var x = 0; // known
   var y = n1; // known
   var z = in1; // unknown
}

component main = A(1, 2);
```

In the code above, the template parameters `n1`, `n2` and the constant value `0` are considered known. Consequently, the variables `x` and `y` are also considered known.


Meanwhile, the signals `in1`, `in2` are considered unknown. Consequently, the variable `z` is also considered unknown.

## Array

A constraint with an array access must have a known accessing position.

```text
pragma circom 2.0.0;

template A(n){
   signal input in;
   signal output out;
   var array[n];
   
   out <== array[in];
   // Error: Non-quadratic constraint was detected statically, using unknown index will cause the constraint to be non-quadratic
}

component main = A(10);
```

In the code above, an array is defined with a known size of value `n` (as template parameters are always considered known), while a constraint is set to be dependent on the array element at an unknown position `in` (as signals are always considered unknown).

An array must also be defined with a known size. 

```text
pragma circom 2.0.0;

template A(){
   signal input in;
   var array[in];
   // Error: The length of every array must known during the constraint generation phase
}

component main = A();
```

In the code above, an array is defined with an unknown size of value `in` (as signals are always considered unknown).

## Control Flow

If `if-else` or `for-loop`blocks have unknown conditions, then the block is considered unknown and no constraint can be generated inside it. Consequently, constraint can only be generated in a control flow with known conditions. 

Take an if-then statement as an example:

```text
pragma circom 2.0.0;

template A(){
   signal input in;
   signal output out;
   
   if (in < 0){
       // Error: There are constraints depending on the value of the condition and it can be unknown during the constraint generation phase
       out <== 0;
   }
}

component main = A();
```

In the code above, a constraint is defined in an if-then statement with a comparative condition involving an unknown value `in` (as signals are always considered unknown).

Similarly, using a for-loop as an example:

```text
pragma circom 2.0.0;

template A(){
   signal input in;
   signal output out;
   
   for (var i = 0; i < in; i++){
       // Error: There are constraints depending on the value of the condition and it can be unknown during the constraint generation phase
       out <== i;
   }
}

component main = A();
```

In the code above, a constraint is defined in a for-loop with a counting condition to an unknown value `in` (as signals are always considered unknown).

For additional details, see [Control Flow](../../control-flow).
