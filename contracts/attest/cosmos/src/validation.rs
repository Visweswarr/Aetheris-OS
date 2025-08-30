use crate::error::ContractError;
use crate::msg::InstantiateMsg;

// ============ INSTANTIATE VALIDATION ============

pub fn validate_instantiate_msg(msg: &InstantiateMsg) -> Result<(), ContractError> {
    // Validate name
    if msg.name.trim().is_empty() {
        return Err(ContractError::invalid_configuration("Registry name cannot be empty"));
    }
    
    if msg.name.len() > 100 {
        return Err(ContractError::invalid_configuration("Registry name too long (max 100 characters)"));
    }
    
    // Validate description
    if msg.description.len() > 500 {
        return Err(ContractError::invalid_configuration("Registry description too long (max 500 characters)"));
    }
    
    // Validate URI
    if !msg.uri.trim().is_empty() && !is_valid_uri(&msg.uri) {
        return Err(ContractError::invalid_configuration("Invalid registry URI format"));
    }
    
    // Validate attestation lifetime constraints
    if msg.min_attestation_lifetime > msg.max_attestation_lifetime {
        return Err(ContractError::invalid_configuration(
            "Minimum attestation lifetime cannot exceed maximum lifetime"
        ));
    }
    
    if msg.min_attestation_lifetime < 60 { // At least 1 minute
        return Err(ContractError::invalid_configuration(
            "Minimum attestation lifetime must be at least 60 seconds"
        ));
    }
    
    if msg.max_attestation_lifetime > 31536000 { // Max 1 year
        return Err(ContractError::invalid_configuration(
            "Maximum attestation lifetime cannot exceed 1 year (31536000 seconds)"
        ));
    }
    
    // Validate tags constraints
    if msg.max_tags_per_attestation < 1 || msg.max_tags_per_attestation > 100 {
        return Err(ContractError::invalid_configuration(
            "Maximum tags per attestation must be between 1 and 100"
        ));
    }
    
    // Validate data size constraints
    if msg.max_attestation_data_size < 64 || msg.max_attestation_data_size > 10240 {
        return Err(ContractError::invalid_configuration(
            "Maximum attestation data size must be between 64 and 10240 bytes"
        ));
    }
    
    // Validate admin address
    if msg.admin.trim().is_empty() {
        return Err(ContractError::invalid_configuration("Admin address cannot be empty"));
    }
    
    Ok(())
}

// ============ URI VALIDATION ============

fn is_valid_uri(uri: &str) -> bool {
    // Basic URI validation - check for common schemes
    uri.starts_with("http://") || 
    uri.starts_with("https://") || 
    uri.starts_with("ipfs://") || 
    uri.starts_with("ar://") || 
    uri.starts_with("data:")
}

// ============ ATTESTATION VALIDATION ============

pub fn validate_attestation_data(data: &[u8], max_size: u32) -> Result<(), ContractError> {
    if data.len() > max_size as usize {
        return Err(ContractError::attestation_data_too_large(data.len(), max_size as usize));
    }
    
    if data.is_empty() {
        return Err(ContractError::invalid_attestation("Attestation data cannot be empty"));
    }
    
    Ok(())
}

pub fn validate_attestation_tags(tags: &[String], max_tags: u32) -> Result<(), ContractError> {
    if tags.len() > max_tags as usize {
        return Err(ContractError::too_many_tags(tags.len(), max_tags as usize));
    }
    
    // Validate individual tags
    for tag in tags {
        if tag.trim().is_empty() {
            return Err(ContractError::invalid_attestation("Tag cannot be empty"));
        }
        
        if tag.len() > 50 {
            return Err(ContractError::invalid_attestation("Tag too long (max 50 characters)"));
        }
        
        // Check for invalid characters
        if tag.chars().any(|c| !c.is_alphanumeric() && c != '-' && c != '_') {
            return Err(ContractError::invalid_attestation("Tag contains invalid characters"));
        }
    }
    
    // Check for duplicate tags
    let mut unique_tags = std::collections::HashSet::new();
    for tag in tags {
        if !unique_tags.insert(tag) {
            return Err(ContractError::invalid_attestation("Duplicate tags not allowed"));
        }
    }
    
    Ok(())
}

pub fn validate_attestation_subject(subject: &str) -> Result<(), ContractError> {
    if subject.trim().is_empty() {
        return Err(ContractError::invalid_attestation("Subject cannot be empty"));
    }
    
    if subject.len() > 100 {
        return Err(ContractError::invalid_attestation("Subject too long (max 100 characters)"));
    }
    
    Ok(())
}

pub fn validate_attestation_uri(uri: &str) -> Result<(), ContractError> {
    if !uri.trim().is_empty() && !is_valid_uri(uri) {
        return Err(ContractError::invalid_attestation("Invalid URI format"));
    }
    
    if uri.len() > 500 {
        return Err(ContractError::invalid_attestation("URI too long (max 500 characters)"));
    }
    
    Ok(())
}

// ============ SCHEMA VALIDATION ============

