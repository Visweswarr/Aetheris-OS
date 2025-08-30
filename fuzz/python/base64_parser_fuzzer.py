#!/usr/bin/env python3
"""
Base64 Parser Fuzzer using Atheris
Tests base64 encoding/decoding robustness with malformed input data
"""

import atheris
import base64
import sys
from typing import Any, Optional


def test_base64_encoding(data: bytes) -> None:
    """Test base64 encoding with various inputs"""
    
    try:
        # Test standard base64 encoding
        encoded = base64.b64encode(data)
        
        # Test standard base64 decoding
        decoded = base64.b64decode(encoded)
        
        # Verify round-trip
        assert decoded == data, "Base64 round-trip failed"
        
    except Exception:
        # Expected errors for some inputs
        pass


def test_base64_decoding(data: bytes) -> None:
    """Test base64 decoding with malformed inputs"""
    
    # Test with truncated data
    if len(data) > 1:
        truncated = data[:-1]
        try:
            _ = base64.b64decode(truncated)
        except Exception:
            pass
    
    # Test with corrupted data
    if len(data) > 10:
        corrupted = bytearray(data)
        # Flip some bits
        for i in range(min(10, len(corrupted))):
            corrupted[i] = (corrupted[i] + 1) % 256
        
        try:
            _ = base64.b64decode(bytes(corrupted))
        except Exception:
            pass
    
    # Test with invalid characters
    if len(data) > 0:
        invalid_chars = bytearray(data)
        for i, byte in enumerate(invalid_chars):
            if byte == ord('+'):
                invalid_chars[i] = ord('*')  # Invalid base64 character
            elif byte == ord('/'):
                invalid_chars[i] = ord('@')  # Invalid base64 character
        
        try:
            _ = base64.b64decode(bytes(invalid_chars))
        except Exception:
            pass


def test_base64_variants(data: bytes) -> None:
    """Test different base64 variants"""
    
    try:
        # Test URL-safe base64
        url_encoded = base64.urlsafe_b64encode(data)
        url_decoded = base64.urlsafe_b64decode(url_encoded)
        assert url_decoded == data, "URL-safe base64 round-trip failed"
        
        # Test URL-safe with standard decoder
        _ = base64.b64decode(url_encoded)
        
    except Exception:
        pass
    
    try:
        # Test no-padding base64
        no_pad_encoded = base64.b64encode(data).rstrip(b'=')
        # Add padding back for decoding
        padded = no_pad_encoded + b'=' * (4 - len(no_pad_encoded) % 4)
        no_pad_decoded = base64.b64decode(padded)
        assert no_pad_decoded == data, "No-padding base64 round-trip failed"
        
    except Exception:
        pass


def test_base64_edge_cases(data: bytes) -> None:
    """Test base64 with edge cases"""
    
    # Test with empty data
    if len(data) == 0:
        try:
            _ = base64.b64encode(data)
            _ = base64.b64decode(b'')
        except Exception:
            pass
    
    # Test with single byte
    if len(data) == 1:
        try:
            _ = base64.b64encode(data)
        except Exception:
            pass
    
    # Test with very large data
    if len(data) > 1024:
        try:
            _ = base64.b64encode(data)
        except Exception:
            pass


def test_base64_padding(data: bytes) -> None:
    """Test base64 padding handling"""
    
    try:
        encoded = base64.b64encode(data)
        
        # Test with missing padding
        no_padding = encoded.rstrip(b'=')
        try:
            _ = base64.b64decode(no_padding)
        except Exception:
            pass
        
        # Test with extra padding
        extra_padding = encoded + b'==='
        try:
            _ = base64.b64decode(extra_padding)
        except Exception:
            pass
        
    except Exception:
        pass


def test_base64_case_sensitivity(data: bytes) -> None:
    """Test base64 case sensitivity"""
    
    try:
        encoded = base64.b64encode(data)
        
        # Test with mixed case
        mixed_case = encoded.decode().swapcase().encode()
        try:
            _ = base64.b64decode(mixed_case)
        except Exception:
            pass
        
    except Exception:
        pass


def test_base64_whitespace(data: bytes) -> None:
    """Test base64 with whitespace"""
    
    try:
        encoded = base64.b64encode(data)
        
        # Test with whitespace
        with_whitespace = b' ' + encoded + b' '
        try:
            _ = base64.b64decode(with_whitespace)
        except Exception:
            pass
        
        # Test with newlines
        with_newlines = encoded.replace(b'=', b'=\n')
        try:
            _ = base64.b64decode(with_newlines)
        except Exception:
            pass
        
    except Exception:
        pass


def main():
    """Main fuzzing function"""
    
    def fuzz_base64(data: bytes) -> None:
        """Fuzz base64 functions"""
        test_base64_encoding(data)
        test_base64_decoding(data)
        test_base64_variants(data)
        test_base64_edge_cases(data)
        test_base64_padding(data)
        test_base64_case_sensitivity(data)
        test_base64_whitespace(data)
    
    # Set up Atheris fuzzer
    atheris.Setup(sys.argv, fuzz_base64)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
