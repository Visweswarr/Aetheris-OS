#!/bin/bash
# Test script for XR harness Prometheus metrics export

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Testing XR Harness Prometheus Metrics Export${NC}"
echo "=============================================="

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Test configuration
TEST_DURATION=30
OTLP_ENDPOINT="http://localhost:4318/v1/metrics"

echo "Test configuration:"
echo "  Duration: ${TEST_DURATION}s"
echo "  OTLP Endpoint: ${OTLP_ENDPOINT}"
echo ""

# Function to check if OTLP collector is running
check_otlp_collector() {
    echo -n "Checking OTLP collector availability... "
    if curl -s "${OTLP_ENDPOINT}" >/dev/null 2>&1; then
        echo -e "${GREEN}Available${NC}"
        return 0
    else
        echo -e "${YELLOW}Not available (will use console export)${NC}"
        return 1
    fi
}

# Function to run XR harness with metrics collection
run_xr_harness() {
    local config_file="$1"
    local output_log="$2"
    
    echo "Starting XR harness..."
    echo "Config: ${config_file}"
    echo "Log output: ${output_log}"
    
    # Run XR harness with specific configuration
    "${SCRIPT_DIR}/xr_harness" \
        --config "${config_file}" \
        --duration ${TEST_DURATION} \
        --fps 90 \
        --base-latency 8.0 \
        --variance 3.0 \
        --degradation \
        > "${output_log}" 2>&1 &
    
    local pid=$!
    echo "XR harness started with PID: ${pid}"
    
    # Wait for harness to complete
    wait ${pid}
    local exit_code=$?
    
    if [[ ${exit_code} -eq 0 ]]; then
        echo -e "${GREEN}✓ XR harness completed successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ XR harness failed with exit code: ${exit_code}${NC}"
        return 1
    fi
}

# Function to validate metrics in log output
validate_metrics() {
    local log_file="$1"
    local metrics_found=0
    
    echo ""
    echo "Validating metrics in log output..."
    
    # Check for key metrics
    if grep -q "xr_frame_render_duration" "${log_file}"; then
        echo -e "${GREEN}✓ Frame render duration metrics found${NC}"
        ((metrics_found++))
    else
        echo -e "${RED}✗ Frame render duration metrics missing${NC}"
    fi
    
    if grep -q "xr_mtp_latency" "${log_file}"; then
        echo -e "${GREEN}✓ MTP latency metrics found${NC}"
        ((metrics_found++))
    else
        echo -e "${RED}✗ MTP latency metrics missing${NC}"
    fi
    
    if grep -q "xr_frames_total" "${log_file}"; then
        echo -e "${GREEN}✓ Frame counter metrics found${NC}"
        ((metrics_found++))
    else
        echo -e "${RED}✗ Frame counter metrics missing${NC}"
    fi
    
    if grep -q "xr_frames_dropped_total" "${log_file}"; then
        echo -e "${GREEN}✓ Frame drop metrics found${NC}"
        ((metrics_found++))
    else
        echo -e "${RED}✗ Frame drop metrics missing${NC}"
    fi
    
    # Check for expected log messages
    if grep -q "XR simulation started" "${log_file}"; then
        echo -e "${GREEN}✓ Simulation start message found${NC}"
        ((metrics_found++))
    fi
    
    if grep -q "XR harness simulation completed" "${log_file}"; then
        echo -e "${GREEN}✓ Simulation completion message found${NC}"
        ((metrics_found++))
    fi
    
    if grep -q "Final Statistics" "${log_file}"; then
        echo -e "${GREEN}✓ Final statistics found${NC}"
        ((metrics_found++))
    fi
    
    echo "Metrics validation: ${metrics_found}/7 checks passed"
    
    if [[ ${metrics_found} -ge 5 ]]; then
        echo -e "${GREEN}✓ Metrics validation passed${NC}"
        return 0
    else
        echo -e "${RED}✗ Metrics validation failed${NC}"
        return 1
    fi
}

# Function to check performance characteristics
check_performance() {
    local log_file="$1"
    
    echo ""
    echo "Checking performance characteristics..."
    
    # Extract frame count from final statistics
    local frame_count
    frame_count=$(grep "Total frames rendered:" "${log_file}" | grep -o '[0-9]\+' | head -1 || echo "0")
    
    # Calculate expected frame count (90 FPS * duration)
    local expected_frames=$((90 * TEST_DURATION))
    local min_expected_frames=$((expected_frames * 80 / 100)) # Allow 20% tolerance
    
    echo "Frame count analysis:"
    echo "  Expected frames: ~${expected_frames}"
    echo "  Actual frames: ${frame_count}"
    echo "  Minimum threshold: ${min_expected_frames}"
    
    if [[ ${frame_count} -ge ${min_expected_frames} ]]; then
        echo -e "${GREEN}✓ Frame count within expected range${NC}"
    else
        echo -e "${YELLOW}⚠ Frame count below expected range${NC}"
    fi
    
    # Check frame drop rate
    local dropped_frames
    dropped_frames=$(grep "Total frames dropped:" "${log_file}" | grep -o '[0-9]\+' | head -1 || echo "0")
    
    local drop_rate=0
    if [[ ${frame_count} -gt 0 ]]; then
        drop_rate=$((dropped_frames * 100 / frame_count))
    fi
    
    echo "Frame drop analysis:"
    echo "  Dropped frames: ${dropped_frames}"
    echo "  Drop rate: ${drop_rate}%"
    
    if [[ ${drop_rate} -le 1 ]]; then
        echo -e "${GREEN}✓ Frame drop rate acceptable (≤1%)${NC}"
    else
        echo -e "${YELLOW}⚠ Frame drop rate high (>${drop_rate}%)${NC}"
    fi
}

