# Code Assertion

**assert(bool_expression);**

This statement introduces conditions to be checked. Here, we distinguish two cases depending if **bool_expression** is unknown at compilation time:

- If the condition only depends on the value of template parameters or field constants, the assert is evaluated in compilation time. If the result of the evaluation is false, then the compilation fails.  Consider the next piece of code:

```
template A(n) {
  signal input in;
  assert(n>0);
  in * in === n;
}

component main = A(0);
```

Here, the assert can be evaluated during the compilation and the result of the evaluation is false. Thus, the compilation ends throwing error *error[T3001]: False assert reached*. If the main component was defined as `component main = A(2);`, then the compilation correctly finishes. 

Recall that, when a constraint like `in * in === n;` is introduced with `===`, then an assert is automatically added in the witness generation code. In this case, `assert(in * in == n)`.

- If the condition or the evaluation of the assert involves the value of a signal, the assert cannot be evaluated in compilation time. The compiler produces an assert which must be satisfied during the witness generation. If the input `in` passed as parameter to produce the witness does not satisfy the assert, then the witness will not be generated.

```text
template Translate(n) {
signal input in;
assert(in<=254);
…..
}
```



