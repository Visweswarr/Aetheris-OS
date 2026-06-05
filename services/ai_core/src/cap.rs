//! Capability token management module for AI Core Service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::error::{AiCoreError, Result};
use crate::ipc::CapToken;

pub struct CapTokenManager {
    config: CapTokenConfig,
    token_cache: Arc<RwLock<HashMap<String, CachedToken>>>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: CapToken,
    validated: bool,
    validated_at: u64,
    expires_at: u64,
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapTokenConfig {
    pub issuers: HashMap<String, IssuerConfig>,
    pub validation: ValidationConfig,
    pub caching: CachingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConfig {
    pub name: String,
    pub public_key: String,
    pub trusted: bool,
    pub max_token_lifetime: u64,
    pub allowed_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub require_signature: bool,
    pub check_expiration: bool,
    pub max_clock_skew: u64,
    pub default_token_lifetime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    pub enabled: bool,
    pub cache_ttl: u64,
    pub max_cache_size: usize,
    pub cache_validation_results: bool,
}

#[derive(Debug, Clone)]
pub struct CapabilityCheckResult {
    pub granted: bool,
    pub reason: String,
    pub matched_capability: Option<String>,
    pub conditions_met: bool,
    pub authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    pub capability: String,
    pub resource: String,
    pub action: String,
    pub issuer: String,
    pub allow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEnforcementResult {
    pub granted: bool,
    pub reason: String,
    pub matched_rule: Option<String>,
}

pub struct PolicyEnforcer {
    policies: Vec<CapabilityPolicy>,
}

impl PolicyEnforcer {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            policies: vec![
                CapabilityPolicy {
                    capability: "ai:chat".to_string(),
                    resource: "ai_core".to_string(),
                    action: "generate_response".to_string(),
                    issuer: "aetheris-system".to_string(),
                    allow: true,
                },
                CapabilityPolicy {
                    capability: "ai:tools".to_string(),
                    resource: "tools".to_string(),
                    action: "execute".to_string(),
                    issuer: "aetheris-system".to_string(),
                    allow: true,
                },
            ],
        })
    }

    pub async fn enforce_policy(
        &self,
        capability: &str,
        resource: &str,
        action: &str,
        issuer: &str,
    ) -> Result<PolicyEnforcementResult> {
        if issuer != "aetheris-system" {
            return Ok(PolicyEnforcementResult {
                granted: false,
                reason: "Unknown issuer".to_string(),
                matched_rule: None,
            });
        }
        if !matches!(capability, "ai:chat" | "ai:tools" | "ai:admin") {
            return Ok(PolicyEnforcementResult {
                granted: false,
                reason: "Unknown capability".to_string(),
                matched_rule: None,
            });
        }
        if capability == "ai:admin" {
            return Ok(PolicyEnforcementResult {
                granted: true,
                reason: "Capability granted by admin policy".to_string(),
                matched_rule: Some("admin".to_string()),
            });
        }
        if let Some(policy) = self.policies.iter().find(|policy| {
            policy.capability == capability
                && policy.resource == resource
                && policy.action == action
                && policy.issuer == issuer
        }) {
            return Ok(PolicyEnforcementResult {
                granted: policy.allow,
                reason: if policy.allow {
                    "Capability granted".to_string()
                } else {
                    "Capability denied".to_string()
                },
                matched_rule: Some(format!(
                    "{}:{}:{}",
                    policy.capability, policy.resource, policy.action
                )),
            });
        }
        Ok(PolicyEnforcementResult {
            granted: false,
            reason: "Capability denied by default policy".to_string(),
            matched_rule: None,
        })
    }
}

