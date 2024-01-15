<div align="center">
<img src="mkdocs/docs/circom-logo-black.png" width="300"/>
</div>
<div align="center">

[![Chat on Telegram][ico-telegram]][link-telegram]
[![Website][ico-website]][link-website]
![GitHub top language](https://img.shields.io/github/languages/top/iden3/circom)

</div>

# About ==>circom

> CIRCUIT COMPILER FOR ZK PROVING SYSTEMS

[Circom](https://iden3.io/circom) is a novel domain-specific language for defining arithmetic circuits that can be used to generate zero-knowledge proofs. `Circom compiler` is a circom language compiler written in Rust that can be used to generate a R1CS file with a set of associated constraints and a program (written either in C++ or WebAssembly) to efficiently compute a valid assignment to all wires of the circuit. One of the main particularities of `circom` is its modularity that allows the programmers to define parameterizable circuits called templates, which can be instantiated to form larger circuits. The idea of building circuits from small individual components makes it easier to test, review, audit, or formally verify large and complex `circom` circuits. In this regard, `circom` users can create their own custom templates or instantiate templates from [circomLib](https://github.com/iden3/circomlib), a publicly available library that comes with hundreds of circuits such as comparators, hash functions, digital signatures, binary and decimal converters, and many more. Circomlib is publicly available to practitioners and developers.

The implementations of proving systems are also available in our libraries including [snarkjs](https://github.com/iden3/snarkjs), written in Javascript and Pure Web Assembly, [wasmsnark](https://github.com/iden3/wasmsnark) written in native Web Assembly, [rapidSnark](https://github.com/iden3/rapidsnark) written in C++ and Intel Assembly.

Circom aims to provide developers a holistic framework to construct arithmetic circuits through an easy-to-use interface and abstracting the complexity of the proving mechanisms.

Circom language reference can be found at [circom language reference](https://docs.circom.io/circom-language/signals).

At this time there are two available syntax highlighters: [circom Visual Studio Code highlight syntax](https://github.com/iden3/circom-highlighting-vscode) and  [circom Vim highlight syntax](https://github.com/iden3/vim-circom-syntax).

# Documentation
All documentation is available in [circom 2 Documentation](https://docs.circom.io/), we encourage you to read it. If you are new start with the [Getting started section](https://docs.circom.io/getting-started/installation/).
Basic background on Zero-knowledge proofs can be found on [Background section](https://docs.circom.io/background/background/).

# Install

Refer to [Installation section](https://docs.circom.io/getting-started/installation/)

## :warning: Deprecation note

The previous `circom 1` compiler written in Javascript is deprecated, but [circom 1 repository](https://github.com/iden3/circom_old) is still available.

# Community
Thank you for considering contributing to the circom & snarkjs framework!

As the `circom` and `snarkjs` community grows new tools, circuits, or projects have appeared. Here we link some of them:

CIRCUITS

+ [0xPARC circom ECDSA circuit](https://github.com/0xPARC/circom-ecdsa)

TOOLS

+ [zkREPL an online playground for zk circuits](https://zkrepl.dev)

+ [Shield: a development framework for circom developers](https://xord.notion.site/SHIELD-5306223ca4f745d19f54b9a5f4004cd6)

+ [Circomspect: a static analyzer for detecting common vulnerabilities in circom circuits that extends the checks performed by the circom flag --inspect](https://github.com/trailofbits/circomspect)

+ [CIVER: a static analyzer for specifying and verifying circom circuits (including weak-safety checks)](https://github.com/costa-group/circom_civer)

+ [Ecne: a static analyzer verifying weak and strong safety for circom circuits](https://github.com/franklynwang/EcneProject)

+ [PICUS: a static analyzer for verifying weak and strong safety for circom circuits](https://github.com/Veridise/Picus)

More information about the notions of weak and strong safety in circom circuits [here](https://ieeexplore.ieee.org/document/10002421).

[ico-website]: https://img.shields.io/website?up_color=blue&up_message=circom&url=https%3A%2F%2Fiden3.io%2Fcircom
[ico-telegram]: https://img.shields.io/badge/@iden3-2CA5E0.svg?style=flat-square&logo=telegram&label=Telegram

[link-website]: https://iden3.io/circom
[link-telegram]: https://t.me/iden3io
