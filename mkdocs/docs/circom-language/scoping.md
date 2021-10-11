# Scoping

Circom has static scoping like C and Rust. However, we have that signals and components must have global scoping and hence they should be defined at the top-level block of the template that defines them. 

```text
pragma circom 2.0.0;

template Multiplier2 (N) {
   //Declaration of signals.
   signal input in;
   signal output out;
   
   //Statements.
   out <== in;
   signal input x;
   if(N > 0){
   	signal output out2;
   	out2 <== x;
   }
}

component main = Multiplier2(5);
```

Signal `out2` must be declared at the top-level block. The next compilation error is produced: _"`out2` is outside the initial scope"_.

Regarding visibility, a signal x of component c is also visible in the template t that has declared c, using the notation c.x. No access to signals of nested sub-components is allowed. For instance, if c is built using another component d, the signals of d cannot be accessed from t.  This can be seen in the next code:

```text
pragma circom 2.0.0;

template d(){
  signal output x;
  x <== 1;
}

template c(){
  signal output out2;
  out2 <== 2;
  component comp2 = d();
}

template t(){
  signal out;
  component c3 = c();
  out <== c3.comp2.x;
}

component main = t();
```

This code produces a compilation error since we cannot access `comp2` of component `c3`.  

A var can be defined at any block and its visibility is reduced to the block like in C or Rust.

