---
description: >-
  This is a detailed description of the json R1CS format produced by the circom compiler when the flag --json is activated.
---
# R1CS json format

The file contains a dictionary with a single entry "constraints" and a list of constraints as value.
```
{
"constraints": [
constraint_1,
...
constraint_n
]
}
```
where every constraint is a list with three elements which are the linear expresions A, B and C that represent the constraint A*B -C = 0.
```
[lin_expr_A,lin_expr_B,lin_expr_C]
```
where the linear expression is represented by a dictionary with the signal numbers as strings occurring in the linear expression (with non-zero coefficient) as entries and their coefficients (as string) as values:
```
{ "sig_num_l1": "coef_1", ... , "sig_num_lm": "coef_m"}`
```

If you also include the ```--sym``` flag, in the generated [sym file](sym.md) you can see the qualified name in the circom program associated to each signal number, with the signal number 0 always expressing the constant 1. This way we can express any constant by having it as coefficient of the signal 0.

Let us consider the following simple circuit in 'basic.circom':

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
circom basic.circom --json --wasm 
```
a file 'basic_contraints.json' is generated and it contains two constraints: 

```text
{
"constraints": [
[{"2":"21888242871839275222246405745257275088548364400416034343698204186575808495616"},{"4":"1"},{"1":"21888242871839275222246405745257275088548364400416034343698204186575808495616"}],
[{},{},{"0":"1","2":"2","3":"1","4":"21888242871839275222246405745257275088548364400416034343698204186575808495616"}]
]
}
```

 As we can see, only constant and renaming (equalities between signals) simplifications have been applied
(since the --O1 simplification is the default).

Instead, if we run

```text
circom basic.circom --json --wasm --O0
```

to indicate that we do not want to apply any simplification the generated file 'basic_constraints.json' contains

```text
{
"constraints": [
[{},{},{"2":"1","5":"21888242871839275222246405745257275088548364400416034343698204186575808495616"}],
[{},{},{"0":"1","2":"2","3":"1","6":"21888242871839275222246405745257275088548364400416034343698204186575808495616"}],
[{},{},{"1":"21888242871839275222246405745257275088548364400416034343698204186575808495616","4":"1"}],
[{"5":"21888242871839275222246405745257275088548364400416034343698204186575808495616"},{"6":"1"},{"4":"21888242871839275222246405745257275088548364400416034343698204186575808495616"}]
]
}
```
Finally, if we run 

```text
circom basic.circom --json --wasm --O2
```

we can see that only one constraint is taken after applying the full simplification:

```text
{
"constraints": [
[{"2":"21888242871839275222246405745257275088548364400416034343698204186575808495616"},{"0":"1","2":"2","3":"1"},{"1":"21888242871839275222246405745257275088548364400416034343698204186575808495616"}]
]
}
```
