use cosmwasm_std::StdError;
use thiserror::Error;

/// Custom error types for the attestation registry
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Attestation not found: {id}")]
    AttestationNotFound { id: u64 },

    #[error("Schema not found: {id}")]
    SchemaNotFound { id: String },

    #[error("Issuer not found: {address}")]
    IssuerNotFound { address: String },

    #[error("Attestation already revoked: {id}")]
    AttestationAlreadyRevoked { id: u64 },

    #[error("Attestation expired: {id}")]
    AttestationExpired { id: u64 },

    #[error("Attestation data too large: {size} bytes (max: {max} bytes)")]
    AttestationDataTooLarge { size: usize, max: usize },

    #[error("Too many tags: {count} (max: {max})")]
    TooManyTags { count: usize, max: usize },

    #[error("Invalid attestation lifetime: {reason}")]
    InvalidAttestationLifetime { reason: String },

    #[error("Issuer not authorized for schema: {issuer} -> {schema}")]
    IssuerNotAuthorizedForSchema { issuer: String, schema: String },

    #[error("Issuer already registered: {address}")]
    IssuerAlreadyRegistered { address: String },

    #[error("Schema already exists: {id}")]
    SchemaAlreadyExists { id: String },

    #[error("Invalid schema: {reason}")]
    InvalidSchema { reason: String },

    #[error("Invalid issuer: {reason}")]
    InvalidIssuer { reason: String },

    #[error("Invalid attestation: {reason}")]
    InvalidAttestation { reason: String },

    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration { reason: String },

    #[error("Invalid pagination: {reason}")]
    InvalidPagination { reason: String },

    #[error("Invalid search parameters: {reason}")]
    InvalidSearchParameters { reason: String },

    #[error("Storage error: {reason}")]
    StorageError { reason: String },

    #[error("Query error: {reason}")]
    QueryError { reason: String },

    #[error("Validation error: {reason}")]
    ValidationError { reason: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("Deserialization error: {reason}")]
    DeserializationError { reason: String },

    #[error("Internal error: {reason}")]
    InternalError { reason: String },

    #[error("Feature not implemented: {feature}")]
    FeatureNotImplemented { feature: String },

    #[error("Rate limit exceeded: {reason}")]
    RateLimitExceeded { reason: String },

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },

    #[error("Contract paused: {reason}")]
    ContractPaused { reason: String },

    #[error("Migration error: {reason}")]
    MigrationError { reason: String },

    #[error("Upgrade error: {reason}")]
    UpgradeError { reason: String },
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        StdError::generic_err(err.to_string())
    }
}

