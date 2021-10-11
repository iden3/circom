# Pragma

All files with .circom extension should start with a first `pragma`instruction specifying the compiler version, like this: 

```text
pragma circom xx.yy.zz;
```

This is to ensure that the circuit is compatible with the compiler version indicated after the `pragma` instruction. Otherwise, the compiler throws a warning. 

If a file does not contain this instruction, it is assumed that the code is compatible with the latest compiler's version and a warning is thrown.

