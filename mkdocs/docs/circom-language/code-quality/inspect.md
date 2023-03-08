Improving security of circuits by using --inspect option 

When using --inspect option, the compiler searches signals that do not appear in constraints. In case, it finds one, then it throws a warning to let the programmer know those unconstrained signals. To avoid such a warning, the compiler could use the underscore notation '_' to inform the compiler that such a situation is expected. Let us see several cases where we can find that situation.

- The compiler throws a warning if a signal defined in a template do not appear in any constraint of such template.

```
template B(n){
  signal input in;
  signal input out;
  out <== in + 1;
}
template A(n){
      signal aux;
      Signal out;
      if(n == 2){
	aux <== 2;
         Out <== B()(aux);
      }
      else{
	 out <== 5;
      }
}

component main = A(3);
```

In this example, `aux` is only used in the if branch. Thus, for the main component, `aux` remains unconstrained and the compiler throws a warning. 

```warning[CA01]: In template "A(3)": Local signal aux does not appear in any constraint```

 To avoid the warning, we can add inside the `else` branch, the instruction `_ <== aux` to indicate the compiler that aux is not used in this case.

- Another case when throwing a warning is using subcomponents inside a template: input and output signals of each subcomponent in a template should appear at least in one constraint of the template.
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

The main component checks if the value of `in` can be represented using 10 bits, and it works as expected. The constraints introduced by the subcomponent check will guarantee that the R1CS system only has a solution if `in` can be represented with 10 bits, but it does not matter which is the specific representation. However, the compiler throws the next warning:

```In template "check_bits(10)": Array of subcomponent input/output signals check.out contains a total of 10 signals that do not appear in any constraint of the father component
 = For example: check.out[0], check.out[1].```

Since we are not interested in the binary representation, the template does not make use of array signal `check.out`. Thus, we should add `_ <== check.out` to indicate the compiler that the binary representation is irrelevant and avoid the warning.

Notice also here that the --inspect option also shows the parameter of the instance that causes a warning (check_bits(10)). In general, we throw as many warnings as instances with different parameters for each template.

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
In template "parity(10)": Array of subcomponent input/output signals check.out contains a total of 9 signals that do not appear in any constraint of the father component
 = For example: check.out[1], check.out[2].```

To fix this example, we can add ` _ <== check.out` at the end of the template to let the compiler know that the remaining positions are irrelevant.

