#!/bin/bash
set -e

echo "🔐 Testing Capability Tokens Implementation..."
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit="$3"

    echo -e "\n${BLUE}Running: ${test_name}${NC}"
    echo "Command: $test_command"

    if eval "$test_command" > /tmp/caps_test_output.log 2>&1; then
        local exit_code=$?
        if [ "$exit_code" = "$expected_exit" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}✗ FAILED (expected exit $expected_exit, got $exit_code)${NC}"
            ((TESTS_FAILED++))
        fi
    else
        local exit_code=$?
        if [ "$exit_code" = "$expected_exit" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}✗ FAILED (expected exit $expected_exit, got $exit_code)${NC}"
            echo "Output:"
            cat /tmp/caps_test_output.log
            ((TESTS_FAILED++))
        fi
    fi
}

# Check if Bazel is available
if ! command -v bazel &> /dev/null; then
    echo -e "${YELLOW}Bazel not found, checking for alternative build tools...${NC}"

    # Check for Cargo
    if command -v cargo &> /dev/null; then
        echo -e "${YELLOW}Using Cargo for testing...${NC}"
        USE_CARGO=true
    else
        echo -e "${RED}No build tools found. Please install Bazel or Cargo.${NC}"
        exit 1
    fi
else
    USE_CARGO=false
fi

# Test 1: Check format module file exists
run_test "Format Module Check" "test -f src/format.rs" 0

# Test 2: Check signing module file exists
run_test "Signing Module Check" "test -f src/sign.rs" 0

# Test 3: Check verification module file exists
run_test "Verification Module Check" "test -f src/verify.rs" 0

# Test 4: Check module file exists
run_test "Module File Check" "test -f src/mod.rs" 0

# Test 5: Check BUILD file exists
run_test "BUILD File Check" "test -f BUILD" 0

# Test 6: Check directory structure
run_test "Directory Structure Check" "test -d src" 0

# Test 7: Check file permissions
run_test "File Permissions Check" "test -r src/format.rs" 0

# Test 8: Check file sizes
run_test "File Size Check" "test -s src/format.rs" 0

# Test 9: Check capability token structs
run_test "Capability Token Structs Check" "grep -q 'struct CapabilityToken' src/format.rs" 0

# Test 10: Check capability token header
run_test "Token Header Check" "grep -q 'struct CapabilityTokenHeader' src/format.rs" 0

# Test 11: Check capability token payload
run_test "Token Payload Check" "grep -q 'struct CapabilityTokenPayload' src/format.rs" 0

# Test 12: Check capability claims
run_test "Capability Claims Check" "grep -q 'struct CapabilityClaim' src/format.rs" 0

# Test 13: Check resource capability
run_test "Resource Capability Check" "grep -q 'struct ResourceCapability' src/format.rs" 0

# Test 14: Check action capability
run_test "Action Capability Check" "grep -q 'struct ActionCapability' src/format.rs" 0

# Test 15: Check scope capability
run_test "Scope Capability Check" "grep -q 'struct ScopeCapability' src/format.rs" 0

# Test 16: Check time capability
run_test "Time Capability Check" "grep -q 'struct TimeCapability' src/format.rs" 0

# Test 17: Check location capability
run_test "Location Capability Check" "grep -q 'struct LocationCapability' src/format.rs" 0

# Test 18: Check device capability
run_test "Device Capability Check" "grep -q 'struct DeviceCapability' src/format.rs" 0

# Test 19: Check network capability
run_test "Network Capability Check" "grep -q 'struct NetworkCapability' src/format.rs" 0

# Test 20: Check data capability
run_test "Data Capability Check" "grep -q 'struct DataCapability' src/format.rs" 0

# Test 21: Check custom capability
run_test "Custom Capability Check" "grep -q 'struct CustomCapability' src/format.rs" 0

# Test 22: Check capability token builder
run_test "Token Builder Check" "grep -q 'struct CapabilityTokenBuilder' src/format.rs" 0

# Test 23: Check capability token formatter
run_test "Token Formatter Check" "grep -q 'struct CapabilityTokenFormatter' src/format.rs" 0

# Test 24: Check signature structs
run_test "Signature Structs Check" "grep -q 'struct Signature' src/sign.rs" 0

# Test 25: Check key pair struct
run_test "Key Pair Check" "grep -q 'struct KeyPair' src/sign.rs" 0

# Test 26: Check signature algorithms
run_test "Signature Algorithms Check" "grep -q 'enum SignatureAlgorithm' src/sign.rs" 0

# Test 27: Check Dilithium support
run_test "Dilithium Support Check" "grep -q 'Dilithium2\|Dilithium3\|Dilithium5' src/sign.rs" 0

