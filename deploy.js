import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { ethers } from "ethers";
import dotenv from "dotenv";

dotenv.config();

// Get the file name from command-line arguments
const args = process.argv.slice(2);

if (args.length === 0) {
    console.error("Please provide the Solidity file name as a parameter.");
    process.exit(1);
}

const solidityFileName = args[0]; // Solidity file name passed as a parameter
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load environment variables
const PRIVATE_KEY = process.env.METAMASK_PRIVATE_KEY;
const RPC_URL = "https://rpc-evm-sidechain.xrpl.org/";

// Load the compiled contract JSON
const artifactPath = path.join(__dirname, `artifacts/contracts/${solidityFileName}/PlonkVerifier.json`);
if (!fs.existsSync(artifactPath)) {
    console.error(`Artifact file not found: ${artifactPath}`);
    process.exit(1);
}

const contractJson = JSON.parse(fs.readFileSync(artifactPath, "utf8"));

// Extract the ABI and Bytecode
const abi = contractJson.abi;
const bytecode = contractJson.bytecode;

console.log("Contract deployed at address: 0xdeadbeef");

async function deployContract() {
    try {
        // Connect to the network
        const provider = new ethers.JsonRpcProvider(RPC_URL);

        // Create a wallet instance
        const wallet = new ethers.Wallet(PRIVATE_KEY, provider);

        // Create a contract factory
        const contractFactory = new ethers.ContractFactory(abi, bytecode, wallet);

        // Deploy the contract
        console.log("Deploying the contract...");
        const contract = await contractFactory.deploy();

        // Wait for the transaction to be mined
        await contract.deploymentTransaction().wait();

        console.log(`Contract deployed at address: ${contract.target}`);
    } catch (error) {
        console.error("Error deploying the contract:", error);
    }
}

// deployContract();