//! Capability management for wallet operations

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::error::WalletError;

/// Wallet capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WalletCapability {
    WalletCreate,
    KeyGenerate,
    KeyDerive,
    KeySign,
    KeyExport,
    KeyImport,
    KeyRevoke,
    DIDCreate,
    DIDResolve,
    VaultList,
    VaultGet,
    VaultPut,
    VaultRemove,
    AuditRead,
}

/// Capability manager for enforcing access control
pub struct CapabilityManager {
    capabilities: HashMap<String, Vec<WalletCapability>>,
    rate_limits: HashMap<String, HashMap<String, RateLimit>>,
    policies: Vec<CapabilityPolicy>,
}

/// Rate limit configuration
#[derive(Debug, Clone)]
struct RateLimit {
    count: u32,
    window_start: SystemTime,
    limit: u32,
    window_duration: Duration,
}

/// Capability policy
#[derive(Debug, Clone)]
struct CapabilityPolicy {
    subject_pattern: String,
    capabilities: Vec<WalletCapability>,
    rate_limits: HashMap<String, u32>, // operation -> limit per day
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        let mut manager = Self {
            capabilities: HashMap::new(),
            rate_limits: HashMap::new(),
            policies: Vec::new(),
        };

        // Set up default policies
        manager.setup_default_policies();
        manager
    }

    /// Check if a subject has a specific capability
    pub async fn check_capability(
        &self,
        subject: &str,
        capability: &WalletCapability,
    ) -> Result<bool, WalletError> {
        // Check explicit capabilities first
        if let Some(subject_caps) = self.capabilities.get(subject) {
            if subject_caps.contains(capability) {
                return Ok(true);
            }
        }

        // Check policy-based capabilities
        for policy in &self.policies {
            if self.matches_subject_pattern(subject, &policy.subject_pattern) {
                if policy.capabilities.contains(capability) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check rate limits for an operation
    pub async fn check_rate_limit(
        &self,
        subject: &str,
        operation: &str,
    ) -> Result<bool, WalletError> {
        let now = SystemTime::now();
        
        if let Some(subject_limits) = self.rate_limits.get(subject) {
            if let Some(rate_limit) = subject_limits.get(operation) {
                // Check if we're still in the current window
                if now.duration_since(rate_limit.window_start).unwrap_or(Duration::ZERO) < rate_limit.window_duration {
                    if rate_limit.count >= rate_limit.limit {
                        return Ok(false); // Rate limit exceeded
                    }
                } else {
                    // Reset the window
                    return Ok(true); // New window, allow
                }
            }
        }

        // Check policy-based rate limits
        for policy in &self.policies {
            if self.matches_subject_pattern(subject, &policy.subject_pattern) {
                if let Some(&limit) = policy.rate_limits.get(operation) {
                    // Check current rate limit for this subject/operation
                    if let Some(subject_limits) = self.rate_limits.get(subject) {
                        if let Some(rate_limit) = subject_limits.get(operation) {
                            if now.duration_since(rate_limit.window_start).unwrap_or(Duration::ZERO) < rate_limit.window_duration {
                                if rate_limit.count >= limit {
                                    return Ok(false);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    /// Grant a capability to a subject
    pub fn grant_capability(&mut self, subject: &str, capability: WalletCapability) {
        self.capabilities
            .entry(subject.to_string())
            .or_insert_with(Vec::new)
            .push(capability);
    }

    /// Revoke a capability from a subject
    pub fn revoke_capability(&mut self, subject: &str, capability: &WalletCapability) {
        if let Some(subject_caps) = self.capabilities.get_mut(subject) {
            subject_caps.retain(|cap| cap != capability);
        }
    }

    /// Set rate limit for an operation
    pub fn set_rate_limit(
        &mut self,
        subject: &str,
        operation: &str,
        limit: u32,
        window_duration: Duration,
    ) {
        let rate_limit = RateLimit {
            count: 0,
            window_start: SystemTime::now(),
            limit,
            window_duration,
        };

        self.rate_limits
            .entry(subject.to_string())
            .or_insert_with(HashMap::new)
            .insert(operation.to_string(), rate_limit);
    }

    /// Record an operation for rate limiting
    pub fn record_operation(&mut self, subject: &str, operation: &str) {
        let now = SystemTime::now();
        
        if let Some(subject_limits) = self.rate_limits.get_mut(subject) {
            if let Some(rate_limit) = subject_limits.get_mut(operation) {
                // Check if we need to reset the window
                if now.duration_since(rate_limit.window_start).unwrap_or(Duration::ZERO) >= rate_limit.window_duration {
                    rate_limit.count = 0;
                    rate_limit.window_start = now;
                }
                
                rate_limit.count += 1;
            }
        }
    }

    /// Add a capability policy
    pub fn add_policy(&mut self, policy: CapabilityPolicy) {
        self.policies.push(policy);
    }

    /// Setup default policies
    fn setup_default_policies(&mut self) {
        // Default policy: allow all operations for admin users
        let admin_policy = CapabilityPolicy {
            subject_pattern: "admin:*".to_string(),
            capabilities: vec![
                WalletCapability::WalletCreate,
                WalletCapability::KeyGenerate,
                WalletCapability::KeyDerive,
                WalletCapability::KeySign,
                WalletCapability::KeyExport,
                WalletCapability::KeyImport,
                WalletCapability::KeyRevoke,
                WalletCapability::DIDCreate,
                WalletCapability::DIDResolve,
                WalletCapability::VaultList,
                WalletCapability::VaultGet,
                WalletCapability::VaultPut,
                WalletCapability::VaultRemove,
                WalletCapability::AuditRead,
            ],
            rate_limits: HashMap::from([
                ("keygen".to_string(), 100), // 100 key generations per day
                ("sign".to_string(), 1000),  // 1000 signatures per day
            ]),
        };

        // Default policy: limited operations for regular users
        let user_policy = CapabilityPolicy {
            subject_pattern: "user:*".to_string(),
            capabilities: vec![
                WalletCapability::WalletCreate,
                WalletCapability::KeyGenerate,
                WalletCapability::KeyDerive,
                WalletCapability::KeySign,
                WalletCapability::KeyExport,
                WalletCapability::DIDCreate,
                WalletCapability::DIDResolve,
                WalletCapability::VaultList,
                WalletCapability::VaultGet,
                WalletCapability::AuditRead,
            ],
            rate_limits: HashMap::from([
                ("keygen".to_string(), 10),  // 10 key generations per day
                ("sign".to_string(), 100),   // 100 signatures per day
            ]),
        };

        // Default policy: read-only access for observers
        let observer_policy = CapabilityPolicy {
            subject_pattern: "observer:*".to_string(),
            capabilities: vec![
                WalletCapability::DIDResolve,
                WalletCapability::VaultList,
                WalletCapability::AuditRead,
            ],
            rate_limits: HashMap::new(),
        };

        self.policies.push(admin_policy);
        self.policies.push(user_policy);
        self.policies.push(observer_policy);
    }

    /// Check if a subject matches a pattern
    fn matches_subject_pattern(&self, subject: &str, pattern: &str) -> bool {
        if pattern.ends_with("*") {
            let prefix = pattern.strip_suffix("*").unwrap();
            subject.starts_with(prefix)
        } else {
            subject == pattern
        }
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capability_manager_creation() {
        let manager = CapabilityManager::new();
        assert!(!manager.policies.is_empty());
    }

    #[tokio::test]
    async fn test_capability_check() {
        let manager = CapabilityManager::new();
        
        // Admin should have all capabilities
        let has_cap = manager.check_capability("admin:test", &WalletCapability::KeyGenerate).await;
        assert!(has_cap.is_ok());
        assert!(has_cap.unwrap());

        // Regular user should have limited capabilities
        let has_cap = manager.check_capability("user:test", &WalletCapability::KeyGenerate).await;
        assert!(has_cap.is_ok());
        assert!(has_cap.unwrap());

        // Observer should not have key generation capability
        let has_cap = manager.check_capability("observer:test", &WalletCapability::KeyGenerate).await;
        assert!(has_cap.is_ok());
        assert!(!has_cap.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = CapabilityManager::new();
        
        // Should allow first operation
        let allowed = manager.check_rate_limit("user:test", "keygen").await;
        assert!(allowed.is_ok());
        assert!(allowed.unwrap());
    }

    #[tokio::test]
    async fn test_capability_grant_revoke() {
        let mut manager = CapabilityManager::new();
        
        // Grant capability
        manager.grant_capability("test_user", WalletCapability::KeyImport);
        let has_cap = manager.check_capability("test_user", &WalletCapability::KeyImport).await;
        assert!(has_cap.is_ok());
        assert!(has_cap.unwrap());

        // Revoke capability
        manager.revoke_capability("test_user", &WalletCapability::KeyImport);
        let has_cap = manager.check_capability("test_user", &WalletCapability::KeyImport).await;
        assert!(has_cap.is_ok());
        assert!(!has_cap.unwrap());
    }
}