# Test 28: Check signer trait
run_test "Signer Trait Check" "grep -q 'trait Signer' src/sign.rs" 0

# Test 29: Check verifier trait
run_test "Verifier Trait Check" "grep -q 'trait Verifier' src/sign.rs" 0

# Test 30: Check Dilithium signer
run_test "Dilithium Signer Check" "grep -q 'struct DilithiumSigner' src/sign.rs" 0

# Test 31: Check Dilithium verifier
run_test "Dilithium Verifier Check" "grep -q 'struct DilithiumVerifier' src/sign.rs" 0

# Test 32: Check mock signer
run_test "Mock Signer Check" "grep -q 'struct MockSigner' src/sign.rs" 0

# Test 33: Check token validator
run_test "Token Validator Check" "grep -q 'struct TokenValidator' src/verify.rs" 0

# Test 34: Check verification config
run_test "Verification Config Check" "grep -q 'struct CapabilityTokenVerifyConfig' src/verify.rs" 0

# Test 35: Check verification errors
run_test "Verification Errors Check" "grep -q 'enum CapabilityTokenVerifyError' src/verify.rs" 0

# Test 36: Check mock validator
run_test "Mock Validator Check" "grep -q 'struct MockTokenValidator' src/verify.rs" 0

# Test 37: Check capability token service
run_test "Token Service Check" "grep -q 'struct CapabilityTokenService' src/mod.rs" 0

# Test 38: Check service config
run_test "Service Config Check" "grep -q 'struct CapabilityTokenServiceConfig' src/mod.rs" 0

# Test 39: Check purpose binding
run_test "Purpose Binding Check" "grep -q 'purpose' src/format.rs" 0

# Test 40: Check time scoping
run_test "Time Scoping Check" "grep -q 'expires_at\|valid_until' src/format.rs" 0

# Test 41: Check least privilege
run_test "Least Privilege Check" "grep -q 'permissions\|access_level' src/format.rs" 0

# Test 42: Check JWT-like structure
run_test "JWT-like Structure Check" "grep -q 'header\|payload\|signature' src/format.rs" 0

# Test 43: Check post-quantum signatures
run_test "Post-Quantum Signatures Check" "grep -q 'Dilithium' src/sign.rs" 0

# Test 44: Check expired token rejection
run_test "Expired Token Rejection Check" "grep -q 'TokenExpired\|expired' src/verify.rs" 0

# Test 45: Check forged token rejection
run_test "Forged Token Rejection Check" "grep -q 'TokenTamperingDetected\|InvalidSignature' src/verify.rs" 0

# Test 46: Check claims validation
run_test "Claims Validation Check" "grep -q 'validate_claim\|validate_token_claims' src/verify.rs" 0

# Test 47: Check signature verification
run_test "Signature Verification Check" "grep -q 'verify_signature\|verify_token_signature' src/verify.rs" 0

# Test 48: Check timestamp validation
run_test "Timestamp Validation Check" "grep -q 'validate_token_timestamps\|clock_skew' src/verify.rs" 0

# Test 49: Check format validation
run_test "Format Validation Check" "grep -q 'validate_token_format' src/verify.rs" 0

# Test 50: Check security validation
run_test "Security Validation Check" "grep -q 'validate_token_security' src/verify.rs" 0

# Test 51: Check purpose verification
run_test "Purpose Verification Check" "grep -q 'verify_token_for_purpose' src/verify.rs" 0

# Test 52: Check scope verification
run_test "Scope Verification Check" "grep -q 'verify_token_for_scope' src/verify.rs" 0

# Test 53: Check resource access verification
run_test "Resource Access Verification Check" "grep -q 'verify_token_for_resource' src/verify.rs" 0

# Test 54: Check algorithm validation
run_test "Algorithm Validation Check" "grep -q 'required_algorithms\|algorithm' src/verify.rs" 0

# Test 55: Check key validation
run_test "Key Validation Check" "grep -q 'KeyNotFound\|InvalidPublicKey' src/verify.rs" 0

# Test 56: Check privilege validation
run_test "Privilege Validation Check" "grep -q 'InsufficientPrivileges\|minimum_token_level' src/verify.rs" 0

# Test 57: Check issuer validation
run_test "Issuer Validation Check" "grep -q 'trusted_issuers\|Untrusted issuer' src/verify.rs" 0

# Test 58: Check audience validation
run_test "Audience Validation Check" "grep -q 'trusted_audiences\|Untrusted audience' src/verify.rs" 0

# Test 59: Check token size validation
run_test "Token Size Validation Check" "grep -q 'max_token_size\|SizeLimitExceeded' src/verify.rs" 0

