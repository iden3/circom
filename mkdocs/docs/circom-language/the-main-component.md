# The Main Component

In order to start the execution, an initial component has to be given. By default, the name of this component is “main”, and hence the component main needs to be instantiated with some template.

This is a special initial component needed to create a circuit and it defines the global input and output signals of a circuit. For this reason, compared to the other components, it has a special attribute: the list of public input signals. The syntax of the creation of the main component is:

```text
component main {public [signal_list]} = tempid(v1,...,vn);
```

where `{public [signal_list]}` is optional. Any input signal of the template that is not included in the list is considered private.

```text
pragma circom 2.0.0;

template A(){
    signal input in1;
    signal input in2;
    signal output out;
    out <== in1 * in2;
}

component main {public [in1]}= A();
```

In this example, we have two input signals `in1` and `in2`. Let us notice that `in1` has been declared as a public signal for the circuit, whereas `in2` is considered a private signal since it does not appear in the list. Finally, output signals are always considered public signals.

Only one main component can be defined, not only in the file being compiled but also in any other circom file included in the program. Otherwise, the compilation fails and the next message is shown: _"Multiple main components in the project structure"_

