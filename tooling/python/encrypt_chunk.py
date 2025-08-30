#!/usr/bin/env python3
"""
NGFS Chunk Encryption Tool

This script encrypts file chunks using the polycrypto library
with XChaCha20-Poly1305 AEAD encryption.
"""

import argparse
import os
import sys
from pathlib import Path

# Add the parent directory to the path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from cffi import FFI
except ImportError:
    print("Error: cffi is required. Install with: pip install cffi")
    sys.exit(1)

# Define the C interface
ffi = FFI()
ffi.cdef("""
    #define AETH_KEY_SIZE 32
    #define AETH_NONCE_SIZE 24
    #define AETH_TAG_SIZE 16
    #define AETH_MAX_PAYLOAD_SIZE (256 * 1024)
    
    #define AETH_OK 0
    #define AETH_ERROR_INVALID_INPUT -1
    #define AETH_ERROR_CRYPTO_FAILURE -2
    #define AETH_ERROR_BUFFER_TOO_SMALL -3
    
    int aeth_xchacha20_seal(
        const uint8_t* key32,
        const uint8_t* nonce24,
        const uint8_t* ad,
        size_t ad_len,
        const uint8_t* pt,
        size_t pt_len,
        uint8_t* ct_out,
        uint8_t* tag16_out
    );
    
    int aeth_xchacha20_open(
        const uint8_t* key32,
        const uint8_t* nonce24,
        const uint8_t* ad,
        size_t ad_len,
        const uint8_t* ct,
        size_t ct_len,
        const uint8_t* tag16,
        uint8_t* pt_out
    );
    
    void aeth_memzero(void* p, size_t n);
""")

def load_polycrypto():
    """Load the polycrypto library"""
    try:
        # Try to load from the expected location
        lib_path = Path(__file__).parent.parent.parent / "c" / "crypto"
        
        if os.name == 'nt':  # Windows
            lib = ffi.dlopen(str(lib_path / "polycrypto.dll"))
        else:  # Unix-like
            lib = ffi.dlopen(str(lib_path / "libpolycrypto.so"))
            
        return lib
    except OSError as e:
        print(f"Error loading polycrypto library: {e}")
        print("Make sure the library is built and available")
        sys.exit(1)

def encrypt_chunk(lib, key, nonce, ad, plaintext):
    """Encrypt a chunk using XChaCha20-Poly1305"""
    if len(key) != 32:
        raise ValueError(f"Key must be 32 bytes, got {len(key)}")
    if len(nonce) != 24:
        raise ValueError(f"Nonce must be 24 bytes, got {len(nonce)}")
    if len(plaintext) > 256 * 1024:
        raise ValueError(f"Plaintext too large: {len(plaintext)} > 256 KiB")
    
    # Allocate output buffers
    ciphertext = ffi.new("uint8_t[]", len(plaintext))
    tag = ffi.new("uint8_t[]", 16)
    
    # Convert Python bytes to C arrays
    key_c = ffi.new("uint8_t[]", key)
    nonce_c = ffi.new("uint8_t[]", nonce)
    ad_c = ffi.new("uint8_t[]", ad) if ad else ffi.NULL
    pt_c = ffi.new("uint8_t[]", plaintext)
    
    # Call the C function
    result = lib.aeth_xchacha20_seal(
        key_c, nonce_c, ad_c, len(ad),
        pt_c, len(plaintext),
        ciphertext, tag
    )
    
    if result != 0:
        raise RuntimeError(f"Encryption failed with code {result}")
    
    # Convert C arrays back to Python bytes
    ct_bytes = bytes(ffi.buffer(ciphertext, len(plaintext)))
    tag_bytes = bytes(ffi.buffer(tag, 16))
    
    return ct_bytes, tag_bytes

def decrypt_chunk(lib, key, nonce, ad, ciphertext, tag):
    """Decrypt a chunk using XChaCha20-Poly1305"""
    if len(key) != 32:
        raise ValueError(f"Key must be 32 bytes, got {len(key)}")
    if len(nonce) != 24:
        raise ValueError(f"Nonce must be 24 bytes, got {len(nonce)}")
    if len(tag) != 16:
        raise ValueError(f"Tag must be 16 bytes, got {len(tag)}")
    
    # Allocate output buffer
    plaintext = ffi.new("uint8_t[]", len(ciphertext))
    
    # Convert Python bytes to C arrays
    key_c = ffi.new("uint8_t[]", key)
    nonce_c = ffi.new("uint8_t[]", nonce)
    ad_c = ffi.new("uint8_t[]", ad) if ad else ffi.NULL
    ct_c = ffi.new("uint8_t[]", ciphertext)
    tag_c = ffi.new("uint8_t[]", tag)
    
    # Call the C function
    result = lib.aeth_xchacha20_open(
        key_c, nonce_c, ad_c, len(ad),
        ct_c, len(ciphertext),
        tag_c, plaintext
    )
    
    if result != 0:
        raise RuntimeError(f"Decryption failed with code {result}")
    
    # Convert C array back to Python bytes
    pt_bytes = bytes(ffi.buffer(plaintext, len(ciphertext)))
    
    return pt_bytes

