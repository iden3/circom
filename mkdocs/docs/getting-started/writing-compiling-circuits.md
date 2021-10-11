---
description: >-
  This tutorial guide you through the process of writing your first program
  using the main features of circom: signals, variables, templates, components,
  and arrays.
---

# Writing and compiling circuits 

`circom` allows programmers to define the [constraints](/circom-language/constraint-generation) that define the arithmetic circuit. All constraints must be of the form A\*B + C = 0, where A, B and C are linear combinations of signals. More details about these equations can be found [here](/circom-language/constraint-generation). 


## Writing our first circuit

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

Then, we use the reserved keyword `template`to define our new circuit, called `Multiplier2`.  Now, we have to define its [signals](/circom-language/signals). Signals can be named with an identifier, e.g.,  `in1, in2, out.`  In this circuit, we have two input signals`in1, in2` and an output signal `out`.  Finally, we use `<==` to set that the value of `out` is the result of multiplying the values of `in1` and `in2`.  Equivalently, we could have also used the operator `==>`, e.g., `in1 * in2 ==> out`.

Let us notice that in each circuit, we first declare its signals, and after that, the assignments to set the value of the output signals.


## Compiling the circuit

Once you have the compiler installed you can see the available options as follows:

```console
circom --help

   Circom Compiler 2.0
   IDEN3
   Compiler for the Circom programming language

   USAGE:
      circom [FLAGS] [OPTIONS] [input]

   FLAGS:
      -h, --help       Prints help information
         --inspect    Does an additional check over the constraints produced
         --O0         No simplification is applied
      -c, --c          Compiles the circuit to c
         --json       outputs the constraints in json format
         --r1cs       outputs the constraints in r1cs format
         --sym        outputs witness in sym format
         --wasm       Compiles the circuit to wasm
         --wat        Compiles the circuit to wat
         --O1         Only applies var to var and var to constant simplification
      -V, --version    Prints version information

   OPTIONS:
         --O2 <full_simplification>    Full constraint simplification [default: full]
      -o, --output <output>             Path to the directory where the output will be written [default: .]

   ARGS:
      <input>    Path to a circuit with a main component [default: ./circuit.circom]
```

We created a template called `Multiplier2` in [Writing our first circuit](/getting-started/writing-compiling-circuits). 
However, to actually create a circuit, we have to create an instance of this template. To do so, create a file with the following content:

```text
pragma circom 2.0.0;

template Multiplier2() {
    signal input a;
    signal input b;
    signal output c;
    c <== a*b;
 }

 component main = Multiplier2();
```

After we write our arithmetic circuit using `circom`, we should save it in a file with the `.circom` extension. Remember that you can create your own circuits or use the templates from our library of circuits [`circomlib`](https://github.com/iden3/circomlib).

In our example, we create a file called *multiplier2.circom*.
Now is time to compile the circuit to get a system of arithmetic equations representing it. As a result of the compilation we will also obtain programs to compute the witness.
We can compile the circuit with the following command:

```text
circom multiplier2.circom --r1cs --wasm --sym --c
```

With these options we generate three types of files:

* `--r1cs`: it generates the file `multiplier2.r1cs` that contains the [R1CS constraint system](/getting-started/background#rank-1-constraint-system) of the circuit in binary format.
* `--wasm`: it generates the directory `multiplier2_js` that contains the `Wasm` code (multiplier2.wasm) and other files needed to generate the [witness](/getting-started/background#witness).
* `--sym` : it generates the file `multiplier2.sym` , a symbols file required for debugging or for printing the constraint system in an annotated mode.
* `--c` : it generates the directory `multiplier2_cpp` that contains several files (multiplier2.cpp, multiplier2.dat, and other common files for every compiled program  like main.cpp, MakeFile, etc)  needed to compile the C code to generate the witness.

We can use the option -o to specify the directory where these files are created. 

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

Finally, every execution starts from an initial [main component](/circom-language/the-main-component) defined as follows.

```text
component main {public [in1,in2,in3]} = Multiplier3();
```

Here, we indicate that the initial component for our first circom program is the circuit `Multiplier3` which has three public signals: `in1, in2` and `in3`.
In circom, all output signals of the main component are public (and cannot be made private), the input signals of the main component are private if not stated otherwise using the keyword public as avobe. The rest of signals are all private and cannot be made public.

## Extending our multiplier to N inputs

When defining a template, we can use [parameters](/circom-language/templates-and-components) to build generic circuits. These parameters must have a [known](/circom-language/circom-insight/unknowns) value at the moment of the instantiation of the template. Following up the previous example, we can implement an N-input multiplier, where `N` is a parameter.

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

In addition to the parameter`N`, two well-known concepts appear in this fragment of code: [arrays](/circom-language/data-types) and [integer variables](/circom-language/data-types). 

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

The instruction `out <== in` not only assigns the value of signal `in` to signal `out`, but it also adds the constraint `out = in` to the set of constraints that define the circuit. Then, when both constraints have solution, it is guaranteed that the output signal is binary. Sometimes, we only want to assign the value of a signal but not adding the corresponding constraint. In this case, we will use the operator `<--` and `-->`. The differences between `<--/-->` and `<==/==>` are described [here](/circom-language/signals).

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


