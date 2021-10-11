# Debugging Operations

In circom there is an operation that can be used while developing circuits to help the programmer debug (note that there are no input/output operations on the standard input/output channels). To this end, the operation `log` has as parameter a non-conditional expression (i.e., not including the _`?`_`;_` operator). The execution of this instruction prints the result of the evaluation of the expression in the standard error stream. As examples consider:

```text
log(135);
log(c.b);
log(x==y);
```



