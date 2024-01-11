---
description: >-
  Here you can find information about the --inspect option and how to solve the warnings.
---
# Improving security of circuits by using --inspect option 

When using --inspect option, the compiler searches for signals that may be underconstrained. In case it finds some, it throws a warning to let the programmer know which are those potentially underconstrained signals. For instance, the compiler throws a warning when some input or output signal of a subcomponent in a template do not appear in any constraint of the father component. In case this is intended, the programmer can use the underscore notation '_' to inform the compiler that such a situation is as expected. A warning is also shown when a signal is not used in any constraint in the component it belongs to. Let us see several cases where we can find that situation. 

1) The compiler throws a warning if a signal defined in a template does not appear in any constraint of such template for the given instantiation.

```
template B(n) {
  signal input in;
  signal input out;
  out <== in + 1;
}

template A(n) {
  signal aux;
  signal out;
  if(n == 2) {
    aux <== 2;
    out <== B()(aux);
  } else {
    out <== 5;
  }
}

component main = A(3);
```

In this example, `aux` is only used in the `if` branch. Thus, for the main component (with `n = 3`) , `aux` remains unconstrained and the compiler throws a warning:

```warning[CA01]: In template "A(3)": Local signal aux does not appear in any constraint```

To avoid the warning, we can add the instruction `_ <== aux;` inside the `else` branch. This indicates to the compiler that `aux` is not used in this case.
```
template A(n) {
  signal aux;
  signal out;
  if(n == 2) {
    aux <== 2;
    out <== B()(aux);
  } else {
    _ <== aux;
    out <== 5;
  }
}
```

Alternatively, since `circom 2.1.5`, we can also define signals inside `if` blocks with conditions known at compilation time and thus, we can use this feature to solve the previous warning as follows:

```
template A(n) {
  signal out;
  if(n == 2) {
    signal aux <== 2;
    out <== B()(aux);
  } else {
    out <== 5;
  }
}
```

- Another case where a warning is thrown is when using subcomponents inside a template, since it is required that every input and output signal of each subcomponent in a template should appear in at least one constraint of the father component.

Although this is the common case, specially for inputs, there are cases where some of the outputs of the subcomponent are ignored on purpose as the component is only used to check some properties. To illustrate this, let us consider the well-known template `Num2Bits(n)` from the circomlib. This template receives an input signal and a parameter `n` which represents a number of bits and returns an output signal array with `n` elements, the binary representation of the input. 

```
include "bitify.circom";

template check_bits(n) {
  signal input in;
  component check = Num2Bits(n);
  check.in <== in;
}

component main = check_bits(10);
```

It is quite common to use the `Num2Bits` template just to check if `in` can be represented with `n`bits. In this case, the main component checks if the value of `in` can be represented using 10 bits, and it works as expected. The constraints introduced by the subcomponent `check` will guarantee that the R1CS system only has a solution if `in` can be represented with 10 bits, but it does not matter which is the specific representation. However, the compiler throws the next warning:

```
In template "check_bits(10)": Array of subcomponent input/output signals check.out contains a total 
of 10 signals that do not appear in any constraint of the father component = For example: check.out[0], check.out[1].
```

Since we are not interested in the binary representation, the template does not make use of array signal `check.out`. Thus, we should add `_ <== check.out` to inform that the binary representation is irrelevant and avoid the warning.

```
template check_bits(n) {
  signal input in;
  component check = Num2Bits(n);
  check.in <== in;
  _ <== check.out;
}
```

or even using anonymous components we can write

```
template check_bits(n){
  signal input in;
  _ <== Num2Bits(n)(in);
}
```

Notice also here that the `--inspect` option also shows the parameter of the instance that causes a warning (`check_bits(10)`). In general, we throw as many warnings as instances with different parameters for each template.

- In the previous example, we have seen that none of the positions of array `check.out` are used, and the warning indicates some of the unused positions. Thus, if some of the positions are used, and others are not, the compiler also notifies some of the unused positions. 

```
include "bitify.circom";

template parity(n) {
  signal input in;
  signal output out;
  component check = Num2Bits(n);
  check.in <== in;
  out <== check.out[0];
}

component main = parity(10);
```

In this case, we are again using a component `Num2Bits(10)` to get the binary representation of signal `in`, but we are only interested in the least-significant bit to know its parity. Then, the warning throws the next warning: 

```
In template "parity(10)": Array of subcomponent input/output signals check.out contains a total of 9 signals 
that do not appear in any constraint of the father component. = For example: check.out[1], check.out[2].
```

To fix this example, we can either add for loop at the end of the template to indicate those positions that are intendedly not used

```
for (var i = 1; i < n; i++) {
  _ <== check.out[i];
}
```

or simply add ` _ <== check.out` at the end of the template to let the compiler know that the remaining positions are irrelevant (as this is not going to affect to the use of check.out[0]).

- Finally, the `--inspect` option also searches for assignments with operator `<--` that can be transformed into assignments with operator `<==`, which automatically include the corresponding constraint to guarantee the code is correct. A typical scenario of this situation is shown below:

```
out <-- in / 4;
out*4 === in;
```

Here, many circom programmers avoid the use of `<==`, since they are using the `/` operator which in many cases turn the expression in non-quadratic. Then, programmers must add the corresponding constraint using `===` to guarantee the code is correct. However, it is important to notice that the inverse of 4 is another field element (which is computed by the compiler), and thus, `in / 4` is a linear expression. Consequently, the previous instructions can be replaced by `out <== in / 4`. In these cases, the compiler suggests to use `<==` instead of `<--`.

