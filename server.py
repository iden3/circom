from flask import Flask, jsonify, request,send_file
from flask_cors import CORS
from web3 import Web3
import subprocess
import threading
import queue
import os
import re
import json
from dotenv import load_dotenv
from xrp_contract import XRPContract
from eth_account import Account

load_dotenv('./.env')

app = Flask(__name__)
CORS(app, resources={r"/*": {
    "origins": ["*"],  # Replace "*" with specific origins, e.g., ["http://localhost:3000"]
    "allow_headers": ["*"],  # Allow all headers
    "methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],  # Specify allowed methods
    "supports_credentials": True  # Allow credentials like cookies or Authorization headers
}})

# Initialize XRP contract with source wallet seed from environment variable
slush_fund_private_key = os.getenv('SLUSH_FUND_PRIVATE_KEY')
withdraw_xrp_contract = XRPContract(metamask_private_key=slush_fund_private_key)

if not slush_fund_private_key.startswith('0x'):
    slush_fund_private_key = '0x' + slush_fund_private_key        
slush_pool = Account.from_key(slush_fund_private_key)

w3 = Web3(Web3.HTTPProvider('https://rpc-evm-sidechain.xrpl.org'))

# Initialize dictionary to exchange proofs for contract details
contracts = {}

def execute_generate_call():
    """Helper function to execute the generate call script"""
    try:
        script_path = os.path.join(os.getcwd(), "make_plonk_contract.sh")
        output_dir = os.path.join(os.getcwd(), "commitmentproof_js")
        output_file_path = os.path.join(output_dir, "generatecall_output.txt")

        if not os.path.exists(script_path):
            return {
                "success": False,
                "message": "The 'make_plonk_contract.sh' script does not exist in the current directory."
            }, 500

        process = subprocess.run(
            ["bash", script_path],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )

        if process.returncode != 0:
            return {
                "success": False,
                "message": "Error occurred while running the script.",
                "error": process.stderr
            }, 500

        contract_deployment_error_match = re.search(r'Error deploying the contract: Error: ([^\(]+)', process.stderr)

        if contract_deployment_error_match:
            return {
                "success": False,
                "message": "Contract deployment failed.",
                "error": contract_deployment_error_match.group(1)
            }, 500

        address_match = re.search(r'Contract deployed at address: (\w+)', process.stdout)
        abi_match = re.search(r'Contract ABI: (\[.*)', process.stdout, re.DOTALL)

        if address_match:
            contract_address = address_match.group(1)
        else:
            contract_address = "Unknown"

        if abi_match:
            contract_abi = abi_match.group(1)
        else:
            contract_abi = "Unknown"

        if os.path.exists(output_file_path):
            with open(output_file_path, 'r') as file:
                output = file.read()
                split_list = output.split("][")
                proof = split_list[0][1:]
                public_signals = split_list[1][:-1]
        else:
            print(f"[SERVER]: Output file not found: {output_file_path}")

        
        return {
            "success": True,
            "message": "Script executed successfully.",
            "contract_address": contract_address,
            "contract_abi": contract_abi,
            "proof": proof,
            "public_signals": public_signals
        }, 200

    except Exception as e:
        return {
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }, 500


@app.route('/mixer_generate_proof', methods=['POST'])
async def mixer_generate_proof():
    data = request.get_json()
    sender = data.get('sender')
    result_queue = queue.Queue()

    def background_task():
        try:
            result, status_code = execute_generate_call()
            result_queue.put((result, status_code))
        except Exception as e:
            result_queue.put(({"success": False, "message": str(e)}, 500))
    
    thread = threading.Thread(target=background_task)
    thread.start()
    thread.join()

    try:
        result, status_code = result_queue.get_nowait()
    except queue.Empty:
        result, status_code = {"success": False, "message": "Background task failed"}, 500
        
    if not isinstance(result, dict) or not isinstance(status_code, int):
        raise ValueError("Invalid response from execute_generate_call()")

    if not result.get("success"):
        return jsonify({
            "success": False,
            "message": "Deposit failed. Could not generate SNARK proof",
            "details": result.get("message")
        }), 500

    proof = result.get("proof", {})
    public_signals = result.get("public_signals", {})
    contract_address = result.get("contract_address", {})
    contract_abi = result.get("contract_abi", {})

    proof_key = proof.replace(" ", "")
    contracts[proof_key] = {
        "contract_address": contract_address,
        "contract_abi": contract_abi,
        "public_signals": public_signals
    }

    return jsonify({
        "success": True,
        "message": "Created proof successfully.",
        "proof": proof,
        "public_signals": public_signals,
        "contract_address": contract_address,
        "contract_abi": contract_abi
    }), 200


