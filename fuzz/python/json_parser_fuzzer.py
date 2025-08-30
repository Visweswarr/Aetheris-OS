#!/usr/bin/env python3
"""
JSON Parser Fuzzer using Atheris
Tests JSON parsing robustness with malformed input data
"""

import atheris
import json
import sys
from typing import Any, Dict, List, Union


def test_json_parsing(data: bytes) -> None:
    """Test JSON parsing with various malformed inputs"""
    
    # Test with UTF-8 string conversion
    try:
        json_str = data.decode('utf-8')
    except UnicodeDecodeError:
        # Skip invalid UTF-8 data
        return
    
    # Test basic JSON parsing
    try:
        parsed = json.loads(json_str)
        # Test JSON serialization back to string
        _ = json.dumps(parsed)
    except (json.JSONDecodeError, TypeError, ValueError, OverflowError):
        # Expected errors for malformed JSON
        pass
    
    # Test with truncated data
    if len(data) > 1:
        truncated = data[:-1]
        try:
            truncated_str = truncated.decode('utf-8')
            _ = json.loads(truncated_str)
        except (UnicodeDecodeError, json.JSONDecodeError):
            pass
    
    # Test with corrupted data
    if len(data) > 10:
        corrupted = bytearray(data)
        # Flip some bits
        for i in range(min(10, len(corrupted)):
            corrupted[i] = (corrupted[i] + 1) % 256
        
        try:
            corrupted_str = corrupted.decode('utf-8')
            _ = json.loads(corrupted_str)
        except (UnicodeDecodeError, json.JSONDecodeError):
            pass
    
    # Test with nested structures
    if len(data) > 20:
        try:
            # Create complex JSON structure
            complex_json = f'''{{
                "string": "{json_str[:10]}",
                "number": {len(data)},
                "boolean": {len(data) % 2 == 0},
                "null": null,
                "array": [1, 2, 3],
                "object": {{"nested": "value"}}
            }}'''
            _ = json.loads(complex_json)
        except (json.JSONDecodeError, TypeError):
            pass
    
    # Test with very large numbers
    if len(data) > 0:
        try:
            large_number = str(len(data) * 1000000)
            _ = json.loads(large_number)
        except (json.JSONDecodeError, ValueError):
            pass
    
    # Test with escaped strings
    if len(data) > 0:
        try:
            escaped_string = f'"{json_str.replace("\\", "\\\\").replace('"', '\\"')}"'
            _ = json.loads(escaped_string)
        except (json.JSONDecodeError, ValueError):
            pass
    
    # Test with unicode data
    if len(data) > 0:
        try:
            unicode_string = f'"{json_str.encode("unicode_escape").decode()}"'
            _ = json.loads(unicode_string)
        except (json.JSONDecodeError, ValueError):
            pass


def test_json_parsing_edge_cases(data: bytes) -> None:
    """Test JSON parsing with edge cases"""
    
    # Test with empty data
    if len(data) == 0:
        try:
            _ = json.loads("")
        except json.JSONDecodeError:
            pass
    
    # Test with single character
    if len(data) == 1:
        try:
            single_char = data.decode('utf-8', errors='ignore')
            _ = json.loads(single_char)
        except (UnicodeDecodeError, json.JSONDecodeError):
            pass
    
    # Test with very large data
    if len(data) > 1024:
        try:
            large_str = data.decode('utf-8', errors='ignore')
            _ = json.loads(large_str)
        except (UnicodeDecodeError, json.JSONDecodeError):
            pass


def test_json_parsing_formats(data: bytes) -> None:
    """Test JSON parsing with different formats"""
    
    try:
        json_str = data.decode('utf-8')
    except UnicodeDecodeError:
        return
    
    # Test with different JSON formats
    formats = [
        json_str,
        json_str.replace('"', "'"),  # Single quotes
        json_str.replace('true', 'True'),  # Python boolean
        json_str.replace('false', 'False'),  # Python boolean
        json_str.replace('null', 'None'),  # Python None
    ]
    
    for fmt in formats:
        try:
            _ = json.loads(fmt)
        except (json.JSONDecodeError, ValueError):
            pass


def main():
    """Main fuzzing function"""
    
    def fuzz_json(data: bytes) -> None:
        """Fuzz JSON parsing functions"""
        test_json_parsing(data)
        test_json_parsing_edge_cases(data)
        test_json_parsing_formats(data)
    
    # Set up Atheris fuzzer
    atheris.Setup(sys.argv, fuzz_json)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
