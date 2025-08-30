use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Main package manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Version of the manifest schema
    pub manifest_version: String,
    /// Unique package identifier
    pub package_id: String,
    /// SHA-256 hash of package content (excluding signatures)
    pub content_hash: String,
    /// ISO 8601 timestamp when package was created
    pub created_at: String,
    /// Optional expiration timestamp
    pub expires_at: Option<String>,
    /// Capability manifest
    pub capabilities: Capabilities,
    /// Software Bill of Materials
    pub sbom: Sbom,
    /// Cryptographic signatures
    pub signatures: Vec<Signature>,
    /// Package metadata
    pub metadata: Metadata,
    /// Package files
    pub files: Vec<PackageFile>,
    /// Package dependencies
    pub dependencies: Vec<PackageDependency>,
    /// Security information
    pub security: Option<Security>,
}

/// Capability manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Required capabilities
    pub required: Vec<Capability>,
    /// Optional capabilities
    pub optional: Vec<Capability>,
    /// Restricted capabilities
    pub restricted: Vec<Capability>,
}

/// Individual capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability type
    pub capability_type: String,
    /// Human-readable description
    pub description: String,
    /// Required version
    pub version: Option<String>,
    /// Capability parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    /// Additional constraints
    pub constraints: Option<Vec<String>>,
}

/// Software Bill of Materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// SBOM format
    pub format: String,
    /// Format version
    pub version: String,
    /// Software components
    pub components: Vec<SbomComponent>,
    /// Component relationships
    pub relationships: Option<Vec<SbomRelationship>>,
}

impl Default for Sbom {
    fn default() -> Self {
        Self {
            format: "spdx".to_string(),
            version: "2.3".to_string(),
            components: Vec::new(),
            relationships: None,
        }
    }
}

/// SBOM component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomComponent {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Component type
    pub component_type: String,
    /// Package URL
    pub purl: Option<String>,
    /// Common Platform Enumeration
    pub cpe: Option<String>,
    /// License
    pub license: Option<String>,
    /// Supplier
    pub supplier: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Checksums
    pub checksums: Vec<Checksum>,
}

/// SBOM relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomRelationship {
    /// Source component
    pub from: String,
    /// Target component
    pub to: String,
    /// Relationship type
    pub relationship_type: String,
    /// Description
    pub description: Option<String>,
}

/// Cryptographic signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature algorithm
    pub algorithm: String,
    /// Key identifier
    pub key_id: String,
    /// Base64-encoded signature
    pub signature: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Signer name
    pub signer: Option<String>,
    /// Base64-encoded certificate
    pub certificate: Option<String>,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Package name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Version
    pub version: Option<String>,
    /// Author
    pub author: Option<String>,
    /// License
    pub license: Option<String>,
    /// Homepage
    pub homepage: Option<String>,
    /// Repository
    pub repository: Option<String>,
    /// Keywords
    pub keywords: Option<Vec<String>>,
    /// Tags
    pub tags: Option<HashMap<String, serde_json::Value>>,
}

/// Package file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    /// File path within package
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// File checksum
    pub checksum: Checksum,
    /// Unix file permissions
    pub permissions: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Whether file is executable
    pub executable: Option<bool>,
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    /// Dependent package ID
    pub package_id: String,
    /// Version constraint
    pub version_constraint: String,
    /// Whether dependency is optional
    pub optional: Option<bool>,
    /// Reason for dependency
    pub reason: Option<String>,
}

/// Checksum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    /// Hash algorithm
    pub algorithm: String,
    /// Hexadecimal hash value
    pub value: String,
}

/// Security information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    /// Known vulnerabilities
    pub vulnerabilities: Option<Vec<Vulnerability>>,
    /// Additional checksums
    pub checksums: Option<HashMap<String, String>>,
    /// Audit information
    pub audit_info: Option<AuditInfo>,
}

/// Security vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Vulnerability identifier
    pub id: String,
    /// Severity level
    pub severity: String,
    /// Description
    pub description: String,
    /// Affected components
    pub affected_components: Option<Vec<String>>,
    /// References
    pub references: Option<Vec<String>>,
}

/// Security audit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    /// Audit timestamp
    pub audited_at: Option<String>,
    /// Auditor name
    pub auditor: Option<String>,
    /// Audit report URL
    pub audit_report: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manifest_serialization() {
        let manifest = PackageManifest {
            manifest_version: "1.0.0".to_string(),
            package_id: "test.package".to_string(),
            content_hash: "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            expires_at: None,
            capabilities: Capabilities {
                required: vec![
                    Capability {
                        name: "system.access".to_string(),
                        capability_type: "system".to_string(),
                        description: "Access to system resources".to_string(),
                        version: None,
                        parameters: None,
                        constraints: None,
                    }
                ],
                optional: Vec::new(),
                restricted: Vec::new(),
            },
            sbom: Sbom::default(),
            signatures: vec![
                Signature {
                    algorithm: "ed25519".to_string(),
                    key_id: "test-key".to_string(),
                    signature: "test-signature".to_string(),
                    timestamp: "2024-01-15T10:30:00Z".to_string(),
                    signer: Some("test-signer".to_string()),
                    certificate: None,
                }
            ],
            metadata: Metadata {
                name: Some("Test Package".to_string()),
                description: Some("A test package".to_string()),
                version: Some("1.0.0".to_string()),
                author: Some("Test Author".to_string()),
                license: Some("MIT".to_string()),
                homepage: None,
                repository: None,
                keywords: Some(vec!["test".to_string()]),
                tags: Some(HashMap::new()),
            },
            files: vec![
                PackageFile {
                    path: "src/main.rs".to_string(),
                    size: 1024,
                    checksum: Checksum {
                        algorithm: "sha256".to_string(),
                        value: "test-checksum".to_string(),
                    },
                    permissions: Some("rw-r--r--".to_string()),
                    mime_type: Some("text/x-rust".to_string()),
                    executable: Some(false),
                }
            ],
            dependencies: Vec::new(),
            security: None,
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let deserialized: PackageManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.package_id, "test.package");
        assert_eq!(deserialized.manifest_version, "1.0.0");
        assert_eq!(deserialized.signatures.len(), 1);
        assert_eq!(deserialized.files.len(), 1);
    }

    #[test]
    fn test_capability_serialization() {
        let capability = Capability {
            name: "network.access".to_string(),
            capability_type: "network".to_string(),
            description: "Access to network resources".to_string(),
            version: Some("1.0.0".to_string()),
            parameters: Some({
                let mut params = HashMap::new();
                params.insert("ports".to_string(), serde_json::Value::Array(vec![
                    serde_json::Value::Number(80.into()),
                    serde_json::Value::Number(443.into()),
                ]));
                params
            }),
            constraints: Some(vec!["localhost_only".to_string()]),
        };

        let json = serde_json::to_string(&capability).unwrap();
        let deserialized: Capability = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "network.access");
        assert_eq!(deserialized.capability_type, "network");
        assert!(deserialized.parameters.is_some());
        assert!(deserialized.constraints.is_some());
    }

    #[test]
    fn test_checksum_serialization() {
        let checksum = Checksum {
            algorithm: "sha256".to_string(),
            value: "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678".to_string(),
        };

        let json = serde_json::to_string(&checksum).unwrap();
        let deserialized: Checksum = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.algorithm, "sha256");
        assert_eq!(deserialized.value, "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678");
    }
}
