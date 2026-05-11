//! Capability token management module for AI Core Service

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};


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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        if self.config.caching.enabled {
            if let Some(cached) = self.get_cached_token(&token.token_id).await {
                if cached.validated && now < cached.expires_at {
                    // Token validation cache hit
                    return Ok(());
                }
            }
        }
        
        if self.config.validation.check_expiration && token.expires_at > 0 && now > token.expires_at {
            return Err(AiCoreError::CapabilityError("Token has expired".to_string()));
        }
        
        if self.config.caching.enabled {
            self.cache_token(token, true).await;
        }
        
        Ok(())
    }

    pub async fn check_capability(&self, token: &CapToken, _resource: &str, _action: &str) -> Result<CapabilityCheckResult> {
        self.validate_token(token).await?;
        
        Ok(CapabilityCheckResult {
            granted: true,
            reason: "Capability granted".to_string(),
            matched_capability: Some(token.capability.clone()),
            conditions_met: true,
            authorized: true,
        })
    }

    pub async fn check_capabilities(&self, cap_token: &CapToken, required_capabilities: &[String]) -> Result<CapabilityCheckResult> {
        self.validate_token(cap_token).await?;
        
        let token_caps: Vec<&str> = cap_token.capability.split(',').collect();
        let has_all = required_capabilities.iter().all(|cap| token_caps.contains(&cap.as_str()) || token_caps.contains(&"admin"));
        
        Ok(CapabilityCheckResult {
            granted: has_all,
            reason: if has_all { "All capabilities granted".to_string() } else { "Missing capabilities".to_string() },
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
            validated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            expires_at: token.expires_at,
            capabilities: vec![token.capability.clone()],
        };
        
        let mut cache = self.token_cache.write().await;
        if cache.len() >= self.config.caching.max_cache_size {
            let oldest_key = cache.iter()
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
        issuers.insert("aetheris-system".to_string(), IssuerConfig {
            name: "Aetheris System".to_string(),
            public_key: "".to_string(),
            trusted: true,
            max_token_lifetime: 3600,
            allowed_capabilities: vec!["ai:chat".to_string(), "ai:tools".to_string(), "ai:admin".to_string()],
        });
        
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
