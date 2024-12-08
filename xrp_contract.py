from xrpl.clients import JsonRpcClient
from xrpl.models.transactions import Payment
from xrpl.wallet import Wallet
from xrpl.utils import xrp_to_drops
from xrpl.transaction import submit_and_wait
from web3 import Web3
from eth_account import Account

class XRPContract:
    def __init__(self, server_url="https://s.altnet.rippletest.net:51234", source_wallet_seed=None, metamask_private_key='2c9e0d3cdc9fbd1bea04dd6bb127f6ac0a2f48df236b70ebaf85a5d6f5f125e8'):
        self.client = JsonRpcClient(server_url)
        self.w3 = Web3(Web3.HTTPProvider('https://rpc-evm-sidechain.xrpl.org'))
        
        if metamask_private_key:
            if not metamask_private_key.startswith('0x'):
                metamask_private_key = '0x' + metamask_private_key
            self.eth_account = Account.from_key(metamask_private_key)
            self.source_wallet = None
            print(f"MetaMask wallet imported successfully. Address: {self.eth_account.address}")
        else:
            self.eth_account = None
            self.source_wallet = self.create_wallet(source_wallet_seed)
            if source_wallet_seed:
                print(f"XRP wallet imported successfully. Address: {self.source_wallet.classic_address}")
            else:
                print(f"New XRP wallet created. Address: {self.source_wallet.classic_address}")
        
    def create_wallet(self, seed=None):
        # Create a new XRP wallet or load existing one
        if seed:
            return Wallet.from_seed(seed)
        return Wallet.create()
    
    async def send_xrp(self, destination_address, amount_xrp=10):
        # Send XRP from source wallet to specified address
        if self.source_wallet:
            payment = Payment(
                account=self.source_wallet.classic_address,
                amount=xrp_to_drops(amount_xrp),
                destination=destination_address
            )
            
            # Submit and wait for validation
            response = await submit_and_wait(payment, self.client, self.source_wallet)
            return response
        # If MetaMask wallet is found
        elif self.eth_account:
            try:
                # Convert XRP to Wei (18 decimals for EVM)
                amount_wei = self.w3.to_wei(amount_xrp, 'ether')
                
                transaction = {
                    'from': self.eth_account.address,
                    'to': destination_address,
                    'value': amount_wei,
                    'nonce': self.w3.eth.get_transaction_count(self.eth_account.address),
                    'gas': 21000, 
                    'gasPrice': self.w3.eth.gas_price,
                    'chainId': self.w3.eth.chain_id
                }
                
                signed_txn = self.eth_account.sign_transaction(transaction)
                tx_hash = self.w3.eth.send_raw_transaction(signed_txn.raw_transaction)
                tx_receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)

                return tx_receipt
            except Exception as e:
                print(f"Detailed error: {str(e)}")
                raise e
        else:
            raise Exception("MetaMask wallet not initialized")
    
    def verify_transaction(self, tx_hash):
        # Verify the transaction was successful
        try:
            receipt = self.w3.eth.get_transaction_receipt(tx_hash)
            if receipt and receipt["status"] == 1:
                return True
            else:
                print(f"Transaction failed or not yet mined: {receipt}")
                return False
        except Exception as e:
            print(f"Detailed error: {str(e)}")
            return False