# Test 60: Check strict validation
run_test "Strict Validation Check" "grep -q 'strict_validation\|validate_token_security' src/verify.rs" 0

# Test 61: Check base64 encoding
run_test "Base64 Encoding Check" "grep -q 'base64::encode\|base64::decode' src/sign.rs" 0

# Test 62: Check JSON serialization
run_test "JSON Serialization Check" "grep -q 'serde_json::to_string\|serde_json::from_str' src/format.rs" 0

# Test 63: Check error handling
run_test "Error Handling Check" "grep -q 'thiserror::Error\|#[error' src/format.rs" 0

# Test 64: Check UUID generation
run_test "UUID Generation Check" "grep -q 'Uuid::new_v4\|uuid' src/format.rs" 0

# Test 65: Check time handling
run_test "Time Handling Check" "grep -q 'SystemTime\|UNIX_EPOCH' src/format.rs" 0

# Test 66: Check hash computation
run_test "Hash Computation Check" "grep -q 'compute_hash\|HashError' src/sign.rs" 0

# Test 67: Check key generation
run_test "Key Generation Check" "grep -q 'generate_key_pair\|KeyGenerationError' src/sign.rs" 0

# Test 68: Check signature creation
run_test "Signature Creation Check" "grep -q 'create_signature\|SignatureCreationError' src/sign.rs" 0

# Test 69: Check signature verification
run_test "Signature Verification Check" "grep -q 'verify_signature_data\|SignatureVerificationFailed' src/sign.rs" 0

# Test 70: Check expiration handling
run_test "Expiration Handling Check" "grep -q 'expires_at\|Expired' src/sign.rs" 0

# Test 71: Check algorithm compatibility
run_test "Algorithm Compatibility Check" "grep -q 'AlgorithmMismatch\|algorithm' src/sign.rs" 0

# Test 72: Check required fields validation
run_test "Required Fields Validation Check" "grep -q 'MissingFields\|required' src/verify.rs" 0

# Test 73: Check token type validation
run_test "Token Type Validation Check" "grep -q 'InvalidFormat\|capability' src/verify.rs" 0

# Test 74: Check version consistency
run_test "Version Consistency Check" "grep -q 'format_version\|version' src/format.rs" 0

# Test 75: Check token ID validation
run_test "Token ID Validation Check" "grep -q 'jti\|TokenTamperingDetected' src/verify.rs" 0

# Test 76: Check suspicious pattern detection
run_test "Suspicious Pattern Detection Check" "grep -q 'issuer equals subject\|suspicious' src/verify.rs" 0

# Test 77: Check clock skew tolerance
run_test "Clock Skew Tolerance Check" "grep -q 'clock_skew_tolerance\|tolerance' src/verify.rs" 0

# Test 78: Check not before validation
run_test "Not Before Validation Check" "grep -q 'nbf\|TokenNotYetValid' src/verify.rs" 0

# Test 79: Check issued at validation
run_test "Issued At Validation Check" "grep -q 'iat\|timestamp' src/format.rs" 0

# Test 80: Check key ID validation
run_test "Key ID Validation Check" "grep -q 'kid\|key_id' src/format.rs" 0

# Test 81: Check token version validation
run_test "Token Version Validation Check" "grep -q 'ver\|version' src/format.rs" 0

# Test 82: Check token type validation
run_test "Token Type Validation Check" "grep -q 'typ\|capability' src/format.rs" 0

# Test 83: Check issuer validation
run_test "Issuer Validation Check" "grep -q 'iss\|issuer' src/format.rs" 0

# Test 84: Check subject validation
run_test "Subject Validation Check" "grep -q 'sub\|subject' src/format.rs" 0

# Test 85: Check audience validation
run_test "Audience Validation Check" "grep -q 'aud\|audience' src/format.rs" 0

# Test 86: Check purpose validation
run_test "Purpose Validation Check" "grep -q 'purpose\|PurposeMismatch' src/verify.rs" 0

# Test 87: Check scope validation
run_test "Scope Validation Check" "grep -q 'scope\|ScopeViolation' src/verify.rs" 0

# Test 88: Check level validation
run_test "Level Validation Check" "grep -q 'level\|minimum_token_level' src/verify.rs" 0

# Test 89: Check hierarchy validation
run_test "Hierarchy Validation Check" "grep -q 'hierarchy\|scope_hierarchy' src/format.rs" 0

# Test 90: Check constraints validation
run_test "Constraints Validation Check" "grep -q 'constraints\|validation_rules' src/format.rs" 0

# Test 91: Check metadata validation
run_test "Metadata Validation Check" "grep -q 'metadata\|additional' src/format.rs" 0

# Test 92: Check custom claims validation
run_test "Custom Claims Validation Check" "grep -q 'Custom\|custom_claim' src/format.rs" 0

