# Constraint Generation

To understand the constructive part of circom, we need to consider the following type of expressions:

* **Constant values**: only a constant value is allowed.
* **Linear expression**: an expression where only addition is used. It can also be written using multiplication of variables by constants. For instance, the expression `2*x + 3*y + 2` is allowed, as it is equivalent to `x + x + y + y + y + 2`. 
* **Quadratic expression**: it is obtained by allowing a multiplication between two linear expressions and addition of a linear expression: A\*B - C, where A, B and C are linear expressions. For instance, `(2*x + 3*y + 2) * (x+y) + 6*x + y – 2`.
* **Non quadratic expressions**: any arithmetic expression which is not of the previous kind.

circom allows programmers to define the constraints that define the arithmetic circuit. All constraints must be quadratic of the form A\*B + C = 0, where A, B and C are linear combinations of signals. circom will apply some minor transformations on the defined constraints in order to meet the format A\*B + C = 0:

* Moves from one side of the equality to the other.
* Applications of commutativity of addition.
* Multiplication (or division) by constants.

A constraint is imposed with the operator `===`,  which creates the simplified form of the given equality constraint.

```text
a*(a-1) === 0;
```

Adding such constraint also implies adding an `assert` statement in the witness code generation.

Constraint generation can be combined with signal assignment with the operator  `<==` with the signal to be assigned on the left hand side of the operator.

```text
out <== 1 - a*b;
```

Which is equivalent to:

```text
out === 1 – a*b;
out <-- 1 - a*b;
```

As mentioned before, assigning a value to a signal using `<--` and `-->` is considered dangerous and should, in general, be combined with adding constraints with `===`, which describe by means of constraints which the assigned values are. For example:

```text
a <-- b/c;
a*c === b;
```

In the constructive phase, a variable can contain arithmetic expressions that are built using multiplication, addition, and other variables or signals and field values. Only quadratic expressions are allowed to be included in constraints. Other arithmetic expressions beyond quadratic or using other arithmetic operators like division or power are not allowed as constraints. 

```text
template multi3() {
	 signal input in;
	 signal input in2;
	 signal input in3;
	 signal output out;
	 out <== in*in2*in3;
}
```

This template produces the error "Non quadratic constraints are not allowed!", since it introduces the constraint `out === in*in2*in3` which is NOT quadratic.

The following example shows the generation of expressions:

```text
 signal input a;
 signal output b;
 var x = a*a;
 x += 3;
 b <== x;
```

The last instruction produces the constraint `b === a * a + 3`.

Finally, programmers sometimes misuse operator `<--`, when starting to work in circom. They usually assign using this operator an expression which is quadratic and, as a consequence, no constraint is added. In this case, the operator needed to both performing the assignment and adding the constraint is operator `<==`. Since version 2.0.8, we throw a warning in this case. 