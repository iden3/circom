# Variables & Mutability

Variables are identifiers that hold non-signal data and are mutable. Variables are declared using the keyword var as in:

```text
var x;
```

They hold either numerical values of the field or arithmetic expressions when they are used to build constraints (see [Constraint Generation](../constraint-generation)). They can be named using a variable identifier or can be stored in arrays.

Variable assignment is made using the equal symbol `=`. Declarations may also include an initialization, as in the following examples:

```text
var x;
x = 234556;
var y = 0;
var z[3] = [1,2,3];
```

An assignment is a statement and does not return any value, hence it cannot be part of an expression, which avoids misleading uses of `=`. Any use of `=` inside an expression will lead to a compilation error.

The two examples below would result in compilation errors:

```text
a = (b = 3) + 2;
```

```text
var x;
if (x = 3) {
   var y = 0;
}
```

