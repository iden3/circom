# Code Assertion

**assert(bool_expression);**

This statement introduces conditions to be checked at execution time. If the condition fails, the witness generation is interrupted and the error is reported.

```text
template Translate(n) {
assert(n<=254);
…..
}
```


Recall that, when a constraint is introduced with ===, then an assert is automatically added in the witness generation code.

