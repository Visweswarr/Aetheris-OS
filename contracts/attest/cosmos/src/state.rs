use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128, Timestamp};
use cw_storage_plus::{Item, Map, IndexedMap, MultiIndex, U64Key, StringKey};

use crate::msg::{Attestation, Schema, Issuer, Config};

// ============ CONFIGURATION ============

/// Registry configuration
pub const CONFIG: Item<Config> = Item::new("config");

/// Registry admin address
pub const ADMIN: Item<Addr> = Item::new("admin");

/// Next attestation ID
pub const NEXT_ATTESTATION_ID: Item<u64> = Item::new("next_attestation_id");

/// Next schema ID
pub const NEXT_SCHEMA_ID: Item<u64> = Item::new("next_schema_id");

// ============ STORAGE MAPS ============

/// Attestations by ID
pub const ATTESTATIONS: Map<U64Key, Attestation> = Map::new("attestations");

/// Schemas by ID
pub const SCHEMAS: Map<StringKey, Schema> = Map::new("schemas");

/// Issuers by address
pub const ISSUERS: Map<StringKey, Issuer> = Map::new("issuers");

/// Issuer schema authorizations
pub const ISSUER_SCHEMA_AUTH: Map<(StringKey, StringKey), bool> = Map::new("issuer_schema_auth");

// ============ INDEXED MAPS ============

/// Attestations by subject (for efficient querying)
pub const SUBJECT_ATTESTATIONS: IndexedMap<&str, U64Key, Attestation, SubjectIndex> = IndexedMap::new(
    "subject_attestations",
    SubjectIndex::new("subject_attestations__subject", "subject_attestations__idx")
);

/// Attestations by issuer (for efficient querying)
pub const ISSUER_ATTESTATIONS: IndexedMap<&str, U64Key, Attestation, IssuerIndex> = IndexedMap::new(
    "issuer_attestations",
    IssuerIndex::new("issuer_attestations__issuer", "issuer_attestations__idx")
);

/// Attestations by schema (for efficient querying)
pub const SCHEMA_ATTESTATIONS: IndexedMap<&str, U64Key, Attestation, SchemaIndex> = IndexedMap::new(
    "schema_attestations",
    SchemaIndex::new("schema_attestations__schema", "schema_attestations__idx")
);

/// Attestations by tags (for efficient querying)
pub const TAG_ATTESTATIONS: IndexedMap<&str, U64Key, Attestation, TagIndex> = IndexedMap::new(
    "tag_attestations",
    TagIndex::new("tag_attestations__tag", "tag_attestations__idx")
);

// ============ INDEX STRUCTURES ============

/// Index for attestations by subject
pub struct SubjectIndex<'a> {
    pub subject: &'a str,
    pub idx: &'a str,
}

impl<'a> SubjectIndex<'a> {
    pub fn new(subject: &'a str, idx: &'a str) -> Self {
        Self { subject, idx }
    }
}

/// Index for attestations by issuer
pub struct IssuerIndex<'a> {
    pub issuer: &'a str,
    pub idx: &'a str,
}

impl<'a> IssuerIndex<'a> {
    pub fn new(issuer: &'a str, idx: &'a str) -> Self {
        Self { issuer, idx }
    }
}

/// Index for attestations by schema
pub struct SchemaIndex<'a> {
    pub schema: &'a str,
    pub idx: &'a str,
}

impl<'a> SchemaIndex<'a> {
    pub fn new(schema: &'a str, idx: &'a str) -> Self {
        Self { schema, idx }
    }
}

/// Index for attestations by tag
pub struct TagIndex<'a> {
    pub tag: &'a str,
    pub idx: &'a str,
}

impl<'a> TagIndex<'a> {
    pub fn new(tag: &'a str, idx: &'a str) -> Self {
        Self { tag, idx }
    }
}

// ============ STORAGE KEYS ============

/// Storage key for attestation ID
pub fn attestation_key(attestation_id: u64) -> U64Key {
    U64Key::new(attestation_id)
}

/// Storage key for schema ID
pub fn schema_key(schema_id: &str) -> StringKey {
    StringKey::new(schema_id)
}

/// Storage key for issuer address
pub fn issuer_key(issuer: &str) -> StringKey {
    StringKey::new(issuer)
}

/// Storage key for issuer-schema authorization
pub fn issuer_schema_auth_key(issuer: &str, schema_id: &str) -> (StringKey, StringKey) {
    (StringKey::new(issuer), StringKey::new(schema_id))
}

// ============ STORAGE HELPERS ============

/// Get next attestation ID and increment
pub fn next_attestation_id(storage: &mut cosmwasm_std::Storage) -> cosmwasm_std::StdResult<u64> {
    let id = NEXT_ATTESTATION_ID.may_load(storage)?.unwrap_or(1);
    NEXT_ATTESTATION_ID.save(storage, &(id + 1))?;
    Ok(id)
}

