from flask import Flask, jsonify, request
import subprocess
import os
import re
import json
from xrp_contract import XRPContract

app = Flask(__name__)
# Initialize XRP contract with source wallet seed from environment variable
xrp_contract = XRPContract(metamask_private_key=os.getenv('METAMASK_PRIVATE_KEY'))

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

        # Regular expression to match the address
        address_match = re.search(r'Contract deployed at address: (\w+)', process.stdout)

        if address_match:
            contract_address = address_match.group(1)
        else:
            contract_address = "Unknown"

        if os.path.exists(output_file_path):
            with open(output_file_path, 'r') as file:
                output = file.read()
            try:
                output_json = json.loads(output)
            except json.JSONDecodeError:
                output_json = output
        else:
            output_json = "Error: generatecall output not found."

        return {
            "success": True,
            "message": "Script executed successfully.",
            "output": output_json,
            "contract_address": contract_address
        }, 200

    except Exception as e:
        return {
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }, 500

@app.route('/generatecall', methods=['POST'])
def generatecall():
    # Endpoint to generate call
    result, status_code = execute_generate_call()
    return jsonify(result), status_code

@app.route('/get_deposit_address', methods=['GET'])
def get_deposit_address():
    """Get the address to deposit XRP"""
    try:
        if xrp_contract.eth_account:
            deposit_address = xrp_contract.eth_account.address
        elif xrp_contract.source_wallet:
            deposit_address = xrp_contract.source_wallet.classic_address
        else:
            return jsonify({
                "success": False,
                "message": "No wallet initialized"
            }), 500
            
        return jsonify({
            "success": True,
            "deposit_address": deposit_address,
            "message": "Send XRP to this address to make a deposit"
        }), 200
    except Exception as e:
        return jsonify({
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }), 500

@app.route('/deploy_contract', methods=['POST'])
async def deploy_contract():
    try:
        data = request.get_json()
        destination_address = data.get('destination_address')
        amount = data.get('amount', 10)  # Default amount is 10 XRP if not specified
        
        if not destination_address:
            return jsonify({
                "success": False,
                "message": "Destination address is required"
            }), 400

        # Send XRP from our wallet to the user's destination address
        response = await xrp_contract.send_xrp(destination_address, amount)
        
        # Verify transaction
        if xrp_contract.verify_transaction(response.result['hash']):
            return jsonify({
                "success": True,
                "message": "XRP transfer successful",
                "transaction_hash": response.result['hash']
            }), 200
        else:
            return jsonify({
                "success": False,
                "message": "XRP transfer failed"
            }), 500

    except Exception as e:
        return jsonify({
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }), 500

if __name__ == '__main__':
    app.run(debug=True)
