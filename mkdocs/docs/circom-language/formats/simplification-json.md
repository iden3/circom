---
description: >-
  This is a detailed description of the json format produced by the circom compiler when the flag --simplification_substitution is activated.
---
# Simplification substitution json format

The file contains a dictionary where the entries are the numbers of the simplified singnals as a string and the values are the linear expresion that has replaced the signal.
```
{
"sig_num_1": lin_expr_1,
...
"sig_num_n": lin_expr_n,
}
```
where the linear expression is represented by a dictionary with the signal numbers as strings occurring in the linear expression (with non-zero coefficient) as entries and their coefficients (as string) as values:
`{ "sig_num_l1": "coef_1", ... , "sig_num_lm": "coef_m"}`

All signals occurring in the linear expression are signals that are not removed. Hence, if you also include the ```--sym``` flag, in the generated [sym file](sym.md) all these signals are associated to a position in the witness list. On the other hand, al signals sig_num_1 ... sig_num_n that appear as entries in the substitution dictionary are associated to -1.

Let us consider the following simple circuit in 'simplify.circom':

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
circom simplify.circom --r1cs --wasm --simplification_substitution
```
a file 'simplify_substitutions.json' is generated that contains

```text
{
"5" : {"2":"1"},
"4" : {"1":"1"},
"6" : {"0":"1","2":"2","3":"1"}
}
```

where we can see that three signals have been substituted (since the --O2 simplification is the default).

Instead, if we run

```text
circom simplify.circom --r1cs --wasm --simplification_substitution --O0
```

to indicate that we do not want to apply any simplification the generated file 'simplify_substitutions.json' contains

```text
{
}
```
Finaly, if we run 

```text
circom simplify.circom --r1cs --wasm --simplification_substitution --O1
```

to indicate that we only want to apply constant and renaming (equalities between signals) simplifications the generated file 'simplify_substitutions.json' contains

```text
{
"5" : {"2":"1"},
"4" : {"1":"1"}
}
```
