use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, Timestamp,
};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
use crate::state::{CONFIG, ADMIN, next_attestation_id, next_schema_id};
use crate::handlers::{
    handle_issue_attestation, handle_revoke_attestation, handle_update_attestation,
    handle_create_schema, handle_deprecate_schema, handle_register_issuer,
    handle_update_issuer, handle_deactivate_issuer, handle_authorize_issuer_for_schema,
    handle_revoke_issuer_schema_authorization, handle_update_config, handle_withdraw_fees
};
use crate::queries::{
    query_attestation, query_subject_attestations, query_issuer_attestations,
    query_schema_attestations, query_schema, query_issuer, query_attestation_validity,
    query_config, query_counts, query_schemas, query_issuers, query_attestations,
    query_search_attestations_by_tags, query_search_attestations_by_issuer,
    query_search_attestations_by_schema
};

// ============ INSTANTIATE ============

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Validate input parameters
    crate::validation::validate_instantiate_msg(&msg)?;
    
    // Set admin
    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;
    
    // Create config
    let config = crate::msg::Config {
        name: msg.name,
        description: msg.description,
        uri: msg.uri,
        min_attestation_lifetime: msg.min_attestation_lifetime,
        max_attestation_lifetime: msg.max_attestation_lifetime,
        attestation_fee: msg.attestation_fee,
        attestation_fee_enabled: msg.attestation_fee_enabled,
        max_tags_per_attestation: msg.max_tags_per_attestation,
        max_attestation_data_size: msg.max_attestation_data_size,
        admin: msg.admin,
        created_at: env.block.time,
        metadata: std::collections::HashMap::new(),
    };
    
    CONFIG.save(deps.storage, &config)?;
    
    // Initialize counters
    next_attestation_id(deps.storage)?;
    next_schema_id(deps.storage)?;
    
    // Create default schema
    crate::handlers::create_default_schema(deps.storage, &env, &info)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("name", config.name))
}

// ============ EXECUTE ============

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::IssueAttestation { subject, schema_id, data, expires_at, uri, tags } => {
            handle_issue_attestation(deps, env, info, subject, schema_id, data, expires_at, uri, tags)
        }
        ExecuteMsg::RevokeAttestation { attestation_id, reason } => {
            handle_revoke_attestation(deps, env, info, attestation_id, reason)
        }
        ExecuteMsg::UpdateAttestation { attestation_id, data, uri, tags } => {
            handle_update_attestation(deps, env, info, attestation_id, data, uri, tags)
        }
        ExecuteMsg::CreateSchema { name, description, fields, field_types, version } => {
            handle_create_schema(deps, env, info, name, description, fields, field_types, version)
        }
        ExecuteMsg::DeprecateSchema { schema_id, reason } => {
            handle_deprecate_schema(deps, env, info, schema_id, reason)
        }
        ExecuteMsg::RegisterIssuer { name, description, uri } => {
            handle_register_issuer(deps, env, info, name, description, uri)
        }
        ExecuteMsg::UpdateIssuer { name, description, uri } => {
            handle_update_issuer(deps, env, info, name, description, uri)
        }
        ExecuteMsg::DeactivateIssuer { reason } => {
            handle_deactivate_issuer(deps, env, info, reason)
        }
        ExecuteMsg::AuthorizeIssuerForSchema { issuer, schema_id } => {
            handle_authorize_issuer_for_schema(deps, env, info, issuer, schema_id)
        }
        ExecuteMsg::RevokeIssuerSchemaAuthorization { issuer, schema_id } => {
            handle_revoke_issuer_schema_authorization(deps, env, info, issuer, schema_id)
        }
        ExecuteMsg::UpdateConfig { min_attestation_lifetime, max_attestation_lifetime, attestation_fee, attestation_fee_enabled, max_tags_per_attestation, max_attestation_data_size } => {
            handle_update_config(deps, env, info, min_attestation_lifetime, max_attestation_lifetime, attestation_fee, attestation_fee_enabled, max_tags_per_attestation, max_attestation_data_size)
        }
        ExecuteMsg::WithdrawFees { recipient } => {
            handle_withdraw_fees(deps, env, info, recipient)
        }
    }
}

// ============ QUERY ============

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAttestation { attestation_id } => {
            to_binary(&query_attestation(deps, attestation_id)?)
        }
        QueryMsg::GetSubjectAttestations { subject } => {
            to_binary(&query_subject_attestations(deps, subject, None, None)?)
        }
        QueryMsg::GetIssuerAttestations { issuer } => {
            to_binary(&query_issuer_attestations(deps, issuer, None, None)?)
        }
        QueryMsg::GetSchemaAttestations { schema_id } => {
            to_binary(&query_schema_attestations(deps, schema_id, None, None)?)
        }
        QueryMsg::GetSchema { schema_id } => {
            to_binary(&query_schema(deps, schema_id)?)
        }
        QueryMsg::GetIssuer { issuer } => {
            to_binary(&query_issuer(deps, issuer)?)
        }
        QueryMsg::IsAttestationValid { attestation_id } => {
            to_binary(&query_attestation_validity(deps, attestation_id)?)
        }
        QueryMsg::GetConfig {} => {
            to_binary(&query_config(deps)?)
        }
        QueryMsg::GetCounts {} => {
            to_binary(&query_counts(deps)?)
        }
        QueryMsg::ListSchemas { start_after, limit } => {
            to_binary(&query_schemas(deps, start_after, limit)?)
        }
        QueryMsg::ListIssuers { start_after, limit } => {
            to_binary(&query_issuers(deps, start_after, limit)?)
        }
        QueryMsg::ListAttestations { start_after, limit } => {
            to_binary(&query_attestations(deps, start_after, limit)?)
        }
        QueryMsg::SearchAttestationsByTags { tags, limit } => {
            to_binary(&query_search_attestations_by_tags(deps, tags, limit)?)
        }
        QueryMsg::SearchAttestationsByIssuer { issuer, limit } => {
            to_binary(&query_search_attestations_by_issuer(deps, issuer, limit)?)
        }
        QueryMsg::SearchAttestationsBySchema { schema_id, limit } => {
            to_binary(&query_search_attestations_by_schema(deps, schema_id, limit)?)
        }
    }
}
