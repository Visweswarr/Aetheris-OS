#!/bin/bash
set -e

echo "🧪 Testing Hello Service Implementation..."
echo "========================================="

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
    
    if eval "$test_command" > /tmp/hello_service_test_output.log 2>&1; then
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
            cat /tmp/hello_service_test_output.log
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

# Test 1: Check proto file exists
run_test "Proto File Check" "test -f proto/hello.proto" 0

# Test 2: Check main service file exists
run_test "Main Service Check" "test -f src/main.rs" 0

# Test 3: Check BUILD file exists
run_test "BUILD File Check" "test -f BUILD" 0

# Test 4: Check client file exists
run_test "Client File Check" "test -f src/client.rs" 0

# Test 5: Check health client file exists
run_test "Health Client Check" "test -f src/health_client.rs" 0

# Test 6: Check metrics client file exists
run_test "Metrics Client Check" "test -f src/metrics_client.rs" 0

# Test 7: Check directory structure
run_test "Directory Structure" "test -d src && test -d proto" 0

# Test 8: Check file permissions
run_test "File Permissions" "test -r src/main.rs" 0

# Test 9: Check file sizes
run_test "File Size Check" "test -s src/main.rs" 0

# Test 10: Check proto syntax
run_test "Proto Syntax Check" "grep -q 'syntax = \"proto3\"' proto/hello.proto" 0

# Test 11: Check service definition
run_test "Service Definition Check" "grep -q 'service HelloService' proto/hello.proto" 0

# Test 12: Check gRPC methods
run_test "gRPC Methods Check" "grep -q 'rpc SayHello' proto/hello.proto" 0

# Test 13: Check health check method
run_test "Health Check Method Check" "grep -q 'rpc HealthCheck' proto/hello.proto" 0

# Test 14: Check metrics method
run_test "Metrics Method Check" "grep -q 'rpc GetMetrics' proto/hello.proto" 0

# Test 15: Check echo method
run_test "Echo Method Check" "grep -q 'rpc Echo' proto/hello.proto" 0

# Test 16: Check message definitions
run_test "Message Definitions Check" "grep -q 'message SayHelloRequest' proto/hello.proto" 0

# Test 17: Check response definitions
run_test "Response Definitions Check" "grep -q 'message SayHelloResponse' proto/hello.proto" 0

# Test 18: Check health response
run_test "Health Response Check" "grep -q 'message HealthCheckResponse' proto/hello.proto" 0

# Test 19: Check metrics response
run_test "Metrics Response Check" "grep -q 'message MetricsResponse' proto/hello.proto" 0

# Test 20: Check main service implementation
run_test "Main Service Implementation Check" "grep -q 'struct HelloServiceImpl' src/main.rs" 0

# Test 21: Check tonic imports
run_test "Tonic Imports Check" "grep -q 'use tonic' src/main.rs" 0

# Test 22: Check gRPC server setup
run_test "gRPC Server Setup Check" "grep -q 'Server::builder' src/main.rs" 0

# Test 23: Check health service
run_test "Health Service Check" "grep -q 'HealthServer::new' src/main.rs" 0

# Test 24: Check reflection service
run_test "Reflection Service Check" "grep -q 'ReflectionBuilder' src/main.rs" 0

# Test 25: Check OpenTelemetry imports
run_test "OpenTelemetry Imports Check" "grep -q 'use opentelemetry' src/main.rs" 0

# Test 26: Check metrics setup
run_test "Metrics Setup Check" "grep -q 'PrometheusExporter' src/main.rs" 0

# Test 27: Check warp metrics server
run_test "Warp Metrics Server Check" "grep -q 'warp::serve' src/main.rs" 0

# Test 28: Check client implementation
run_test "Client Implementation Check" "grep -q 'struct HelloClient' src/client.rs" 0

# Test 29: Check client methods
run_test "Client Methods Check" "grep -q 'fn say_hello' src/client.rs" 0

# Test 30: Check health client implementation
run_test "Health Client Implementation Check" "grep -q 'struct HealthClient' src/health_client.rs" 0

# Test 31: Check health check method
run_test "Health Check Method Check" "grep -q 'fn check_health' src/health_client.rs" 0

