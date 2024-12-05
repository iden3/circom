from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import subprocess
import os
import re
import json
from xrp_contract import XRPContract
import asyncio
from dotenv import load_dotenv
import threading
import queue
from concurrent.futures import ThreadPoolExecutor
import time
from typing import Optional

load_dotenv('./.env')

app = FastAPI()

# 配置 CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 初始化 XRP contract
xrp_contract = XRPContract(metamask_private_key=os.getenv('METAMASK_PRIVATE_KEY'))

# 定义请求模型
class DepositRequest(BaseModel):
    amount: float
    currency: Optional[str] = "XRP"

class WithdrawRequest(BaseModel):
    proof: dict
    recipient: str

async def execute_generate_call():
    """Helper function to execute the generate call script"""
    try:
        script_path = os.path.join(os.getcwd(), "make_plonk_contract.sh")
        output_dir = os.path.join(os.getcwd(), "commitmentproof_js")
        output_file_path = os.path.join(output_dir, "generatecall_output.txt")

        if not os.path.exists(script_path):
            raise HTTPException(status_code=500, detail="The 'make_plonk_contract.sh' script does not exist")

        process = await asyncio.create_subprocess_exec(
            "bash", script_path,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
        
        stdout, stderr = await process.communicate()
        
        if process.returncode != 0:
            raise HTTPException(status_code=500, detail=f"Error occurred while running the script: {stderr.decode()}")

        address_match = re.search(r'Contract deployed at address: (\w+)', stdout.decode())
        contract_address = address_match.group(1) if address_match else "Unknown"

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
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/deposit")
async def deposit_get():
    return {"success": True, "message": "GET request successful"}

@app.post("/deposit")
async def deposit_post(request: DepositRequest):
    try:
        if request.currency != 'XRP':
            raise HTTPException(status_code=400, detail="Unsupported currency. Only XRP is allowed.")

        # 获取存款地址
        if xrp_contract.eth_account:
            deposit_address = xrp_contract.eth_account.address
        elif xrp_contract.source_wallet:
            deposit_address = xrp_contract.source_wallet.classic_address
        else:
            raise HTTPException(status_code=500, detail="No wallet initialized")

        result = await execute_generate_call()
        
        proof = result.get("output", {})
        public_signals = result.get("output", {})

        return {
            "success": True,
            "message": "Deposit information generated successfully",
            "deposit_address": deposit_address,
            "amount": request.amount,
            "currency": request.currency,
            "snark_proof": proof,
            "public_signals": public_signals
        }

    except HTTPException as e:
        raise e
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/withdraw")
async def withdraw(request: WithdrawRequest):
    try:
        # 定义默认提现金额
        amount = 10

        # 执行 XRP 转账
        response = await xrp_contract.send_xrp(request.recipient, amount)

        if response and xrp_contract.verify_transaction(response.result['hash']):
            return {
                "success": True,
                "message": "Withdrawal successful",
                "transaction_hash": response.result['hash']
            }
        else:
            raise HTTPException(status_code=500, detail="Withdrawal failed. Transaction could not be verified.")

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=5000)