impl ContractError {
    /// Create an unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized { message: message.into() }
    }

    /// Create an attestation not found error
    pub fn attestation_not_found(id: u64) -> Self {
        Self::AttestationNotFound { id }
    }

    /// Create a schema not found error
    pub fn schema_not_found(id: impl Into<String>) -> Self {
        Self::SchemaNotFound { id: id.into() }
    }

    /// Create an issuer not found error
    pub fn issuer_not_found(address: impl Into<String>) -> Self {
        Self::IssuerNotFound { address: address.into() }
    }

    /// Create an attestation already revoked error
    pub fn attestation_already_revoked(id: u64) -> Self {
        Self::AttestationAlreadyRevoked { id }
    }

    /// Create an attestation expired error
    pub fn attestation_expired(id: u64) -> Self {
        Self::AttestationExpired { id }
    }

    /// Create an attestation data too large error
    pub fn attestation_data_too_large(size: usize, max: usize) -> Self {
        Self::AttestationDataTooLarge { size, max }
    }

    /// Create a too many tags error
    pub fn too_many_tags(count: usize, max: usize) -> Self {
        Self::TooManyTags { count, max }
    }

    /// Create an invalid attestation lifetime error
    pub fn invalid_attestation_lifetime(reason: impl Into<String>) -> Self {
        Self::InvalidAttestationLifetime { reason: reason.into() }
    }

    /// Create an issuer not authorized for schema error
    pub fn issuer_not_authorized_for_schema(issuer: impl Into<String>, schema: impl Into<String>) -> Self {
        Self::IssuerNotAuthorizedForSchema { 
            issuer: issuer.into(), 
            schema: schema.into() 
        }
    }

    /// Create an issuer already registered error
    pub fn issuer_already_registered(address: impl Into<String>) -> Self {
        Self::IssuerAlreadyRegistered { address: address.into() }
    }

    /// Create a schema already exists error
    pub fn schema_already_exists(id: impl Into<String>) -> Self {
        Self::SchemaAlreadyExists { id: id.into() }
    }

    /// Create an invalid schema error
    pub fn invalid_schema(reason: impl Into<String>) -> Self {
        Self::InvalidSchema { reason: reason.into() }
    }

    /// Create an invalid issuer error
    pub fn invalid_issuer(reason: impl Into<String>) -> Self {
        Self::InvalidIssuer { reason: reason.into() }
    }

    /// Create an invalid attestation error
    pub fn invalid_attestation(reason: impl Into<String>) -> Self {
        Self::InvalidAttestation { reason: reason.into() }
    }

    /// Create an invalid configuration error
    pub fn invalid_configuration(reason: impl Into<String>) -> Self {
        Self::InvalidConfiguration { reason: reason.into() }
    }

    /// Create an invalid pagination error
    pub fn invalid_pagination(reason: impl Into<String>) -> Self {
        Self::InvalidPagination { reason: reason.into() }
    }

    /// Create an invalid search parameters error
    pub fn invalid_search_parameters(reason: impl Into<String>) -> Self {
        Self::InvalidSearchParameters { reason: reason.into() }
    }

    /// Create a storage error
    pub fn storage_error(reason: impl Into<String>) -> Self {
        Self::StorageError { reason: reason.into() }
    }

    /// Create a query error
    pub fn query_error(reason: impl Into<String>) -> Self {
        Self::QueryError { reason: reason.into() }
    }

    /// Create a validation error
    pub fn validation_error(reason: impl Into<String>) -> Self {
        Self::ValidationError { reason: reason.into() }
    }

    /// Create a serialization error
    pub fn serialization_error(reason: impl Into<String>) -> Self {
        Self::SerializationError { reason: reason.into() }
    }

    /// Create a deserialization error
    pub fn deserialization_error(reason: impl Into<String>) -> Self {
        Self::DeserializationError { reason: reason.into() }
    }

    /// Create an internal error
    pub fn internal_error(reason: impl Into<String>) -> Self {
        Self::InternalError { reason: reason.into() }
    }

    /// Create a feature not implemented error
    pub fn feature_not_implemented(feature: impl Into<String>) -> Self {
        Self::FeatureNotImplemented { feature: feature.into() }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(reason: impl Into<String>) -> Self {
        Self::RateLimitExceeded { reason: reason.into() }
    }

    /// Create an insufficient funds error
    pub fn insufficient_funds(required: impl Into<String>, available: impl Into<String>) -> Self {
        Self::InsufficientFunds { 
            required: required.into(), 
            available: available.into() 
        }
    }

    /// Create a contract paused error
    pub fn contract_paused(reason: impl Into<String>) -> Self {
        Self::ContractPaused { reason: reason.into() }
    }

    /// Create a migration error
    pub fn migration_error(reason: impl Into<String>) -> Self {
        Self::MigrationError { reason: reason.into() }
    }

    /// Create an upgrade error
    pub fn upgrade_error(reason: impl Into<String>) -> Self {
        Self::UpgradeError { reason: reason.into() }
    }
}

// ============ VALIDATION HELPERS ============

/// Validation result type
pub type ValidationResult<T> = Result<T, ContractError>;

/// Validate that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> ValidationResult<()> {
    if value.trim().is_empty() {
        return Err(ContractError::validation_error(
            format!("{} cannot be empty", field_name)
        ));
    }
    Ok(())
}

/// Validate that a string has a maximum length
pub fn validate_max_length(value: &str, field_name: &str, max_length: usize) -> ValidationResult<()> {
    if value.len() > max_length {
        return Err(ContractError::validation_error(
            format!("{} too long: {} characters (max: {})", field_name, value.len(), max_length)
        ));
    }
    Ok(())
}

