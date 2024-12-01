#!/bin/bash

UUID=$(uuidgen)
commitmentproof_circom="commitmentproof_${UUID}.circom"
commitmentproof_wasm="commitmentproof_${UUID}.wasm"
commitmentproof_js="commitmentproof_js_${UUID}"
commitmentproof_r1cs="commitmentproof_${UUID}.r1cs"
commitmentproof_zkey="commitmentproof_${UUID}.zkey"
commitmentproof_witness="witness_${UUID}.wtns"
verification_key="verification_key_${UUID}.json"
proof_json="proof.json"
public_json="public_${UUID}.json"
verifier_sol="verifier_${UUID}.sol"
random_name="Contribution_$(uuidgen)"

echo "Generating commitmentproof.circom..."
cp commitmentproof.circom $commitmentproof_circom

echo "Compiling the circom file..."
circom $commitmentproof_circom --r1cs --wasm --sym --c

cd commitmentproof_js

echo "Running generate_secrets.py..."
python3 generate_secrets.py

echo "Generated input.json in the current directory..."

echo "Generating witness..."
node generate_witness.js commitmentproof.wasm input.json $commitmentproof_witness

echo "Generating powers of tau..."
snarkjs powersoftau new bn128 12 pot12_0000.ptau -v

echo "Feeding random contribution name..."
echo "$random_name" | snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v

echo "Preparing phase 2..."
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

echo "Performing the setup phase..."
snarkjs plonk setup ../$commitmentproof_r1cs pot12_final.ptau $commitmentproof_zkey

echo "Exporting the verification key..."
snarkjs zkey export verificationkey $commitmentproof_zkey $verification_key

echo "Generating the proof..."
snarkjs plonk prove $commitmentproof_zkey $commitmentproof_witness $proof_json $public_json

echo "Verifying the proof..."
snarkjs plonk verify $verification_key $public_json $proof_json

echo "Exporting the Solidity Verifier..."
snarkjs zkey export solidityverifier $commitmentproof_zkey $verifier_sol

echo "Generating output of generatecall ..."
snarkjs generatecall $public_json > generatecall_output.txt

echo "The verifier Solidity contract is saved as verifier_${UUID}.sol"

echo "Fixing parameter syntax error in contract..."
python3 fix_sol_files.py verifier_${UUID}.sol

cd ..
cp -f commitmentproof_js/verifier_${UUID}.sol contracts/verifier_${UUID}.sol

echo "Compiling contract..."
npx hardhat compile

echo "Deploying contract..."
node deploy.mjs verifier_${UUID}.sol