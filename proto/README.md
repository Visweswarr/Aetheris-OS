# Polymera OS Protocol Buffer Linting

This directory contains the Protocol Buffer definitions for Polymera OS along with comprehensive linting, formatting, and breaking change detection using [Buf](https://buf.build/).

## 🚀 Quick Start

```bash
# Install Buf CLI
curl -sSL \
  "https://github.com/bufbuild/buf/releases/download/v1.28.1/buf-$(uname -s)-$(uname -m)" \
  -o "buf"
chmod +x buf
sudo mv buf /usr/local/bin/

# Verify installation
buf --version

# Run linting
cd proto
buf lint

# Check for breaking changes
buf breaking --against .git#subdir=proto,ref=HEAD~1

# Format proto files
buf format -w

# Generate code
buf generate
```

## 📋 Configuration Files

### `buf.yaml` - Main Configuration
- **Linting Rules**: Uses `DEFAULT` rules with exceptions for `PACKAGE_VERSION_SUFFIX` and `RPC_NO_STREAM`
- **Breaking Change Detection**: Checks `FILE`, `PACKAGE`, and `WIRE_JSON` compatibility
- **Dependencies**: Includes Google APIs and Envoy validation rules
- **Ignored Files**: Excludes `google/type/expr.proto` from checks

### `buf.gen.yaml` - Code Generation
- **Go**: Generates Go code with gRPC support
- **Rust**: Generates Rust code with gRPC support  
- **TypeScript**: Generates TypeScript code with gRPC support
- **Python**: Generates Python code with gRPC support
- **Managed Mode**: Enabled for consistent Go package naming

### `buf.work.yaml` - Workspace Configuration
- **Directories**: Includes `kernel/`, `test/`, and root proto files
- **Workspace**: Defines the scope for Buf operations

## 🔍 Linting Rules

### Default Rules Applied
- **Style**: Consistent naming conventions
- **Format**: Proper proto3 syntax
- **Best Practices**: Following Protocol Buffer guidelines
- **Documentation**: Required comments and descriptions

### Exceptions
- `PACKAGE_VERSION_SUFFIX`: Allows version suffixes in package names
- `RPC_NO_STREAM`: Allows non-streaming RPC methods

### Ignored Files
- `google/type/expr.proto`: Excluded from linting and breaking change checks

## 🚨 Breaking Change Detection

### What Triggers Breaking Changes
1. **Field Number Changes**: Modifying existing field numbers
2. **Field Type Changes**: Changing field types (e.g., string → int32)
3. **Field Removal**: Deleting existing fields
4. **Required Field Addition**: Adding required fields to existing messages
5. **Service Method Changes**: Modifying RPC method signatures

### What's Safe
1. **Adding Optional Fields**: New fields with default values
2. **Adding New Messages**: New message types don't affect existing ones
3. **Adding New Services**: New service definitions are safe
4. **Adding New Methods**: New RPC methods don't break existing clients

### Breaking Change Rules
- **FILE**: Detects changes within individual files
- **PACKAGE**: Detects package-level changes
- **WIRE_JSON**: Detects wire format compatibility issues

## 🎯 Bazel Integration

### Build Targets
```bash
# Run Buf lint check
bazel build //proto:buf_lint

# Run breaking change check
bazel build //proto:buf_breaking

# Run format check
bazel build //proto:buf_format

# Run all Buf checks
bazel build //proto:buf_all_checks

# Generate code with Buf
bazel build //proto:buf_generate
```

### CI Integration
```bash
# CI lint job target
bazel build //proto:ci_buf_lint

# CI breaking change job target
bazel build //proto:ci_buf_breaking
```

## 🔧 CI/CD Integration

### GitHub Actions Workflow
The `buf-lint.yml` workflow automatically runs on:
- **Push**: To `main` and `develop` branches
- **Pull Request**: To `main` and `develop` branches
- **Path Changes**: Any changes to proto files or Buf configuration

### CI Job Steps
1. **Setup**: Install Go and Buf CLI
2. **Lint**: Run style and format checks
3. **Breaking**: Detect API compatibility issues
4. **Generate**: Test code generation (dry run)
5. **Results**: Upload artifacts and comment on PRs

### Breaking Change Detection
```bash
# CI automatically detects breaking changes
buf breaking --against ".git#subdir=proto,ref=$BASE_COMMIT"

# Base commit is determined by:
# - PR: Base branch commit
# - Push: Previous commit (HEAD~1)
```

## 🧪 Testing Breaking Changes

### Test Scenarios
The `proto/test/` directory contains examples for testing:

#### 1. Breaking Change Test
```bash
# Introduce breaking changes
./proto/test/introduce_breaking_change.sh

# This will:
# - Change field types (string → int32)
# - Change field numbers (4 → 5)
# - Remove existing fields
# - Add new service methods

# Test breaking change detection
buf breaking --against .git#subdir=proto,ref=HEAD~1
# Should FAIL with breaking change errors
```

#### 2. Safe Change Test
```bash
# Introduce safe changes
./proto/test/introduce_safe_change.sh

# This will:
# - Add new optional fields
# - Add new message types
# - Add new service methods
# - Maintain backward compatibility

# Test breaking change detection
buf breaking --against .git#subdir=proto,ref=HEAD~1
# Should PASS with no breaking changes
```

### Restore Original Files
```bash
# After testing, restore original files
cp proto/test/breaking_change_test.proto.backup proto/test/breaking_change_test.proto
```

## 📚 Best Practices

### Proto File Structure
```protobuf
syntax = "proto3";

package polymera.service.name;

import "google/protobuf/timestamp.proto";

// Service description
service ServiceName {
  // Method description
  rpc MethodName(RequestMessage) returns (ResponseMessage);
}

// Message description
message RequestMessage {
  // Field description
  string field_name = 1;
  int32 field_number = 2;
}
```

### Naming Conventions
- **Packages**: Use lowercase with dots (e.g., `polymera.kernel.core`)
- **Services**: Use PascalCase with "Service" suffix (e.g., `KernelService`)
- **Messages**: Use PascalCase (e.g., `CreateProcessRequest`)
- **Fields**: Use snake_case (e.g., `process_name`)
- **RPC Methods**: Use PascalCase (e.g., `CreateProcess`)

### Field Management
- **Never change field numbers** of existing fields
- **Never change field types** of existing fields
- **Never remove fields** from existing messages
- **Always add new fields** as optional
- **Use descriptive field names** and comments

## 🔍 Troubleshooting

### Common Issues

#### 1. Lint Failures
```bash
# Check specific lint rules
buf lint --config buf.yaml

# Fix formatting issues
buf format -w

# Check for specific rule violations
buf lint --error-format=json
```

#### 2. Breaking Change Detection
```bash
# Check against specific commit
buf breaking --against .git#subdir=proto,ref=abc123

# Check against remote branch
buf breaking --against .git#subdir=proto,ref=origin/main

# Ignore specific files
buf breaking --against .git#subdir=proto,ref=HEAD~1 --exclude-path google/type/expr.proto
```

#### 3. Code Generation Issues
```bash
# Test generation without writing files
buf generate --dry-run

# Check plugin availability
buf plugins ls

# Update dependencies
buf mod update
```

### Debug Commands
```bash
# Show Buf configuration
buf config ls-env

# Show workspace structure
buf work ls

# Show module dependencies
buf mod ls

# Show plugin information
buf plugins ls
```

## 📖 Additional Resources

### Documentation
- [Buf Official Docs](https://docs.buf.build/)
- [Protocol Buffer Style Guide](https://developers.google.com/protocol-buffers/docs/style)
- [gRPC Best Practices](https://grpc.io/docs/guides/best-practices/)

### Community
- [Buf GitHub](https://github.com/bufbuild/buf)
- [Protocol Buffers Community](https://developers.google.com/protocol-buffers/community)
- [gRPC Community](https://grpc.io/community/)

### Tools
- [Buf Studio](https://studio.buf.build/) - Web-based proto editor
- [Buf CLI](https://github.com/bufbuild/buf) - Command-line tools
- [Buf VS Code Extension](https://marketplace.visualstudio.com/items?itemName=bufbuild.vscode-buf)

---

**Happy Protocol Buffering! 🎉**

For questions or issues, please refer to the troubleshooting section or create an issue in the repository.
