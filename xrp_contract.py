from xrpl.clients import JsonRpcClient
from xrpl.models.transactions import Payment
from xrpl.wallet import Wallet
from xrpl.utils import xrp_to_drops
from xrpl.transaction import submit_and_wait

from web3 import Web3
from eth_account import Account
from dotenv import load_dotenv
import os

class XRPContract:
    def __init__(self, server_url="https://s.altnet.rippletest.net:51234", source_wallet_seed=None, metamask_private_key=None):
        # Variables:
        # 
        # client - XRPL RPC Client
        # w3 - XRPL EVM Sidechain
        # slush_pool - Wallet used to store deposited XRP
        # source_wallet - User's wallet
        # eth_account - User's wallet (MetaMask)

        self.client = JsonRpcClient(server_url)
        self.w3 = Web3(Web3.HTTPProvider('https://rpc-evm-sidechain.xrpl.org'))

        slush_fund_private_key = os.getenv('SLUSH_FUND_PRIVATE_KEY')
        if not slush_fund_private_key.startswith('0x'):
            slush_fund_private_key = '0x' + slush_fund_private_key        
        self.slush_pool = Account.from_key(slush_fund_private_key)
        print(f"MetaMask slush pool wallet imported successfully. Address: {self.slush_pool.address}")

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
    
    async def send_xrp(self, action="deposit", destination_address=None, amount_xrp=10):
        # Send XRP to specified address
        if action != "deposit" and action != "withdraw":
            raise Exception(f"parameter action must be \"deposit\" or \"withdraw\", not {action}")

        if self.source_wallet:
            if action == "deposit":
                sender = self.source_wallet.classic_address
                receiver = self.slush_pool.address
            else:
                sender = self.slush_pool.address
                receiver = destination_address
            
            try:
                payment = Payment(
                    account=sender,
                    amount=xrp_to_drops(amount_xrp),
                    destination=receiver
                )

                response = await submit_and_wait(payment, self.client, self.source_wallet)
                return response
            except Exception as e:
                print(f"Source Wallet Error: {str(e)}")
                raise e
        
        # If MetaMask wallet is found
        elif self.eth_account:
            if action == "deposit":
                sender = self.eth_account.address
                receiver = self.slush_pool.address
            else:
                sender = self.slush_pool.address
                receiver = destination_address
            
            print(sender)
            print(receiver)
            print(amount_xrp)
            try:
                # Convert XRP to Wei (18 decimals for EVM)
                amount_wei = self.w3.to_wei(amount_xrp, 'ether')                
                transaction = {
                    'from': sender,
                    'to': receiver,
                    'value': amount_wei,
                    'nonce': self.w3.eth.get_transaction_count(sender),
                    'gas': 21000, 
                    'gasPrice': self.w3.eth.gas_price,
                    'chainId': self.w3.eth.chain_id
                }
                
                signed_txn = self.eth_account.sign_transaction(transaction)
                tx_hash = self.w3.eth.send_raw_transaction(signed_txn.raw_transaction)
                tx_receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)
                return tx_receipt
            except Exception as e:
                print(f"MetaMask Wallet Error: {str(e)}")
                raise e
        else:
            raise Exception("Issue initializing wallets")

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