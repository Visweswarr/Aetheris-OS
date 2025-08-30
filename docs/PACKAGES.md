# Polymera OS Package Format

Content-addressed package system for Polymera OS with SBOM, signatures, and capability manifests.

## 🚀 Features

- **Content Addressing**: Immutable packages identified by SHA-256 content hash
- **SBOM Generation**: Software Bill of Materials in SPDX, CycloneDX, and SWID formats
- **Digital Signatures**: Cryptographic verification with Ed25519, RSA, and ECDSA
- **Capability Security**: Fine-grained permission system for package execution
- **Package Verification**: Comprehensive integrity and security validation
- **JSON Schema**: Well-defined manifest format with validation

## 📦 Package Structure

### Content-Addressed Bundles

Packages are identified by their content hash, ensuring immutability and integrity:

```
package_hash = SHA256(manifest_content + file_contents)
```

### Package Components

```
package.pkg/
├── manifest.json          # Package manifest with metadata
├── src/                   # Source files
├── bin/                   # Executable files
├── lib/                   # Library files
└── docs/                  # Documentation
```

## 🔧 Quick Start

### Building a Package

```rust
use polymera_pack::{
    PackageBuilderConfig, build_package, Capability
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PackageBuilderConfig {
        source_dir: PathBuf::from("./src"),
        output_dir: PathBuf::from("./dist"),
        package_id: "my-service".to_string(),
        version: "1.0.0".to_string(),
        name: "My Service".to_string(),
        description: "A sample service".to_string(),
        author: "Developer".to_string(),
        license: "MIT".to_string(),
        generate_sbom: true,
        sign_package: true,
        ..Default::default()
    };

    let manifest = build_package(config).await?;
    println!("Package built: {}", manifest.package_id);
    println!("Content hash: {}", manifest.content_hash);
    
    Ok(())
}
```

### Verifying a Package

```rust
use polymera_pack::{verify_package, VerificationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = VerificationConfig {
        verify_content_hash: true,
        verify_file_checksums: true,
        verify_signatures: true,
        verify_capabilities: true,
        ..Default::default()
    };

    let result = verify_package(Path::new("./package"), Some(config)).await?;
    
    if result.is_success() {
        println!("✅ Package verification passed");
        println!("{}", result.summary());
    } else {
        println!("❌ Package verification failed");
        for error in &result.errors {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}
```

## 📋 Manifest Schema

### Core Fields

```json
{
  "manifest_version": "1.0.0",
  "package_id": "core.kernel",
  "content_hash": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678",
  "created_at": "2024-01-15T10:30:00Z",
  "expires_at": "2025-01-15T10:30:00Z",
  "capabilities": { ... },
  "sbom": { ... },
  "signatures": [ ... ],
  "metadata": { ... },
  "files": [ ... ],
  "dependencies": [ ... ],
  "security": { ... }
}
```

### Capability Manifest

```json
{
  "capabilities": {
    "required": [
      {
        "name": "system.access",
        "type": "system",
        "description": "Access to system resources",
        "version": "1.0.0",
        "parameters": {
          "max_memory": "1GB",
          "allowed_ports": [80, 443]
        },
        "constraints": ["no_root_access"]
      }
    ],
    "optional": [
      {
        "name": "network.https",
        "type": "network",
        "description": "HTTPS network access",
        "version": "1.0.0"
      }
    ],
    "restricted": [
      {
        "name": "system.admin",
        "type": "system",
        "description": "Administrative access",
        "version": "1.0.0"
      }
    ]
  }
}
```

### SBOM (Software Bill of Materials)

```json
{
  "sbom": {
    "format": "spdx",
    "version": "2.3",
    "components": [
      {
        "name": "my-service",
        "version": "1.0.0",
        "type": "application",
        "purl": "pkg:polymera/my-service@1.0.0",
        "license": "MIT",
        "supplier": "Developer",
        "checksums": [
          {
            "algorithm": "sha256",
            "value": "component_hash_here"
          }
        ]
      }
    ],
    "relationships": [
      {
        "from": "my-service",
        "to": "polymera-os",
        "type": "depends_on",
        "description": "Depends on Polymera OS runtime"
      }
    ]
  }
}
```

### Digital Signatures

```json
{
  "signatures": [
    {
      "algorithm": "ed25519",
      "key_id": "developer-key-2024",
      "signature": "base64_encoded_signature",
      "timestamp": "2024-01-15T10:30:00Z",
      "signer": "Developer Name",
      "certificate": "base64_encoded_certificate"
    }
  ]
}
```

## 🛡️ Security Features

### Content Integrity

- **SHA-256 Hashing**: All package content is hashed for integrity
- **File Checksums**: Individual file verification
- **Manifest Signing**: Cryptographic signatures prevent tampering

### Capability Security

- **Required Capabilities**: Must be available for execution
- **Optional Capabilities**: Enhance functionality if available
- **Restricted Capabilities**: Explicitly denied access
- **Parameter Constraints**: Fine-grained permission control

### SBOM Security

- **Component Tracking**: Complete dependency visibility
- **Vulnerability Scanning**: Known security issues identification
- **License Compliance**: Open source license tracking
- **Audit Trail**: Security review information

