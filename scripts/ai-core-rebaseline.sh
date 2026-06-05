#!/bin/bash
# AI Core Service Golden Test Rebaseline Script
# This script rebaselines the AI Core Service golden tests

set -e

echo "🔄 Rebaselining AI Core Service Golden Tests..."
echo "=============================================="

# Check if we're in the right directory
if [ ! -f "scripts/golden/ai_core_golden.rs" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Ensure artifacts directory exists
mkdir -p artifacts/ai_core

# Check if AI Core Service is running
if [ ! -S "/tmp/ai_core.sock" ]; then
    echo "⚠️  Warning: AI Core Service socket not found at /tmp/ai_core.sock"
    echo "   Starting AI Core Service in background..."
    
    # Start AI Core Service in background
    if command -v cargo >/dev/null 2>&1; then
        cd services/ai_core
        cargo run --bin ai_core &
        AI_CORE_PID=$!
        cd ../..
        
        # Wait for service to start
        echo "   Waiting for AI Core Service to start..."
        sleep 5
        
        # Check if service is running
        if ! kill -0 $AI_CORE_PID 2>/dev/null; then
            echo "❌ Error: Failed to start AI Core Service"
            exit 1
        fi
        
        echo "✅ AI Core Service started (PID: $AI_CORE_PID)"
    else
        echo "❌ Error: cargo not found, cannot start AI Core Service"
        exit 1
    fi
else
    echo "✅ AI Core Service is running"
    AI_CORE_PID=""
fi

# Run golden tests in rebaseline mode
echo ""
echo "🧪 Running golden tests in rebaseline mode..."

if command -v cargo >/dev/null 2>&1; then
    cd scripts/golden
    cargo run --bin ai_core_golden -- --rebaseline
    cd ../..
else
    echo "⚠️  cargo not found, using fallback method"
    rustc scripts/golden/ai_core_golden.rs -o scripts/golden/ai_core_golden
    ./scripts/golden/ai_core_golden --rebaseline
fi

# Check if rebaseline was successful
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Golden tests rebaselined successfully!"
    echo ""
    echo "📊 Summary:"
    echo "  - Configuration: scripts/golden/ai_core_config.json"
    echo "  - Golden file: artifacts/ai_core/golden.json"
    echo "  - Report: artifacts/ai_core/golden_report.json"
    echo ""
    echo "🔍 To verify the rebaseline:"
    echo "  make golden-ai-core"
else
    echo ""
    echo "❌ Golden test rebaseline failed!"
    exit 1
fi

# Clean up background process if we started it
if [ -n "$AI_CORE_PID" ]; then
    echo ""
    echo "🛑 Stopping AI Core Service (PID: $AI_CORE_PID)..."
    kill $AI_CORE_PID 2>/dev/null || true
    wait $AI_CORE_PID 2>/dev/null || true
    echo "✅ AI Core Service stopped"
fi

echo ""
echo "🎉 Rebaseline complete!"
