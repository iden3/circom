---
description: This tutorial will guide you through the installation of circom and snarkJS.
---

<!-- 
TODO add and mini explain ffjavascript
Put links to all the docs
-->

# Installing the circom ecosystem

## &#9888; Important deprecation note

The old `circom` compiler written in Javascript will be frozen, but it can still be downloaded from the [old circom repository](https://github.com/iden3/circom_old).

## Installing dependencies

You need several dependencies in your system to 
run `circom` and its associated tools.

   * The core tool is the `circom` compiler which is written in Rust.
   To have Rust available in your system, you can install `rustup`. If you’re using Linux or macOS, open a terminal and enter the following command:

<!-- 
TODO remove the command and put a link to rustup site 
-->

```shell
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

   * We also distribute a series of npm packages so `Node.js` and some package manager like `npm` or `yarn` should be available in your system. Recent versions of `Node.js` include big integer support and web assembly compilers that help run code faster, so to get a better performance, install version 10 or higher.

## Installing circom

To install from our sources, clone the `circom` repository: 

```text
git clone https://github.com/iden3/circom.git
```

Enter the circom directory and use the cargo build to compile:

```text
cargo build --release
```

The installation takes around 3 minutes to be completed.
When the command successfully finishes, it generates the `circom` binary in the directory `target/release`. 
You can install this binary as follows:

```text
cargo install --path circom
```

The previous command will install the `circom` binary in the directory 
`$HOME/.cargo/bin`. 

Now, you should be able to see all the options of the executable by using the `help` flag:

```console
circom --help

   Circom Compiler 2.0.0
   IDEN3
   Compiler for the Circom programming language

   USAGE:
      circom [FLAGS] [OPTIONS] [input]

   FLAGS:
      -h, --help       Prints help information
         --inspect    Does an additional check over the constraints produced
         --O0         No simplification is applied
      -c, --c          Compiles the circuit to c
         --json       outputs the constraints in json format
         --r1cs       outputs the constraints in r1cs format
         --sym        outputs witness in sym format
         --wasm       Compiles the circuit to wasm
         --wat        Compiles the circuit to wat
         --O1         Only applies var to var and var to constant simplification
      -V, --version    Prints version information

   OPTIONS:
         --O2 <full_simplification>    Full constraint simplification [default: full]
      -o, --output <output>             Path to the directory where the output will be written [default: .]

   ARGS:
      <input>    Path to a circuit with a main component [default: ./circuit.circom]
```

## Installing snarkjs <a id="installing-the-tools"></a>

`snarkjs` is a npm package that contains code to generate and validate ZK proofs from the artifacts produced by `circom`. 

You can install `snarkjs` with the following command:

```text
npm install -g snarkjs
```
