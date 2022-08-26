---
description: >-
  This tutorial guide you through the process of writing your first program
  using the main features of circom: signals, variables, templates, components,
  and arrays.
---
# Compiling our circuit

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

We created a template called `Multiplier2` in [Writing our first circuit](../writing-circuits). 
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

* `--r1cs`: it generates the file `multiplier2.r1cs` that contains the [R1CS constraint system](../../background/background#rank-1-constraint-system) of the circuit in binary format.
* `--wasm`: it generates the directory `multiplier2_js` that contains the `Wasm` code (multiplier2.wasm) and other files needed to generate the [witness](../../background/background#witness).
* `--sym` : it generates the file `multiplier2.sym` , a symbols file required for debugging or for printing the constraint system in an annotated mode.
* `--c` : it generates the directory `multiplier2_cpp` that contains several files (multiplier2.cpp, multiplier2.dat, and other common files for every compiled program  like main.cpp, MakeFile, etc)  needed to compile the C code to generate the witness.

We can use the option `-o` to specify the directory where these files are created. 

Since version 2.0.8, we can use the option `-l` to indicate the directory where the directive `include` should look for the circuits indicated.