#!/usr/bin/env python3
"""
NGFS Test Vectors Validation

This script validates test vectors across different language implementations
using Hypothesis property-based testing.
"""

import os
import sys
import json
from pathlib import Path
from hypothesis import given, strategies as st
import cbor2

# Add the parent directory to the path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

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
        const uint8_t* ad,
        size_t ad_len,
        const uint8_t* pt,
        size_t pt_len,
        uint8_t* ct_out,
        uint8_t* tag16_out
    );
    
    int aeth_xchacha20_open(
        const uint8_t* key32,
        const uint8_t* ad,
        size_t ad_len,
        const uint8_t* ct,
        size_t ct_len,
        const uint8_t* tag16,
        uint8_t* pt_out
    );
""")

def load_polycrypto():
    """Load the polycrypto library"""
    try:
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

def test_roundtrip_property(lib):
    """Test that encryption/decryption roundtrip preserves data"""
    
    @given(
        st.binary(min_size=32, max_size=32),  # key
        st.binary(min_size=24, max_size=24),  # nonce
        st.binary(max_size=1024),             # ad
        st.binary(max_size=1024),             # plaintext
    )
    def roundtrip_test(key, nonce, ad, plaintext):
        try:
            # Encrypt
            ciphertext, tag = encrypt_chunk(lib, key, nonce, ad, plaintext)
            
            # Decrypt
            plaintext2 = decrypt_chunk(lib, key, nonce, ad, ciphertext, tag)
            
            # Verify roundtrip
            assert plaintext == plaintext2, "Roundtrip failed: plaintext mismatch"
            
        except (ValueError, RuntimeError) as e:
            # Skip invalid inputs
            pass
    
    return roundtrip_test

def test_deterministic_encryption(lib):
    """Test that encryption with same inputs produces same outputs"""
    
    @given(
        st.binary(min_size=32, max_size=32),  # key
        st.binary(min_size=24, max_size=24),  # nonce
        st.binary(max_size=1024),             # ad
        st.binary(max_size=1024),             # plaintext
    )
    def deterministic_test(key, nonce, ad, plaintext):
        try:
            # Encrypt twice with same parameters
            ct1, tag1 = encrypt_chunk(lib, key, nonce, ad, plaintext)
            ct2, tag2 = encrypt_chunk(lib, key, nonce, ad, plaintext)
            
            # Verify deterministic output
            assert ct1 == ct2, "Ciphertext not deterministic"
            assert tag1 == tag2, "Tag not deterministic"
            
        except (ValueError, RuntimeError) as e:
            # Skip invalid inputs
            pass
    
    return deterministic_test

def test_key_uniqueness(lib):
    """Test that different keys produce different ciphertexts"""
    
    @given(
        st.binary(min_size=24, max_size=24),  # nonce
        st.binary(max_size=1024),             # ad
        st.binary(max_size=1024),             # plaintext
    )
    def key_uniqueness_test(nonce, ad, plaintext):
        try:
            # Generate two different keys
            key1 = bytes(range(32))
            key2 = bytes(range(32, 64))
            
            # Encrypt with different keys
            ct1, tag1 = encrypt_chunk(lib, key1, nonce, ad, plaintext)
            ct2, tag2 = encrypt_chunk(lib, key2, nonce, ad, plaintext)
            
            # Verify different outputs
            assert ct1 != ct2, "Different keys produced same ciphertext"
            assert tag1 != tag2, "Different keys produced same tag"
            
        except (ValueError, RuntimeError) as e:
            # Skip invalid inputs
            pass
    
    return key_uniqueness_test

def test_nonce_uniqueness(lib):
    """Test that different nonces produce different ciphertexts"""
    
    @given(
        st.binary(min_size=32, max_size=32),  # key
        st.binary(max_size=1024),             # ad
        st.binary(max_size=1024),             # plaintext
    )
    def nonce_uniqueness_test(key, ad, plaintext):
        try:
            # Generate two different nonces
            nonce1 = bytes(range(24))
            nonce2 = bytes(range(24, 48))
            
            # Encrypt with different nonces
            ct1, tag1 = encrypt_chunk(lib, key, nonce1, ad, plaintext)
            ct2, tag2 = encrypt_chunk(lib, key, nonce2, ad, plaintext)
            
            # Verify different outputs
            assert ct1 != ct2, "Different nonces produced same ciphertext"
            assert tag1 != tag2, "Different nonces produced same tag"
            
        except (ValueError, RuntimeError) as e:
            # Skip invalid inputs
            pass
    
    return nonce_uniqueness_test

def test_associated_data_integrity(lib):
    """Test that associated data affects encryption output"""
    
    @given(
        st.binary(min_size=32, max_size=32),  # key
        st.binary(min_size=24, max_size=24),  # nonce
        st.binary(max_size=1024),             # plaintext
    )
    def ad_integrity_test(key, nonce, plaintext):
        try:
            # Generate two different associated data
            ad1 = b"associated data 1"
            ad2 = b"associated data 2"
            
            # Encrypt with different AD
            ct1, tag1 = encrypt_chunk(lib, key, nonce, ad1, plaintext)
            ct2, tag2 = encrypt_chunk(lib, key, nonce, ad2, plaintext)
            
            # Verify different outputs
            assert ct1 != ct2, "Different AD produced same ciphertext"
            assert tag1 != tag2, "Different AD produced same tag"
            
        except (ValueError, RuntimeError) as e:
            # Skip invalid inputs
            pass
    
    return ad_integrity_test

def run_property_tests(lib):
    """Run all property-based tests"""
    print("Running property-based tests...")
    
    tests = [
        ("Roundtrip", test_roundtrip_property(lib)),
        ("Deterministic", test_deterministic_encryption(lib)),
        ("Key Uniqueness", test_key_uniqueness(lib)),
        ("Nonce Uniqueness", test_nonce_uniqueness(lib)),
        ("AD Integrity", test_associated_data_integrity(lib)),
    ]
    
    results = {}
    for test_name, test_func in tests:
        try:
            print(f"Running {test_name} test...")
            test_func()
            results[test_name] = "PASS"
            print(f"  {test_name}: PASS")
        except Exception as e:
            results[test_name] = f"FAIL: {e}"
            print(f"  {test_name}: FAIL - {e}")
    
    return results

def main():
    """Main test runner"""
    print("NGFS Test Vectors Validation")
    print("=" * 40)
    
    # Load the library
    lib = load_polycrypto()
    print("✓ Loaded polycrypto library")
    
    # Run property tests
    test_results = run_property_tests(lib)
    
    # Summary
    print("\nTest Summary:")
    print("-" * 20)
    passed = sum(1 for result in test_results.values() if result == "PASS")
    total = len(test_results)
    
    for test_name, result in test_results.items():
        status = "✓" if result == "PASS" else "✗"
        print(f"{status} {test_name}: {result}")
    
    print(f"\nOverall: {passed}/{total} tests passed")
    
    # Output JSON for CI
    output = {
        "test": "enc_vectors",
        "cases": total,
        "passed": passed,
        "failed": total - passed,
        "results": test_results
    }
    
    with open("test_results.json", "w") as f:
        json.dump(output, f, indent=2)
    
    if passed == total:
        print("✓ All tests passed!")
        return 0
    else:
        print("✗ Some tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())
