# Proving circuits

After compiling the circuit and running the witness calculator with 
an appropriate input, we will have a file with extension .wtns that 
contains all the computed signals and, a file with extension .r1cs that contains the constraints describing the circuit. Both files will be used to create our proof.

Now, we will use the `snarkjs` tool to generate and validate a proof for our input. In particular, using the multiplier2, **we will prove that we are able to provide the two factors of the number 33**. That is, we will show that we know two integers `a` and `b` such that when we multiply them, it results in the number 33.

We are going to use the [Groth16](https://eprint.iacr.org/2016/260) zk-SNARK protocol.
To use this protocol, you will need to generate a [trusted setup](../../background/background#trusted-setup).
**Groth16 requires a per circuit trusted setup**. In more detail, the trusted setup consists of 2 parts:

- The powers of tau, which is independent of the circuit.
- The phase 2, which depends on the circuit. 

Next, we provide a very basic ceremony for creating the trusted setup and we also provide the basic commands to create and verify [Groth16](https://eprint.iacr.org/2016/260) proofs. Review the related [Background](../../background/background) section and check [the snarkjs tutorial](https://github.com/iden3/snarkjs) for further information.

## Powers of Tau <a id="my-first-trusted-setup"></a>

<!-- 
You can access the **help** of `snarkjs` by typing the command:

`$ snarkjs --help`

You can get general **statistics** of the circuit and print the **constraints**. Just run:

```text
snarkjs info -c multiplier2.r1cs 
snarkjs print -r multiplier2.r1cs -s multiplier2.sym
```
-->

First, we start a new "powers of tau" ceremony:

```text
snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
```

Then, we contribute to the ceremony:

```text
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v
```

Now, we have the contributions to the powers of tau in the file *pot12_0001.ptau* and 
we can proceed with the Phase 2.

## Phase 2 <a id="my-first-trusted-setup"></a>

The **phase 2** is **circuit-specific**. If you are using a proving system that does not require a trusted setup, you don't need to go through phase 2 and can directly proceed to the next [section](#generating-a-proof). In this example we are using [Groth16](https://eprint.iacr.org/2016/260), which requires a trusted setup.

Execute the following command to start the generation of this phase:

```text
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
```

Next, we generate a `.zkey` file that will contain the proving and verification keys together with all phase 2 contributions.
Execute the following command to start a new zkey:

```text
snarkjs groth16 setup multiplier2.r1cs pot12_final.ptau multiplier2_0000.zkey
```

Contribute to the phase 2 of the ceremony:

```text
snarkjs zkey contribute multiplier2_0000.zkey multiplier2_0001.zkey --name="1st Contributor Name" -v
```

We should be prompted to enter some text to provide a random source of entropy.

Finalize phase 2 of the trusted setup by adding a beacon on the latest zkey.
This is necessary in order to generate a final zkey file and finalize phase 2 of the trusted setup.

```text
snarkjs zkey beacon multiplier2_0001.zkey multiplier2_final.zkey 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon phase2"
```

We can perform a final check in order to verify the final zkey.
```text
snarkjs zkey verify multiplier2.r1cs pot12_final.ptau multiplier2_final.zkey
```

Finally, export the verification key:
```text
snarkjs zkey export verificationkey multiplier2_final.zkey verification_key.json
```

Please ensure that you only use the final `zkey` file for this step, as using the other `zkey` files might result in invalid verification keys.

## Generating a Proof

Once the witness is computed and the trusted setup is already executed, we can **generate a zk-proof** associated to the circuit and the witness:

```text
snarkjs groth16 prove multiplier2_final.zkey witness.wtns proof.json public.json
```

This command generates a [Groth16](https://eprint.iacr.org/2016/260) proof and outputs two files:

* `proof.json`: it contains the proof.
* `public.json`: it contains the values of the public inputs and outputs.

## Verifying a Proof

To **verify the proof**, execute the following command:

```text
snarkjs groth16 verify verification_key.json public.json proof.json
```

The command uses the files `verification_key.json` we exported earlier,`proof.json` and `public.json` to check if the proof is valid. If the proof is valid, the command outputs an `OK`.

A valid proof not only proves that we know a set of signals that satisfy the circuit, but also that the public inputs and outputs that we use match the ones described in the `public.json` file.

## Verifying from a Smart Contract

​👉 It is also possible to generate a **Solidity verifier** that allows **verifying proofs on Ethereum blockchain**.

First, we need to generate the Solidity code using the command:

```text
snarkjs zkey export solidityverifier multiplier2_final.zkey verifier.sol
```

This command takes validation key `multiplier2_final.zkey` and outputs Solidity code in a file named `verifier.sol`. You can take the code from this file and cut and paste it in Remix. You will see that the code contains two contracts: `Pairing` and `Verifier`. You only need to deploy the `Verifier` contract.

Note that you should only generate the verifier using the final `zkey` file. If you use the other `zkey` files the verifier generated will not properly verify the proof given.

You may want to use first a testnet like Goerli or Sepolia. You can also use the JavaScript VM, but in some browsers the verification takes long and the page may freeze.

The `Verifier` has a `view` function called `verifyProof` that returns `TRUE` if and only if the proof and the inputs are valid. To facilitate the call, you can use `snarkJS` to generate the parameters of the call by typing:

```text
snarkjs generatecall
```

Cut and paste the output of the command to the parameters field of the `verifyProof` method in Remix. If everything works fine, this method should return `TRUE`. You can try to change just a single bit of the parameters, and you will see that the result is verifiable `FALSE`.

