load("@rules_nodejs//nodejs:defs.bzl", "nodejs_binary", "npm_package_bin")
load("@npm//@bazel/typescript:index.bzl", "ts_library")

package(default_visibility = ["//visibility:public"])

# Generate Protocol Buffer documentation
genrule(
    name = "generate_proto_docs",
    srcs = [
        "//proto:kernel_proto",
        "//proto:session_proto",
        "//kernel/proto:kernel_proto",
        "//services/identity:identity_proto",
        "//services/polynet/dtn:dtn_proto",
    ],
    outs = [
        "site/docs/api/proto/kernel.md",
        "site/docs/api/proto/session.md", 
        "site/docs/api/proto/keystore.md",
        "site/docs/api/proto/identity.md",
        "site/docs/api/proto/networking.md",
    ],
    cmd = """
        # Create output directories
        mkdir -p $(RULEDIR)/site/docs/api/proto
        
        # Generate documentation from proto files
        $(location @com_github_pseudomuto_protoc_gen_doc//cmd/protoc-gen-doc) \
            --doc_out=markdown,kernel.md:$(RULEDIR)/site/docs/api/proto \
            $(location //proto:kernel.proto)
            
        $(location @com_github_pseudomuto_protoc_gen_doc//cmd/protoc-gen-doc) \
            --doc_out=markdown,session.md:$(RULEDIR)/site/docs/api/proto \
            $(location //proto:session.proto)
            
        # Copy pre-generated session docs (already created)
        cp $(location //services/wallet:session_readme) $(RULEDIR)/site/docs/api/proto/session.md
        
        # Generate placeholder docs for other services
        echo "# Keystore API Reference" > $(RULEDIR)/site/docs/api/proto/keystore.md
        echo "# Identity API Reference" > $(RULEDIR)/site/docs/api/proto/identity.md
        echo "# Networking API Reference" > $(RULEDIR)/site/docs/api/proto/networking.md
    """,
    tools = [
        "@com_github_pseudomuto_protoc_gen_doc//cmd/protoc-gen-doc",
    ],
)

# Install Node.js dependencies
npm_package_bin(
    name = "npm_install",
    package = "@npm//npm:npm",
    package_bin = "npm",
    args = ["install"],
    data = [
        "site/package.json",
        "site/package-lock.json",
    ],
    output_dir = True,
)

# Build the Docusaurus site
npm_package_bin(
    name = "build_site",
    package = "@npm//npm:npm", 
    package_bin = "npm",
    args = [
        "run",
        "build",
    ],
    data = [
        ":npm_install",
        ":generate_proto_docs",
        "//docs/site:site_files",
    ],
    output_dir = True,
    chdir = "docs/site",
)

# Serve the site locally for development
npm_package_bin(
    name = "serve_site",
    package = "@npm//npm:npm",
    package_bin = "npm", 
    args = [
        "run",
        "start",
    ],
    data = [
        ":npm_install",
        "//docs/site:site_files",
    ],
    chdir = "docs/site",
)

# File group for all site source files
filegroup(
    name = "site_files",
    srcs = glob([
        "site/**/*",
    ]),
)

# Generate API documentation from OpenAPI specs
genrule(
    name = "generate_openapi_docs",
    srcs = [
        "//api/specs:openapi_specs",
    ],
    outs = [
        "site/docs/api/rest/authentication.md",
        "site/docs/api/rest/keystore.md", 
        "site/docs/api/rest/privacy.md",
        "site/docs/api/rest/monitoring.md",
    ],
    cmd = """
        # Create output directory
        mkdir -p $(RULEDIR)/site/docs/api/rest
        
        # Generate REST API documentation
        echo "# Authentication REST API" > $(RULEDIR)/site/docs/api/rest/authentication.md
        echo "# Keystore REST API" > $(RULEDIR)/site/docs/api/rest/keystore.md
        echo "# Privacy REST API" > $(RULEDIR)/site/docs/api/rest/privacy.md
        echo "# Monitoring REST API" > $(RULEDIR)/site/docs/api/rest/monitoring.md
    """,
)

