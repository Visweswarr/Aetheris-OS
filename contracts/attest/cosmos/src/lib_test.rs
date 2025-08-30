use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Addr, Binary, Uint128, Timestamp};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use polymera_attestation::contract::{execute, instantiate, query};
use polymera_attestation::error::ContractError;
use polymera_attestation::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, AttestationResponse, AttestationsResponse,
    SchemaResponse, IssuerResponse, ConfigResponse, CountsResponse
};

// ============ TEST CONSTANTS ============

const ADMIN: &str = "admin";
const ISSUER1: &str = "issuer1";
const ISSUER2: &str = "issuer2";
const SUBJECT1: &str = "subject1";
const SUBJECT2: &str = "subject2";
const USER: &str = "user";

// ============ HELPER FUNCTIONS ============

fn mock_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        name: "Test Registry".to_string(),
        description: "Test attestation registry".to_string(),
        uri: "https://example.com".to_string(),
        min_attestation_lifetime: 86400, // 1 day
        max_attestation_lifetime: 31536000, // 1 year
        attestation_fee: Uint128::zero(),
        attestation_fee_enabled: false,
        max_tags_per_attestation: 10,
        max_attestation_data_size: 1024,
        admin: ADMIN.to_string(),
    }
}

fn mock_attestation_data() -> Binary {
    Binary::from(r#"{"name": "John Doe", "age": 30}"#.as_bytes())
}

fn mock_tags() -> Vec<String> {
    vec!["person".to_string(), "identification".to_string()]
}

// ============ INSTANTIATE TESTS ============

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "instantiate");
    assert_eq!(res.attributes[1].key, "admin");
    assert_eq!(res.attributes[1].value, ADMIN);
    assert_eq!(res.attributes[2].key, "name");
    assert_eq!(res.attributes[2].value, "Test Registry");
}

#[test]
fn test_instantiate_invalid_name() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);
    let mut msg = mock_instantiate_msg();
    msg.name = "".to_string();

    let err = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::InvalidConfiguration { reason } => {
            assert!(reason.contains("cannot be empty"));
        }
        _ => panic!("unexpected error: {}", err),
    }
}

