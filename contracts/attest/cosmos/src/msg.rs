use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128, Timestamp};

/// Instantiate message for the attestation registry
#[cw_serde]
pub struct InstantiateMsg {
    /// Registry name
    pub name: String,
    /// Registry description
    pub description: String,
    /// Registry URI for metadata
    pub uri: String,
    /// Minimum attestation lifetime in seconds
    pub min_attestation_lifetime: u64,
    /// Maximum attestation lifetime in seconds
    pub max_attestation_lifetime: u64,
    /// Attestation fee amount
    pub attestation_fee: Uint128,
    /// Whether attestation fee is enabled
    pub attestation_fee_enabled: bool,
    /// Maximum tags per attestation
    pub max_tags_per_attestation: u32,
    /// Maximum attestation data size in bytes
    pub max_attestation_data_size: u32,
    /// Admin address
    pub admin: String,
}

/// Execute messages for the attestation registry
#[cw_serde]
pub enum ExecuteMsg {
    /// Issue a new attestation
    IssueAttestation {
        /// Subject address being attested
        subject: String,
        /// Schema identifier
        schema_id: String,
        /// Attestation data (encoded)
        data: Binary,
        /// Expiration timestamp (0 = never)
        expires_at: Option<Timestamp>,
        /// URI to additional metadata
        uri: String,
        /// Tags for categorization
        tags: Vec<String>,
    },
    
    /// Revoke an attestation
    RevokeAttestation {
        /// Attestation identifier
        attestation_id: u64,
        /// Reason for revocation
        reason: String,
    },
    
    /// Update attestation data
    UpdateAttestation {
        /// Attestation identifier
        attestation_id: u64,
        /// New attestation data
        data: Binary,
        /// New URI
        uri: String,
        /// New tags
        tags: Vec<String>,
    },
    
    /// Create a new schema
    CreateSchema {
        /// Schema name
        name: String,
        /// Schema description
        description: String,
        /// Field identifiers
        fields: Vec<String>,
        /// Field type definitions
        field_types: Vec<String>,
        /// Schema version
        version: String,
    },
    
    /// Deprecate a schema
    DeprecateSchema {
        /// Schema identifier
        schema_id: String,
        /// Reason for deprecation
        reason: String,
    },
    
    /// Register a new issuer
    RegisterIssuer {
        /// Issuer name
        name: String,
        /// Issuer description
        description: String,
        /// URI to issuer metadata
        uri: String,
    },
    
    /// Update issuer information
    UpdateIssuer {
        /// New issuer name
        name: String,
        /// New issuer description
        description: String,
        /// New URI
        uri: String,
    },
    
    /// Deactivate an issuer
    DeactivateIssuer {
        /// Reason for deactivation
        reason: String,
    },
    
    /// Authorize issuer for schema
    AuthorizeIssuerForSchema {
        /// Issuer address
        issuer: String,
        /// Schema identifier
        schema_id: String,
    },
    
    /// Revoke issuer schema authorization
    RevokeIssuerSchemaAuthorization {
        /// Issuer address
        issuer: String,
        /// Schema identifier
        schema_id: String,
    },
    
    /// Update registry configuration
    UpdateConfig {
        /// New minimum attestation lifetime
        min_attestation_lifetime: Option<u64>,
        /// New maximum attestation lifetime
        max_attestation_lifetime: Option<u64>,
        /// New attestation fee
        attestation_fee: Option<Uint128>,
        /// Whether attestation fee is enabled
        attestation_fee_enabled: Option<bool>,
        /// New maximum tags per attestation
        max_tags_per_attestation: Option<u32>,
        /// New maximum attestation data size
        max_attestation_data_size: Option<u32>,
    },
    
    /// Withdraw collected fees
    WithdrawFees {
        /// Recipient address
        recipient: String,
    },
}

/// Query messages for the attestation registry
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get attestation by ID
    #[returns(AttestationResponse)]
    GetAttestation { attestation_id: u64 },
    
    /// Get attestations for a subject
    #[returns(AttestationsResponse)]
    GetSubjectAttestations { subject: String },
    
    /// Get attestations issued by an issuer
    #[returns(AttestationsResponse)]
    GetIssuerAttestations { issuer: String },
    
    /// Get attestations for a schema
    #[returns(AttestationsResponse)]
    GetSchemaAttestations { schema_id: String },
    
    /// Get schema by ID
    #[returns(SchemaResponse)]
    GetSchema { schema_id: String },
    
    /// Get issuer by address
    #[returns(IssuerResponse)]
    GetIssuer { issuer: String },
    
    /// Check if attestation is valid
    #[returns(AttestationValidityResponse)]
    IsAttestationValid { attestation_id: u64 },
    
    /// Get registry configuration
    #[returns(ConfigResponse)]
    GetConfig {},
    
    /// Get total counts
    #[returns(CountsResponse)]
    GetCounts {},
    
    /// List all schemas
    #[returns(SchemasResponse)]
    ListSchemas { start_after: Option<String>, limit: Option<u32> },
    
    /// List all issuers
    #[returns(IssuersResponse)]
    ListIssuers { start_after: Option<String>, limit: Option<u32> },
    
    /// List all attestations
    #[returns(AttestationsResponse)]
    ListAttestations { start_after: Option<u64>, limit: Option<u32> },
    
    /// Search attestations by tags
    #[returns(AttestationsResponse)]
    SearchAttestationsByTags { tags: Vec<String>, limit: Option<u32> },
    
    /// Search attestations by issuer
    #[returns(AttestationsResponse)]
    SearchAttestationsByIssuer { issuer: String, limit: Option<u32> },
    
    /// Search attestations by schema
    #[returns(AttestationsResponse)]
    SearchAttestationsBySchema { schema_id: String, limit: Option<u32> },
}

