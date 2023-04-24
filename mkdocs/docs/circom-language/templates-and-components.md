# Templates & Components

## Templates

The mechanism to create generic circuits in Circom is the so-called templates.

They are normally parametric on some values that must be instantiated when the template is used. The instantiation of a template is a new circuit object, which can be used to compose other circuits, so as part of larger circuits. Since templates define circuits by instantiation, they have their own signals \(input, output, etc\).

```text
template tempid ( param_1, ... , param_n ) {
 signal input a;
 signal output b;

 .....

}
```

Templates cannot include local functions or template definitions.

Assigning a value to an input signal inside the same template where it has been defined also generates the error _"Exception caused by invalid assignment"_ as can be seen in the next example.

```text
pragma circom 2.0.0;

template wrong (N) {
 signal input a;
 signal output b;
 a <== N;
}

component main = wrong(1);
```

The instantiation of a template is made using the keyword component and by providing the necessary parameters.

```text
component c = tempid(v1,...,vn);
```

The values of the parameters should be known constants at compile time. The next code produces this compilation error message: _"Every component instantiation must be resolved during the constraint generation phase"._

```text
pragma circom 2.0.0;

template A(N1,N2){
   signal input in;
   signal output out; 
   out <== N1 * in * N2;
}


template wrong (N) {
 signal input a;
 signal output b;
 component c = A(a,N); 
}

component main {public [a]} = wrong(1);
```


## Components

A component defines an arithmetic circuit and, as such, it receives N input signals and produces M output signals and K intermediate signals. Additionally, it can produce a set of constraints.

In order to access the input or output signals of a component, we will use **dot notation**. No other signals are visible outside the component.

```text
c.a <== y*z-1;
var x;
x = c.b;
```

The **component instantiation** will not be triggered until all its input signals are assigned to concrete values. Therefore the instantiation might be delayed and hence the component creation instruction does not imply the execution of the component object, but the creation of the instantiation process that will be completed when all the inputs are set. The output signals of a component can only be used when all inputs are set, otherwise a compiler error is generated. For instance, the following piece of code would result in an error:

```text
pragma circom 2.0.0;

template Internal() {
   signal input in[2];
   signal output out;
   out <== in[0]*in[1];
}

template Main() {
   signal input in[2];
   signal output out;
   component c = Internal ();
   c.in[0] <== in[0];
   c.out ==> out;  // c.in[1] is not assigned yet
   c.in[1] <== in[1];  // this line should be placed before calling c.out
}

component main = Main();
```

**Components are immutable** (like signals). A component can be declared first and initialized in a second step. If there are several initialization instructions (in different execution paths) they all need to be instantiations of the same template (maybe with different values for the parameters).

```text
template A(N){
   signal input in;
   signal output out;
   out <== in;
}

template C(N){
   signal output out;
   out <== N;
}
template B(N){
  signal output out;
  component a;
  if(N > 0){
     a = A(N);
  }
  else{
     a = A(0);
  }
}

component main = B(1);
```

If the instruction `a = A(0);`is replaced with `a = C(0)`, the compilation fails and the next error message is shown: _"Assignee and assigned types do not match"_.

We can define **arrays of components** following the same restrictions on the size given before. Moreover, initialization in the definition of arrays of components is not allowed, and instantiation can only be made component by component, accessing the positions of the array. All components in the array have to be instances of the same template as it can be seen in the next example.

```text
template MultiAND(n) {
    signal input in[n];
    signal output out;
    component and;
    component ands[2];
    var i;
    if (n==1) {
        out <== in[0];
    } else if (n==2) {
          and = AND();
        and.a <== in[0];
        and.b <== in[1];
        out <== and.out;
    } else {
        and = AND();
        var n1 = n\2;
        var n2 = n-n\2;
        ands[0] = MultiAND(n1);
        ands[1] = MultiAND(n2);
        for (i=0; i<n1; i++) ands[0].in[i] <== in[i];
        for (i=0; i<n2; i++) ands[1].in[i] <== in[n1+i];
        and.a <== ands[0].out;
        and.b <== ands[1].out;
        out <== and.out;
    }
}
```

When components are independent (the inputs do not depend on each others’ outputs), the computation of these parts can be done in parallel using the tag `parallel`, like shown in the next line.

```text
template parallel NameTemplate(...){...}
```

If this tag is used, the resulting C++ file will contain the parallelized code to compute the witness. Parallelization becomes particularly relevant when dealing with large circuits.

Notice that the previous parallelism is declared at template level. Sometimes, it can be useful declare the parallelism for each component. Since version 2.0.8, the tag parallel can be also used at component level, with the parallel tag indicated right before the call to the template.

```text
component comp = parallel NameTemplate(...){...}
```
A real example of use case is the following piece of code from the rollup code:
```
component rollupTx[nTx];
for (i = 0; i < nTx; i++) {
        rollupTx[i] = parallel RollupTx(nLevels, maxFeeTx);
}
```

It is important to highlight again that this parallelism can only be exploited in C++ witness generator.
 
## Custom templates

Since version 2.0.6, the language allows the definition of a new type of templates, custom templates. This new construction works similarly to standard templates: they are declared analogously, just adding the keyword `custom` in its declaration after `template`; and are instantiated in the exact same way. That is, a custom template `Example` is defined and then instantiated as follows:

```text
pragma circom 2.0.6; // note that custom templates are only allowed since version 2.0.6
pragma custom_templates;

template custom Example() {
   // custom template's code
}

template UsingExample() {
   component example = Example(); // instantiation of the custom template
}
```

However, the way in which their computation is encoded is different from the one for standard templates. Instead of producing r1cs constraints, the usage of each defined custom template will be treated in a later stage by [snarkjs](../index.md#snarkjs) to generate and validate the zk proof, in this case using the PLONK scheme (and using the custom template's definitions as PLONK's custom gates, see [here](../circom-language/custom-templates-snarkjs.md) how). Information about the definition and usages of custom templates will be exported in the `.r1cs` file (see [here](https://github.com/iden3/r1csfile/blob/master/doc/r1cs_bin_format.md) sections 4 and 5). This means that custom templates cannot introduce any constraint inside their body, nor declare any subcomponent.