/// Get next schema ID and increment
pub fn next_schema_id(storage: &mut cosmwasm_std::Storage) -> cosmwasm_std::StdResult<u64> {
    let id = NEXT_SCHEMA_ID.may_load(storage)?.unwrap_or(1);
    NEXT_SCHEMA_ID.save(storage, &(id + 1))?;
    Ok(id)
}

/// Check if attestation exists
pub fn attestation_exists(storage: &cosmwasm_std::Storage, attestation_id: u64) -> bool {
    ATTESTATIONS.has(storage, attestation_key(attestation_id))
}

/// Check if schema exists
pub fn schema_exists(storage: &cosmwasm_std::Storage, schema_id: &str) -> bool {
    SCHEMAS.has(storage, schema_key(schema_id))
}

/// Check if issuer exists
pub fn issuer_exists(storage: &cosmwasm_std::Storage, issuer: &str) -> bool {
    ISSUERS.has(storage, issuer_key(issuer))
}

/// Check if issuer is authorized for schema
pub fn is_issuer_authorized_for_schema(
    storage: &cosmwasm_std::Storage,
    issuer: &str,
    schema_id: &str
) -> bool {
    ISSUER_SCHEMA_AUTH
        .may_load(storage, issuer_schema_auth_key(issuer, schema_id))
        .unwrap_or(false)
}

// ============ STORAGE OPERATIONS ============

/// Save attestation to storage
pub fn save_attestation(
    storage: &mut cosmwasm_std::Storage,
    attestation: &Attestation
) -> cosmwasm_std::StdResult<()> {
    // Save to main attestations map
    ATTESTATIONS.save(storage, attestation_key(attestation.id), attestation)?;
    
    // Save to indexed maps
    SUBJECT_ATTESTATIONS.save(storage, attestation.subject.as_str(), attestation_key(attestation.id), attestation)?;
    ISSUER_ATTESTATIONS.save(storage, attestation.issuer.as_str(), attestation_key(attestation.id), attestation)?;
    SCHEMA_ATTESTATIONS.save(storage, attestation.schema_id.as_str(), attestation_key(attestation.id), attestation)?;
    
    // Save to tag index for each tag
    for tag in &attestation.tags {
        TAG_ATTESTATIONS.save(storage, tag.as_str(), attestation_key(attestation.id), attestation)?;
    }
    
    Ok(())
}

/// Update attestation in storage
pub fn update_attestation(
    storage: &mut cosmwasm_std::Storage,
    attestation: &Attestation
) -> cosmwasm_std::StdResult<()> {
    // Update main attestations map
    ATTESTATIONS.save(storage, attestation_key(attestation.id), attestation)?;
    
    // Update indexed maps
    SUBJECT_ATTESTATIONS.save(storage, attestation.subject.as_str(), attestation_key(attestation.id), attestation)?;
    ISSUER_ATTESTATIONS.save(storage, attestation.issuer.as_str(), attestation_key(attestation.id), attestation)?;
    SCHEMA_ATTESTATIONS.save(storage, attestation.schema_id.as_str(), attestation_key(attestation.id), attestation)?;
    
    // Update tag index for each tag
    for tag in &attestation.tags {
        TAG_ATTESTATIONS.save(storage, tag.as_str(), attestation_key(attestation.id), attestation)?;
    }
    
    Ok(())
}

/// Save schema to storage
pub fn save_schema(
    storage: &mut cosmwasm_std::Storage,
    schema: &Schema
) -> cosmwasm_std::StdResult<()> {
    SCHEMAS.save(storage, schema_key(&schema.id), schema)
}

/// Save issuer to storage
pub fn save_issuer(
    storage: &mut cosmwasm_std::Storage,
    issuer: &Issuer
) -> cosmwasm_std::StdResult<()> {
    ISSUERS.save(storage, issuer_key(&issuer.addr), issuer)
}

/// Authorize issuer for schema
pub fn authorize_issuer_for_schema(
    storage: &mut cosmwasm_std::Storage,
    issuer: &str,
    schema_id: &str
) -> cosmwasm_std::StdResult<()> {
    ISSUER_SCHEMA_AUTH.save(storage, issuer_schema_auth_key(issuer, schema_id), &true)
}

/// Revoke issuer schema authorization
pub fn revoke_issuer_schema_authorization(
    storage: &mut cosmwasm_std::Storage,
    issuer: &str,
    schema_id: &str
) -> cosmwasm_std::StdResult<()> {
    ISSUER_SCHEMA_AUTH.save(storage, issuer_schema_auth_key(issuer, schema_id), &false)
}

// ============ QUERY HELPERS ============

