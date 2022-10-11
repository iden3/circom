# Release notes
## October 11, 2022 circom 2.1.0
#### New features
- Tags: more information [here](https://github.com/iden3/circom/blob/master/mkdocs/docs/circom-language/tags.md).
- Anonymous Components: more information [here](https://github.com/iden3/circom/blob/master/mkdocs/docs/circom-language/anonymous-components-and-tuples.md).

#### Extensions
- Improving the memory consumption during the C++ witness generation.

## September 21, 2022 circom 2.0.9
 #### Extensions
 - Adding a warning  if the programmer is using the operator <-- when it is possible to use <== instead (if the right side is a quadratic expression and the instruction is not contained in a custom template).
 - Signal ids in custom templates changed to 64 bits.
 - Array sizes are expected to be usize. Now, we throw an error in other case.
 - Separating optimization option -O2 in two different options: --O2 and --O2rounds. Explanation can be found [here](https://github.com/iden3/circom/blob/master/mkdocs/docs/circom-language/include.md). The default option is currently --O2.
- Writing Sections 4 and 5 of the r1cs file, only if "pragma custom_templates" is used (which is needed if custom templates are present).
 - Improving --O1 optimization. 
 - Adding a new documentation section about the different compilation options and flags. 

 #### Fixed bugs
 - Fixing -l option to disallow several values for one option: each value must have its own -l option.

## August 26, 2022 circom 2.0.8

#### Extensions
- Adding a link option -l that works as usual in other programming languages, to include a directory to look for the circuits indicated by the directive include. 
- Adding a warning if the programmer is using the operator <-- when it is possible to use <== instead (if the right side is a quadratic expression).
- circom returns 0 if everything was correct and a number greater than 0 if something was wrong.
- Changing the log operator to work as usual in other programming languages. 
- The keyword parallel can be used per instance instead of per template. Now, parallel can be indicated before the instantiation call to make parallel such a particular instance. 
- Wasm Functions getMinorVersion and getPatchVersion to obtain the minor and the patch version. 

#### Fixed Bugs
- Fixing main.cpp to allow handling a main component without inputs.
- New log version has to be applied in every version of wasm files. (By a mistake, it was not updated for every wasm files.)

## August 19, 2022 circom 2.0.7

#### Extensions
- Log operation receives as parameters a list of expressions and string. (Documentation is [here](https://github.com/iden3/circom/blob/master/mkdocs/docs/circom-language/code-quality/debugging-operations.md).
- New heuristics that improves the constraint simplification process is added by default. The old version of the simplification process can be still used using the option "--use_old_simplification_heuristics".
- Initialization of every array position to 0 in both C and WASM witness generator.
- New check of size vector when vector assignment:
           + If a vector is assigned in a smaller vector, then an error is produced. 
           + If a vector is assigned in a larger vector, then a warning is produced. (The remaining positions will have their previous values, or 0     
             otherwise. 
- Improvement of the trace error message.

## July 23, 2022 circom 2.0.6

#### Extensions
- Adding three new prime numbers in circom and a flag to choose which one to use. 
- Adding custom templates to circom. (Documentation is [here](https://github.com/iden3/circom/edit/master/mkdocs/docs/circom-language/custom-templates-snarkjs.md)). 
- Adding published binaries for mac, linux and windows.


## June 23, 2022 circom 2.0.5

#### Extensions
- Removing non-determinism in linear constraints that cannot be removed.
- Making deterministic the code generation. 
- Adding signal one in the wires counting for optimization option '-O0'.


#### Fixed Bugs
- Bug in conditional creation of components inside loops

## April 24, 2022 circom 2.0.4

#### Extensions
-	Improvement of unused signals removal in constraint optimization.
-	macos (x86_64) support for cpp backend added: use template to generate makefile (Not available for M1).
-	wabt dependency swapped to wast dependency instead.
-	Improvement of the known/unknown analysis.
-	Single signal declaration with initialization added for both <== and <--.
-	Input signal size check added in wasm/JS and C++ generated code.
-	Recommendation of using C++ when the memory needed is beyond WebAssembly limit added.
-	Making deterministic the R1CS file: constraints will be always written in the R1CS file in the same order. 

#### Fixed Bugs
-	Bug in C++ error trace management for asserts in functions.
-	Bug in multiple line(s) comments.
-	Bug in unassigned inputs of arrays of size 0.
-	Bug in the use of constant division operation on constraints.
-	Bug in evaluation of expressions involving subcomponents signals.

## Dec 23, 2021 circom 2.0.3

#### Extensions
-	Improvement in the check all array dimensions in its definition have known size.
-	A new verbose flag is added: If –verbose is set the compiler shows log messages during constraint generation.

#### Fixed Bugs
-	Bug in functions that return calls to other functions.

## Dec 10, 2021 circom 2.0.2

#### Extensions
-	A check that all inputs are set is added in the wasm/JS and C++ generated code.
-	Improvement of the “merge_code” implementation in code generators. 

## Nov 9, 2021 circom 2.0.1

#### Extensions
-	Error trace printed in the C++ code when an assert fails.
-	Compiler versions in circom programs handling added (improving the pragma instruction).
-	Arrays of components with uninstantiated positions are handled.
-	Comments in the wat code only generated when –wat flag is used.

#### Fixed bug

-	Bug in the line number generation for wasm error message.
-	Bug: R1CS map is an array of 64bit elements (instead of 32bit).
-	Bug on the initial memory size (too large) defined in the generated wasm code and error message handling in wasm generation added.
-	Bug with use of circuit file names with dots.
