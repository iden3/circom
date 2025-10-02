If we use the command ```circom --help```, we can see all the options and flags that we can use during the compilation.

```console 
USAGE:
    circom [FLAGS] [OPTIONS] [--] [input]

FLAGS:
        --r1cs                                 Outputs the constraints in r1cs format
        --sym                                  Outputs witness in sym format
        --wasm                                 Compiles the circuit to wasm
        --json                                 Outputs the constraints in json format
        --wat                                  Compiles the circuit to wat
    -c, --c                                    Compiles the circuit to C++
        --O0                                   No simplification is applied
        --O1                                   Only applies signal to signal and signal to constant simplification. This
                                               is the default option
        --O2                                   Full constraint simplification
        --verbose                              Shows logs during compilation
        --inspect                              Does an additional check over the constraints produced
        --use_old_simplification_heuristics    Applies the old version of the heuristics when performing linear
                                               simplification
        --simplification_substitution          Outputs the substitution applied in the simplification phase in json
                                               format
        --no_asm                               Does not use asm files in witness generation code in C++
        --no_init                              Removes initializations to 0 of variables ("var") in the witness
                                               generation code
    -h, --help                                 Prints help information
    -V, --version                              Prints version information

OPTIONS:
    -o, --output <output>                    Path to the directory where the output will be written [default: .]
    -p, --prime <prime>                      To choose the prime number to use to generate the circuit. Receives the
                                             name of the curve (bn128, bls12377, bls12381, goldilocks, grumpkin, pallas,
                                             secq256r1, vesta) [default: bn128]
    -l <link_libraries>...                   Adds directory to library search path
        --O2round <simplification_rounds>    Maximum number of rounds of the simplification process
        --sanity_check <sanity_check>        Selects the level of sanity checks to be included in the witness generation
                                             code generated. It receives the value 0, 1, 2, or 3. [default: 2]

ARGS:
    <input>    Path to a circuit with a main component [default: ./circuit.circom]
```

In the following, we explain these options.


##### Flags and options related to the compiler's output
* Flag ```--r1cs``` outputs the constraints in binary R1CS format (see the detailed format [here](https://github.com/iden3/r1csfile/blob/master/doc/r1cs_bin_format.md)).
* Flag ```--sym``` outputs for every signal of the circuit: the unique number given by the compiler, the circom qualified name, the number of the witness signal that contains it and the (unique) number of the component (given by the compiler) it belongs (see the detailed format and examples [here](../circom-language/formats/sym.md)).
* Flag ```--simplification_substitution``` outputs the substitutions performed by the --O1 (default) and --O2 constraint simplification options in json format (see the detailed format [here](../circom-language/formats/simplification-json.md)).
* Flag ```--wasm``` produces a WebAssembly program that receives the private and public inputs and generates the circuit witness.
* Flag ```-c / --c``` produces a C++ program that receives the private and public inputs and generates the circuit witness.
    * When flag ```--no_asm``` is activated the generated C code does not use asm files, this is a new version that can be used in all architectures
* Flag ```--wat``` compiles the circuit to wat.
* Flag ```--json``` outputs the R1CS system in JSON format (see the detailed format [here](../circom-language/formats/constraints-json.md)).
* Flag ```--no_asm``` (combined with the flag ```--c```) generates C++ code without using asm files. This makes the generated code compatible with any architecture.
* Flag ```--no_init``` avoids the introduction of an initialization to 0 in the wasm and C++ code for every declared ```var``` in the circom program.
* Option ```-o / --output <output>``` allows to indicate the path to the directory where the output will be written. By default the path is ```.```. 
* Option ```--sanity_check```  controls the amount of runtime checks inserted into the witness generation code:
  0 (no sanity checks are added), 1 (insert assert statements for <== assignments), 2 (in addition to level 1, each component verifies that all its subcomponents have been executed with all required inputs set). 
  By default the option is 2. 

##### Flags and options related to the constraint generation process
* Flag ```--verbose``` shows logs with known values at compilation time during the constraint generation process. 
* Flag ```--inspect``` does an additional check over the R1CS system produced. (see [--inspect](../circom-language/code-quality/inspect.md)).
* Flag ```--use_old_simplification_heuristics``` allows to use an old heuristics of the optimization algorithm. However, it is not recommended since the new heuristics has produced better results in practice.

##### Flags and options related to the R1CS optimization
In the following, we explain the different optimizations that we can apply to the final R1CS during the constraint generation phase. More details on the simplification process can be found [here](../circom-language/circom-insight/simplification.md)

* Flag ```--O0``` does not apply any kind of simplification.
  
* Flag ```--O1``` removes two kinds of simple constraints: a) ```signal = K```, being K is a constant in $F_p$ and b) ```signal1 = signal2```, which usually appears when linking components inputs and outputs. 
  
* Flag ```--O2``` applies Gauss elimination to remove as many linear constraints as possible. After applying the substitutions discovered by the algorithm, non-linear constraints may become linear. Thus, the Gauss elimination is applied during several rounds until no more linear constraints are discovered.

* Option ```--O2round <simplification_rounds>``` is similar to ```--O2```but it limits the maximum number of rounds applied during the optimization. In ```<simplification_rounds>```, user needs to indicate the number of rounds. 

Only one of these flags/options must be used during the compilation.

##### Other flags and options
* Option ```-p, --prime <prime>``` allows the user indicate which prime must be used during the compilation. Currently, it admits six different primes: bn128, bls12377, bls12381, goldilocks, grumpkin, pallas, secq256r1 and vesta. If not indicated, the default prime is bn128.

* Option ```-l <link_libraries>``` adds the provided directory in ```<link_libraries>```to the library search path. It is possible to add as much ```-l <link_libraries>``` as needed, but only one directory per option.

* Flag ```--no_init``` removes variable initializations to 0. 

* Flag ```-v / --version``` prints the version information.
* Flag ```-h / --help``` prints the help information.
