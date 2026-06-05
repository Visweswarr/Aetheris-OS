#!/usr/bin/env bash
# Test script for SBOM generation

# Mock syft command for testing
mock_syft() {
    echo "Mock syft version: 1.0.0"
    echo '{"version": "1.0.0"}'
}

# Test the generate script with mock syft
export PATH="$(pwd)/tools/sbom:$PATH"

# Create mock syft
cat > tools/sbom/syft << 'EOF'
#!/bin/bash
if [[ "$1" == "version" ]]; then
    echo '{"version": "1.0.0"}'
else
    echo '{"packages": []}'
fi
EOF

chmod +x tools/sbom/syft

# Run the generate script
bash tools/sbom/generate.sh

# Cleanup
rm -f tools/sbom/syft
