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

[Circom](https://iden3.io/circom) is a novel domain-specific language for defining arithmetic circuits that can be used to generate zero-knowledge proofs. `Circom compiler` is a Circom language compiler written in Rust that can be used to generate a R1CS file with a set of associated constraints and a program (written either in C++ or WebAssembly) to efficiently compute a valid assignment to all wires of the circuit. One of the main particularities of `CIRCOM` is that it is a modular language that allows the definition of parameterizable small circuits called templates, which can be instantiated to form larger circuits. The idea of building circuits from small individual components makes it easier to test, review, audit, or formally verify large and complex CIRCOM circuits.In this regard, CIRCOM users can create their own custom templates, but they can also use templates from [CircomLib](https://github.com/iden3/circomlib), a publicly available library that counts with hundreds of circuits such as comparators, hash functions, digital signatures, binary and decimal converters, and many more. Circomlib is publicly available to practitioners and developers.

The implementations of proving systems are also available in our libraries including [SnarkJS](https://github.com/iden3/snarkjs), written in Javascript and Pure Web Assembly, [wasmsnark](https://github.com/iden3/wasmsnark) written in native Web Assembly, [rapidSnark](https://github.com/iden3/rapidsnark) written in C++ and Intel Assembly.

Circom aims to provide developers a holistic framework to construct arithmetic circuits through an easy-to-use interface and abstract the complexity of the proving mechanisms.

Circom language reference can be found at [Circom language reference](https://docs.circom.io/circom-language/signals)

At this time there are two available syntax highlighters: [Circom Visual Studio Code highlight syntax](https://github.com/iden3/circom-highlighting-vscode) and  [Circom Vim highlight syntax](https://github.com/iden3/vim-circom-syntax)

# Documentation
All documentation is available in [Circom 2 Documentation](https://docs.circom.io/), we encourage you to read it. If you are new start with the [Getting started section](https://docs.circom.io/getting-started/installation/).
Basic background on Zero-knowledge proofs can be found on [Background section](https://docs.circom.io/background/background/)

# Install

Refer to [Installation section](Installing the circom ecosystem)

## :warning: Deprecation note

The previous `circom 1` compiler written in Javascript is deprecated, but [circom 1 repository](https://github.com/iden3/circom_old) is still available

# Community
Thank you for considering contributing to the Circom & SnarkJS framework!

As the `circom` and `snarkjs` community grows new tools, circuits, or projects have appeared. Here we link some of them:

CIRCUITS

+ [0xPARC CIRCOM ECDSA circuit](https://github.com/0xPARC/circom-ecdsa)

TOOLS

+ [zkREPL an online playground for zk circuits](https://zkrepl.dev)



[ico-website]: https://img.shields.io/website?up_color=blue&up_message=circom&url=https%3A%2F%2Fiden3.io%2Fcircom
[ico-telegram]: https://img.shields.io/badge/@iden3-2CA5E0.svg?style=flat-square&logo=telegram&label=Telegram

[link-website]: https://iden3.io/circom
[link-telegram]: https://t.me/iden3io
