from flask import Flask, jsonify, request,send_file
import subprocess
import os
import re
import json
from xrp_contract import XRPContract
import asyncio
from dotenv import load_dotenv
from flask_cors import CORS
import threading
import queue

load_dotenv('./.env')

app = Flask(__name__)
# CORS(app)
CORS(app, resources={r"/*": {
    "origins": ["*"],  # Replace "*" with specific origins, e.g., ["http://localhost:3000"]
    "allow_headers": ["*"],  # Allow all headers
    "methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],  # Specify allowed methods
    "supports_credentials": True  # Allow credentials like cookies or Authorization headers
}})

# Initialize XRP contract with source wallet seed from environment variable
# xrp_contract = XRPContract(metamask_private_key=os.getenv('METAMASK_PRIVATE_KEY'))
xrp_contract = XRPContract()

    

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
            print(f"Output file not found: {output_file_path}")

        
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


@app.route('/deposit', methods=['GET', 'POST'])
async def deposit():
    """
    Endpoint to handle deposit requests.
    Input:
        - amount: The amount of XRP to transfer.
        - currency: The currency type (e.g., XRP).
    """
    if request.method == 'GET':
        return jsonify({"success": True, "message": "GET request successful"}), 200
    elif request.method == 'POST':
        try:
            data = request.get_json()
            print(data)
            amount = data.get('amount')
            currency = data.get('currency', 'XRP')  # Default currency is XRP

            if not amount:
                return jsonify({
                    "success": False,
                    "message": "Amount is required"
                }), 400

            if currency != 'XRP':
                return jsonify({
                    "success": False,
                    "message": "Unsupported currency. Only XRP is allowed."
                }), 400

            # Step 1: Get deposit address
            if xrp_contract.eth_account:
                deposit_address = xrp_contract.eth_account.address
                print(f'deposit_address: {deposit_address}')
            elif xrp_contract.source_wallet:
                deposit_address = xrp_contract.source_wallet.classic_address
                print(f'deposit_address: {deposit_address}')
            else:
                print("No wallet initialized")
                return jsonify({
                    "success": False,
                    "message": "No wallet initialized"
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
            print(f'result: {result}')
            print(f'status_code: {status_code}')
            
            if not isinstance(result, dict) or not isinstance(status_code, int):
                raise ValueError("Invalid response from execute_generate_call()")

            if not result.get("success"):
                return jsonify({
                    "success": False,
                    "message": "Failed to generate SNARK proof",
                    "details": result.get("message")
                }), 500
            
            
            proof = result.get("proof", {})
            print(f'proof: {proof}')
            public_signals = result.get("public_signals", {})
            print(f'public_signals: {public_signals}')

            contract_address = result.get("contract_address", {})
            contract_abi = result.get("contract_abi", {})
      

            return jsonify({
                "success": True,
                "message": "Deposit information generated successfully",
                "deposit_address": deposit_address,
                "amount": amount,
                "currency": currency,
                "snark_proof": proof,
                "public_signals": public_signals,
                "contract_address": contract_address,
                "contract_abi": contract_abi
            }), 200

        except Exception as e:
            return jsonify({
                "success": False,
                "message": "An exception occurred.",
                "error": str(e)
            }), 500



# @app.route('/withdraw', methods=['POST'])
# async def withdraw():
#     """
#     Endpoint to handle withdrawal requests.
#     Input:
#         - proof: The SNARK proof for verification.
#         - recipient: The recipient's XRP address.
#     Output:
#         - success: Whether the withdrawal was successful.
#         - message: A status message.
#     """
#     try:
#         # Parse the input data
#         data = request.get_json()
#         print(data)
#         proof = data.get('proof')
#         recipient = data.get('recipient')

#         print(f'proof: {proof}')
#         print(f'recipient: {recipient}')

#         # Validate inputs
#         if not proof:
#             return jsonify({
#                 "success": False,
#                 "message": "Proof is required"
#             }), 400

#         if not recipient:
#             return jsonify({
#                 "success": False,
#                 "message": "Recipient address is required"
#             }), 400

#         # Define the amount to transfer
#         amount = 10  # Default amount for withdrawal

#         # Perform the XRP transfer
#         response = await xrp_contract.send_xrp(recipient, amount)

#         print(f'response: {response}')

#         # Verify transaction success
#         if response and xrp_contract.verify_transaction(response.result['hash']):
#             return jsonify({
#                 "success": True,
#                 "message": "Withdrawal successful",
#                 "transaction_hash": response.result['hash']
#             }), 200
#         else:
#             return jsonify({
#                 "success": False,
#                 "message": "Withdrawal failed. Transaction could not be verified."
#             }), 500

#     except Exception as e:
#         return jsonify({
#             "success": False,
#             "message": "An exception occurred.",
#             "error": str(e)
#         }), 500


# @app.route('/withdraw2', methods=['GET', 'POST'])
# async def withdraw2():
#     """
#     Endpoint to handle withdrawal requests.
#     """
#     try:
#         # Parse the input data
#         data = request.get_json()
#         proof = data.get('proof')
#         recipient = data.get('recipient')

#         print(f"Received data: {data}", flush=True)

#         # Validate inputs
#         if not proof:
#             return jsonify({"success": False, "message": "Proof is required"}), 400
#         if not recipient:
#             return jsonify({"success": False, "message": "Recipient address is required"}), 400

#         amount = 10  # Default withdrawal amount

#         # 异步调用 xrp_contract.send_xrp
#         response = await xrp_contract.send_xrp(recipient, amount)

#         # 验证交易
#         if response and xrp_contract.verify_transaction(response.result['hash']):
#             return jsonify({
#                 "success": True,
#                 "message": "Withdrawal successful",
#                 "transaction_hash": response.result['hash']
#             }), 200
#         else:
#             return jsonify({
#                 "success": False,
#                 "message": "Withdrawal failed. Transaction could not be verified."
#             }), 500

#     except Exception as e:
#         print(f"Error: {str(e)}", flush=True)
#         return jsonify({
#             "success": False,
#             "message": "An exception occurred.",
#             "error": str(e)
#         }), 500

# @app.route('/withdraw3', methods=['GET', 'POST'])
# def withdraw3():
#     """
#     Endpoint to handle withdrawal requests.
#     """
#     try:
#         # 获取输入数据
#         data = request.get_json()
#         proof = data.get('proof')
#         recipient = data.get('recipient')

#         print(f"Received data: {data}", flush=True)

#         # 验证输入
#         if not proof:
#             return jsonify({"success": False, "message": "Proof is required"}), 400
#         if not recipient:
#             return jsonify({"success": False, "message": "Recipient address is required"}), 400

#         amount = 10  # 默认提现金额

#         # 使用队列和线程处理异步任务
#         result_queue = queue.Queue()

#         def background_task():
#             try:
#                 # 异步调用 send_xrp
#                 loop = asyncio.new_event_loop()
#                 asyncio.set_event_loop(loop)
#                 response = loop.run_until_complete(xrp_contract.send_xrp(recipient, amount))

#                 # loop.close()

#                 # loop = asyncio.get_event_loop()

#                 # 提交协程到事件循环中运行
#                 # future = asyncio.run_coroutine_threadsafe(xrp_contract.send_xrp(recipient, amount), loop)
#                 # response = future.result()  # 获取协程返回值

#                 if response and xrp_contract.verify_transaction(response.result['hash']):
#                     result_queue.put({
#                         "success": True,
#                         "message": "Withdrawal successful",
#                         "transaction_hash": response.result['hash']
#                     })
#                 else:
#                     result_queue.put({
#                         "success": False,
#                         "message": "Withdrawal failed. Transaction could not be verified."
#                     })
#             except Exception as e:
#                 result_queue.put({
#                     "success": False,
#                     "message": f"An error occurred: {str(e)}"
#                 })

#         # 启动后台任务
#         thread = threading.Thread(target=background_task)
#         thread.start()
#         thread.join()

#         # 获取队列中的结果
#         try:
#             result = result_queue.get_nowait()
#             print(f"Result: {result}", flush=True)
#         except queue.Empty:
#             result = {"success": False, "message": "Background task failed."}

#         return jsonify(result), (200 if result.get("success") else 500)

#     except Exception as e:
#         print(f"Error: {str(e)}", flush=True)
#         return jsonify({
#             "success": False,
#             "message": "An exception occurred.",
#             "error": str(e)
#         }), 500

@app.route('/withdraw', methods=['POST'])
async def withdraw():
    """
    Endpoint to handle withdrawal requests.
    """
    try:
        data = request.get_json()
        proof = data.get('proof')
        recipient = data.get('recipient')

        if not proof:
            return jsonify({"success": False, "message": "Proof is required"}), 400
        if not recipient:
            return jsonify({"success": False, "message": "Recipient address is required"}), 400

        amount = 10

        # 异步调用 submit_and_wait
        response = await xrp_contract.send_xrp(recipient, amount)
        print(f"Response: {response}", flush=True)

        if response and xrp_contract.verify_transaction(response['transactionHash'].hex()):
            return jsonify({
                "success": True,
                "message": "Withdrawal successful",
                "transaction_hash": response['transactionHash'].hex()
            }), 200
        else:
            return jsonify({
                "success": False,
                "message": "Withdrawal failed. Transaction could not be verified."
            }), 500

    except Exception as e:
        print(f"Error: {str(e)}", flush=True)
        return jsonify({
            "success": False,
            "message": "An exception occurred.",
            "error": str(e)
        }), 500


@app.route('/', methods=['GET'])
def index():
    return send_file('website.html')


if __name__ == '__main__':
    app.run(debug=False, host='0.0.0.0', port=5002)
    # print(app.url_map)

    # app.run(debug=True, host='0.0.0.0', port=5002)