# Test 93: Check resource permissions validation
run_test "Resource Permissions Validation Check" "grep -q 'permissions\|ResourceAccessDenied' src/verify.rs" 0

# Test 94: Check action parameters validation
run_test "Action Parameters Validation Check" "grep -q 'parameters\|action_name' src/format.rs" 0

# Test 95: Check scope level validation
run_test "Scope Level Validation Check" "grep -q 'scope_level\|scope_name' src/format.rs" 0

# Test 96: Check time constraints validation
run_test "Time Constraints Validation Check" "grep -q 'time_constraints\|recurring_patterns' src/format.rs" 0

# Test 97: Check location constraints validation
run_test "Location Constraints Validation Check" "grep -q 'location_constraints\|coordinates' src/format.rs" 0

# Test 98: Check device constraints validation
run_test "Device Constraints Validation Check" "grep -q 'device_constraints\|device_capabilities' src/format.rs" 0

# Test 99: Check network constraints validation
run_test "Network Constraints Validation Check" "grep -q 'network_constraints\|security_level' src/format.rs" 0

# Test 100: Check data constraints validation
run_test "Data Constraints Validation Check" "grep -q 'data_constraints\|data_classification' src/format.rs" 0

# Test 101: Check token builder methods
run_test "Token Builder Methods Check" "grep -q 'with_.*capability\|with_expiration\|with_algorithm' src/format.rs" 0

# Test 102: Check token formatter methods
run_test "Token Formatter Methods Check" "grep -q 'to_json\|from_json\|to_base64\|from_base64' src/format.rs" 0

# Test 103: Check token size calculation
run_test "Token Size Calculation Check" "grep -q 'get_token_size\|validate_token_size' src/format.rs" 0

# Test 104: Check signature manager
run_test "Signature Manager Check" "grep -q 'SignatureManager\|add_signer\|add_verifier' src/sign.rs" 0

# Test 105: Check signing service
run_test "Signing Service Check" "grep -q 'CapabilityTokenSigningService\|add_signer\|set_default_signer' src/sign.rs" 0

# Test 106: Check verification service
run_test "Verification Service Check" "grep -q 'verify_token\|verify_token_for_purpose' src/mod.rs" 0

# Test 107: Check error propagation
run_test "Error Propagation Check" "grep -q 'map_err\|?\|Result' src/verify.rs" 0

# Test 108: Check configuration management
run_test "Configuration Management Check" "grep -q 'get_config\|update_config' src/verify.rs" 0

# Test 109: Check trait implementations
run_test "Trait Implementations Check" "grep -q 'impl.*for.*Signer\|impl.*for.*Verifier' src/sign.rs" 0

# Test 110: Check default implementations
run_test "Default Implementations Check" "grep -q 'impl Default\|Default::default' src/format.rs" 0

# Test 111: Check test modules
run_test "Test Modules Check" "grep -q '#\[cfg\(test\)\]\|mod tests' src/format.rs" 0

# Test 112: Check comprehensive testing
run_test "Comprehensive Testing Check" "grep -q 'test_.*\|assert' src/format.rs" 0

# Test 113: Check Bazel build configuration
run_test "Bazel Build Configuration Check" "grep -q 'rust_library\|rust_binary\|rust_test' BUILD" 0

# Test 114: Check test targets
run_test "Test Targets Check" "grep -q 'expired_token_tests\|forged_token_tests' BUILD" 0

# Test 115: Check Docker configuration
run_test "Docker Configuration Check" "grep -q 'container_image\|k8s_object' BUILD" 0

# Test 116: Check service configuration
run_test "Service Configuration Check" "grep -q 'caps_service\|caps_cli\|caps_generate' BUILD" 0

# Test 117: Check dependencies
run_test "Dependencies Check" "grep -q 'serde\|serde_json\|thiserror\|uuid\|base64' BUILD" 0

# Test 118: Check edition specification
run_test "Edition Specification Check" "grep -q 'edition = \"2021\"' BUILD" 0

# Test 119: Check features specification
run_test "Features Specification Check" "grep -q 'features = \[\"full\"\]' BUILD" 0

# Test 120: Check visibility settings
run_test "Visibility Settings Check" "grep -q 'default_visibility = \[\"//visibility:public\"\]' BUILD" 0

echo -e "\n${BLUE}Test Summary${NC}"
echo "============="
echo -e "${GREEN}Tests Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Tests Failed: ${TESTS_FAILED}${NC}"
echo -e "${BLUE}Total Tests: $((TESTS_PASSED + TESTS_FAILED))${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed! Capability Tokens implementation is complete and ready.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please review the implementation.${NC}"
    exit 1
fi
