---
description: >-
  Here you can find information about the --inspect option and how to solve the warnings.
---
# Improving security of circuits by using --inspect option 

When using --inspect option, the compiler searches for signals that do not appear in any constraint. In case it finds some, then it throws a warning to let the programmer know those unconstrained signals. The compiler also throws a warning when some input or output signal of a subcomponent in a template do not appear in any onstraint of the father component. To avoid these warnings, the compiler could use the underscore notation '_' to inform the compiler that such a situation is expected. Let us see several cases where we can find that situation.

- The compiler throws a warning if a signal defined in a template do not appear in any constraint of such template.

```
template B(n){
  signal input in;
  signal input out;
  out <== in + 1;
}
template A(n){
      signal aux;
      signal out;
      if(n == 2){
	aux <== 2;
        out <== B()(aux);
      }
      else{
	 out <== 5;
      }
}

component main = A(3);
```

In this example, `aux` is only used in the `if` branch. Thus, for the main component (with `n = 3`) , `aux` remains unconstrained and the compiler throws a warning:

```warning[CA01]: In template "A(3)": Local signal aux does not appear in any constraint```

 To avoid the warning, we can add inside the `else` branch, the instruction `_ <== aux;` to indicate the compiler that aux is not used in this case.
 Since `circom 2.1.5`, we can also define signals inside `if` blocks with conditions known at compilation time and thus, use this feature to solve the previous warning as follows:
 ```
template A(n){
      signal out;
      if(n == 2){
        signal aux <== 2;
        out <== B()(aux);
      }
      else{
        out <== 5;
      }
}
 ```

- Another case when throwing a warning is using subcomponents inside a template: input and output signals of each subcomponent in a template should appear at least in one constraint of the father component.
However, it is very common the use of subcomponents to check some property but ignoring the output of the subcomponent.

To illustrate this, let us consider the well-known template `Num2Bits(n)` from the circomlib. This template receives an input signal and a parameter `n` which represents a number of bits and returns and output signal array with n elements, the binary representation of the input. 

```
include "bitify.circom";

template check_bits(n){
	signal input in;
        component check = Num2Bits(n);
        check.in <== in;
}

component main = check_bits(10);
```

It is very common to use the `Num2Bits` template to check if `in` can be represented with `n`bits. In this case, the main component checks if the value of `in` can be represented using 10 bits, and it works as expected. The constraints introduced by the subcomponent check will guarantee that the R1CS system only has a solution if `in` can be represented with 10 bits, but it does not matter which is the specific representation. However, the compiler throws the next warning:
```
In template "check_bits(10)": Array of subcomponent input/output signals check.out contains a total 
of 10 signals that do not appear in any constraint of the father component = For example: check.out[0], check.out[1].
```
 

Since we are not interested in the binary representation, the template does not make use of array signal `check.out`. Thus, we should add `_ <== check.out` to inform that the binary representation is irrelevant and avoid the warning.

Notice also here that the `--inspect option also shows the parameter of the instance that causes a warning (`check_bits(10)`). In general, we throw as many warnings as instances with different parameters for each template.

- In the previous example, we have seen that none of the positions of array `check.out` are used, and the warning indicates some of the unused positions. Thus, if some of the positions are used, and others are not, the compiler also notifies some of the unused positions. 

```
include "bitify.circom";

template parity(n){
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

To fix this example, we can add ` _ <== check.out` at the end of the template to let the compiler know that the remaining positions are irrelevant.

- Finally, the `--inspect` option also searches for assignments with operator `<--` that can be transformed into assignments with operator `<==`, which automatically include the corresponding constraint to guarantee the code is correct. A typical scenario of this situation is shown below:

```
 out <-- in / 4;
 out*4 === in;
```

Here, many circom programmers avoid the use of `<==`, since they are using the `/` operator which in many cases turn the expression in non-quadratic. Then, programmer must add the corresponding constraint using `===` to guarantee the code is correct. However, it is important to notice that the inverse of 4 is another field element (which is computed by the compiler), and thus, `in / 4` is a linear expression. Consequently, the previous instructions can be replaced by `out <== in / 4`. In these cases, the compiler suggests to use `<==` instead of `<--`.


