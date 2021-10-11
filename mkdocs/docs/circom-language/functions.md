# Functions

In circom, functions define generic abstract pieces of code that can perform some computations to obtain a value or an expression to be returned.

```text
function funid ( param1, ... , paramn ) {

 .....

 return x;
}
```

Functions compute numeric (or arrays of) values or expressions. Functions can be recursive. Consider the [next function](https://github.com/iden3/circomlib/blob/master/circuits/binsum.circom) from the circom library.

```text
/*
    This function calculates the number of extra bits 
    in the output to do the full sum.
 */

function nbits(a) {
    var n = 1;
    var r = 0;
    while (n-1<a) {
        r++;
        n *= 2;
    }
    return r;
}
```

 Functions cannot declare signals or generate constraints (use templates if you need so). The next function produces the error message: "Template operator found".

```text
function nbits(a) {
    signal input in; //This is not allowed.
    var n = 1;
    var r = 0;
    while (n-1<a) {
        r++;
        n *= 2;
    }
    r === a; //This is also not allowed.
    return r;
}
```

As usual, there can be many return statements, but every execution trace must end in a return statement (otherwise, a compile error will be produced). The execution of the return statement returns the control to the caller of the function. 

```text
function example(N){
	 if(N >= 0){ return 1;}
//	 else{ return 0;}
}
```

The compilation of function `example` produces the next error message: "In example there are paths without return".