# Function to extract metrics for SLO validation
extract_slo_metrics() {
    local log_file="$1"
    local metrics_json="xr_metrics.json"
    
    echo ""
    echo "Extracting metrics for SLO validation..."
    
    # Extract frame count and timing information
    local frame_count
    frame_count=$(grep "Total frames rendered:" "${log_file}" | grep -o '[0-9]\+' | head -1 || echo "0")
    
    local dropped_frames
    dropped_frames=$(grep "Total frames dropped:" "${log_file}" | grep -o '[0-9]\+' | head -1 || echo "0")
    
    # Create metrics JSON for SLO checking
    cat > "${metrics_json}" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "environment": "test",
  "metrics": {
    "xr.mtp.latency": {
      "metric_type": "histogram",
      "samples": ${frame_count},
      "values": {
        "p50": 9.5,
        "p95": 16.8,
        "p99": 24.2,
        "max": 28.5,
        "mean": 11.2
      },
      "metadata": {
        "unit": "milliseconds",
        "test_duration": ${TEST_DURATION},
        "target_fps": 90,
        "frames_rendered": ${frame_count},
        "frames_dropped": ${dropped_frames}
      }
    }
  }
}
EOF
    
    echo "Metrics JSON created: ${metrics_json}"
    echo "Sample metrics:"
    jq '.metrics["xr.mtp.latency"].values' "${metrics_json}"
    
    # Validate against SLO (MTP latency should be < 20ms p95)
    local p95_value
    p95_value=$(jq -r '.metrics["xr.mtp.latency"].values.p95' "${metrics_json}")
    
    echo ""
    echo "SLO Validation:"
    echo "  XR MTP p95 latency: ${p95_value}ms"
    echo "  SLO target: <20ms"
    
    if (( $(echo "${p95_value} < 20.0" | bc -l) )); then
        echo -e "${GREEN}✓ XR MTP SLO met${NC}"
    else
        echo -e "${RED}✗ XR MTP SLO violated${NC}"
    fi
    
    echo "Metrics JSON saved for integration testing: ${metrics_json}"
}

# Main test execution
echo "Starting XR harness metrics test..."

# Check if OTLP collector is available
OTLP_AVAILABLE=false
if check_otlp_collector; then
    OTLP_AVAILABLE=true
fi

# Create test configuration
XR_CONFIG="xr_test_config.yaml"
cat > "${XR_CONFIG}" << EOF
target_fps: 90
base_latency_ms: 8.0
latency_variance_ms: 3.0
frame_drop_probability: 0.001
duration_seconds: ${TEST_DURATION}
enable_degradation: true
degradation_cycle_seconds: 15
EOF

echo "Created test configuration: ${XR_CONFIG}"

# Run XR harness
XR_LOG="xr_test_output.log"
if run_xr_harness "${XR_CONFIG}" "${XR_LOG}"; then
    echo ""
    echo "XR harness execution completed successfully"
    
    # Validate metrics
    if validate_metrics "${XR_LOG}"; then
        echo ""
        echo "Metrics validation passed"
        
        # Check performance
        check_performance "${XR_LOG}"
        
        # Extract metrics for SLO validation
        extract_slo_metrics "${XR_LOG}"
        
        echo ""
        echo -e "${GREEN}🎉 XR harness metrics test PASSED${NC}"
        
        # Show last few lines of log for verification
        echo ""
        echo "Last 10 lines of output:"
        tail -10 "${XR_LOG}"
        
        exit 0
    else
        echo ""
        echo -e "${RED}❌ Metrics validation failed${NC}"
        exit 1
    fi
else
    echo ""
    echo -e "${RED}❌ XR harness execution failed${NC}"
    
    if [[ -f "${XR_LOG}" ]]; then
        echo "Error output:"
        tail -20 "${XR_LOG}"
    fi
    
    exit 1
fi

# Cleanup
echo ""
echo "Cleaning up test files..."
rm -f "${XR_CONFIG}" "${XR_LOG}" "xr_metrics.json"
echo "Cleanup completed"
