# Include

Templates, like other code, can be found in other files like in libraries. In order to use code in other files, we have to include them in our program by using the keyword include, with the corresponding name of the file (.circom extension is the default).

```text
include "montgomery.circom";
include "mux3.circom";
include "babyjub.circom";
```

This piece of code includes the files `montgomery.circom`, `mux3.circom` and `babyjub.circom` from the circom library.

Since circom 2.0.8, option `-l` is available to indicate the paths where searching the files to be included. 