// ============ RESPONSE STRUCTS ============

/// Response for attestation queries
#[cw_serde]
pub struct AttestationResponse {
    /// Attestation data
    pub attestation: Option<Attestation>,
}

/// Response for multiple attestations
#[cw_serde]
pub struct AttestationsResponse {
    /// List of attestations
    pub attestations: Vec<Attestation>,
    /// Total count
    pub total: u64,
}

/// Response for schema queries
#[cw_serde]
pub struct SchemaResponse {
    /// Schema data
    pub schema: Option<Schema>,
}

/// Response for multiple schemas
#[cw_serde]
pub struct SchemasResponse {
    /// List of schemas
    pub schemas: Vec<Schema>,
    /// Total count
    pub total: u64,
}

/// Response for issuer queries
#[cw_serde]
pub struct IssuerResponse {
    /// Issuer data
    pub issuer: Option<Issuer>,
}

/// Response for multiple issuers
#[cw_serde]
pub struct IssuersResponse {
    /// List of issuers
    pub issuers: Vec<Issuer>,
    /// Total count
    pub total: u64,
}

/// Response for attestation validity check
#[cw_serde]
pub struct AttestationValidityResponse {
    /// Whether attestation is valid
    pub is_valid: bool,
    /// Reason if invalid
    pub reason: Option<String>,
}

/// Response for configuration queries
#[cw_serde]
pub struct ConfigResponse {
    /// Registry configuration
    pub config: Config,
}

/// Response for count queries
#[cw_serde]
pub struct CountsResponse {
    /// Total attestation count
    pub total_attestations: u64,
    /// Total schema count
    pub total_schemas: u64,
    /// Total issuer count
    pub total_issuers: u64,
}

// ============ DATA STRUCTURES ============

/// Attestation data structure
#[cw_serde]
pub struct Attestation {
    /// Unique attestation ID
    pub id: u64,
    /// Address that issued the attestation
    pub issuer: String,
    /// Address being attested
    pub subject: String,
    /// Schema identifier for the attestation
    pub schema_id: String,
    /// Attestation data (encoded)
    pub data: Binary,
    /// Timestamp when issued
    pub issued_at: Timestamp,
    /// Timestamp when expires (None = never)
    pub expires_at: Option<Timestamp>,
    /// Whether attestation is revoked
    pub revoked: bool,
    /// Timestamp when revoked (None = not revoked)
    pub revoked_at: Option<Timestamp>,
    /// Address that revoked the attestation
    pub revoked_by: Option<String>,
    /// URI to additional metadata
    pub uri: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Version of the attestation
    pub version: u32,
}

/// Schema data structure
#[cw_serde]
pub struct Schema {
    /// Unique schema identifier
    pub id: String,
    /// Human-readable schema name
    pub name: String,
    /// Schema description
    pub description: String,
    /// Field identifiers
    pub fields: Vec<String>,
    /// Field type definitions
    pub field_types: Vec<String>,
    /// Whether schema is required
    pub required: bool,
    /// Timestamp when created
    pub created_at: Timestamp,
    /// Address that created the schema
    pub created_by: String,
    /// Whether schema is deprecated
    pub deprecated: bool,
    /// Timestamp when deprecated
    pub deprecated_at: Option<Timestamp>,
    /// Schema version
    pub version: String,
}

/// Issuer data structure
#[cw_serde]
pub struct Issuer {
    /// Issuer address
    pub addr: String,
    /// Human-readable name
    pub name: String,
    /// Issuer description
    pub description: String,
    /// URI to issuer metadata
    pub uri: String,
    /// Whether issuer is active
    pub active: bool,
    /// Timestamp when registered
    pub registered_at: Timestamp,
    /// Total attestations issued
    pub total_attestations: u64,
    /// Schemas issuer can use
    pub authorized_schemas: Vec<String>,
}

/// Registry configuration
#[cw_serde]
pub struct Config {
    /// Registry name
    pub name: String,
    /// Registry description
    pub description: String,
    /// Registry URI for metadata
    pub uri: String,
    /// Minimum attestation lifetime in seconds
    pub min_attestation_lifetime: u64,
    /// Maximum attestation lifetime in seconds
    pub max_attestation_lifetime: u64,
    /// Attestation fee amount
    pub attestation_fee: Uint128,
    /// Whether attestation fee is enabled
    pub attestation_fee_enabled: bool,
    /// Maximum tags per attestation
    pub max_tags_per_attestation: u32,
    /// Maximum attestation data size in bytes
    pub max_attestation_data_size: u32,
    /// Admin address
    pub admin: String,
    /// Registry creation timestamp
    pub created_at: Timestamp,
}

// ============ MIGRATION MESSAGES ============

/// Migration message for contract upgrades
#[cw_serde]
pub struct MigrateMsg {
    /// New version
    pub version: String,
    /// Migration data
    pub data: Option<Binary>,
}