@app.route('/mixer_verify_proof', methods=['POST'])
def mixer_verify_proof():
    data = request.get_json()
    sender = data.get('sender')
    proof = data.get('proof')
    # print(f'contracts: {contracts}')
    contract_info = contracts[proof]
    contract_address = contract_info["contract_address"]
    contract_abi = contract_info["contract_abi"]
    public_signals = contract_info["public_signals"]

    # print(f"[{sender}]: Deployed proof validation smart contract at {contract_address}")

    # Verify proof
    proof_contract = w3.eth.contract(address=contract_address, abi=json.loads(contract_abi))
    proof_param = [int(elem, 16) for elem in proof.replace("\"", "").replace(" ", "").split(",")]
    public_signals_param = [int(public_signals.replace("\"", "").replace("]", ""), 16)]
    # print('---------------------------verifiy proof---------------------------')
    # print(f"proof: {proof}")
    # print(f"contract_address: {contract_address}")
    # print(f"contract_abi: {contract_abi}")
    # print(f"proof_param: {proof_param}")
    # print(f"public_signals_param: {public_signals_param}")
    # print(f"send: {send}")
    try:
        result = proof_contract.functions.verifyProof(proof_param, public_signals_param).call({'from': sender})
        print(f'result of verification on server.py: {result}')
        if result == False:
            return jsonify({
                "success": False,
                "message": "Deposit failed. Smart contract function verifyProof returned false."
            }), 400  
        else:
            print(f"[{sender}]: Verified proof")
            return jsonify({
                "success": True,
                "message": "Proof verified successfully."
            }), 200  
    except Exception as e:
        return jsonify({
            "success": False,
            "message": "Deposit failed. Smart contract function verifyProof had an error.",
            "error": str(e)
        }), 500            
    

@app.route('/deposit', methods=['POST'])
async def deposit():
    """
    Endpoint to handle deposit requests.
    """
    try:
        data = request.get_json()
        sender = data.get('sender')
        amount = data.get('amount')
        currency = data.get('currency', 'XRP')  # Default currency is XRP
        print(f"[{sender}]: Creating deposit transaction...")

        if not amount:
            return jsonify({
                "success": False,
                "message": "Amount is required."
            }), 400

        if currency != 'XRP':
            return jsonify({
                "success": False,
                "message": "Unsupported currency. Only XRP is allowed."
            }), 400

        # Step 1: Get address of recipient
        if slush_pool:
            recipient = slush_pool.address
        else:
            print(f"[{sender}]: Slush pool wallet not initialized")
            return jsonify({
                "success": False,
                "message": "Deposit failed. Slush pool wallet not initialized"
            }), 500

        # Step 2: Generate the SNARK proof using the `generatecall` function
        result_queue = queue.Queue()

        def background_task():
            try:
                result, status_code = execute_generate_call()
                result_queue.put((result, status_code))
            except Exception as e:
                result_queue.put(({"success": False, "message": str(e)}, 500))
        
        thread = threading.Thread(target=background_task)
        thread.start()
        thread.join()

        try:
            result, status_code = result_queue.get_nowait()
        except queue.Empty:
            result, status_code = {"success": False, "message": "Background task failed"}, 500
            
        if not isinstance(result, dict) or not isinstance(status_code, int):
            raise ValueError("Invalid response from execute_generate_call()")

        if not result.get("success"):
            return jsonify({
                "success": False,
                "message": "Deposit failed. Could not generate SNARK proof",
                "details": result.get("message")
            }), 500

        proof = result.get("proof", {})
        public_signals = result.get("public_signals", {})
        contract_address = result.get("contract_address", {})
        contract_abi = result.get("contract_abi", {})
        print(f"[{sender}]: Deployed proof validation smart contract at {contract_address}")

        # Step 3: Verify proof
        proof_contract = w3.eth.contract(address=contract_address, abi=json.loads(contract_abi))
        proof_param = [int(elem, 16) for elem in proof.replace("\"", "").replace(" ", "").split(",")]
        public_signals_param = [int(public_signals.replace("\"", "").replace("]", ""), 16)]
        try:
            result = proof_contract.functions.verifyProof(proof_param, public_signals_param).call({'from': sender})
            if result == False:
                return jsonify({
                    "success": False,
                    "message": "Deposit failed. Smart contract function verifyProof returned false."
                }), 400  
        except Exception as e:
            return jsonify({
                "success": False,
                "message": "Deposit failed. Smart contract function verifyProof had an error.",
                "error": str(e)
            }), 500            
        print(f"[{sender}]: Verified proof")

        # Step 4: Save data to server
        proof_key = proof.replace(" ", "")
        contracts[proof_key] = {
            "amount": amount,
            "contract_address": contract_address,
            "contract_abi": contract_abi,
            "public_signals": public_signals
        }
        print(f"[{sender}]: Saved contract address, contract abi, and public signals to dictionary")

        # Step 5: Prepare XRP Deposit to Slush Pool
        amount_wei = w3.to_wei(amount, 'ether')
                
        transaction = {
            'from': sender,
            'to': recipient,
            'value': hex(amount_wei),
            'nonce': hex(w3.eth.get_transaction_count(sender)),
            'gas': hex(21000),
            'gasPrice': hex(w3.eth.gas_price),
            'chainId': w3.eth.chain_id
        }
        print(f"[{sender}]: Created transaction to send {amount} {currency} to {recipient}\n")

        return jsonify({
            "success": True,
            "message": "Created deposit transaction.",
            "transaction": transaction,
            "proof_key": proof_key
        }), 200

    except Exception as e:
        return jsonify({
            "success": False,
            "message": "Deposit failed. An exception occurred.",
            "error": str(e)
        }), 500



