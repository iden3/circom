---
description: >-
  This is a detailed description of the sym format produced by the circom compiler when the flag --sym is activated.
---
# sym format

The format provides a list of lines, where each line contains the information about a signal of the programmed circom circuit given as

```
#s, #w, #c, name
```
where
* #s: a positive number. It is the unique number starting in 1 (0 is reserved to the signal holding the constant value 1) which is assigned by the circom compiler to each signal in the circuit.
* #w: an integer larger than or equal to -1. It gives either the position in the witness where the signal occurs or -1 if the signal is not public and does not occur in any constraint in the generated R1CS. Note that many signals do not appear in the final R1CS because they have been replaced by a linear combination of other signals that is equivalent to it in the simplification phase. In order to know the substitution applied to a removed signal one can add the flag --simplification_substitution to the circom call and check the generated [json file](simplification-json.md). All witness positions except 0 (which is again reserved to the constant value 1) must occur once in the sym file. The length of the witness coincides with the number of (different) signals occurring in the generated R1CS plus one (for the constant 1).
* #c: a non-negative integer (starting in 0). It is the unique number given by the compiler to the component the signal belongs to. 
* name: is a string containing the qualified name of the signal (including the complete component path).

Let us consider the following simple circuit in 'symbols.circom':

```text
pragma circom 2.0.0;

template Internal() {
   signal input in[2];
   signal output out;
   out <== in[0]*in[1];
}

template Main() {
   signal input in[2];
   signal output out;
   component c = Internal ();
   c.in[0] <== in[0];
   c.in[1] <== in[1]+2*in[0]+1;
   c.out ==> out;
}
```
if we run

```text
circom symbols.circom --r1cs --wasm --sym 
```
a file 'symbols.sym' is generated that contains

```text
1,1,1,main.out
2,2,1,main.in[0]
3,3,1,main.in[1]
4,-1,0,main.c.out
5,-1,0,main.c.in[0]
6,-1,0,main.c.in[1]
```

where we can see that three signals have been eliminated (since the --O2 simplification is the default).

Instead, if we run

```text
circom symbols.circom --r1cs --wasm --sym --O0
```

to indicate that we do not want to apply any simplification the generated file 'symbols.sym' contains

```text
1,1,1,main.out
2,2,1,main.in[0]
3,3,1,main.in[1]
4,4,0,main.c.out
5,5,0,main.c.in[0]
6,6,0,main.c.in[1]
```
Finaly, if we run 

```text
circom symbols.circom --r1cs --wasm --sym --O1
```

to indicate that we only want to apply constant and renaming (equalities between signals) simplifications the generated file 'symbols.sym' contains

```text
1,1,1,main.out
2,2,1,main.in[0]
3,3,1,main.in[1]
4,-1,0,main.c.out
5,-1,0,main.c.in[0]
6,4,0,main.c.in[1]
```