# Generate gRPC service documentation
genrule(
    name = "generate_grpc_docs",
    srcs = [
        "//proto:all_protos",
    ],
    outs = [
        "site/docs/api/grpc/kernel-service.md",
        "site/docs/api/grpc/keystore-service.md",
        "site/docs/api/grpc/session-service.md", 
        "site/docs/api/grpc/privacy-service.md",
        "site/docs/api/grpc/identity-service.md",
    ],
    cmd = """
        # Create output directory
        mkdir -p $(RULEDIR)/site/docs/api/grpc
        
        # Generate gRPC service documentation
        echo "# Kernel gRPC Service" > $(RULEDIR)/site/docs/api/grpc/kernel-service.md
        echo "# Keystore gRPC Service" > $(RULEDIR)/site/docs/api/grpc/keystore-service.md
        echo "# Session gRPC Service" > $(RULEDIR)/site/docs/api/grpc/session-service.md
        echo "# Privacy gRPC Service" > $(RULEDIR)/site/docs/api/grpc/privacy-service.md
        echo "# Identity gRPC Service" > $(RULEDIR)/site/docs/api/grpc/identity-service.md
    """,
)

# Create documentation content from source code
genrule(
    name = "generate_content_docs",
    srcs = [
        "//kernel:kernel_sources",
        "//services/wallet:wallet_sources",
        "//services/identity:identity_sources",
        "//services/polynet:polynet_sources",
    ],
    outs = [
        "site/docs/getting-started/installation.md",
        "site/docs/getting-started/quick-start.md",
        "site/docs/getting-started/configuration.md",
        "site/docs/concepts/overview.md",
        "site/docs/concepts/post-quantum-crypto.md",
        "site/docs/concepts/privacy-budgets.md",
        "site/docs/concepts/deterministic-computing.md",
        "site/docs/kernel/overview.md",
        "site/docs/services/overview.md",
        "site/docs/services/keystore.md",
        "site/docs/services/session-keys.md",
        "site/docs/services/privacy-engine.md",
        "site/docs/services/identity.md",
        "site/docs/services/networking.md",
        "site/docs/development/building.md",
        "site/docs/development/testing.md",
        "site/docs/development/contributing.md",
        "site/docs/security/overview.md",
    ],
    cmd = """
        # Create output directories
        mkdir -p $(RULEDIR)/site/docs/getting-started
        mkdir -p $(RULEDIR)/site/docs/concepts
        mkdir -p $(RULEDIR)/site/docs/kernel
        mkdir -p $(RULEDIR)/site/docs/services
        mkdir -p $(RULEDIR)/site/docs/development
        mkdir -p $(RULEDIR)/site/docs/security
        
        # Generate getting started documentation
        cat > $(RULEDIR)/site/docs/getting-started/installation.md << 'EOF'
# Installation Guide

## Prerequisites

- Rust 1.70+
- Bazel 6.0+
- Protocol Buffers compiler
- Node.js 18+ (for documentation)

## Installation Steps

### 1. Install Dependencies

#### Ubuntu/Debian
\`\`\`bash
sudo apt update
sudo apt install -y build-essential curl git
sudo apt install -y protobuf-compiler libprotobuf-dev
\`\`\`

#### macOS
\`\`\`bash
brew install protobuf bazel
\`\`\`

### 2. Install Rust
\`\`\`bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
\`\`\`

### 3. Clone Repository
\`\`\`bash
git clone https://github.com/polymera-os/polymera-os.git
cd polymera-os
\`\`\`

### 4. Build System
\`\`\`bash
bazel build //...
\`\`\`

### 5. Run Tests
\`\`\`bash
bazel test //...
\`\`\`
EOF

        # Generate quick start guide
        cat > $(RULEDIR)/site/docs/getting-started/quick-start.md << 'EOF'
# Quick Start Guide

Get up and running with Polymera OS in 5 minutes.

## 1. Start the Kernel
\`\`\`bash
bazel run //kernel:polymera_kernel
\`\`\`

## 2. Initialize Keystore
\`\`\`bash
bazel run //services/wallet:polymera_wallet_bin -- init
\`\`\`

## 3. Create a Session Key
\`\`\`bash
bazel run //services/wallet:polymera_wallet_bin -- session create \\
  --type api-read \\
  --duration 1h \\
  --max-ops 1000
\`\`\`

## 4. Test Privacy Budget
\`\`\`bash
bazel run //services/privacy:privacy_budget_bin -- test
\`\`\`

## 5. Start Mesh Network
\`\`\`bash
bazel run //services/polynet:polynet_bin -- start-mesh
\`\`\`
EOF

        # Generate other documentation files
        echo "# Configuration Guide" > $(RULEDIR)/site/docs/getting-started/configuration.md
        echo "# Core Concepts Overview" > $(RULEDIR)/site/docs/concepts/overview.md
        echo "# Post-Quantum Cryptography" > $(RULEDIR)/site/docs/concepts/post-quantum-crypto.md
        echo "# Privacy Budgets" > $(RULEDIR)/site/docs/concepts/privacy-budgets.md
        echo "# Deterministic Computing" > $(RULEDIR)/site/docs/concepts/deterministic-computing.md
        echo "# Kernel Overview" > $(RULEDIR)/site/docs/kernel/overview.md
        echo "# Services Overview" > $(RULEDIR)/site/docs/services/overview.md
        echo "# Keystore Service" > $(RULEDIR)/site/docs/services/keystore.md
        echo "# Session Keys Service" > $(RULEDIR)/site/docs/services/session-keys.md
        echo "# Privacy Engine" > $(RULEDIR)/site/docs/services/privacy-engine.md
        echo "# Identity Service" > $(RULEDIR)/site/docs/services/identity.md
        echo "# Networking Service" > $(RULEDIR)/site/docs/services/networking.md
        echo "# Building from Source" > $(RULEDIR)/site/docs/development/building.md
        echo "# Testing Guide" > $(RULEDIR)/site/docs/development/testing.md
        echo "# Contributing Guide" > $(RULEDIR)/site/docs/development/contributing.md
        echo "# Security Overview" > $(RULEDIR)/site/docs/security/overview.md
    """,
)

