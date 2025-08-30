#!/bin/bash

set -e

echo "🧪 Testing Polymera OS Observability Setup..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the observability directory"
    exit 1
fi

# Build the observability crate
echo "🔨 Building observability crate..."
cargo build --features full

# Run unit tests
echo "🧪 Running unit tests..."
cargo test --features full

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --features full --test integration

# Test the test binary
echo "📊 Testing observability test binary..."
cargo run --bin observability-test --features full

# Check if monitoring stack is running
echo "🔍 Checking monitoring stack status..."
if curl -f http://localhost:3000/api/health > /dev/null 2>&1; then
    echo "✅ Grafana is running"
    
    # Test metrics endpoint
    if curl -f http://localhost:9090/-/healthy > /dev/null 2>&1; then
        echo "✅ Prometheus is running"
        
        # Test OpenTelemetry Collector
        if curl -f http://localhost:8888/ > /dev/null 2>&1; then
            echo "✅ OpenTelemetry Collector is running"
            
            # Test metrics collection
            echo "📈 Testing metrics collection..."
            
            # Simulate some metrics
            curl -X POST http://localhost:4318/v1/metrics \
                -H "Content-Type: application/json" \
                -d '{
                    "resourceMetrics": [{
                        "resource": {
                            "attributes": [{
                                "key": "service.name",
                                "value": { "stringValue": "test-service" }
                            }]
                        },
                        "scopeMetrics": [{
                            "scope": {
                                "name": "test-scope"
                            },
                            "metrics": [{
                                "name": "test_counter",
                                "unit": "1",
                                "sum": {
                                    "dataPoints": [{
                                        "timeUnixNano": "'$(date +%s%N)'",
                                        "value": 42
                                    }]
                                }
                            }]
                        }]
                    }]
                }'
            
            echo "✅ Test metrics sent successfully"
            
        else
            echo "❌ OpenTelemetry Collector is not responding"
        fi
    else
        echo "❌ Prometheus is not responding"
    fi
else
    echo "❌ Grafana is not running. Please start the monitoring stack first:"
    echo "   cd infra/grafana && ./up.sh"
fi

# Test privacy features
echo "🔒 Testing privacy features..."
cargo test --features full privacy

# Test performance
echo "⚡ Testing performance..."
cargo run --bin observability-bench --features full

echo ""
echo "🎉 Observability testing completed!"
echo ""
echo "📊 To view metrics:"
echo "   - Grafana: http://localhost:3000 (admin/polymera123)"
echo "   - Prometheus: http://localhost:9090"
echo "   - Jaeger: http://localhost:16686"
echo ""
echo "🔧 To integrate with your services:"
echo "   1. Add polymera-observability as a dependency"
echo "   2. Initialize observability in your main function"
echo "   3. Use RequestTimer for automatic metrics"
echo "   4. Create spans for distributed tracing"
echo ""
echo "📚 For more information, see the README.md file"
