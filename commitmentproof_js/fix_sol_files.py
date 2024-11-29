import os
import argparse

def fix_file(file_path):
    try:
        if not os.path.isfile(file_path):
            print(f"File not found: {file_path}")
            return
        
        with open(file_path, 'r') as file:
            content = file.read()
        
        updated_content = content.replace(' _proof', '')
        
        with open(file_path, 'w') as file:
            file.write(updated_content)
        
    except Exception as e:
        print(f"An error occurred: {e}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Remove ' _proof' from a Solidity file.")
    parser.add_argument("file_path", type=str, help="Path to the Solidity file")
    args = parser.parse_args()

    fix_file(args.file_path)