def generate_test_vectors(lib):
    """Generate test vectors for cross-language validation"""
    print("Generating test vectors...")
    
    # Fixed test data
    key = bytes(range(1, 33))  # 1, 2, 3, ..., 32
    nonce = bytes(range(100, 124))  # 100, 101, 102, ..., 123
    ad = b"associated data for testing"
    plaintext = b"plaintext data for encryption testing"
    
    # Encrypt
    ciphertext, tag = encrypt_chunk(lib, key, nonce, ad, plaintext)
    
    # Decrypt to verify
    plaintext2 = decrypt_chunk(lib, key, nonce, ad, ciphertext, tag)
    
    if plaintext != plaintext2:
        raise RuntimeError("Roundtrip failed: plaintext mismatch")
    
    # Write test vectors to files
    test_vectors = {
        "key.bin": key,
        "nonce.bin": nonce,
        "ad.bin": ad,
        "pt.bin": plaintext,
        "ct.bin": ciphertext,
        "tag.bin": tag,
    }
    
    for filename, data in test_vectors.items():
        with open(filename, "wb") as f:
            f.write(data)
        print(f"Generated {filename} ({len(data)} bytes)")
    
    print("Test vectors generated successfully")

def main():
    parser = argparse.ArgumentParser(description="NGFS Chunk Encryption Tool")
    parser.add_argument("--in", dest="input_file", help="Input file to encrypt")
    parser.add_argument("--out", dest="output_file", help="Output file for encrypted data")
    parser.add_argument("--key", help="32-byte hex key")
    parser.add_argument("--nonce", help="24-byte hex nonce")
    parser.add_argument("--ad", help="Associated data (hex)")
    parser.add_argument("--generate", action="store_true", help="Generate test vectors")
    parser.add_argument("--decrypt", action="store_true", help="Decrypt instead of encrypt")
    
    args = parser.parse_args()
    
    # Load the library
    lib = load_polycrypto()
    
    if args.generate:
        generate_test_vectors(lib)
        return
    
    if args.decrypt:
        # Decryption mode
        if not all([args.input_file, args.output_file, args.key, args.nonce, args.ad]):
            print("Error: --decrypt requires --in, --out, --key, --nonce, and --ad")
            sys.exit(1)
        
        # Read input files
        with open(args.input_file, "rb") as f:
            ciphertext = f.read()
        
        # Parse hex arguments
        key = bytes.fromhex(args.key)
        nonce = bytes.fromhex(args.nonce)
        ad = bytes.fromhex(args.ad) if args.ad else b""
        
        # Extract tag from ciphertext (last 16 bytes)
        if len(ciphertext) < 16:
            print("Error: Ciphertext too short")
            sys.exit(1)
        
        ct = ciphertext[:-16]
        tag = ciphertext[-16:]
        
        # Decrypt
        plaintext = decrypt_chunk(lib, key, nonce, ad, ct, tag)
        
        # Write output
        with open(args.output_file, "wb") as f:
            f.write(plaintext)
        
        print(f"Decrypted {len(plaintext)} bytes to {args.output_file}")
        
    else:
        # Encryption mode
        if not all([args.input_file, args.output_file, args.key, args.nonce]):
            print("Error: Encryption requires --in, --out, --key, and --nonce")
            sys.exit(1)
        
        # Read input file
        with open(args.input_file, "rb") as f:
            plaintext = f.read()
        
        # Parse hex arguments
        key = bytes.fromhex(args.key)
        nonce = bytes.fromhex(args.nonce)
        ad = bytes.fromhex(args.ad) if args.ad else b""
        
        # Encrypt
        ciphertext, tag = encrypt_chunk(lib, key, nonce, ad, plaintext)
        
        # Combine ciphertext and tag
        encrypted_data = ciphertext + tag
        
        # Write output
        with open(args.output_file, "wb") as f:
            f.write(encrypted_data)
        
        print(f"Encrypted {len(plaintext)} bytes to {args.output_file}")
        print(f"Ciphertext: {len(ciphertext)} bytes, Tag: {len(tag)} bytes")

if __name__ == "__main__":
    main()
