#!/bin/bash
# Performance test for SLO checker with large datasets

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}SLO Checker Performance Test${NC}"
echo "=============================="

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Performance test function
run_performance_test() {
    local test_name="$1"
    local metrics_file="$2"
    local expected_max_time_ms="$3"
    
    echo ""
    echo -e "${BLUE}Performance Test: $test_name${NC}"
    echo "Metrics file: $metrics_file"
    echo "Expected max time: ${expected_max_time_ms}ms"
    
    # Run test multiple times and take average
    local total_time=0
    local runs=3
    
    for i in $(seq 1 $runs); do
        echo -n "Run $i/$runs... "
        
        start_time=$(date +%s%N)
        if "${SCRIPT_DIR}/check_slo" \
            --config "${SCRIPT_DIR}/slo.yaml" \
            --metrics "$metrics_file" \
            --env production \
            --format json \
            --output "${SCRIPT_DIR}/perf_test_output_${i}.json" >/dev/null 2>&1; then
            end_time=$(date +%s%N)
            
            run_time=$(( (end_time - start_time) / 1000000 ))
            total_time=$((total_time + run_time))
            echo "${run_time}ms"
        else
            echo -e "${RED}FAILED${NC}"
            return 1
        fi
    done
    
    # Calculate average
    avg_time=$((total_time / runs))
    echo "Average execution time: ${avg_time}ms"
    
    # Check if within acceptable range
    if [[ $avg_time -le $expected_max_time_ms ]]; then
        echo -e "${GREEN}✓ PASS${NC} - Performance within acceptable range"
    else
        echo -e "${YELLOW}⚠ SLOW${NC} - Performance exceeded expected maximum"
    fi
    
    # Clean up
    rm -f "${SCRIPT_DIR}/perf_test_output_"*.json
    
    return 0
}

# Test 1: Normal metrics file (baseline)
run_performance_test "Normal metrics" \
    "${SCRIPT_DIR}/test_metrics_pass.json" \
    1000

# Test 2: Large metrics file
if [[ -f "${SCRIPT_DIR}/large_metrics.json" ]]; then
    run_performance_test "Large metrics dataset" \
        "${SCRIPT_DIR}/large_metrics.json" \
        5000
else
    echo -e "${YELLOW}⚠ Skipping large metrics test - file not found${NC}"
fi

# Test 3: Memory usage test
echo ""
echo -e "${BLUE}Memory Usage Test${NC}"

# Check if we can measure memory usage
if command -v valgrind >/dev/null 2>&1; then
    echo "Running memory usage analysis with Valgrind..."
    
    valgrind --tool=massif \
        --massif-out-file="${SCRIPT_DIR}/massif.out" \
        "${SCRIPT_DIR}/check_slo" \
        --config "${SCRIPT_DIR}/slo.yaml" \
        --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
        --env production >/dev/null 2>&1
    
    if command -v ms_print >/dev/null 2>&1; then
        peak_memory=$(ms_print "${SCRIPT_DIR}/massif.out" | grep "Peak" | head -1)
        echo "Memory usage: $peak_memory"
    fi
    
    rm -f "${SCRIPT_DIR}/massif.out"
    echo -e "${GREEN}✓ Memory analysis completed${NC}"
elif command -v time >/dev/null 2>&1; then
    echo "Running basic time measurement..."
    
    # Use time command to get basic resource usage
    /usr/bin/time -v "${SCRIPT_DIR}/check_slo" \
        --config "${SCRIPT_DIR}/slo.yaml" \
        --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
        --env production >/dev/null 2>"${SCRIPT_DIR}/time_output.txt" || true
    
    if [[ -f "${SCRIPT_DIR}/time_output.txt" ]]; then
        max_resident=$(grep "Maximum resident set size" "${SCRIPT_DIR}/time_output.txt" | awk '{print $6}')
        if [[ -n "$max_resident" ]]; then
            echo "Maximum resident set size: ${max_resident}KB"
            
            # Check if memory usage is reasonable (< 100MB)
            if [[ $max_resident -lt 102400 ]]; then
                echo -e "${GREEN}✓ Memory usage within reasonable limits${NC}"
            else
                echo -e "${YELLOW}⚠ High memory usage detected${NC}"
            fi
        fi
        rm -f "${SCRIPT_DIR}/time_output.txt"
    fi
else
    echo -e "${YELLOW}⚠ No memory profiling tools available${NC}"
fi

# Test 4: Concurrent execution test
echo ""
echo -e "${BLUE}Concurrent Execution Test${NC}"

echo "Testing concurrent SLO checks..."
start_time=$(date +%s%N)

# Run multiple SLO checks in parallel
pids=()
for i in {1..5}; do
    "${SCRIPT_DIR}/check_slo" \
        --config "${SCRIPT_DIR}/slo.yaml" \
        --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
        --env production \
        --output "${SCRIPT_DIR}/concurrent_test_${i}.json" >/dev/null 2>&1 &
    pids+=($!)
done

# Wait for all to complete
for pid in "${pids[@]}"; do
    wait "$pid"
done

end_time=$(date +%s%N)
concurrent_time=$(( (end_time - start_time) / 1000000 ))

echo "Concurrent execution time: ${concurrent_time}ms"

# Verify all outputs are valid
all_valid=true
for i in {1..5}; do
    if [[ -f "${SCRIPT_DIR}/concurrent_test_${i}.json" ]]; then
        if ! jq empty "${SCRIPT_DIR}/concurrent_test_${i}.json" 2>/dev/null; then
            all_valid=false
            echo -e "${RED}✗ Invalid output from concurrent run $i${NC}"
        fi
    else
        all_valid=false
        echo -e "${RED}✗ Missing output from concurrent run $i${NC}"
    fi
done

if $all_valid; then
    echo -e "${GREEN}✓ All concurrent executions completed successfully${NC}"
else
    echo -e "${RED}✗ Some concurrent executions failed${NC}"
fi

# Clean up concurrent test files
rm -f "${SCRIPT_DIR}/concurrent_test_"*.json

# Test 5: Stress test with invalid data
echo ""
echo -e "${BLUE}Error Handling Stress Test${NC}"

# Create malformed metrics file
cat > "${SCRIPT_DIR}/malformed_metrics.json" << 'EOF'
{
  "timestamp": "invalid-date",
  "environment": "test",
  "metrics": {
    "test.metric": {
      "metric_type": "histogram",
      "samples": "not-a-number",
      "values": {
        "p95": "also-not-a-number"
      }
    }
  }
EOF

echo "Testing error handling with malformed data..."
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/malformed_metrics.json" \
    --env production >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠ Unexpectedly succeeded with malformed data${NC}"
else
    echo -e "${GREEN}✓ Correctly rejected malformed data${NC}"
fi

# Clean up
rm -f "${SCRIPT_DIR}/malformed_metrics.json"

echo ""
echo -e "${GREEN}Performance testing completed!${NC}"