@app.route('/remove_withdrawal_request', methods=['POST'])
async def remove_withdrawal_request():
    """
    Endpoint to handle requests to remove dictionary KV pairs.
    """
    try:
        data = request.get_json()
        proof_key = data.get('proof_key')
        print("[SERVER]: Remove withdrawal request executing...")

        # Step 1: Remove data to server
        if proof_key not in contracts:
            return jsonify({
                "success": False,
                "message": "Removal failed. No contract associated with provided proof."
            }), 400
        contracts.pop(proof_key, None)
        print(f"[SERVER]: Removed KV pair from dictionary\n")

        return jsonify({
            "success": True,
            "message": "Removed withdrawal request.",
        }), 200

    except Exception as e:
        return jsonify({
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }), 500



@app.route('/withdraw', methods=['POST'])
async def withdraw():
    """
    Endpoint to handle withdrawal requests.
    """
    try:
        data = request.get_json()
        sender = data.get('sender')
        proof = data.get('proof')
        recipient = data.get('recipient')
        print(f"[{sender}]: Withdraw executing...")

        if not proof:
            return jsonify({"success": False, "message": "Proof is required"}), 400
        if not recipient:
            return jsonify({"success": False, "message": "Recipient address is required"}), 400
        
        # Step 1: Verify proof
        if proof not in contracts:
            return jsonify({
                "success": False,
                "message": "Withdrawal failed. No contract associated with provided proof."
            }), 400

        contract_info = contracts[proof]
        amount = contract_info["amount"]
        contract_address = contract_info["contract_address"]
        contract_abi = contract_info["contract_abi"]
        public_signals = contract_info["public_signals"]

        proof_contract = w3.eth.contract(address=contract_address, abi=json.loads(contract_abi))
        proof_param = [int(elem, 16) for elem in proof.replace("\"", "").replace(" ", "").split(",")]
        public_signals_param = [int(public_signals.replace("\"", "").replace("]", ""), 16)]
        try:
            result = proof_contract.functions.verifyProof(proof_param, public_signals_param).call({'from': sender})
            if result == False:
                return jsonify({
                    "success": False,
                    "message": "Withdrawal failed. Smart contract function verifyProof returned false."
                }), 400  
        except Exception as e:
            return jsonify({
                "success": False,
                "message": "Withdrawal failed. Smart contract function verifyProof had an error.",
                "error": str(e)
            }), 500            
        print(f"[{sender}]: Verified proof")

        # Step 2: Withdraw XRP to Slush Pool
        response = await withdraw_xrp_contract.send_xrp(destination_address=recipient, amount_xrp=amount)
        if not response or not withdraw_xrp_contract.verify_transaction(response['transactionHash'].hex()):
            return jsonify({
                "success": False,
                "message": "Withdrawal failed. Transaction failed or could not be verified."
            }), 400
        print(f"[{sender}]: Withdrew {amount} XRP from the slush pool to {recipient}")

        # Step 3: Remove data from server
        contracts.pop(proof, None)
        print(f"[{sender}]: Removed KV pair from dictionary\n")

        # Step 4: Return data to frontend
        return jsonify({
            "success": True,
            "message": "Withdrawal successful.",
        }), 200
    
    except Exception as e:
        return jsonify({
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }), 500



@app.route('/baseline_send_xrp', methods=['POST'])
async def send():
    """
    Endpoint to handle send XRP requests.
    """
    try:
        data = request.get_json()
        sender = data.get('sender')
        amount = data.get('amount')
        recipient = data.get('recipient')
        xrp_contract = XRPContract(metamask_private_key=os.getenv('METAMASK_PRIVATE_KEY'))
        
        response = await xrp_contract.send_xrp(destination_address=recipient, amount_xrp=amount)
        if not response or not xrp_contract.verify_transaction(response['transactionHash'].hex()):
            return jsonify({
                "success": False,
                "message": "XRP send transaction could not be verified.",
                "error": str(e)
            }), 500

        return jsonify({
            "success": True,
            "message": "Send successful.",
        }), 200
    except Exception as e:
        return jsonify({
            "success": False,
            "message": "Send failed. An exception occurred.",
            "error": str(e)
        }), 500


@app.route('/', methods=['GET'])
def index():
    return send_file('website.html')


if __name__ == '__main__':
    app.run(debug=False, host='0.0.0.0', port=5002)