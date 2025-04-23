from trivium import Trivium, int_to_bits, bits_to_int

def hex_to_bits(hex_str, total_bits):
    n = int(hex_str, 16)
    return int_to_bits(n, total_bits)

def bits_to_hex(bits):
    return hex(bits_to_int(bits))[2:].zfill(len(bits) // 4)

# === Example NIST/eSTREAM test vector ===
key_hex = '00000000000000000000'   # 80-bit key (20 hex chars)
iv_hex  = '00000000000000000000'   # 80-bit IV

key_bits = hex_to_bits(key_hex, 80)
iv_bits  = hex_to_bits(iv_hex, 80)

# Create cipher and generate keystream
trivium = Trivium(key_bits, iv_bits)
keystream_bits = trivium.get_keystream(64)  # Generate 64 bits of keystream

print("Keystream (bits):", keystream_bits)
print("Keystream (hex) :", bits_to_hex(keystream_bits))