/// Validate that a string has a minimum length
pub fn validate_min_length(value: &str, field_name: &str, min_length: usize) -> ValidationResult<()> {
    if value.len() < min_length {
        return Err(ContractError::validation_error(
            format!("{} too short: {} characters (min: {})", field_name, value.len(), min_length)
        ));
    }
    Ok(())
}

/// Validate that a string matches a pattern
pub fn validate_pattern(value: &str, field_name: &str, pattern: &str) -> ValidationResult<()> {
    if !value.matches(pattern).next().is_some() {
        return Err(ContractError::validation_error(
            format!("{} does not match pattern: {}", field_name, pattern)
        ));
    }
    Ok(())
}

/// Validate that a number is within a range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    field_name: &str,
    min: T,
    max: T
) -> ValidationResult<()> {
    if value < min || value > max {
        return Err(ContractError::validation_error(
            format!("{} out of range: {} (min: {}, max: {})", field_name, value, min, max)
        ));
    }
    Ok(())
}

/// Validate that a number is greater than a minimum
pub fn validate_min<T: PartialOrd + std::fmt::Display>(
    value: T,
    field_name: &str,
    min: T
) -> ValidationResult<()> {
    if value < min {
        return Err(ContractError::validation_error(
            format!("{} too small: {} (min: {})", field_name, value, min)
        ));
    }
    Ok(())
}

/// Validate that a number is less than a maximum
pub fn validate_max<T: PartialOrd + std::fmt::Display>(
    value: T,
    field_name: &str,
    max: T
) -> ValidationResult<()> {
    if value > max {
        return Err(ContractError::validation_error(
            format!("{} too large: {} (max: {})", field_name, value, max)
        ));
    }
    Ok(())
}

/// Validate that a vector has a maximum length
pub fn validate_vec_max_length<T>(
    vec: &[T],
    field_name: &str,
    max_length: usize
) -> ValidationResult<()> {
    if vec.len() > max_length {
        return Err(ContractError::validation_error(
            format!("{} too many items: {} (max: {})", field_name, vec.len(), max_length)
        ));
    }
    Ok(())
}

/// Validate that a vector has a minimum length
pub fn validate_vec_min_length<T>(
    vec: &[T],
    field_name: &str,
    min_length: usize
) -> ValidationResult<()> {
    if vec.len() < min_length {
        return Err(ContractError::validation_error(
            format!("{} too few items: {} (min: {})", field_name, vec.len(), min_length)
        ));
    }
    Ok(())
}

/// Validate that a vector has no duplicate items
pub fn validate_vec_no_duplicates<T: std::hash::Hash + Eq>(
    vec: &[T],
    field_name: &str
) -> ValidationResult<()> {
    let mut seen = std::collections::HashSet::new();
    for item in vec {
        if !seen.insert(item) {
            return Err(ContractError::validation_error(
                format!("{} contains duplicate items", field_name)
            ));
        }
    }
    Ok(())
}

/// Validate that a pagination limit is reasonable
pub fn validate_pagination_limit(limit: Option<u32>, max_limit: u32) -> ValidationResult<u32> {
    let limit = limit.unwrap_or(30);
    if limit > max_limit {
        return Err(ContractError::invalid_pagination(
            format!("Limit too large: {} (max: {})", limit, max_limit)
        ));
    }
    if limit == 0 {
        return Err(ContractError::invalid_pagination("Limit cannot be zero"));
    }
    Ok(limit)
}

/// Validate that a timestamp is in the future
pub fn validate_future_timestamp(
    timestamp: cosmwasm_std::Timestamp,
    field_name: &str,
    current_time: cosmwasm_std::Timestamp
) -> ValidationResult<()> {
    if timestamp <= current_time {
        return Err(ContractError::validation_error(
            format!("{} must be in the future", field_name)
        ));
    }
    Ok(())
}

/// Validate that a timestamp is in the past
pub fn validate_past_timestamp(
    timestamp: cosmwasm_std::Timestamp,
    field_name: &str,
    current_time: cosmwasm_std::Timestamp
) -> ValidationResult<()> {
    if timestamp >= current_time {
        return Err(ContractError::validation_error(
            format!("{} must be in the past", field_name)
        ));
    }
    Ok(())
}
