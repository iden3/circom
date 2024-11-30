from xrpl.clients import JsonRpcClient
from xrpl.models.transactions import Payment
from xrpl.wallet import Wallet
from xrpl.utils import xrp_to_drops
from xrpl.transaction import submit_and_wait
from web3 import Web3
from eth_account import Account
import secrets

class XRPContract:
    def __init__(self, server_url="https://s.altnet.rippletest.net:51234", source_wallet_seed=None, metamask_private_key=None):
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
        else:
            raise Exception("XRP wallet not initialized")
    
    def verify_transaction(self, tx_hash):
        # Verify the transaction was successful
        try:
            tx_response = self.client.request(
                "tx",
                {"transaction": tx_hash}
            )
            return tx_response.result.get("validated", False)
        except Exception as e:
            return False
