---
description: >-
  This tutorial guide you through the process of writing your first program
  using the main features of circom: signals, variables, templates, components,
  and arrays.
---

# Writing circuits 

`circom` allows programmers to define the [constraints](../../circom-language/constraint-generation) that define the arithmetic circuit. All constraints must be of the form A\*B + C = 0, where A, B and C are linear combinations of signals. More details about these equations can be found [here](../../circom-language/constraint-generation). 

The arithmetic circuits built using `circom` operate on signals. Let us define our first circuit that simply multiplies two input signals and produces an output signal.

```text  
pragma circom 2.0.0;
  
/*This circuit multiplies in1 and in2.*/  

template Multiplier2 () {  

   // Declaration of signals.  
   signal input in1;  
   signal input in2;  
   signal output out;  
     
   // Statements.  
   out <== in1 * in2;  
}

component main = Multiplier2();
```

First, the `pragma` instruction is used to specify the compiler version. This is to ensure that the circuit is compatible with the compiler version indicated after the `pragma` instruction. Otherwise, the compiler will throw a warning. 

Then, we use the reserved keyword `template`to define our new circuit, called `Multiplier2`.  Now, we have to define its [signals](../../circom-language/signals). Signals can be named with an identifier, e.g.,  `in1, in2, out.`  In this circuit, we have two input signals`in1, in2` and an output signal `out`.  Finally, we use `<==` to set that the value of `out` is the result of multiplying the values of `in1` and `in2`.  Equivalently, we could have also used the operator `==>`, e.g., `in1 * in2 ==> out`.

Let us notice that in each circuit, we first declare its signals, and after that, the assignments to set the value of the output signals.