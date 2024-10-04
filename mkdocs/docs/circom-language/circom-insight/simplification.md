# Constraint simplification

Constraint simplification is a key part of the `circom` compiler. A fast simplification `--O1` is activated by default (it only applies constant and renaming simplifications), and its associated flag is `--O1` (see the [compilation options](../../getting-started/compilation-options.md)). Simplification is not applied when the flag `--O0` is activated, and a full form of simplification is applied when using the flag `--O2`.

Let us explain the kind of simplification we can perform in detail.

As pointed out in Section 2.3 (Quadratic arithmetic programs) of the [Groth16 paper](https://eprint.iacr.org/2016/260) (where ZK-SNARKs based on arithmetic circuits were introduced): 

> Addition gates are handled for free in the sums defining the equations, i.e., if a<sub>i</sub> + a<sub>j</sub> = a<sub>k</sub> and a<sub>k</sub> is multiplied by a<sub>l</sub>, we may simply write (a<sub>i</sub> + a<sub>j</sub>) * a<sub>l</sub> and skip the calculation of a<sub>k</sub>.

Note that since we can skip its calculation, it will not be part of the witness (the values of the signals that satisfy the arithmetic circuit, i.e. the quadratic constraints).

This means that we can remove any constraint E = 0 if E is linear, by choosing one of the signals in E, say x, and expressing E = 0 as x = E' and replacing x by E' in all other constraints. This way, we may skip the calculation of x since it is not in the resulting problem.

In the context of [Groth16], the statement to be proved is that given the public inputs and outputs and the relation between them expressed by means of quadratic constrains of the form A*B-C = 0 (where A, B and C are linear expressions) we know a witness (an assignment to the signals that includes the given public inputs and outputs) that satisfies the relation (i.e. the constraints for the given public inputs and outputs). Therefore, we cannot remove the public inputs and outputs (even if they occur in a linear constraint) but we can remove any other private signal if it is equivalent to a linear combination of the other signals (i.e. just using additions), since `we can skip the computation of such signal` (because `addition gates are handled for free`) and we are not changing the relation between public inputs and outputs, i.e. the statement.

In case we are using the PLONK proof system (instead of Groth16), since additions are not free we cannot remove linear constraints anymore. Still we can remove equalities between signals or equalities between signals and constants which is made with the flag --O1 (see below). Moreover, note that if we apply linear simplification to a constraint system in PLONK format, the resulting constraints will in general not be in PLONK format anymore, and transforming the result back to PLONK format may lead to a worse result than the original. For this reason, when using PLONK, it is always recommended to use the --O1 flag.

Once we have explained why removing any private signal (including the private inputs) and applying linear simplification is correct, let us explain what kind of simplification is applied when we enable the flag `--O1` (which is activated by default) or the flag `--O2`. Notice that if we do not want to apply any simplification we must use the flag `--O0`.

* Flag ```--O1``` removes two kinds of simple constraints: a) ```signal = K```, being K is a constant in $F_p$ and b) ```signal1 = signal2```. In both cases, at least one of the signals must be private, and it is the one that will be replaced by the other side. Note that there are usually many equalities between two signals in constraints defined by circom programs as they are many times used to connect components with their sub components.
  
* Flag ```--O2``` applies first the same simplification as in `--O1` and then applies a lazy form of Gaussian elimination to remove as many linear constraints containing at least a private signal as possible. After applying the substitutions discovered by the algorithm, non-linear constraints may have become linear. Thus, the Gauss elimination is applied as many rounds as needed until no more linear constraints containing at least a private signal are found.

* As a special case, the flag ```--O2round <simplification_rounds>``` applies the same simplification as in ```--O2```but it limits the maximum number of rounds applied during the optimization to the number given in ```<simplification_rounds>```.

* Finally, as said, flag ```--O0``` indicates that we do not want to apply any kind of simplification.
  
Only one of these flags/options can be enabled in the compilation.

In case we want to see the simplification applied we can use the flag [```--simplification_substitution```](../../getting-started/compilation-options.md) to obtain a json file whose format is described [here](../formats/simplification-json.md).

Since circom 2.2.0, we have set `--O1` as the default simplification option. This decision aligns with the growing use of Plonk, as `--O2` is not compatible with it.

Note that, using the full simplification `--O2` can significantly reduce the number of constraints and signals, which has a positive impact in the time and space needed to compute the proof. However, this is the most time and space consuming phase of the compilation process. Hence, with large circuits, say with millions of constraints, compilation can take a long time (even minutes or hours) and can run in out-of-memory exceptions. In such cases, it is recommended to only use the `--O2` flag in the final steps of the project development.

[Groth16] Jens Groth. "On the Size of Pairing-Based Non-interactive Arguments". Advances in Cryptology -- EUROCRYPT 2016, pages 305--326. Springer Berlin Heidelberg, 2016.
