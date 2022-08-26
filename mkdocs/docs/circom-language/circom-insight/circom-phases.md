---
description: >-
  Here you can find information about the compiler and the messages you may get
  from it.
---

# circom Compiler

`circom` has two compilation phases:

1. The **construction** phase, where the constraints are generated. 
2. The **code generation** phase, where the code to compute the witness is generated.

If an error is produced in any of these two phases, circom will finish with an error code greater than 0. Otherwise, if the compiler finish successfully, it finishes returning 0.

