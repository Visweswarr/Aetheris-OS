use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, Uint128, Timestamp,
    Addr, StdResult,
};

use crate::error::ContractError;
use crate::msg::{Attestation, Schema, Issuer, Config};
use crate::state::{
    ATTESTATIONS, SCHEMAS, ISSUERS, ISSUER_SCHEMA_AUTH,
    next_attestation_id, next_schema_id,
    save_attestation, update_attestation, save_schema, save_issuer,
    authorize_issuer_for_schema, revoke_issuer_schema_authorization,
    CONFIG, ADMIN, is_issuer_authorized_for_schema, issuer_exists,
    schema_exists, attestation_exists,
    validate_attestation_data_size, validate_attestation_tags_count, validate_attestation_lifetime,
};

// ============ ATTESTATION HANDLERS ============

pub fn handle_issue_attestation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    subject: String,
    schema_id: String,
    data: cosmwasm_std::Binary,
    expires_at: Option<Timestamp>,
    uri: String,
    tags: Vec<String>,
) -> Result<Response, ContractError> {
    // Validate inputs
    validate_attestation_data_size(deps.storage, data.len())?;
    validate_attestation_tags_count(deps.storage, tags.len())?;
    validate_attestation_lifetime(deps.storage, expires_at)?;
    
    // Check if issuer is authorized for schema
    if !is_issuer_authorized_for_schema(deps.storage, &info.sender.to_string(), &schema_id) {
        return Err(ContractError::issuer_not_authorized_for_schema(
            info.sender.to_string(), schema_id
        ));
    }
    
    // Get next attestation ID
    let attestation_id = next_attestation_id(deps.storage)?;
    
    // Create attestation
    let attestation = Attestation {
        id: attestation_id,
        issuer: info.sender.to_string(),
        subject,
        schema_id,
        data,
        issued_at: env.block.time,
        expires_at,
        revoked: false,
        revoked_at: None,
        revoked_by: None,
        uri,
        tags,
        version: 1,
    };
    
    // Save attestation
    save_attestation(deps.storage, &attestation)?;
    
    // Update issuer stats
    if let Ok(mut issuer) = ISSUERS.load(deps.storage, info.sender.as_str()) {
        issuer.total_attestations += 1;
        save_issuer(deps.storage, &issuer)?;
    }
    
    Ok(Response::new()
        .add_attribute("method", "issue_attestation")
        .add_attribute("attestation_id", attestation_id.to_string())
        .add_attribute("issuer", info.sender)
        .add_attribute("subject", attestation.subject))
}

pub fn handle_revoke_attestation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attestation_id: u64,
    reason: String,
) -> Result<Response, ContractError> {
    // Check if attestation exists
    if !attestation_exists(deps.storage, attestation_id) {
        return Err(ContractError::attestation_not_found(attestation_id));
    }
    
    // Load attestation
    let mut attestation = ATTESTATIONS.load(deps.storage, attestation_id)?;
    
    // Check if already revoked
    if attestation.revoked {
        return Err(ContractError::attestation_already_revoked(attestation_id));
    }
    
    // Check if expired
    if let Some(expires_at) = attestation.expires_at {
        if env.block.time >= expires_at {
            return Err(ContractError::attestation_expired(attestation_id));
        }
    }
    
    // Check authorization (issuer or admin)
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != attestation.issuer && info.sender != admin {
        return Err(ContractError::unauthorized("Only issuer or admin can revoke attestation"));
    }
    
    // Revoke attestation
    attestation.revoked = true;
    attestation.revoked_at = Some(env.block.time);
    attestation.revoked_by = Some(info.sender.to_string());
    
    // Update attestation
    update_attestation(deps.storage, &attestation)?;
    
    Ok(Response::new()
        .add_attribute("method", "revoke_attestation")
        .add_attribute("attestation_id", attestation_id.to_string())
        .add_attribute("revoked_by", info.sender)
        .add_attribute("reason", reason))
}

