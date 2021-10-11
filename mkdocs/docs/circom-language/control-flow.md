# Control Flow

We have standard constructions for defining the control flow of the program.

## Conditional statement: if-then-else

**if ( boolean_condition ) block_of_code else block_of_code**

The else part is optional. When omitted, it means “else do nothing”.

```text
var x = 0;
var y = 1;
if (x >= 0) {
   x = y + 1;
   y += 1;
} else {
   y = x;
}
```

## Loop statement: for

**for ( initialization_code ; boolean_condition ; step_code ) block_of_code**

If the initialization_code includes a var declaration then its scope is reduced to the for statement and hence, using it later on (without defining it again) will produce a compilation error.

```text
var y = 0;
for(var i = 0; i < 100; i++){
    y++;
}
```

## Loop statement: while

**while ( boolean_condition ) block_of_code**

It executes the block of code while the condition holds. The condition is checked every time before executing the block of code.

```text
var y = 0;
var i = 0;
while(i < 100){
    i++;
    y += y;
}
```

**Important**: when constraints are generated in any block inside an if-then-else or loop statement, the condition cannot be unknown (see [Unknowns](../circom-insight/unknowns)). This is because the constraint generation must be unique and cannot depend on unknown input signals.

In case the expression in the condition is unknown and some constraint is generated, the compiler will generate the next error message: "_There are constraints depending on the value of the condition and it can be unknown during the constraint generation phase_".

```text
pragma circom 2.0.0;

template A(){}
template wrong(N1){
    signal input in;
    component c;
    if(in > N1){
      c = A();
    }
}
component main {public [in]} = wrong(1);
```

In this example, the condition depends on the input signal `in` whose value is unknown at compilation time.

Let us also notice that if the body of the statement does not involve any signal or component; or a constraint does not depend on a value involved with unknown values, then the compilation will succeed as it can be seen in the next example.

```text
template right(N){
    signal input in;
    var x = 2;
    var t = 5;
    if(in > N){
      t = 2;
    }
}
```

This template is correct, since no constraint depends on the unknown value of `in`.

```text
template right(N1,N2){
    signal input in;
    var x = 2;
    var t = 5;
    if(N1 > N2){
      t = 2;
    }
    x === t;
}
```

This template is correct since the values of variables involved in the constraint only depend on known values of parameter `N1` and `N2`.

**Important**: Another compilation error is generated when the content of a var depends on some unknown condition: that is when the var takes its value inside an if-then-else or loop statement with an unknown condition. Then, the content of the variable is a non-quadratic expression and, as such, cannot be used in the generation of a constraint.

```text
template wrong(){
    signal input in;
    var x; 
    var t = 5;
    if(in > 3){
      t = 2;
    }
    x === t;
}
```

This template produces a compilation error, since the value of variable `t` involved in the last constraint depends on the unknown value of variable `in`.

The control flow of the computations is like in other imperative languages, but the [instantiation of components](../templates-and-components) may not follow the sequential structure of the code because component instantiation will not be triggered until all input signals have a concrete value assigned.

```text
template mult(){
  signal input in[2];
  signal output out;
  out <== in[0] * in[1];
}

template mult4(){
  signal input in[4];
  component comp1 = mult();
  component comp2 = mult();
  comp1.in[0] = in[0];
  comp2.in[0] = in[1];
  comp2.in[1] = in[2];
  comp1.in[1] = in[3];
}
```

In this example, `comp2`is instantiated before `comp1`, since `comp2`'s input signals have concrete values before `comp1`'s input signals. Consequently, `comp2.out` obtains the value after the execution of line 13, whereas `comp1.out` obtains it after the execution of line 14.

