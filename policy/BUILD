load("@rules_opa//opa:defs.bzl", "opa_library", "opa_test", "opa_wasm")

package(default_visibility = ["//visibility:public"])

# Wallet Spend Limits Policy
opa_library(
    name = "wallet_spend_limits_lib",
    srcs = ["samples/wallet_spend_limits.rego"],
)

opa_test(
    name = "wallet_spend_limits_test",
    srcs = [
        "samples/wallet_spend_limits.rego",
        "samples/wallet_spend_limits_test.rego",
    ],
    deps = [":wallet_spend_limits_lib"],
)

opa_wasm(
    name = "wallet_spend_limits_wasm",
    src = "samples/wallet_spend_limits.rego",
    out = "out/wallet_spend_limits.wasm",
    entrypoint = "polymera/wallet/spend_limits/allow",
)

# Network Rate Limits Policy
opa_library(
    name = "network_rate_limits_lib",
    srcs = ["samples/network_rate_limits.rego"],
)

opa_test(
    name = "network_rate_limits_test",
    srcs = [
        "samples/network_rate_limits.rego",
        "samples/network_rate_limits_test.rego",
    ],
    deps = [":network_rate_limits_lib"],
)

opa_wasm(
    name = "network_rate_limits_wasm",
    src = "samples/network_rate_limits.rego",
    out = "out/network_rate_limits.wasm",
    entrypoint = "polymera/network/rate_limits/allow",
)

# Filesystem Access Scopes Policy
opa_library(
    name = "fs_access_scopes_lib",
    srcs = ["samples/fs_access_scopes.rego"],
)

opa_test(
    name = "fs_access_scopes_test",
    srcs = [
        "samples/fs_access_scopes.rego",
        "samples/fs_access_scopes_test.rego",
    ],
    deps = [":fs_access_scopes_lib"],
)

opa_wasm(
    name = "fs_access_scopes_wasm",
    src = "samples/fs_access_scopes.rego",
    out = "out/fs_access_scopes.wasm",
    entrypoint = "polymera/filesystem/access_scopes/allow",
)

# Additional WASM targets for rationale functions
opa_wasm(
    name = "wallet_spend_limits_rationale_wasm",
    src = "samples/wallet_spend_limits.rego",
    out = "out/wallet_spend_limits_rationale.wasm",
    entrypoint = "polymera/wallet/spend_limits/rationale",
)

opa_wasm(
    name = "network_rate_limits_rationale_wasm",
    src = "samples/network_rate_limits.rego",
    out = "out/network_rate_limits_rationale.wasm",
    entrypoint = "polymera/network/rate_limits/rationale",
)

opa_wasm(
    name = "fs_access_scopes_rationale_wasm",
    src = "samples/fs_access_scopes.rego",
    out = "out/fs_access_scopes_rationale.wasm",
    entrypoint = "polymera/filesystem/access_scopes/rationale",
)

# Test data files
filegroup(
    name = "test_data",
    srcs = [
        "test_data/wallet_test_scenarios.json",
        "test_data/network_test_scenarios.json",
        "test_data/fs_test_scenarios.json",
    ],
)

# Manual WASM compilation using OPA CLI (fallback for custom builds)
genrule(
    name = "compile_wallet_policy_manual",
    srcs = ["samples/wallet_spend_limits.rego"],
    outs = ["out/wallet_spend_limits_manual.wasm"],
    cmd = """
        opa build -t wasm -e polymera/wallet/spend_limits/allow $(SRCS) -o /tmp/bundle.tar.gz
        cd /tmp && tar -xzf bundle.tar.gz
        cp /tmp/policy.wasm $(location out/wallet_spend_limits_manual.wasm)
    """,
    tools = ["@opa//:opa"],
)

genrule(
    name = "compile_network_policy_manual",
    srcs = ["samples/network_rate_limits.rego"],
    outs = ["out/network_rate_limits_manual.wasm"],
    cmd = """
        opa build -t wasm -e polymera/network/rate_limits/allow $(SRCS) -o /tmp/bundle.tar.gz
        cd /tmp && tar -xzf bundle.tar.gz
        cp /tmp/policy.wasm $(location out/network_rate_limits_manual.wasm)
    """,
    tools = ["@opa//:opa"],
)

genrule(
    name = "compile_fs_policy_manual",
    srcs = ["samples/fs_access_scopes.rego"],
    outs = ["out/fs_access_scopes_manual.wasm"],
    cmd = """
        opa build -t wasm -e polymera/filesystem/access_scopes/allow $(SRCS) -o /tmp/bundle.tar.gz
        cd /tmp && tar -xzf bundle.tar.gz
        cp /tmp/policy.wasm $(location out/fs_access_scopes_manual.wasm)
    """,
    tools = ["@opa//:opa"],
)