pub fn handle_update_attestation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attestation_id: u64,
    data: cosmwasm_std::Binary,
    uri: String,
    tags: Vec<String>,
) -> Result<Response, ContractError> {
    // Check if attestation exists
    if !attestation_exists(deps.storage, attestation_id) {
        return Err(ContractError::attestation_not_found(attestation_id));
    }
    
    // Load attestation
    let mut attestation = ATTESTATIONS.load(deps.storage, attestation_id)?;
    
    // Check if revoked
    if attestation.revoked {
        return Err(ContractError::attestation_already_revoked(attestation_id));
    }
    
    // Check if expired
    if let Some(expires_at) = attestation.expires_at {
        if env.block.time >= expires_at {
            return Err(ContractError::attestation_expired(attestation_id));
        }
    }
    
    // Check authorization (issuer or admin)
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != attestation.issuer && info.sender != admin {
        return Err(ContractError::unauthorized("Only issuer or admin can update attestation"));
    }
    
    // Validate inputs
    validate_attestation_data_size(deps.storage, data.len())?;
    validate_attestation_tags_count(deps.storage, tags.len())?;
    
    // Update attestation
    attestation.data = data;
    attestation.uri = uri;
    attestation.tags = tags;
    attestation.version += 1;
    
    // Save updated attestation
    update_attestation(deps.storage, &attestation)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_attestation")
        .add_attribute("attestation_id", attestation_id.to_string())
        .add_attribute("updated_by", info.sender)
        .add_attribute("version", attestation.version.to_string()))
}

// ============ SCHEMA HANDLERS ============

pub fn handle_create_schema(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    fields: Vec<String>,
    field_types: Vec<String>,
    version: String,
) -> Result<Response, ContractError> {
    // Validate inputs
    if fields.len() != field_types.len() {
        return Err(ContractError::invalid_schema("Fields and types count mismatch"));
    }
    
    if fields.is_empty() {
        return Err(ContractError::invalid_schema("Schema must have at least one field"));
    }
    
    // Generate schema ID
    let schema_id = format!("{}-{}", name, version);
    
    // Check if schema already exists
    if schema_exists(deps.storage, &schema_id) {
        return Err(ContractError::schema_already_exists(schema_id.clone()));
    }
    
    // Create schema
    let schema = Schema {
        id: schema_id.clone(),
        name,
        description,
        fields,
        field_types,
        required: false,
        created_at: env.block.time,
        created_by: info.sender.to_string(),
        deprecated: false,
        deprecated_at: None,
        version,
    };
    
    // Save schema
    save_schema(deps.storage, &schema)?;
    
    Ok(Response::new()
        .add_attribute("method", "create_schema")
        .add_attribute("schema_id", schema_id)
        .add_attribute("created_by", info.sender))
}

pub fn handle_deprecate_schema(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    schema_id: String,
    reason: String,
) -> Result<Response, ContractError> {
    // Check if schema exists
    if !schema_exists(deps.storage, &schema_id) {
        return Err(ContractError::schema_not_found(schema_id.clone()));
    }
    
    // Load schema
    let mut schema = SCHEMAS.load(deps.storage, &schema_id)?;
    
    // Check authorization (creator or admin)
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != schema.created_by && info.sender != admin {
        return Err(ContractError::unauthorized("Only creator or admin can deprecate schema"));
    }
    
    // Deprecate schema
    schema.deprecated = true;
    schema.deprecated_at = Some(env.block.time);
    
    // Save schema
    save_schema(deps.storage, &schema)?;
    
    Ok(Response::new()
        .add_attribute("method", "deprecate_schema")
        .add_attribute("schema_id", schema_id)
        .add_attribute("deprecated_by", info.sender)
        .add_attribute("reason", reason))
}

// ============ ISSUER HANDLERS ============

pub fn handle_register_issuer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    uri: String,
) -> Result<Response, ContractError> {
    // Check if issuer already exists
    if issuer_exists(deps.storage, &info.sender.to_string()) {
        return Err(ContractError::issuer_already_registered(info.sender.to_string()));
    }
    
    // Create issuer
    let issuer = Issuer {
        addr: info.sender.to_string(),
        name,
        description,
        uri,
        active: true,
        registered_at: env.block.time,
        total_attestations: 0,
        authorized_schemas: Vec::new(),
    };
    
    // Save issuer
    save_issuer(deps.storage, &issuer)?;
    
    Ok(Response::new()
        .add_attribute("method", "register_issuer")
        .add_attribute("address", info.sender)
        .add_attribute("name", issuer.name))
}

pub fn handle_update_issuer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    uri: String,
) -> Result<Response, ContractError> {
    // Check if issuer exists
    if !issuer_exists(deps.storage, &info.sender.to_string()) {
        return Err(ContractError::issuer_not_found(info.sender.to_string()));
    }
    
    // Load issuer
    let mut issuer = ISSUERS.load(deps.storage, &info.sender.to_string())?;
    
    // Update issuer
    issuer.name = name.clone();
    issuer.description = description.clone();
    issuer.uri = uri.clone();
    
    // Save issuer
    save_issuer(deps.storage, &issuer)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_issuer")
        .add_attribute("address", info.sender)
        .add_attribute("name", name))
}

