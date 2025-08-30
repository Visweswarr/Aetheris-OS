#!/bin/bash

# This script demonstrates how to introduce a breaking change
# for testing the CI breaking change detection

set -e

echo "🚨 Introducing Breaking Change for Testing"
echo "=========================================="

# Create a backup of the original file
cp breaking_change_test.proto breaking_change_test.proto.backup

echo "📝 Original file backed up to breaking_change_test.proto.backup"

# Introduce a breaking change by modifying the proto file
cat > breaking_change_test.proto << 'EOF'
syntax = "proto3";

package polymera.test.breaking;

import "google/protobuf/timestamp.proto";

// This service demonstrates breaking changes for testing
service BreakingChangeService {
  // Original method signature
  rpc OriginalMethod(OriginalRequest) returns (OriginalResponse);
  
  // BREAKING CHANGE: Modified method signature
  rpc BreakingMethod(BreakingRequest) returns (BreakingResponse);
  
  // BREAKING CHANGE: Added new method (this is safe)
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

// BREAKING CHANGE: This request message has been modified
message BreakingRequest {
  string message = 1;
  int32 value = 2;
  
  // BREAKING: Changed field type from string to int32
  int32 new_optional_field = 3;  // This was string, now int32
  
  // BREAKING: Changed field number
  int32 new_field_with_default = 5;  // This was field 4, now field 5
}

// BREAKING CHANGE: This response message has been modified
message BreakingResponse {
  string reply = 1;
  bool success = 2;
  
  // BREAKING: Removed field
  // string new_optional_field = 3;  // This field was removed
  
  // SAFE: Adding new optional field is safe
  int32 new_safe_field = 4;
}

// New message types (these are safe additions)
message NewRequest {
  string message = 1;
  int32 value = 2;
}

message NewResponse {
  string reply = 1;
  bool success = 2;
}

# Example of breaking changes introduced:
# 1. ✅ Changed field type from string to int32 in BreakingRequest.new_optional_field
# 2. ✅ Changed field number from 4 to 5 in BreakingRequest.new_field_with_default  
# 3. ✅ Removed field new_optional_field from BreakingResponse
# 4. ✅ Added new service method (this is actually safe)
# 5. ✅ Added new message types (this is safe)
EOF

echo "🚨 Breaking changes introduced:"
echo "   - Changed field type from string to int32"
echo "   - Changed field number from 4 to 5"
echo "   - Removed field new_optional_field"
echo "   - Added new service method and message types"

echo ""
echo "🔍 Now run: buf breaking --against .git#subdir=proto,ref=HEAD~1"
echo "   This should detect the breaking changes and fail the CI"
echo ""
echo "🔄 To restore the original file:"
echo "   cp breaking_change_test.proto.backup breaking_change_test.proto"