## 🔍 Package Verification

### Verification Levels

```rust
let config = VerificationConfig {
    // Content verification
    verify_content_hash: true,      // Verify package integrity
    verify_file_checksums: true,    // Verify individual files
    verify_file_sizes: true,        // Verify file sizes
    
    // Security verification
    verify_signatures: true,        // Verify digital signatures
    verify_capabilities: true,      // Verify capability requirements
    verify_dependencies: true,      // Verify dependency constraints
    verify_sbom: true,             // Verify SBOM integrity
    verify_security: true,          // Verify security information
    
    // Expiration checking
    check_expiration: true,         // Check package expiration
    expiration_warning_days: 30,    // Warning threshold
    
    // Trust configuration
    strict_mode: false,             // Fail on warnings
    trusted_key_ids: vec![         // Trusted signing keys
        "official-key-2024".to_string(),
        "developer-key-2024".to_string()
    ],
    trusted_signers: vec![          // Trusted signers
        "Polymera OS Team".to_string(),
        "Verified Developer".to_string()
    ],
};
```

### Verification Results

```rust
let result = verify_package(&package_dir, Some(config)).await?;

println!("Verification: {}", result.summary());
println!("Success: {}", result.is_success());
println!("Errors: {}", result.errors.len());
println!("Warnings: {}", result.warnings.len());

// Access verification details
if let Some(files_verified) = result.details.get("files_verified") {
    println!("Files verified: {}", files_verified);
}

if let Some(content_hash_verified) = result.details.get("content_hash_verified") {
    println!("Content hash verified: {}", content_hash_verified);
}
```

## 🧪 Testing

### Building Test Packages

```rust
use polymera_pack::create_test_package;

#[tokio::test]
async fn test_package_building() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let output_dir = temp_dir.path().join("output");
    
    // Create test source files
    fs::create_dir(&source_dir)?;
    fs::write(source_dir.join("main.rs"), "fn main() {}")?;
    fs::write(source_dir.join("Cargo.toml"), "[package]")?;
    
    // Build test package
    let manifest = create_test_package(&source_dir, &output_dir).await?;
    assert_eq!(manifest.package_id, "test.package");
    
    Ok(())
}
```

### Testing Verification Failure

```rust
use polymera_pack::corrupt_package_for_testing;

#[tokio::test]
async fn test_verification_failure() -> Result<(), Box<dyn std::error::Error>> {
    // Build a valid package
    let manifest = create_test_package(&source_dir, &output_dir).await?;
    
    // Verify it passes
    let result = verify_package(&package_dir, None).await?;
    assert!(result.is_success());
    
    // Corrupt the package
    corrupt_package_for_testing(&package_dir).await?;
    
    // Verify it now fails
    let result = verify_package(&package_dir, None).await?;
    assert!(!result.is_success());
    assert!(result.errors.iter().any(|e| e.contains("Content hash mismatch")));
    
    Ok(())
}
```

## 🔧 Advanced Configuration

### Custom Capabilities

```rust
let mut builder = PackageBuilder::new(config);

// Add required capabilities
builder.add_capability(Capability {
    name: "crypto.encryption".to_string(),
    capability_type: "crypto".to_string(),
    description: "Access to encryption functions".to_string(),
    version: Some("2.0.0".to_string()),
    parameters: Some({
        let mut params = HashMap::new();
        params.insert("algorithms".to_string(), serde_json::Value::Array(vec![
            serde_json::Value::String("AES-256".to_string()),
            serde_json::Value::String("ChaCha20".to_string()),
        ]));
        params.insert("key_size".to_string(), serde_json::Value::Number(256.into()));
        params
    }),
    constraints: Some(vec!["no_export".to_string()]),
});

// Add dependencies
builder.add_dependency(PackageDependency {
    package_id: "crypto.liboqs".to_string(),
    version_constraint: ">=1.0.0,<2.0.0".to_string(),
    optional: Some(false),
    reason: Some("Required for post-quantum cryptography".to_string()),
});
```

### Custom Metadata

```rust
builder.add_metadata("build_timestamp".to_string(), 
    serde_json::Value::String(Utc::now().to_rfc3339()));

builder.add_metadata("build_environment".to_string(), 
    serde_json::Value::String("production".to_string()));

builder.add_metadata("ci_pipeline".to_string(), 
    serde_json::Value::String("github-actions".to_string()));
```

### File Filtering

```rust
let config = PackageBuilderConfig {
    // Exclude patterns
    exclude_patterns: vec![
        "*.tmp".to_string(),
        "*.log".to_string(),
        ".git".to_string(),
        "target".to_string(),
        "node_modules".to_string(),
        "*.swp".to_string(),
    ],
    
    // Include patterns
    include_patterns: vec![
        "*.rs".to_string(),
        "*.toml".to_string(),
        "*.md".to_string(),
        "LICENSE".to_string(),
    ],
    
    // File size limits
    max_file_size: Some(10 * 1024 * 1024), // 10MB
    
    // Hidden files
    include_hidden: false,
    
    ..Default::default()
};
```

## 📊 SBOM Formats

