# Compiler Messages



The compiler messages are basically of three kinds: hints, warnings and errors.

## A hint

This message means that it is allowed but uncommon, and hence it is better to check if it was done on purpose.

## A warning

This message means that it is allowed but should not happen in general.

For instance, if a signal is not used in any constraint, a warning message will be generated (when compiling the program with the `--inspect` option). Moreover, if it is an input signal x, then the compiler would suggest adding a constraint of the form x \* 0 === 0;

```text
pragma circom 2.0.0;

template A(N){
   signal input in;
   signal intermediate;
   signal output out;
   intermediate <== 1;
   out <== intermediate;
}
component main {public [in]} = A(1);
```

## An error

This message means that it is not allowed and the compilation of the program fails. For instance, one of the most common errors we can make when starting to program in circom is trying to assign a value to a signal using `=`.

```text
pragma circom 2.0.0;

template A(){
  signal in;
  in = 1;
}

component main = A();
```

The compilation fails and the next error is received: _"Assignee and assigned types do not match operator."_ 