pub fn validate_schema_name(name: &str) -> Result<(), ContractError> {
    if name.trim().is_empty() {
        return Err(ContractError::invalid_schema("Schema name cannot be empty"));
    }
    
    if name.len() > 100 {
        return Err(ContractError::invalid_schema("Schema name too long (max 100 characters)"));
    }
    
    // Check for invalid characters
    if name.chars().any(|c| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_') {
        return Err(ContractError::invalid_schema("Schema name contains invalid characters"));
    }
    
    Ok(())
}

pub fn validate_schema_description(description: &str) -> Result<(), ContractError> {
    if description.len() > 500 {
        return Err(ContractError::invalid_schema("Schema description too long (max 500 characters)"));
    }
    
    Ok(())
}

pub fn validate_schema_fields(fields: &[String], field_types: &[String]) -> Result<(), ContractError> {
    if fields.len() != field_types.len() {
        return Err(ContractError::invalid_schema("Fields and types count mismatch"));
    }
    
    if fields.is_empty() {
        return Err(ContractError::invalid_schema("Schema must have at least one field"));
    }
    
    if fields.len() > 50 {
        return Err(ContractError::invalid_schema("Schema cannot have more than 50 fields"));
    }
    
    // Validate individual fields
    for field in fields {
        if field.trim().is_empty() {
            return Err(ContractError::invalid_schema("Field name cannot be empty"));
        }
        
        if field.len() > 50 {
            return Err(ContractError::invalid_schema("Field name too long (max 50 characters)"));
        }
        
        // Check for invalid characters
        if field.chars().any(|c| !c.is_alphanumeric() && c != '_') {
            return Err(ContractError::invalid_schema("Field name contains invalid characters"));
        }
    }
    
    // Validate field types
    let valid_types = [
        "string", "int", "uint", "float", "bool", "bytes", "address", "timestamp"
    ];
    
    for field_type in field_types {
        if !valid_types.contains(&field_type.as_str()) {
            return Err(ContractError::invalid_schema(
                format!("Invalid field type: {} (valid types: {})", 
                    field_type, valid_types.join(", "))
            ));
        }
    }
    
    // Check for duplicate field names
    let mut unique_fields = std::collections::HashSet::new();
    for field in fields {
        if !unique_fields.insert(field) {
            return Err(ContractError::invalid_schema("Duplicate field names not allowed"));
        }
    }
    
    Ok(())
}

pub fn validate_schema_version(version: &str) -> Result<(), ContractError> {
    if version.trim().is_empty() {
        return Err(ContractError::invalid_schema("Schema version cannot be empty"));
    }
    
    if version.len() > 20 {
        return Err(ContractError::invalid_schema("Schema version too long (max 20 characters)"));
    }
    
    // Check version format (e.g., "1.0.0", "2.1", "beta-1")
    if !version.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
        return Err(ContractError::invalid_schema("Schema version contains invalid characters"));
    }
    
    Ok(())
}

// ============ ISSUER VALIDATION ============

pub fn validate_issuer_name(name: &str) -> Result<(), ContractError> {
    if name.trim().is_empty() {
        return Err(ContractError::invalid_issuer("Issuer name cannot be empty"));
    }
    
    if name.len() > 100 {
        return Err(ContractError::invalid_issuer("Issuer name too long (max 100 characters)"));
    }
    
    Ok(())
}

pub fn validate_issuer_description(description: &str) -> Result<(), ContractError> {
    if description.len() > 500 {
        return Err(ContractError::invalid_issuer("Issuer description too long (max 500 characters)"));
    }
    
    Ok(())
}

pub fn validate_issuer_uri(uri: &str) -> Result<(), ContractError> {
    if !uri.trim().is_empty() && !is_valid_uri(uri) {
        return Err(ContractError::invalid_issuer("Invalid URI format"));
    }
    
    if uri.len() > 500 {
        return Err(ContractError::invalid_issuer("URI too long (max 500 characters)"));
    }
    
    Ok(())
}

// ============ GENERAL VALIDATION ============

pub fn validate_address(address: &str) -> Result<(), ContractError> {
    if address.trim().is_empty() {
        return Err(ContractError::validation_error("Address cannot be empty"));
    }
    
    if address.len() > 100 {
        return Err(ContractError::validation_error("Address too long (max 100 characters)"));
    }
    
    Ok(())
}

pub fn validate_reason(reason: &str) -> Result<(), ContractError> {
    if reason.trim().is_empty() {
        return Err(ContractError::validation_error("Reason cannot be empty"));
    }
    
    if reason.len() > 200 {
        return Err(ContractError::validation_error("Reason too long (max 200 characters)"));
    }
    
    Ok(())
}

pub fn validate_pagination_params(
    page_size: Option<u32>,
    max_page_size: u32,
) -> Result<u32, ContractError> {
    let page_size = page_size.unwrap_or(30);
    
    if page_size == 0 {
        return Err(ContractError::invalid_pagination("Page size cannot be zero"));
    }
    
    if page_size > max_page_size {
        return Err(ContractError::invalid_pagination(
            format!("Page size too large (max {})", max_page_size)
        ));
    }
    
    Ok(page_size)
}
