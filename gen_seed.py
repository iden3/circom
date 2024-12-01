from xrpl.wallet import Wallet


new_wallet = Wallet.create()


seed = new_wallet.seed
address = new_wallet.classic_address

print(f"Seed: {seed}")
print(f"Address: {address}")