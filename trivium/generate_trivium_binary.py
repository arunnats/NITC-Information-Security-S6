import random
import os
from trivium import *
from concurrent.futures import ProcessPoolExecutor

# Constants
num_sequences = 100
sequence_length = 1_000_000  # 1 million bits
output_dir = "data"

os.makedirs(output_dir, exist_ok=True)

def generate_sequence(i):
    # Generate unique key and IV
    key = random.getrandbits(80)
    iv = random.getrandbits(80)

    # Initialize Trivium
    trivium = Trivium(key, iv)
    keystream = trivium.generate_keystream(sequence_length)

    # Save to binary file
    path = os.path.join(output_dir, f"trivium_keystream_{i}.bin")
    with open(path, 'wb') as f:
        current_byte = 0
        bit_count = 0
        for bit in keystream:
            current_byte = (current_byte << 1) | bit
            bit_count += 1
            if bit_count == 8:
                f.write(bytes([current_byte]))
                current_byte = 0
                bit_count = 0
        if bit_count > 0:
            current_byte <<= (8 - bit_count)
            f.write(bytes([current_byte]))
    return i

# Run in parallel
if __name__ == "__main__":
    with ProcessPoolExecutor() as executor:
        results = list(executor.map(generate_sequence, range(num_sequences)))
    print(f"Finished generating {len(results)} sequences.")

    # Concatenate all sequences into one file
    # with open('data/trivium_all_sequences.bin', 'wb') as outfile:
    #     for i in range(num_sequences):
    #         with open(f'data/trivium_keystream_{i}.bin', 'rb') as infile:
    #             outfile.write(infile.read())