from xrpl.clients import JsonRpcClient
from xrpl.models.transactions import Payment
from xrpl.wallet import Wallet
from xrpl.utils import xrp_to_drops
from xrpl.transaction import submit_and_wait

class XRPContract:
    def __init__(self, server_url="https://s.altnet.rippletest.net:51234", source_wallet_seed=None):
        self.client = JsonRpcClient(server_url)
        # self.source_wallet = self.create_wallet("2c9e0d3cdc9fbd1bea04dd6bb127f6ac0a2f48df236b70ebaf85a5d6f5f125e8") # Private key for MetaMask wallet connected to XRP EVM sidechain
        self.source_wallet = self.create_wallet(source_wallet_seed)
        
    def create_wallet(self, seed=None):
        # Create a new XRP wallet or load existing one
        if seed:
            return Wallet.from_seed(seed)
        return Wallet.create()
    
    async def send_xrp(self, destination_address, amount_xrp=10):
        # Send XRP from source wallet to specified address
        payment = Payment(
            account=self.source_wallet.classic_address,
            amount=xrp_to_drops(amount_xrp),
            destination=destination_address
        )
        
        # Submit and wait for validation
        response = await submit_and_wait(payment, self.client, self.source_wallet)
        return response
    
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