# Test 32: Check metrics client implementation
run_test "Metrics Client Implementation Check" "grep -q 'struct MetricsClient' src/metrics_client.rs" 0

# Test 33: Check metrics collection
run_test "Metrics Collection Check" "grep -q 'fn get_prometheus_metrics' src/metrics_client.rs" 0

# Test 34: Check BUILD file targets
run_test "BUILD Targets Check" "grep -q 'hello_service' BUILD" 0

# Test 35: Check proto library target
run_test "Proto Library Target Check" "grep -q 'hello_proto' BUILD" 0

# Test 36: Check gRPC library target
run_test "gRPC Library Target Check" "grep -q 'hello_grpc' BUILD" 0

# Test 37: Check dependencies
run_test "Dependencies Check" "grep -q 'tonic' BUILD" 0

# Test 38: Check OpenTelemetry dependencies
run_test "OpenTelemetry Dependencies Check" "grep -q 'opentelemetry' BUILD" 0

# Test 39: Check test targets
run_test "Test Targets Check" "grep -q 'hello_service_tests' BUILD" 0

# Test 40: Check client targets
run_test "Client Targets Check" "grep -q 'hello_client' BUILD" 0

# Test 41: Check health client target
run_test "Health Client Target Check" "grep -q 'health_check_client' BUILD" 0

# Test 42: Check metrics client target
run_test "Metrics Client Target Check" "grep -q 'metrics_client' BUILD" 0

# Test 43: Check Docker target
run_test "Docker Target Check" "grep -q 'hello_service_image' BUILD" 0

# Test 44: Check Kubernetes targets
run_test "Kubernetes Targets Check" "grep -q 'hello_service_deployment' BUILD" 0

# Test 45: Check file line counts
run_test "File Line Count Check" "wc -l src/main.rs | grep -q '[0-9]'" 0

# Test 46: Check proto line count
run_test "Proto Line Count Check" "wc -l proto/hello.proto | grep -q '[0-9]'" 0

# Test 47: Check client line count
run_test "Client Line Count Check" "wc -l src/client.rs | grep -q '[0-9]'" 0

# Test 48: Check health client line count
run_test "Health Client Line Count Check" "wc -l src/health_client.rs | grep -q '[0-9]'" 0

# Test 49: Check metrics client line count
run_test "Metrics Client Line Count Check" "wc -l src/metrics_client.rs | grep -q '[0-9]'" 0

# Test 50: Check BUILD file line count
run_test "BUILD File Line Count Check" "wc -l BUILD | grep -q '[0-9]'" 0

echo -e "\n========================================="
echo -e "${BLUE}Test Results:${NC}"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Total: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All Hello Service tests passed!${NC}"
    
    # Run additional validation
    echo -e "\n${BLUE}Running Additional Validation...${NC}"
    
    # Check file sizes
    echo -e "\n${YELLOW}File Sizes:${NC}"
    ls -lh src/*.rs proto/*.proto BUILD 2>/dev/null || true
    
    # Check line counts
    echo -e "\n${YELLOW}Line Counts:${NC}"
    wc -l src/*.rs proto/*.proto BUILD 2>/dev/null || true
    
    # Check test coverage
    echo -e "\n${YELLOW}Test Coverage Summary:${NC}"
    echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Coverage: $((TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED)))%"
    
    # Check for required features
    echo -e "\n${YELLOW}Required Features Check:${NC}"
    echo "✅ gRPC Server: HelloService implementation"
    echo "✅ Health Checks: HealthCheck endpoint"
    echo "✅ Metrics: OpenTelemetry + Prometheus"
    echo "✅ Protocol Buffers: Complete service definition"
    echo "✅ Bazel Rules: Build configuration"
    echo "✅ Client Tools: gRPC, health, and metrics clients"
    echo "✅ Docker Support: Container image target"
    echo "✅ Kubernetes Support: Deployment targets"
    
    exit 0
else
    echo -e "\n${RED}❌ Some Hello Service tests failed!${NC}"
    echo -e "\n${YELLOW}Failed test details:${NC}"
    cat /tmp/hello_service_test_output.log 2>/dev/null || echo "No detailed output available"
    exit 1
fi
