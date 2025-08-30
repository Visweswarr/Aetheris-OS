use cosmwasm_std::{Deps, StdResult, Uint128};
use crate::error::ContractError;
use crate::msg::{
    AttestationResponse, AttestationsResponse, SchemaResponse, SchemasResponse,
    IssuerResponse, IssuersResponse, AttestationValidityResponse, ConfigResponse,
    CountsResponse, Attestation, Schema, Issuer, Config
};
use crate::state::{
    ATTESTATIONS, SCHEMAS, ISSUERS, CONFIG, count_attestations, count_schemas, count_issuers,
    get_subject_attestations, get_issuer_attestations, get_schema_attestations,
    get_tag_attestations, validate_pagination_limit
};

// ============ ATTESTATION QUERIES ============

pub fn query_attestation(deps: Deps, attestation_id: u64) -> Result<AttestationResponse, ContractError> {
    let attestation = ATTESTATIONS.may_load(deps.storage, attestation_id)?;
    Ok(AttestationResponse { attestation })
}

pub fn query_subject_attestations(
    deps: Deps,
    subject: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    let attestations = get_subject_attestations(deps.storage, &subject, start_after, Some(limit))?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}

pub fn query_issuer_attestations(
    deps: Deps,
    issuer: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    let attestations = get_issuer_attestations(deps.storage, &issuer, start_after, Some(limit))?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}

pub fn query_schema_attestations(
    deps: Deps,
    schema_id: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    let attestations = get_schema_attestations(deps.storage, &schema_id, start_after, Some(limit))?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}

pub fn query_attestation_validity(
    deps: Deps,
    attestation_id: u64,
) -> Result<AttestationValidityResponse, ContractError> {
    let attestation = ATTESTATIONS.load(deps.storage, attestation_id)?;
    
    let mut is_valid = true;
    let mut reason = None;
    
    // Check if revoked
    if attestation.revoked {
        is_valid = false;
        reason = Some("Attestation is revoked".to_string());
    }
    
    // Check if expired
    if let Some(expires_at) = attestation.expires_at {
        if deps.api.block_info().time >= expires_at {
            is_valid = false;
            reason = Some("Attestation is expired".to_string());
        }
    }
    
    Ok(AttestationValidityResponse {
        is_valid,
        reason,
    })
}

// ============ SCHEMA QUERIES ============

pub fn query_schema(deps: Deps, schema_id: String) -> Result<SchemaResponse, ContractError> {
    let schema = SCHEMAS.may_load(deps.storage, &schema_id)?;
    Ok(SchemaResponse { schema })
}

pub fn query_schemas(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<SchemasResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    
    let schemas: Result<Vec<_>, _> = SCHEMAS
        .range(deps.storage, start_after.as_deref().map(|s| s.as_str()), None, cosmwasm_std::Order::Ascending)
        .take(limit as usize)
        .map(|item| item.map(|(_, schema)| schema))
        .collect();
    
    let schemas = schemas?;
    let total = count_schemas(deps.storage)?;
    
    Ok(SchemasResponse {
        schemas,
        total,
    })
}

// ============ ISSUER QUERIES ============

pub fn query_issuer(deps: Deps, issuer: String) -> Result<IssuerResponse, ContractError> {
    let issuer_data = ISSUERS.may_load(deps.storage, &issuer)?;
    Ok(IssuerResponse { issuer: issuer_data })
}

pub fn query_issuers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<IssuersResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    
    let issuers: Result<Vec<_>, _> = ISSUERS
        .range(deps.storage, start_after.as_deref().map(|s| s.as_str()), None, cosmwasm_std::Order::Ascending)
        .take(limit as usize)
        .map(|item| item.map(|(_, issuer)| issuer))
        .collect();
    
    let issuers = issuers?;
    let total = count_issuers(deps.storage)?;
    
    Ok(IssuersResponse {
        issuers,
        total,
    })
}

// ============ CONFIG QUERIES ============

pub fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_counts(deps: Deps) -> Result<CountsResponse, ContractError> {
    let total_attestations = count_attestations(deps.storage)?;
    let total_schemas = count_schemas(deps.storage)?;
    let total_issuers = count_issuers(deps.storage)?;
    
    Ok(CountsResponse {
        total_attestations,
        total_schemas,
        total_issuers,
    })
}

// ============ LISTING QUERIES ============

pub fn query_attestations(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    
    let attestations: Result<Vec<_>, _> = ATTESTATIONS
        .range(deps.storage, start_after.map(|id| id + 1), None, cosmwasm_std::Order::Ascending)
        .take(limit as usize)
        .map(|item| item.map(|(_, attestation)| attestation))
        .collect();
    
    let attestations = attestations?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}

// ============ SEARCH QUERIES ============

pub fn query_search_attestations_by_tags(
    deps: Deps,
    tags: Vec<String>,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    let attestations = get_tag_attestations(deps.storage, &tags, Some(limit))?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}

pub fn query_search_attestations_by_issuer(
    deps: Deps,
    issuer: String,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    let attestations = get_issuer_attestations(deps.storage, &issuer, None, Some(limit))?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}

pub fn query_search_attestations_by_schema(
    deps: Deps,
    schema_id: String,
    limit: Option<u32>,
) -> Result<AttestationsResponse, ContractError> {
    let limit = validate_pagination_limit(limit, 100)?;
    let attestations = get_schema_attestations(deps.storage, &schema_id, None, Some(limit))?;
    let total = count_attestations(deps.storage)?;
    
    Ok(AttestationsResponse {
        attestations,
        total,
    })
}