impl CapTokenManager {
    pub async fn new(_config_path: &Path) -> Result<Self> {
        // Initializing Capability Token Manager
        let config = Self::create_default_config();
        Ok(Self {
            config,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn new_mock() -> Self {
        Self {
            config: Self::create_default_config(),
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn validate_token(&self, token: &CapToken) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if self.config.caching.enabled {
            if let Some(cached) = self.get_cached_token(&token.token_id).await {
                if cached.validated && now < cached.expires_at {
                    // Token validation cache hit
                    return Ok(());
                }
            }
        }

        if self.config.validation.check_expiration && token.expires_at > 0 && now > token.expires_at
        {
            return Err(AiCoreError::CapabilityError(
                "Token has expired".to_string(),
            ));
        }

        if self.config.caching.enabled {
            self.cache_token(token, true).await;
        }

        Ok(())
    }

    pub async fn check_capability(
        &self,
        token: &CapToken,
        resource: &str,
        action: &str,
    ) -> Result<CapabilityCheckResult> {
        self.validate_token(token).await?;
        let enforcement = self
            .enforce_capability_policy(&token.capability, resource, action, &token.issuer)
            .await?;

        Ok(CapabilityCheckResult {
            granted: enforcement.granted,
            reason: enforcement.reason,
            matched_capability: enforcement
                .matched_rule
                .as_ref()
                .map(|_| token.capability.clone()),
            conditions_met: enforcement.granted,
            authorized: enforcement.granted,
        })
    }

    pub async fn check_capability_with_deny(
        &self,
        token: &CapToken,
        resource: &str,
        action: &str,
    ) -> Result<CapabilityCheckResult> {
        let result = self.check_capability(token, resource, action).await?;
        if result.granted {
            Ok(result)
        } else {
            Err(AiCoreError::CapDenied(result.reason))
        }
    }

    pub async fn enforce_capability_policy(
        &self,
        capability: &str,
        resource: &str,
        action: &str,
        issuer: &str,
    ) -> Result<PolicyEnforcementResult> {
        PolicyEnforcer::new()
            .await?
            .enforce_policy(capability, resource, action, issuer)
            .await
    }

    pub async fn check_capabilities(
        &self,
        cap_token: &CapToken,
        required_capabilities: &[String],
    ) -> Result<CapabilityCheckResult> {
        self.validate_token(cap_token).await?;

        let token_caps: Vec<&str> = cap_token.capability.split(',').collect();
        let has_all = required_capabilities
            .iter()
            .all(|cap| token_caps.contains(&cap.as_str()) || token_caps.contains(&"admin"));

        Ok(CapabilityCheckResult {
            granted: has_all,
            reason: if has_all {
                "All capabilities granted".to_string()
            } else {
                "Missing capabilities".to_string()
            },
            matched_capability: Some(cap_token.capability.clone()),
            conditions_met: has_all,
            authorized: has_all,
        })
    }

    async fn get_cached_token(&self, token_id: &str) -> Option<CachedToken> {
        self.token_cache.read().await.get(token_id).cloned()
    }

    async fn cache_token(&self, token: &CapToken, validated: bool) {
        let cached = CachedToken {
            token: token.clone(),
            validated,
            validated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expires_at: token.expires_at,
            capabilities: vec![token.capability.clone()],
        };

        let mut cache = self.token_cache.write().await;
        if cache.len() >= self.config.caching.max_cache_size {
            let oldest_key = cache
                .iter()
                .min_by_key(|(_, v)| v.validated_at)
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }
        cache.insert(token.token_id.clone(), cached);
    }

    fn create_default_config() -> CapTokenConfig {
        let mut issuers = HashMap::new();
        issuers.insert(
            "aetheris-system".to_string(),
            IssuerConfig {
                name: "Aetheris System".to_string(),
                public_key: "".to_string(),
                trusted: true,
                max_token_lifetime: 3600,
                allowed_capabilities: vec![
                    "ai:chat".to_string(),
                    "ai:tools".to_string(),
                    "ai:admin".to_string(),
                ],
            },
        );

        CapTokenConfig {
            issuers,
            validation: ValidationConfig {
                require_signature: false,
                check_expiration: true,
                max_clock_skew: 300,
                default_token_lifetime: 3600,
            },
            caching: CachingConfig {
                enabled: true,
                cache_ttl: 300,
                max_cache_size: 1000,
                cache_validation_results: true,
            },
        }
    }
}
