# Tuples and Anonymous Components

## Anonymous Components

circom 2.1.0 introduces a new feature called __anonymous component__. An anonymous component allows in a single instruction 1) the implicit declaration of a new component, 2) the assignment of every input signal and, finally,  3) the assignment of every output signal. 
This section is divided in the next subsections:
1. Syntax and semantics of __anonymous components__.
2. What if the anonymous component is an instance of a template with more than an output signal?
   __We introduce the tuples in circom.__
3. What if the anonymous component is an instance of a template which input/output signals are arrays?
   __We introduce the element-wise assignment for signal arrays.__
4. What if we are not interested in collecting one of the outputs?
__We introduce the use of "_" to indicate that a signal is not relevant.__


### Syntax and semantics of anonymous components
Let us see a typical circom program. 
```text
template A(n){
   signal input a, b;
   signal output c;
   c <== a*b;
}
template B(n){
   signal input in[n];
   signal out;
   component temp_a = A(n);
   temp_a.a <== in[0]; 
   temp_a.b <== in[1];
   out <== temp_a.c;
}
component main = B(2);
```
Thanks to anonymous components, we can make the above program much cleaner.

```text
template A(n){
   signal input a, b;
   signal output c;
   c <== a*b;
}
template B(n){
   signal input in[n];
   signal out <== A(n)(in[0],in[1]);
}
component main = B(2);
```

It is important to highlight that both circuits are equivalent: they have the same witnesses and the same constraint, that is, `out === in[0]*in[1]`.

The anonymous components are a new kind of circom expression whose syntax is as follows: ```temp_name(arg1,...,argN)(input1,...,inputM)``` 
assuming that we have a template temp_name with N arguments and M input signals. 

Let us clarify two points:
1. `arg1`, ..., `argN` are template arguments. We can use them as usual. They can be arithmetic operations expressions or constants. The important thing is its value must be known at compilation time. 
2. `input1`, ..., `inputM` are input signals. We can pass another signals, (just like in the example) constants or other anonymous components (in a compositional way), if and only if, __such components only have 1 output signal__.

The order of the signals in the anonymous component matters: the ith input signal receives the value of the ith signal passed as parameter. Since circom 2.1.1, it is also allowed to indicate the name of the input signal corresponding to each parameter, followed by `<==` and then the expression in R1CS format). In that case, we can uses any order in giving the inputs provided all the subcomponent inputs are given. Note that either we use the notation with `<==` for all inputs or none. Let us see this new feature in the previous example:

```text
template A(n){
   signal input a, b;
   signal output c;
   c <== a*b;
}
template B(n){
   signal input in[n];
   signal out <== A(n)(b <== in[1], a <== in[0]);
}
component main = B(2);
```

The value returned by the anonymous components depends on the number of template's output signals.

1. __If the template does not define any output signal__ (for instance, if it only defines constraints based on the input signals),  we can use the anonymous component like if it was an statement 
   `temp_name(arg1,...,argN)(inp1,...,inpM);`

2. __If the template defines a single output signal__, we can use any of the well-known operators to collect the output signal. It is important to highlight that we can use with the anonymous components any of the operators `<==`, `==>` and `=`  with the usual semantics, but not `<--` and `-->`, since there is no way to add additional constraints including the signals of the anonymous components (which will end up in security issues in most of the cases). For instance,
`signal out <== temp_name(a1,...,aN)(i1,...,iM);`

1. __If the template defines more than an output signal__, we need to use a new kind of expression to collect all the outputs: __the tuples__, whose syntax is the usual in other programming languages.

```
signal output o1, ..., oK;
(o1,...,oK) <== temp_name(a1,...,aN)(i1,...,iM);
```

```
var  v1, ..., vK;
(v1,...,vK) = temp_name(a1,...,aN)(i1,...,iM);
```

##The use of tuples
Tuples are a typical expression in other programming languages, which allows us to do multiple assignments like in the previous example.

We have introduced tuples because of the previous feature. When using templates with several outputs is necessary being able to collect all the outputs at the same time, since the component is anonymous and later cannot be accessed.

Apart from the main use of the tuples, we can use this new kind of expressions with every kind of assignment  `<==`,`=` and `<--`. However, the latter is not allowed when getting the result of an anonymous component and its use is in general discouraged. Tuples can only be used in combination of any of these operators whenever there are tuples in both sides of the statement. In this case, the semantics of this multiple assignment is the element-wise assignment. 

Let us see a non-trivial example to illustrate the importance of the order of the tuple elements.

```
var a = 0, b = 0; component c;
(a, b, c) = (1,a+1, A(2));
```

This is internally translated by the compiler to
```
a = 1; 
b = a + 1; 
c = A(2);
```
Then, the final value of a and b is 1 and 2, respectively. Notice that c is an instance of template A and we could use now statements to access its inputs and outputs.

### The use of <== for signal arrays
One more extension must be added to circom in order to enable the use of anonymous components. 

How could we use a template as anonymous component if it makes use of input/output signal arrays? So far, this could not be handled by circom. 

In circom 2.1.0, we have overloaded the operator `<==` for signal arrays with the same dimension to express the element-wise assignment. For instance, let us consider the next code.

```
template Ex(n,m){ 
   signal input in[n];
   signal output out[m];
   out <== in;
}
```

If `n != m`, then the compiler reports an error, since both arrays have not the same size. Otherwise, the code is equivalent to:

```
template Ex(n, m){ 
   signal input in[n];
   signal output out[m];
   var i = 0;
   while(i < n) { 
      out[i] <== in[i];
      i += 1;
   }
}
```

Let us use this template to illustrate how this new feature is combined with the use of an anonymous component. 

```
template A{
   signal input i[4];
   signal output o[4];
   o <== Ex(4,4)(i);
}
```
Here, we can see that we pass as first signal parameter a 4-size array. Notice that previously we can only write a program similar to:

```
template A{
   signal input i[4];
   signal output o[4];
   component anon = Ex(4,4);
   var i = 0;
   while(i < 4){ 
      anon.in[i] <== i[i];
      i += 1;
   }
   i = 0;
   while(i < 4){
      o[i] <== anon.out[i];
      i += 1;
   }
}
```

### The use of _

The underscore __"_"__ allows to ignore any amount of output signals of the anonymous components. 

```text
template A(n){
   signal input a, b, c;
   signal output d;
   d <== a*b+c;
   a * b === c;
}
template B(n){
   signal input in[n];
   _ <== A(n)(in[0],in[1],in[2]);
}
component main = B(3);
```

In the previous example, we are interested in adding the constraint  `a * b = c` to the R1CS, but we can ignore the output signal `d`. 

In case the anonymous component has one more than one output, we can ignore the ones we are not interested. 

```text
template A(n){
   signal input a;
   signal output b, c, d;
   b <== a * a;
   c <== a + 2;
   d <== a * a + 2;
}
template B(n){
   signal input in;
   signal output out1;
   (_,out1,_) <== A(n)(in);
}
component main = B(3);
```

In this example, we are only interested in `out1 = in + 2`.

