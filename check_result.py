import os
import json

output_file_path = os.path.join('./commitmentproof_js', 'generatecall_output.txt')

with open(output_file_path, 'r') as file:
    output = file.read()
    # output_json = json.loads(output)
    split_list = output.split("][")
    proof = split_list[0][1:]
    public_signals = split_list[1][:-1]
    print(f'proof: {proof}')
    print(f'public_signals: {public_signals}')