/// Get attestations by subject with pagination
pub fn get_subject_attestations(
    storage: &cosmwasm_std::Storage,
    subject: &str,
    start_after: Option<u64>,
    limit: Option<u32>
) -> cosmwasm_std::StdResult<Vec<Attestation>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|id| attestation_key(id + 1));
    
    let attestations: Result<Vec<_>, _> = SUBJECT_ATTESTATIONS
        .range(storage, subject, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, attestation)| attestation))
        .collect();
    
    attestations
}

/// Get attestations by issuer with pagination
pub fn get_issuer_attestations(
    storage: &cosmwasm_std::Storage,
    issuer: &str,
    start_after: Option<u64>,
    limit: Option<u32>
) -> cosmwasm_std::StdResult<Vec<Attestation>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|id| attestation_key(id + 1));
    
    let attestations: Result<Vec<_>, _> = ISSUER_ATTESTATIONS
        .range(storage, issuer, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, attestation)| attestation))
        .collect();
    
    attestations
}

/// Get attestations by schema with pagination
pub fn get_schema_attestations(
    storage: &cosmwasm_std::Storage,
    schema_id: &str,
    start_after: Option<u64>,
    limit: Option<u32>
) -> cosmwasm_std::StdResult<Vec<Attestation>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|id| attestation_key(id + 1));
    
    let attestations: Result<Vec<_>, _> = SCHEMA_ATTESTATIONS
        .range(storage, schema_id, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, attestation)| attestation))
        .collect();
    
    attestations
}

/// Get attestations by tags with pagination
pub fn get_tag_attestations(
    storage: &cosmwasm_std::Storage,
    tags: &[String],
    limit: Option<u32>
) -> cosmwasm_std::StdResult<Vec<Attestation>> {
    let limit = limit.unwrap_or(30) as usize;
    let mut all_attestations = std::collections::HashMap::new();
    
    for tag in tags {
        let attestations: Result<Vec<_>, _> = TAG_ATTESTATIONS
            .range(storage, tag, None, None, cosmwasm_std::Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, attestation)| attestation))
            .collect();
        
        for attestation in attestations? {
            all_attestations.insert(attestation.id, attestation);
        }
    }
    
    let mut result: Vec<_> = all_attestations.into_values().collect();
    result.sort_by_key(|a| a.id);
    result.truncate(limit);
    
    Ok(result)
}

/// Count total attestations
pub fn count_attestations(storage: &cosmwasm_std::Storage) -> cosmwasm_std::StdResult<u64> {
    Ok(ATTESTATIONS.range(storage, None, None, cosmwasm_std::Order::Ascending).count() as u64)
}

/// Count total schemas
pub fn count_schemas(storage: &cosmwasm_std::Storage) -> cosmwasm_std::StdResult<u64> {
    Ok(SCHEMAS.range(storage, None, None, cosmwasm_std::Order::Ascending).count() as u64)
}

/// Count total issuers
pub fn count_issuers(storage: &cosmwasm_std::Storage) -> cosmwasm_std::StdResult<u64> {
    Ok(ISSUERS.range(storage, None, None, cosmwasm_std::Order::Ascending).count() as u64)
}

// ============ VALIDATION HELPERS ============

/// Validate attestation data size
pub fn validate_attestation_data_size(
    storage: &cosmwasm_std::Storage,
    data_size: usize
) -> cosmwasm_std::StdResult<()> {
    let config = CONFIG.load(storage)?;
    if data_size > config.max_attestation_data_size as usize {
        return Err(cosmwasm_std::StdError::generic_err("Attestation data too large"));
    }
    Ok(())
}

/// Validate attestation tags count
pub fn validate_attestation_tags_count(
    storage: &cosmwasm_std::Storage,
    tags_count: usize
) -> cosmwasm_std::StdResult<()> {
    let config = CONFIG.load(storage)?;
    if tags_count > config.max_tags_per_attestation as usize {
        return Err(cosmwasm_std::StdError::generic_err("Too many tags"));
    }
    Ok(())
}

/// Validate attestation lifetime
pub fn validate_attestation_lifetime(
    storage: &cosmwasm_std::Storage,
    expires_at: Option<Timestamp>
) -> cosmwasm_std::StdResult<()> {
    let config = CONFIG.load(storage)?;
    let now = cosmwasm_std::Timestamp::from_seconds(cosmwasm_std::BlockInfo::time_seconds());
    
    if let Some(expires_at) = expires_at {
        let min_expiry = now.plus_seconds(config.min_attestation_lifetime);
        let max_expiry = now.plus_seconds(config.max_attestation_lifetime);
        
        if expires_at < min_expiry {
            return Err(cosmwasm_std::StdError::generic_err("Expiration too soon"));
        }
        if expires_at > max_expiry {
            return Err(cosmwasm_std::StdError::generic_err("Expiration too far"));
        }
    }
    
    Ok(())
}