### SPDX (Software Package Data Exchange)

```json
{
  "sbom": {
    "format": "spdx",
    "version": "2.3",
    "components": [
      {
        "name": "my-service",
        "version": "1.0.0",
        "type": "application",
        "purl": "pkg:polymera/my-service@1.0.0",
        "license": "MIT",
        "supplier": "Developer"
      }
    ]
  }
}
```

### CycloneDX

```json
{
  "sbom": {
    "format": "cyclonedx",
    "version": "1.5",
    "components": [
      {
        "name": "my-service",
        "version": "1.0.0",
        "type": "application",
        "purl": "pkg:polymera/my-service@1.0.0",
        "license": "MIT",
        "supplier": "Developer"
      }
    ]
  }
}
```

### SWID (Software Identification)

```json
{
  "sbom": {
    "format": "swid",
    "version": "2.0",
    "components": [
      {
        "name": "my-service",
        "version": "1.0.0",
        "type": "application",
        "purl": "pkg:polymera/my-service@1.0.0",
        "license": "MIT",
        "supplier": "Developer"
      }
    ]
  }
}
```

## 🔐 Cryptographic Signatures

### Supported Algorithms

- **Ed25519**: Fast, secure elliptic curve signatures
- **RSA-2048**: Traditional RSA with SHA-256
- **RSA-4096**: High-security RSA with SHA-512
- **ECDSA-P256**: NIST P-256 curve signatures
- **ECDSA-P384**: NIST P-384 curve signatures

### Signature Verification

```rust
// Verify signatures
if self.verify_signatures {
    for signature in &manifest.signatures {
        // Check signature format
        if signature.signature.is_empty() {
            result.add_error("Empty signature".to_string());
            continue;
        }
        
        // Verify timestamp
        if let Ok(timestamp) = DateTime::parse_from_rfc3339(&signature.timestamp) {
            let age = Utc::now().signed_duration_since(timestamp);
            if age > Duration::days(365) {
                result.add_error("Signature too old".to_string());
            }
        }
        
        // Check trusted keys
        if !self.config.trusted_key_ids.is_empty() {
            if !self.config.trusted_key_ids.contains(&signature.key_id) {
                result.add_warning("Untrusted key ID".to_string());
            }
        }
    }
}
```

## 🚨 Security Considerations

### Package Integrity

- **Content Hashing**: SHA-256 ensures package immutability
- **File Verification**: Individual file checksums prevent tampering
- **Manifest Signing**: Cryptographic signatures prevent forgery

### Capability Isolation

- **Required Capabilities**: Explicit permission requirements
- **Restricted Capabilities**: Explicit access denials
- **Parameter Constraints**: Fine-grained permission control

### SBOM Security

- **Component Tracking**: Complete dependency visibility
- **Vulnerability Scanning**: Known security issues identification
- **License Compliance**: Open source license tracking

### Trust Management

- **Trusted Keys**: Only accept signatures from known keys
- **Trusted Signers**: Only accept packages from known developers
- **Expiration Checking**: Reject expired packages

## 🔍 Troubleshooting

### Common Issues

1. **Content Hash Mismatch**
   - Check if files were modified after packaging
   - Verify manifest.json integrity
   - Rebuild package from source

2. **Signature Verification Failed**
   - Check signing key validity
   - Verify signature timestamp
   - Confirm trusted key configuration

3. **Capability Check Failed**
   - Review capability requirements
   - Check for capability conflicts
   - Verify capability descriptions

4. **SBOM Validation Failed**
   - Check SBOM format compliance
   - Verify component information
   - Review relationship definitions

### Debug Mode

```rust
let config = VerificationConfig {
    strict_mode: false,  // Allow warnings
    ..Default::default()
};

let result = verify_package(&package_dir, Some(config)).await?;

// Print detailed verification information
println!("Verification Details:");
for (key, value) in &result.details {
    println!("  {}: {}", key, value);
}

// Print warnings
if !result.warnings.is_empty() {
    println!("Warnings:");
    for warning in &result.warnings {
        println!("  - {}", warning);
    }
}
```

## 📚 API Reference

### Core Types

- `PackageManifest` - Main package manifest structure
- `Capabilities` - Required, optional, and restricted capabilities
- `Sbom` - Software Bill of Materials
- `Signature` - Cryptographic signature information
- `PackageFile` - Package file metadata

### Builder Types

- `PackageBuilder` - Package creation interface
- `PackageBuilderConfig` - Builder configuration
- `PackageBuildError` - Building error types

### Verifier Types

- `PackageVerifier` - Package verification interface
- `VerificationConfig` - Verification configuration
- `VerificationResult` - Verification results
- `PackageVerifyError` - Verification error types

### Functions

- `build_package()` - Build package from configuration
- `verify_package()` - Verify package integrity
- `create_test_package()` - Create test package
- `corrupt_package_for_testing()` - Corrupt package for testing

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

MIT OR Apache-2.0

## 🆘 Support

- **Documentation**: [Polymera OS Docs](https://docs.polymera-os.org)
- **Issues**: [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- **Discord**: [Polymera OS Community](https://discord.gg/polymera-os)
