from flask import Flask, jsonify, request
import subprocess
import os
import json
from xrp_contract import XRPContract
import asyncio

app = Flask(__name__)
xrp_contract = XRPContract()

@app.route('/generatecall', methods=['POST'])
def generatecall():
    try:
        # Define the script path in the current working directory
        script_path = os.path.join(os.getcwd(), "make_plonk_contract.sh")
        
        # Define the directory for the output file
        output_dir = os.path.join(os.getcwd(), "commitmentproof_js")
        output_file_path = os.path.join(output_dir, "generatecall_output.txt")

        # Ensure the script exists in the current directory
        if not os.path.exists(script_path):
            return jsonify({
                "success": False,
                "message": "The 'make_plonk_contract.sh' script does not exist in the current directory."
            }), 500

        # Execute the script directly
        process = subprocess.run(
            ["bash", script_path],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )

        # Capture the output
        stdout = process.stdout
        stderr = process.stderr

        # Check for errors
        if process.returncode != 0:
            return jsonify({
                "success": False,
                "message": "Error occurred while running the script.",
                "error": stderr
            }), 500

        # Check if the output file exists in the commitmentproof_js directory
        if os.path.exists(output_file_path):
            with open(output_file_path, 'r') as file:
                output = file.read()

            # Ensure proper JSON formatting, remove extra escaping
            try:
                # Parse the JSON array and return it as a proper JSON response
                output_json = json.loads(output)
            except json.JSONDecodeError:
                output_json = output  # If not valid JSON, return the raw output as a string
        else:
            output_json = "Error: generatecall output not found."

        # Return the output as JSON
        return jsonify({
            "success": True,
            "message": "Script executed successfully.",
            "output": output_json
        })

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
        
        if not destination_address:
            return jsonify({
                "success": False,
                "message": "Destination address is required"
            }), 400

        # Initialize contract wallet
        contract_wallet = xrp_contract.create_wallet()
        
        # Send XRP
        response = await xrp_contract.send_xrp(
            contract_wallet,
            destination_address
        )
        
        # Verify transaction
        if xrp_contract.verify_transaction(response.result['hash']):
            # Generate snarkjs call after successful XRP transfer
            return await generatecall()
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
