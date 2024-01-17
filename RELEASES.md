# Release notes

## Jan 17, 2024 circom 2.1.8

#### Extensions
- Adding flag --simplification_substitutions: outputs a .json file including information about the substitutions performed during the constraint simplifications
- Adding the prime number secq256r1.
- Removing non-determinism in R1CS generation: fixing assignment order of anonymous components inputs
- Improving error message: case assignment to signal of not initialized component
- Removing unnecessary clones
- Improving documentation: types of signal assignments, output-file format descriptions, compilation options,
- Improving the README file with more community tools and projects. 

#### Fixed bugs
- Printing errors and warnings in custom gates correctly
- Removing panic caused by edge case in function display_trace()

  
## Dec 15, 2023 circom 2.1.7

#### Extensions
- Fixing typos and improving documentation.
- Improving error messages: missing tags in inputs and use of operator ++x.
- Adding info of number of private inputs in witness in R1CS message and removing message of circom safe.
  
#### Fixed Bugs 
- Fixing a panic produced when the two branches of an if instruction are not compatible.
- Fixing C++ code generated: input counter decrease inside assert => Moved outside the assert.
- Fixing c++ code generated for macos with clang 14.0.0.
- Fixing a bug in parallel definition at component level not working as expected.
- Fixing a bug in parallel components not working as expected when considering assignments of the inputs given by names (A()(in_1 <== x, in_2 <== y)).
- Fixing a bug in c++ code generation produced when the last assigned input has size 0.
- Fixing a panic: handling case array of components with different signals (signals defined inside blocks ifs).
- Fixing a panic: error when processing sizes of arrays of variables in functions, panic in merger when processing complex expressions.

  
## June 22, 2023 circom 2.1.6

#### Extensions
- Improving tag propagation: array case.
- Handling new prime numbers: pallas, vesta, grumpkin
- Improving array access index computation in the code generated intermediate representation: using cheap addition and multiplication operations when possible.
- Updating the documentation.
- Added check on the name of the circom file when --C is used to avoid clashes with reserved names. When the file is called main.circom, fr.circom or calcwit.circom it is changed to main_c, fr_c and calcwit_c respectively.

#### Fixed Bugs 
- Fixing a bug while parsing anonymous components.
- Fixing a problem in calls to anonymous components with signal names.
- Fixing a bug in wasm witness generation that happened when doing a call inside an array index.
- Executing the main method without inputs in wasm witness generation.
  
  
## March 15, 2023 circom 2.1.5

#### Extensions
- Definition of signals and components can be done now inside if blocks IF the condition is known at compilation time. If the condition is unknown and depends on the value of signals, then the compiler throws an error. 
- Improving the --inspect option. It now detects underconstrained signals and assignments using <-- in which <== could be used.
- Improving the efficiency of the compiler. It does not execute the inspect and constraint generation phase only if there are not the corresponding flags.
- Improving --O1 simplification: removing signals that do not appear in any constraint and avoiding unnecessary constraint normalizations.
- Improving parallel: array assignments of outputs and efficiency by updating numThread while waiting and setting maxThreads to 32.
- Handling better the parser errors and improving error messages to output more information. (parser, type checking and constraint generation errors).
- Showing warnings when errors are found.
- Reducing writing time for output files.
- Updating the documentation.

#### Fixed Bugs
- Fixing a problem with the memory release of the components (in C).
- Fixing a problem with the parallel execution during the witness generation (in C).
- Fixing a bug: functions are executed even if they output signals when they contain logs or asserts. 
- Fixing a bug: During the witness generation, the computation of expressions like x**x was buggy (in wasm).

## February 10, 2023 circom 2.1.4

 #### Extensions
 - Improving the efficiency of the parser regarding the anonnymous components and tuples. 
 - Improving the substitution process: better compilation times for --O1 and --O2.
 - Improving the handling of the underscore substitution.
 - Extending the substitution to allow the inheritance of signal tags.
 - Removing unused signal when applying --O1. (If a signal does not appear in any constraint, it is removed).  

 #### Fixed Bugs
 - Solving bug in the release of the memory of the components.

## January 16, 2023 circom 2.1.3
 #### Extensions
 - Improving error messages: invalid access and invalid assignment.
 - Avoiding side effects in out of bounds log operations.
 - Adding check to detect components that are created but do not receive all their inputs.
 - Fixing field size of goldilocks when writing r1cs file.

 #### Fixed Bugs
 - Fixing a problem with the use of integer division and ===, <== operators. If an integer division is involved in the expression of one of these operators, then we consider the expression as no quadratic.
 - Fixing bug in code generation of constraint equalities with arrays


## November 7, 2022 circom 2.1.2

 #### Fixed bugs
 - Fixed bug in C++ witness generation: function release_memory_component failed when releasing the memory of an array of components with some empty positions
 - Fixed bug in logging of arithmetic expressions 
 
## November 4, 2022 circom 2.1.1
 #### Extensions
 - New feature of anonymous components: programmers can pass the parameters indicate the input names receiving the values.[See here](https://github.com/iden3/circom/blob/master/mkdocs/docs/circom-language/anonymous-components-and-tuples.md).
 - circom now exits with 0 when it finishes successfully (last version exists with Exist(0) which broke some projects).
 - Improving tags assignment: case multiple assignments in an array giving the same value to a tag.
 - Allowing in cpp the use of binary, octal and hexadecimal numbers as inputs from a json file
 - Adding support for non-64bit architectures.
 - Witness_calculator adapted to work with negative numbers in the json input.

 #### Fixed bugs
 - Fixing bug in C++ witness generation: function Fr_toInt in fr.asm
 - Improving error handling division by zero (instead of throwing a panic)
 
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
