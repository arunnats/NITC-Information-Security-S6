class Trivium:
    def __init__(self, key, iv):
        # Convert key and IV to bit arrays (80 bits each)
        self.key = self._convert_to_bits(key, 80)
        self.iv = self._convert_to_bits(iv, 80)
        self.state = [0] * 288
        self._initialize()
        
    def _convert_to_bits(self, value, length):
        # Convert an integer to a bit array of specified length
        bits = [0] * length
        for i in range(length):
            bits[i] = (value >> i) & 1
        return bits
    
    def _initialize(self):
        # Initialize state with key, IV and constants
        # (s1...s93) = (k1...k80, 0,...,0)
        for i in range(80):
            self.state[i] = self.key[i]
        
        # (s94...s177) = (iv1...iv80, 0,...,0)
        for i in range(80):
            self.state[i + 93] = self.iv[i]
        
        # (s178...s288) = (0,...,0,1,1,1)
        self.state[285] = self.state[286] = self.state[287] = 1
        
        # Warm-up: update state 4*288 = 1152 times
        for _ in range(1152):
            self._update_state()
    
    def _update_state(self):
        # Calculate feedback bits
        t1 = self.state[65] ^ self.state[92]
        t2 = self.state[161] ^ self.state[176]
        t3 = self.state[242] ^ self.state[287]
        
        t1 ^= (self.state[90] & self.state[91]) ^ self.state[170]
        t2 ^= (self.state[174] & self.state[175]) ^ self.state[263]
        t3 ^= (self.state[285] & self.state[286]) ^ self.state[68]
        
        # Shift registers and insert feedback
        for i in range(287, 0, -1):
            self.state[i] = self.state[i-1]
        
        self.state[0] = t3
        self.state[93] = t1
        self.state[177] = t2
        
        return self.state[65] ^ self.state[92] ^ self.state[161] ^ self.state[176] ^ self.state[242] ^ self.state[287]
    
    def generate_keystream(self, length):
        # Generate keystream bits
        keystream = []
        for _ in range(length):
            keystream.append(self._update_state())
        return keystream