#[test]
fn test_instantiate_invalid_lifetime() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);
    let mut msg = mock_instantiate_msg();
    msg.min_attestation_lifetime = 31536000; // 1 year
    msg.max_attestation_lifetime = 86400; // 1 day

    let err = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::InvalidConfiguration { reason } => {
            assert!(reason.contains("cannot exceed maximum"));
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ SCHEMA TESTS ============

#[test]
fn test_create_schema() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Create schema
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::CreateSchema {
        name: "Person".to_string(),
        description: "Person identification schema".to_string(),
        fields: vec!["name".to_string(), "age".to_string()],
        field_types: vec!["string".to_string(), "uint256".to_string()],
        version: "1.0.0".to_string(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "create_schema");
}

#[test]
fn test_create_schema_empty_fields() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to create schema with empty fields
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::CreateSchema {
        name: "Empty".to_string(),
        description: "Empty schema".to_string(),
        fields: vec![],
        field_types: vec![],
        version: "1.0.0".to_string(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::InvalidSchema { reason } => {
            assert!(reason.contains("must have at least one field"));
        }
        _ => panic!("unexpected error: {}", err),
    }
}

#[test]
fn test_create_schema_mismatched_fields() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to create schema with mismatched fields
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::CreateSchema {
        name: "Mismatch".to_string(),
        description: "Mismatched schema".to_string(),
        fields: vec!["name".to_string(), "age".to_string()],
        field_types: vec!["string".to_string()], // Only one type
        version: "1.0.0".to_string(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::InvalidSchema { reason } => {
            assert!(reason.contains("count mismatch"));
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ ISSUER TESTS ============

#[test]
fn test_register_issuer() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "register_issuer");
}

#[test]
fn test_register_issuer_duplicate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer first time
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
    // Try to register again
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Another Issuer".to_string(),
        description: "Another description".to_string(),
        uri: "https://example.com/another".to_string(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::IssuerAlreadyRegistered { address } => {
            assert_eq!(address, ISSUER1);
        }
        _ => panic!("unexpected error: {}", err),
    }
}

#[test]
fn test_update_issuer() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
    // Update issuer
    let msg = ExecuteMsg::UpdateIssuer {
        name: "Updated Issuer".to_string(),
        description: "Updated description".to_string(),
        uri: "https://example.com/updated".to_string(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "update_issuer");
}

#[test]
fn test_update_issuer_not_found() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to update non-existent issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::UpdateIssuer {
        name: "Updated Issuer".to_string(),
        description: "Updated description".to_string(),
        uri: "https://example.com/updated".to_string(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::IssuerNotFound { address } => {
            assert_eq!(address, ISSUER1);
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ AUTHORIZATION TESTS ============

#[test]
fn test_authorize_issuer_for_schema() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Create schema
    let info = mock_info(ISSUER2, &[]);
    let msg = ExecuteMsg::CreateSchema {
        name: "Person".to_string(),
        description: "Person identification schema".to_string(),
        fields: vec!["name".to_string()],
        field_types: vec!["string".to_string()],
        version: "1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Authorize issuer for schema
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "Person-1.0.0".to_string(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "authorize_issuer_for_schema");
}

#[test]
fn test_authorize_issuer_for_schema_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to authorize from non-admin account
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER2.to_string(),
        schema_id: "Person-1.0.0".to_string(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::Unauthorized { message } => {
            assert!(message.contains("Only admin can authorize"));
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ ATTESTATION TESTS ============

#[test]
fn test_issue_attestation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Authorize issuer for default schema
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Issue attestation
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "issue_attestation");
}

#[test]
fn test_issue_attestation_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to issue attestation without authorization
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::IssuerNotAuthorizedForSchema { issuer, schema } => {
            assert_eq!(issuer, ISSUER1);
            assert_eq!(schema, "default-1.0.0");
        }
        _ => panic!("unexpected error: {}", err),
    }
}

#[test]
fn test_issue_attestation_too_many_tags() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Authorize issuer for default schema
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to issue attestation with too many tags
    let info = mock_info(ISSUER1, &[]);
    let too_many_tags: Vec<String> = (0..11).map(|i| format!("tag{}", i)).collect();
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: too_many_tags,
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::TooManyTags { count, max } => {
            assert_eq!(count, 11);
            assert_eq!(max, 10);
        }
        _ => panic!("unexpected error: {}", err),
    }
}