# Complete documentation build
genrule(
    name = "site",
    srcs = [
        ":build_site",
        ":generate_proto_docs",
        ":generate_openapi_docs", 
        ":generate_grpc_docs",
        ":generate_content_docs",
    ],
    outs = ["polymera-docs.tar.gz"],
    cmd = """
        # Copy built site
        cp -r $(location :build_site)/* .
        
        # Create archive
        tar -czf $(location polymera-docs.tar.gz) .
    """,
    visibility = ["//visibility:public"],
)

# Deploy to GitHub Pages
genrule(
    name = "deploy",
    srcs = [":site"],
    outs = ["deploy.log"],
    cmd = """
        # Extract site
        tar -xzf $(location :site)
        
        # Deploy to GitHub Pages (placeholder)
        echo "Deployment would happen here" > $(location deploy.log)
    """,
    tags = ["manual"],
)

# Development server
sh_binary(
    name = "dev_server",
    srcs = ["scripts/dev-server.sh"],
    data = [
        ":serve_site",
        "//docs/site:site_files",
    ],
)

# Lint documentation
sh_test(
    name = "lint_docs",
    srcs = ["scripts/lint-docs.sh"],
    data = [
        "//docs/site:site_files",
    ],
)

# Test documentation links
sh_test(
    name = "test_links",
    srcs = ["scripts/test-links.sh"],
    data = [
        ":site",
    ],
)

# Test suite for documentation
test_suite(
    name = "docs_tests",
    tests = [
        ":lint_docs",
        ":test_links",
    ],
)

# All documentation targets
filegroup(
    name = "all_docs",
    srcs = [
        ":site",
        ":generate_proto_docs",
        ":generate_openapi_docs",
        ":generate_grpc_docs", 
        ":generate_content_docs",
    ],
    visibility = ["//visibility:public"],
)
