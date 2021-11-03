---
description: >-
  This tutorial guide you through the process of writing your first program
  using the main features of circom: signals, variables, templates, components,
  and arrays.
---

# More basic circuits

## Extending our multiplier to three inputs

Building on top of the 2-input multiplier, we can build a 3-input multiplier.

```text
pragma circom 2.0.0;

template Multiplier2(){
     /*Code from the previous example.*/
}

//This circuit multiplies in1, in2, and in3.
template Multiplier3 () {
   //Declaration of signals and components.
   signal input in1;
   signal input in2;
   signal input in3;
   signal output out;
   component mult1 = Multiplier2();
   component mult2 = Multiplier2();

   //Statements.
   mult1.in1 <== in1;
   mult1.in2 <== in2;
   mult2.in1 <== mult1.out;
   mult2.in2 <== in3;
   out <== mult2.out;
}

component main = Multiplier3();
```

As expected, we first declare three input signals `in1, in2, in3,` and an output signal `out` and two instances of `Multiplier2` . Instantiations of templates are done using the keyword `component`. We need an instance `mult1` to multiply `in1` and `in2`. In order to assign the values of the input signals of `mult1` we use the dot notation `"."`. Once `mult1.in1` and `mult1.in2` have their values set, then the value of `mult1.out` is computed. This value can be now used to set the input value of `mult2`  of the second instance of `Multiplier2`to multiply `in1*in2` and `in3` obtaining the final result  `in1*in2*in3`.

Finally, every execution starts from an initial [main component](../../circom-language/the-main-component) defined as follows.

```text
component main {public [in1,in2,in3]} = Multiplier3();
```

Here, we indicate that the initial component for our first circom program is the circuit `Multiplier3` which has three public signals: `in1, in2` and `in3`.
In circom, all output signals of the main component are public (and cannot be made private), the input signals of the main component are private if not stated otherwise using the keyword public as above. The rest of signals are all private and cannot be made public.

## Extending our multiplier to N inputs

When defining a template, we can use [parameters](../../circom-language/templates-and-components) to build generic circuits. These parameters must have a [known](../../circom-language/circom-insight/unknowns) value at the moment of the instantiation of the template. Following up the previous example, we can implement an N-input multiplier, where `N` is a parameter.

```text
pragma circom 2.0.0; 

template Multiplier2(){
     /*Code from the previous example.*/
}

template MultiplierN (N){
   //Declaration of signals and components.
   signal input in[N];
   signal output out;
   component comp[N-1];
   
   //Statements.
   for(var i = 0; i < N-1; i++){
   	   comp[i] = Multiplier2();
   }

   // ... some more code (see below)
   
}

component main = MultiplierN(4);
```

In addition to the parameter`N`, two well-known concepts appear in this fragment of code: [arrays](../../circom-language/data-types) and [integer variables](../../circom-language/data-types). 

As we have seen for a 3-input multiplier, we need 3 input signals and 2 components of `Multiplier2`. Then, for an N-input multiplier, we need an N-dimensional array of input signals and an \(N-1\)-dimensional array of components of `Multiplier2`. 

We also need an integer variable `i` to instantiate each component `comp[i]`. Once this is done, we have to set the signals for each component as follows:

```text
   comp[0].in1 <== in[0];
   comp[0].in2 <== in[1];
   for(var i = 0; i < N-2; i++){
	   comp[i+1].in1 <== comp[i].out;
	   comp[i+1].in2 <== in[i+2]; 
   }
   out <== comp[N-2].out; 
}
```

Similarly to `Multiplier3`, each output signal of a component becomes one of the input signals of the next component. Finally, `out` is set as the output signal of the last component and its value will be `in[0]*in[1]*...*in[N-1]`. Finally, we define as main component a `MultiplierN` with `N = 3`.

```text
component main {public [in]} = MultiplierN(3);
```