pub fn handle_deactivate_issuer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reason: String,
) -> Result<Response, ContractError> {
    // Check if issuer exists
    if !issuer_exists(deps.storage, &info.sender.to_string()) {
        return Err(ContractError::issuer_not_found(info.sender.to_string()));
    }
    
    // Load issuer
    let mut issuer = ISSUERS.load(deps.storage, &info.sender.to_string())?;
    
    // Deactivate issuer
    issuer.active = false;
    
    // Save issuer
    save_issuer(deps.storage, &issuer)?;
    
    Ok(Response::new()
        .add_attribute("method", "deactivate_issuer")
        .add_attribute("address", info.sender)
        .add_attribute("reason", reason))
}

// ============ AUTHORIZATION HANDLERS ============

pub fn handle_authorize_issuer_for_schema(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    issuer: String,
    schema_id: String,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::unauthorized("Only admin can authorize issuers"));
    }
    
    // Check if issuer exists
    if !issuer_exists(deps.storage, &issuer) {
        return Err(ContractError::issuer_not_found(issuer.clone()));
    }
    
    // Check if schema exists
    if !schema_exists(deps.storage, &schema_id) {
        return Err(ContractError::schema_not_found(schema_id.clone()));
    }
    
    // Authorize issuer for schema
    authorize_issuer_for_schema(deps.storage, &issuer, &schema_id)?;
    
    Ok(Response::new()
        .add_attribute("method", "authorize_issuer_for_schema")
        .add_attribute("issuer", issuer)
        .add_attribute("schema_id", schema_id)
        .add_attribute("authorized_by", info.sender))
}

pub fn handle_revoke_issuer_schema_authorization(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    issuer: String,
    schema_id: String,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::unauthorized("Only admin can revoke issuer authorizations"));
    }
    
    // Revoke authorization
    revoke_issuer_schema_authorization(deps.storage, &issuer, &schema_id)?;
    
    Ok(Response::new()
        .add_attribute("method", "revoke_issuer_schema_authorization")
        .add_attribute("issuer", issuer)
        .add_attribute("schema_id", schema_id)
        .add_attribute("revoked_by", info.sender))
}

// ============ CONFIG HANDLERS ============

pub fn handle_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    min_attestation_lifetime: Option<u64>,
    max_attestation_lifetime: Option<u64>,
    attestation_fee: Option<Uint128>,
    attestation_fee_enabled: Option<bool>,
    max_tags_per_attestation: Option<u32>,
    max_attestation_data_size: Option<u32>,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::unauthorized("Only admin can update config"));
    }
    
    // Load current config
    let mut config = CONFIG.load(deps.storage)?;
    
    // Update config fields
    if let Some(lifetime) = min_attestation_lifetime {
        config.min_attestation_lifetime = lifetime;
    }
    if let Some(lifetime) = max_attestation_lifetime {
        config.max_attestation_lifetime = lifetime;
    }
    if let Some(fee) = attestation_fee {
        config.attestation_fee = fee;
    }
    if let Some(enabled) = attestation_fee_enabled {
        config.attestation_fee_enabled = enabled;
    }
    if let Some(max_tags) = max_tags_per_attestation {
        config.max_tags_per_attestation = max_tags;
    }
    if let Some(max_size) = max_attestation_data_size {
        config.max_attestation_data_size = max_size;
    }
    
    // Save config
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("updated_by", info.sender))
}

pub fn handle_withdraw_fees(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::unauthorized("Only admin can withdraw fees"));
    }
    
    // Validate recipient address
    let _recipient = deps.api.addr_validate(&recipient)?;
    
    // For now, just return success
    // In a real implementation, this would transfer funds
    Ok(Response::new()
        .add_attribute("method", "withdraw_fees")
        .add_attribute("recipient", recipient)
        .add_attribute("withdrawn_by", info.sender))
}

// ============ UTILITY FUNCTIONS ============

pub fn create_default_schema(
    storage: &mut cosmwasm_std::Storage,
    env: &Env,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let schema_id = "default-1.0.0".to_string();
    
    let schema = Schema {
        id: schema_id.clone(),
        name: "Default Attestation".to_string(),
        description: "Basic attestation schema for general use".to_string(),
        fields: vec!["type".to_string(), "value".to_string()],
        field_types: vec!["string".to_string(), "string".to_string()],
        required: true,
        created_at: env.block.time,
        created_by: info.sender.to_string(),
        deprecated: false,
        deprecated_at: None,
        version: "1.0.0".to_string(),
    };
    
    save_schema(storage, &schema)?;
    
    // Authorize the contract creator for the default schema
    authorize_issuer_for_schema(storage, &info.sender.to_string(), &schema_id)?;
    
    Ok(())
}
