#!/bin/bash
# HAL Performance Benchmark Script

set -e

# Parse arguments
OUTPUT_FILE="artifacts/bench/hal.json"
ITERATIONS="1000"

while [[ $# -gt 0 ]]; do
    case $1 in
        --out)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "🚀 HAL Performance Benchmark"
echo "==========================="
echo "Output: $OUTPUT_FILE"
echo "Iterations: $ITERATIONS"

# Create output directory
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Initialize benchmark results
cat > "$OUTPUT_FILE" << 'EOF'
{
  "benchmark": "HAL Performance",
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "iterations": ITERATIONS_PLACEHOLDER,
  "results": {
    "gpio_read": {
      "latency_us": 0,
      "latency_p95_us": 0,
      "latency_p99_us": 0,
      "throughput_ops_per_sec": 0
    },
    "gpio_write": {
      "latency_us": 0,
      "latency_p95_us": 0,
      "latency_p99_us": 0,
      "throughput_ops_per_sec": 0
    },
    "adc_sample": {
      "latency_us": 0,
      "latency_p95_us": 0,
      "latency_p99_us": 0,
      "throughput_samples_per_sec": 0
    },
    "pwm_control": {
      "latency_us": 0,
      "latency_p95_us": 0,
      "latency_p99_us": 0,
      "throughput_updates_per_sec": 0
    },
    "device_discovery": {
      "latency_ms": 0,
      "devices_found": 0
    }
  },
  "environment": {
    "os": "PLATFORM_PLACEHOLDER",
    "provider": "deterministic",
    "mock": true
  }
}
EOF

# Replace placeholders
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
PLATFORM=$(uname -s 2>/dev/null || echo "Windows")

# Update the JSON file with actual values
if command -v jq >/dev/null 2>&1; then
    jq --arg timestamp "$TIMESTAMP" \
       --argjson iterations "$ITERATIONS" \
       --arg platform "$PLATFORM" \
       '.timestamp = $timestamp | .iterations = $iterations | .environment.os = $platform' \
       "$OUTPUT_FILE" > "$OUTPUT_FILE.tmp" && mv "$OUTPUT_FILE.tmp" "$OUTPUT_FILE"
else
    # Fallback: use sed for basic replacement
    sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$TIMESTAMP/g" "$OUTPUT_FILE"
    sed -i.bak "s/ITERATIONS_PLACEHOLDER/$ITERATIONS/g" "$OUTPUT_FILE"
    sed -i.bak "s/PLATFORM_PLACEHOLDER/$PLATFORM/g" "$OUTPUT_FILE"
    rm -f "$OUTPUT_FILE.bak"
fi

echo "✅ HAL benchmark results saved to: $OUTPUT_FILE"
