#!/usr/bin/env python3
"""
Simple test script to validate protobuf structure and generation.
"""

import sys
import os
import json
from pathlib import Path

# Add the tooling directories to the path
sys.path.insert(0, str(Path(__file__).parent.parent / "tooling" / "python" / "ai_core"))

def test_protobuf_structure():
    """Test that our protobuf structure is valid."""
    print("Testing protobuf structure...")
    
    try:
        # Test importing the generated protobuf modules
        import ai_core_pb2
        print("✓ Successfully imported ai_core_pb2")
        
        # Test creating basic messages
        from message_builder import create_ping_request, create_chat_request
        
        # Test ping request
        ping_msg = create_ping_request("test_client", "1.0.0")
        print("✓ Successfully created ping request")
        
        # Test chat request
        chat_msg = create_chat_request("Hello, AI!")
        print("✓ Successfully created chat request")
        
        # Test CBOR utilities
        from cbor_utils import CborUtils
        
        cbor_utils = CborUtils()
        test_data = {"key": "value", "number": 42}
        
        # Test encoding
        encoded = cbor_utils.encode_to_json(test_data)
        print("✓ Successfully encoded test data")
        
        # Test decoding
        decoded = cbor_utils.decode_from_json(encoded)
        print("✓ Successfully decoded test data")
        
        assert decoded == test_data, "Decoded data doesn't match original"
        print("✓ CBOR encoding/decoding test passed")
        
        return True
        
    except ImportError as e:
        print(f"✗ Import error: {e}")
        return False
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def test_message_validation():
    """Test message validation."""
    print("\nTesting message validation...")
    
    try:
        from cbor_utils import MessageValidator
        from message_builder import create_ping_request
        
        validator = MessageValidator()
        
        # Test valid message
        valid_msg = create_ping_request("test_client", "1.0.0")
        is_valid = validator.validate_message(valid_msg)
        
        if is_valid:
            print("✓ Message validation passed")
        else:
            print("✗ Message validation failed")
            return False
            
        return True
        
    except Exception as e:
        print(f"✗ Validation error: {e}")
        return False

def test_error_codes():
    """Test error code enumeration."""
    print("\nTesting error codes...")
    
    try:
        import ai_core_pb2
        
        # Test that error codes are accessible
        error_codes = [
            ai_core_pb2.UNKNOWN_ERROR,
            ai_core_pb2.INVALID_REQUEST,
            ai_core_pb2.UNAUTHORIZED,
            ai_core_pb2.CAPABILITY_DENIED,
            ai_core_pb2.MODEL_NOT_AVAILABLE,
            ai_core_pb2.TOOL_NOT_FOUND,
            ai_core_pb2.TOOL_EXECUTION_FAILED,
            ai_core_pb2.RATE_LIMITED,
            ai_core_pb2.INTERNAL_ERROR,
            ai_core_pb2.TIMEOUT,
            ai_core_pb2.VALIDATION_ERROR,
            ai_core_pb2.NOT_FOUND,
            ai_core_pb2.ALREADY_EXISTS,
            ai_core_pb2.INVALID_ARGUMENT,
            ai_core_pb2.UNSUPPORTED_OPERATION,
            ai_core_pb2.SERVICE_UNAVAILABLE,
            ai_core_pb2.EXTERNAL_SERVICE_ERROR,
            ai_core_pb2.NETWORK_ERROR,
            ai_core_pb2.SERIALIZATION_ERROR,
            ai_core_pb2.DESERIALIZATION_ERROR,
            ai_core_pb2.CONFIGURATION_ERROR,
            ai_core_pb2.RESOURCE_EXHAUSTED,
            ai_core_pb2.CONCURRENT_MODIFICATION,
            ai_core_pb2.DEPENDENCY_FAILURE,
            ai_core_pb2.AUTHENTICATION_FAILED,
            ai_core_pb2.AUTHORIZATION_FAILED,
            ai_core_pb2.TOKEN_EXPIRED,
            ai_core_pb2.TOKEN_INVALID,
            ai_core_pb2.SESSION_EXPIRED,
            ai_core_pb2.SESSION_INVALID,
            ai_core_pb2.QUOTA_EXCEEDED,
            ai_core_pb2.BANDWIDTH_EXCEEDED,
            ai_core_pb2.STORAGE_FULL,
            ai_core_pb2.FILE_NOT_FOUND,
            ai_core_pb2.PERMISSION_DENIED,
            ai_core_pb2.INVALID_FORMAT,
            ai_core_pb2.CORRUPTED_DATA,
            ai_core_pb2.VERSION_MISMATCH,
            ai_core_pb2.FEATURE_NOT_AVAILABLE,
            ai_core_pb2.MAINTENANCE_MODE,
            ai_core_pb2.UPGRADE_REQUIRED
        ]
        
        print(f"✓ Found {len(error_codes)} error codes")
        
        # Test service states
        service_states = [
            ai_core_pb2.STARTING,
            ai_core_pb2.RUNNING,
            ai_core_pb2.STOPPING,
            ai_core_pb2.STOPPED,
            ai_core_pb2.ERROR
        ]
        
        print(f"✓ Found {len(service_states)} service states")
        
        return True
        
    except Exception as e:
        print(f"✗ Error code test failed: {e}")
        return False

def main():
    """Main test function."""
    print("AI Core Service Protobuf Validation Test")
    print("=" * 50)
    
    tests = [
        test_protobuf_structure,
        test_message_validation,
        test_error_codes
    ]
    
    passed = 0
    total = len(tests)
    
    for test in tests:
        if test():
            passed += 1
        else:
            print(f"Test failed: {test.__name__}")
    
    print("\n" + "=" * 50)
    print(f"Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("✓ All tests passed!")
        return 0
    else:
        print("✗ Some tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())
