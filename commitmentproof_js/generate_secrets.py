import os
import poseidon
import json

FIELD_MODULUS = 18446744073709551359

def generate_secret():
    s = int.from_bytes(os.urandom(32), 'big')  # Secret value as an integer
    r = int.from_bytes(os.urandom(32), 'big')  # Random nonce as an integer
    return s, r

def create_commitment(s, r):
    s = s % FIELD_MODULUS
    r = r % FIELD_MODULUS

    poseidon_simple, t = poseidon.parameters.case_simple()

    input_vec = [s, r]

    poseidon_digest = poseidon_simple.run_hash(input_vec)
    return poseidon_digest

def main():
    s, r = generate_secret()
    poseidon_digest = create_commitment(s, r) 

    # Output the results as a JSON structure
    data = {
        "s": hex(s),
        "r": hex(r),
        "commitment": hex(int(poseidon_digest))
    }

    # Save the output to input.json
    with open('input.json', 'w') as json_file:
        json.dump(data, json_file, indent=4)

    # Print the results for verification
    print("Generated s:", hex(s))
    print("Generated r:", hex(r))
    print("Commitment:", hex(int(poseidon_digest)))

if __name__ == "__main__":
    main()