# Comprehensive test suite
test_suite(
    name = "policy_tests",
    tests = [
        ":wallet_spend_limits_test",
        ":network_rate_limits_test",
        ":fs_access_scopes_test",
    ],
)

# WASM compilation targets
filegroup(
    name = "wasm_policies",
    srcs = [
        ":wallet_spend_limits_wasm",
        ":network_rate_limits_wasm",
        ":fs_access_scopes_wasm",
        ":wallet_spend_limits_rationale_wasm",
        ":network_rate_limits_rationale_wasm",
        ":fs_access_scopes_rationale_wasm",
    ],
)

# Manual WASM compilation for all policies
filegroup(
    name = "wasm_policies_manual",
    srcs = [
        ":compile_wallet_policy_manual",
        ":compile_network_policy_manual",
        ":compile_fs_policy_manual",
    ],
)

# Bundle all policies for deployment
genrule(
    name = "policy_bundle",
    srcs = [
        "samples/wallet_spend_limits.rego",
        "samples/network_rate_limits.rego",
        "samples/fs_access_scopes.rego",
    ],
    outs = ["out/polymera_policies.tar.gz"],
    cmd = """
        mkdir -p /tmp/policies/samples
        cp $(location samples/wallet_spend_limits.rego) /tmp/policies/samples/
        cp $(location samples/network_rate_limits.rego) /tmp/policies/samples/
        cp $(location samples/fs_access_scopes.rego) /tmp/policies/samples/
        
        # Create metadata file
        cat > /tmp/policies/metadata.json << 'EOF'
{
  "version": "1.0.0",
  "policies": [
    {
      "name": "wallet_spend_limits",
      "package": "polymera.wallet.spend_limits",
      "entrypoint": "allow",
      "description": "Wallet spending limits and controls"
    },
    {
      "name": "network_rate_limits", 
      "package": "polymera.network.rate_limits",
      "entrypoint": "allow",
      "description": "Network access rate limiting and controls"
    },
    {
      "name": "fs_access_scopes",
      "package": "polymera.filesystem.access_scopes", 
      "entrypoint": "allow",
      "description": "Filesystem access scoping and permissions"
    }
  ]
}
EOF
        
        cd /tmp && tar -czf policies.tar.gz policies/
        cp /tmp/policies.tar.gz $(location out/polymera_policies.tar.gz)
    """,
)

# Policy validation script
sh_test(
    name = "validate_policies",
    srcs = ["scripts/validate_policies.sh"],
    data = [
        "samples/wallet_spend_limits.rego",
        "samples/network_rate_limits.rego", 
        "samples/fs_access_scopes.rego",
        ":test_data",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Performance benchmarking
sh_test(
    name = "benchmark_policies",
    srcs = ["scripts/benchmark_policies.sh"],
    data = [
        ":wasm_policies",
        ":test_data",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
    size = "large",
)

# WASM size optimization test
sh_test(
    name = "test_wasm_size",
    srcs = ["scripts/test_wasm_size.sh"],
    data = [":wasm_policies"],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Integration test with mock runtime
sh_test(
    name = "integration_test",
    srcs = ["scripts/integration_test.sh"],
    data = [
        ":wasm_policies",
        ":test_data",
        "//tools:wasm_runner",  # Mock WASM runtime for testing
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Documentation generation
genrule(
    name = "generate_policy_docs",
    srcs = [
        "samples/wallet_spend_limits.rego",
        "samples/network_rate_limits.rego",
        "samples/fs_access_scopes.rego",
    ],
    outs = ["out/policy_documentation.md"],
    cmd = """
        # Generate documentation from Rego comments and structure
        cat > $(location out/policy_documentation.md) << 'EOF'
# Polymera OS Policy Documentation

## Generated Policy Documentation

This documentation is automatically generated from the Rego policy files.

EOF
        
        for policy in $(SRCS); do
            echo "## Policy: $$(basename $$policy .rego)" >> $(location out/policy_documentation.md)
            echo "" >> $(location out/policy_documentation.md)
            grep "^#" $$policy | sed 's/^# //' >> $(location out/policy_documentation.md)
            echo "" >> $(location out/policy_documentation.md)
        done
    """,
)

# Alias for easy building
alias(
    name = "build_all",
    actual = ":wasm_policies",
)

alias(
    name = "test_all",
    actual = ":policy_tests",
)