```text
pragma circom 2.0.0;

template Multiplier2(){
   //Declaration of signals.
   signal input in1;
   signal input in2;
   signal output out;
   
   //Statements.
   out <== in1 * in2;
}

template Multiplier3 () {
   //Declaration of signals.
   signal input in1;
   signal input in2;
   signal input in3;
   signal output out;
   component mult1 = Multiplier2();
   component mult2 = Multiplier2();
   
   //Statements.
   mult1.in1 <== in1;
   mult1.in2 <== in2;
   mult2.in1 <== mult1.out;
   mult2.in2 <== in3;
   out <== mult2.out;
}

template MultiplierN (N){
   //Declaration of signals.
   signal input in[N];
   signal output out;
   component comp[N-1];
 
   //Statements.
   for(var i = 0; i < N-1; i++){
   	   comp[i] = Multiplier2();
   }
   comp[0].in1 <== in[0];
   comp[0].in2 <== in[1];
   for(var i = 0; i < N-2; i++){
	   comp[i+1].in1 <== comp[i].out;
	   comp[i+1].in2 <== in[i+2];
   	   
   }
   out <== comp[N-2].out; 
}

component main {public [in]} = MultiplierN(3);
```


## Writing a circuit for binary checks

Let us build a circuit that checks if the input signal is binary. In case it is, the circuit returns an output signal with the same value than`in`. 

```text
pragma circom 2.0.0;

template binaryCheck () {

   // Declaration of signals.
   
   signal input in;
   signal output out;
   
   // Statements.
   
   in * (in-1) === 0;
   out <== in;
}

component main = binaryCheck();
```

After declaring the signals of the circuit, we use the operator `===`to introduce the constraint `in * (in -1) = 0`. The solutions of this constraint are `in = 0` and `in = 1`. This means that the constraint has solution if and only if the input signal is binary.

The instruction `out <== in` not only assigns the value of signal `in` to signal `out`, but it also adds the constraint `out = in` to the set of constraints that define the circuit. Then, when both constraints have solution, it is guaranteed that the output signal is binary. Sometimes, we only want to assign the value of a signal but not adding the corresponding constraint. In this case, we will use the operator `<--` and `-->`. The differences between `<--/-->` and `<==/==>` are described [here](../../circom-language/signals).

## Writing a logic gate AND with two inputs

We are going to use the circuits `Multiplier2` and `binaryCheck` to build a 2-gate logic AND.

```text
pragma circom 2.0.0;

template Multiplier2(){
   //Declaration of signals
   signal input in1;
   signal input in2;
   signal output out;
   
   //Statements.
   out <== in1 * in2;
}

template binaryCheck () {
   //Declaration of signals.
   signal input in;
   signal output out;
   
   //Statements.
   in * (in-1) === 0;
   out <== in;
}

template And2(){
   //Declaration of signals and components.
   signal input in1;
   signal input in2;
   signal output out;
   component mult = Multiplier2();
   component binCheck[2];
   
   //Statements.
   binCheck[0] = binaryCheck();
   binCheck[0].in <== in1;
   binCheck[1] = binaryCheck();
   binCheck[1].in <== in2;
   mult.in1 <== binCheck[0].out;
   mult.in2 <== binCheck[1].out;
   out <== mult.out;
}

component main = And2();
```

Simplifying, the 2-gate AND circuit can be defined by the next constraints:

`in1 * (in1 - 1) = 0`, `in2 * (in2 - 1) = 0`, `out = in1 * in2`

These constraints are satisfiable if and only  if `in1, in2` are binary signals. Consequently, `out` will also be binary.

## Extending our AND logic gate to N inputs

Finally, let us build an N-gate logic AND using circuit `Multiplier2` and `binaryCheck`.

```text
pragma circom 2.0.0;

template binaryCheck () {
   /*Code from previous example*/
}

template Multiplier2 () {
   /*Code from previous example*/
}

template AndN (N){
   //Declaration of signals and components.
   signal input in[N];
   signal output out;
   component mult[N-1];
   component binCheck[N];
   
   //Statements.
   for(var i = 0; i < N; i++){
   	   binCheck[i] = binaryCheck();
	     binCheck[i].in <== in[i];
   }
   for(var i = 0; i < N-1; i++){
   	   mult[i] = Multiplier2();
   }
   mult[0].in1 <== binCheck[0].out;
   mult[0].in2 <== binCheck[1].out;
   for(var i = 0; i < N-2; i++){
	   mult[i+1].in1 <== mult[i].out;
	   mult[i+1].in2 <== binCheck[i+2].out;
   	   
   }
   out <== mult[N-2].out; 
}

component main = AndN(4);
```

This program is very similar to `MultiplierN`, but every  signal involved in it is binary.

It is important to highlight that we cannot use a (2N-1)-dimensional array to instantiate all the components since, every component of an array must be an instance of the same template with (optionally) different parameters.
