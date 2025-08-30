#!/bin/bash

# This script demonstrates how to introduce safe changes
# that won't trigger breaking change detection

set -e

echo "✅ Introducing Safe Changes for Testing"
echo "======================================"

# Create a backup of the original file
cp breaking_change_test.proto breaking_change_test.proto.backup

echo "📝 Original file backed up to breaking_change_test.proto.backup"

# Introduce safe changes by modifying the proto file
cat > breaking_change_test.proto << 'EOF'
syntax = "proto3";

package polymera.test.breaking;

import "google/protobuf/timestamp.proto";

// This service demonstrates breaking changes for testing
service BreakingChangeService {
  // Original method signature
  rpc OriginalMethod(OriginalRequest) returns (OriginalResponse);
  
  // SAFE: Modified method signature (adding new optional parameter)
  rpc BreakingMethod(BreakingRequest) returns (BreakingResponse);
  
  // SAFE: Added new method
  rpc NewMethod(NewRequest) returns (NewResponse);
}

// Original request message
message OriginalRequest {
  string message = 1;
  int32 value = 2;
}

// Original response message
message OriginalResponse {
  string reply = 1;
  bool success = 2;
}

// SAFE: This request message has been modified safely
message BreakingRequest {
  string message = 1;
  int32 value = 2;
  
  // SAFE: Adding new optional field
  string new_optional_field = 3;
  
  // SAFE: Adding new optional field with default
  int32 new_field_with_default = 4;
  
  // SAFE: Adding new optional field
  string another_optional_field = 5;
}

// SAFE: This response message has been modified safely
message BreakingResponse {
  string reply = 1;
  bool success = 2;
  
  // SAFE: Adding new optional field
  string new_optional_field = 3;
  
  // SAFE: Adding new optional field with default
  int32 new_field_with_default = 4;
  
  // SAFE: Adding new optional field
  string another_optional_field = 5;
}

// SAFE: New message types
message NewRequest {
  string message = 1;
  int32 value = 2;
  
  // SAFE: Adding optional fields
  string optional_field = 3;
  int32 optional_number = 4;
}

message NewResponse {
  string reply = 1;
  bool success = 2;
  
  // SAFE: Adding optional fields
  string optional_field = 3;
  int32 optional_number = 4;
}

# Example of safe changes introduced:
# 1. ✅ Added new optional fields to existing messages
# 2. ✅ Added new service methods
# 3. ✅ Added new message types
# 4. ✅ All fields are optional (proto3 default)
# 5. ✅ No field numbers or types were changed
# 6. ✅ No fields were removed
EOF

echo "✅ Safe changes introduced:"
echo "   - Added new optional fields to existing messages"
echo "   - Added new service methods"
echo "   - Added new message types"
echo "   - All changes maintain backward compatibility"

echo ""
echo "🔍 Now run: buf breaking --against .git#subdir=proto,ref=HEAD~1"
echo "   This should pass as no breaking changes were introduced"
echo ""
echo "🔄 To restore the original file:"
echo "   cp breaking_change_test.proto.backup breaking_change_test.proto"