#[test]
fn test_issue_attestation_data_too_large() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Register issuer
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Authorize issuer for default schema
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to issue attestation with data too large
    let info = mock_info(ISSUER1, &[]);
    let large_data = Binary::from(vec![0u8; 1025]); // 1KB + 1 byte
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: large_data,
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::AttestationDataTooLarge { size, max } => {
            assert_eq!(size, 1025);
            assert_eq!(max, 1024);
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ REVOCATION TESTS ============

#[test]
fn test_revoke_attestation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup: create attestation first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
    // Revoke attestation
    let msg = ExecuteMsg::RevokeAttestation {
        attestation_id: 1,
        reason: "Information incorrect".to_string(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "revoke_attestation");
}

#[test]
fn test_revoke_attestation_not_found() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to revoke non-existent attestation
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RevokeAttestation {
        attestation_id: 999,
        reason: "Test reason".to_string(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::AttestationNotFound { id } => {
            assert_eq!(id, 999);
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ UPDATE TESTS ============

#[test]
fn test_update_attestation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup: create attestation first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
    // Update attestation
    let new_data = Binary::from(r#"{"name": "John Doe", "age": 31}"#.as_bytes());
    let new_tags = vec!["person".to_string(), "updated".to_string()];
    let msg = ExecuteMsg::UpdateAttestation {
        attestation_id: 1,
        data: new_data.clone(),
        uri: "https://example.com/attestation/1/updated".to_string(),
        tags: new_tags.clone(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "update_attestation");
}

// ============ QUERY TESTS ============

#[test]
fn test_query_attestation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup: create attestation first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Query attestation
    let msg = QueryMsg::GetAttestation { attestation_id: 1 };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let attestation_response: AttestationResponse = from_binary(&res).unwrap();
    
    assert!(attestation_response.attestation.is_some());
    let attestation = attestation_response.attestation.unwrap();
    assert_eq!(attestation.id, 1);
    assert_eq!(attestation.issuer, ISSUER1);
    assert_eq!(attestation.subject, SUBJECT1);
}

#[test]
fn test_query_attestation_not_found() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Query non-existent attestation
    let msg = QueryMsg::GetAttestation { attestation_id: 999 };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let attestation_response: AttestationResponse = from_binary(&res).unwrap();
    
    assert!(attestation_response.attestation.is_none());
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Query config
    let msg = QueryMsg::GetConfig {};
    let res = query(deps.as_ref(), env, msg).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    
    assert_eq!(config_response.config.name, "Test Registry");
    assert_eq!(config_response.config.description, "Test attestation registry");
    assert_eq!(config_response.config.uri, "https://example.com");
    assert_eq!(config_response.config.admin, ADMIN);
}

#[test]
fn test_query_counts() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Query counts
    let msg = QueryMsg::GetCounts {};
    let res = query(deps.as_ref(), env, msg).unwrap();
    let counts_response: CountsResponse = from_binary(&res).unwrap();
    
    assert_eq!(counts_response.total_attestations, 0);
    assert_eq!(counts_response.total_schemas, 1); // Default schema
    assert_eq!(counts_response.total_issuers, 0);
}

// ============ CONFIG TESTS ============

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Update config
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        min_attestation_lifetime: Some(172800), // 2 days
        max_attestation_lifetime: Some(63072000), // 2 years
        attestation_fee: Some(Uint128::from(1000u128)),
        attestation_fee_enabled: Some(true),
        max_tags_per_attestation: Some(20),
        max_attestation_data_size: Some(2048),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 2);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "update_config");
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Try to update config from non-admin account
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        min_attestation_lifetime: Some(172800),
        max_attestation_lifetime: None,
        attestation_fee: None,
        attestation_fee_enabled: None,
        max_tags_per_attestation: None,
        max_attestation_data_size: None,
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::Unauthorized { message } => {
            assert!(message.contains("Only admin can update"));
        }
        _ => panic!("unexpected error: {}", err),
    }
}

// ============ EDGE CASES ============

#[test]
fn test_attestation_with_expiry() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup: create attestation with expiry
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Issue attestation with expiry
    let info = mock_info(ISSUER1, &[]);
    let expires_at = Timestamp::from_seconds(env.block.time.seconds() + 365 * 24 * 60 * 60); // 1 year from now
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: mock_attestation_data(),
        expires_at: Some(expires_at),
        uri: "https://example.com/attestation/1".to_string(),
        tags: mock_tags(),
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "issue_attestation");
}

#[test]
fn test_attestation_with_empty_data() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup: create attestation with empty data
    let info = mock_info(ADMIN, &[]);
    let msg = mock_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ISSUER1, &[]);
    let msg = ExecuteMsg::RegisterIssuer {
        name: "Test Issuer".to_string(),
        description: "Test issuer description".to_string(),
        uri: "https://example.com/issuer".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let info = mock_info(ADMIN, &[]);
    let msg = ExecuteMsg::AuthorizeIssuerForSchema {
        issuer: ISSUER1.to_string(),
        schema_id: "default-1.0.0".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Issue attestation with empty data
    let info = mock_info(ISSUER1, &[]);
    let empty_data = Binary::from(vec![]);
    let msg = ExecuteMsg::IssueAttestation {
        subject: SUBJECT1.to_string(),
        schema_id: "default-1.0.0".to_string(),
        data: empty_data,
        expires_at: None,
        uri: "https://example.com/attestation/1".to_string(),
        tags: vec![],
    };
    
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "issue_attestation");
}
