# Scoping

Circom has static scoping like C and Rust. However, we have that signals and components must have global scoping and hence they should be defined at the top-level block of the template that defines them or, since circom 2.1.5, inside (nested) `if` blocks, but only if conditions are known at compilation time. 

```text
pragma circom 2.1.5;

template Cubes (N) {
   //Declaration of signals.
   signal input in[N];
   signal output out[N];
   
   //Statements.
   for (var i = 0; i < N; i++) {
      signal aux;
      aux <== in[i]*in[i];
      out[i] <== aux*in[i];      
   }
}

component main = Cubes(5);
```

Signal `aux` cannot be declared in the block of the `for` instruction. The next compilation error is produced: _"`aux` Is outside the initial scope"_.

Instead the following program compiles correctly.

```text
pragma circom 2.1.5;
template A(n){
   signal input in;
   signal output outA;
   var i = 0;
   if(i < n){
    signal out <== 2;
    i = out;
   } 
   outA <== i;
}
component main = A(5);
```

since the condition `i < n` is known at compilation time, and then the declaration of signal `out` is allowed. However, if the condition was `in < n`, since it is not known at compilation time, it would output an error message because the declaration in that case is not allowed. 

In any case, we apply a static scoping like in C++ or Rust, and a signal declared inside an `if` block is only visible inside the block it is declared.

Regarding visibility of signals of subcomponent, a signal `x` of component `c` is also visible in the template `t` that has declared `c`, using the notation `c.x`, if `x` is an input or and output of `c`. No access to intermediate signals of sub-components or signals of nested sub-components is allowed. For instance, if `c` is built using another component `d`, the signals of `d` cannot be accessed from `t`.  This can be seen in the next code:

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
That rises and error on `c3.comp2.x`: _"Signal not found in component: only accesses to input/output signals are allowed"_.

A var can be defined at any block and its visibility is reduced to the block like in C or Rust.
