//! Rate Limiting Library for Polymera OS
//!
//! Provides token bucket rate limiting with DID-based tracking, burst allowances,
//! and penalty mechanisms. Designed as reusable middleware for all services.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn, error};

/// Errors that can occur during rate limiting operations
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for DID: {did}")]
    RateLimitExceeded { did: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Time error: {0}")]
    Time(String),

    #[error("Penalty applied: {penalty_type} until {until:?}")]
    PenaltyActive { penalty_type: String, until: SystemTime },
}

/// Result type for rate limiting operations
pub type RateLimitResult<T> = Result<T, RateLimitError>;

/// Rate limiting decision outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitDecision {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Number of tokens remaining
    pub tokens_remaining: u64,
    /// Time until bucket refills
    pub refill_time: Duration,
    /// When the rate limit resets
    pub reset_time: SystemTime,
    /// Current penalty level if any
    pub penalty: Option<PenaltyInfo>,
    /// HTTP headers to include in response
    pub headers: HashMap<String, String>,
}

/// Configuration for a rate limiting bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketConfig {
    /// Maximum number of tokens in the bucket
    pub capacity: u64,
    /// Number of tokens to refill per interval
    pub refill_amount: u64,
    /// Refill interval
    pub refill_interval: Duration,
    /// Initial token count (defaults to capacity)
    pub initial_tokens: Option<u64>,
    /// Whether to allow burst above capacity
    pub allow_burst: bool,
    /// Maximum burst allowance
    pub burst_capacity: Option<u64>,
    /// Burst cooldown period
    pub burst_cooldown: Option<Duration>,
}

/// Rate limiting rules for different request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRules {
    /// Default bucket configuration
    pub default: BucketConfig,
    /// Per-endpoint specific configurations
    pub endpoints: HashMap<String, BucketConfig>,
    /// Per-DID specific configurations (for premium users, etc.)
    pub did_overrides: HashMap<String, BucketConfig>,
    /// Global rate limit across all DIDs
    pub global_limit: Option<BucketConfig>,
}

/// Penalty configuration and escalation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyConfig {
    /// Enable penalty system
    pub enabled: bool,
    /// Violation threshold to trigger penalty
    pub violation_threshold: u32,
    /// Base penalty duration
    pub base_penalty_duration: Duration,
    /// Penalty escalation multiplier
    pub escalation_multiplier: f64,
    /// Maximum penalty duration
    pub max_penalty_duration: Duration,
    /// Time window for counting violations
    pub violation_window: Duration,
    /// Penalty types and their multipliers
    pub penalty_types: HashMap<String, f64>,
}

/// Information about an active penalty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyInfo {
    /// Type of penalty applied
    pub penalty_type: String,
    /// When the penalty was applied
    pub applied_at: SystemTime,
    /// When the penalty expires
    pub expires_at: SystemTime,
    /// Number of violations that triggered this penalty
    pub violation_count: u32,
    /// Escalation level
    pub escalation_level: u32,
}

/// Token bucket state for a specific DID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBucket {
    /// Current number of tokens
    pub tokens: f64,
    /// Maximum bucket capacity
    pub capacity: u64,
    /// Refill rate (tokens per second)
    pub refill_rate: f64,
    /// Last refill timestamp
    pub last_refill: SystemTime,
    /// Number of burst tokens used
    pub burst_tokens_used: u64,
    /// When burst cooldown ends
    pub burst_cooldown_until: Option<SystemTime>,
    /// Request count in current window
    pub request_count: u64,
    /// Current window start time
    pub window_start: SystemTime,
}

/// Rate limit violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    /// When the violation occurred
    pub timestamp: SystemTime,
    /// Endpoint that was violated
    pub endpoint: String,
    /// Severity of the violation
    pub severity: ViolationSeverity,
    /// Tokens attempted to consume
    pub tokens_requested: u64,
    /// Tokens available at time of violation
    pub tokens_available: f64,
}

/// Severity levels for rate limit violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    Minor,      // Slightly over limit
    Moderate,   // Significantly over limit
    Severe,     // Extreme violation
    Critical,   // Potential abuse
}

/// DID-specific rate limiting state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidRateLimitState {
    /// DID identifier
    pub did: String,
    /// Token buckets per endpoint
    pub buckets: HashMap<String, TokenBucket>,
    /// Penalty information if any
    pub penalty: Option<PenaltyInfo>,
    /// Violation history
    pub violations: Vec<ViolationRecord>,
    /// Total requests made
    pub total_requests: u64,
    /// First seen timestamp
    pub first_seen: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
}

/// Rate limiting service implementation
#[derive(Debug)]
pub struct RateLimiter {
    /// Rate limiting rules
    rules: RateLimitRules,
    /// Penalty configuration
    penalty_config: PenaltyConfig,
    /// DID states (DID -> State)
    states: Arc<RwLock<HashMap<String, DidRateLimitState>>>,
    /// Global state for cross-DID limits
    global_state: Arc<RwLock<TokenBucket>>,
    /// Configuration for cleanup intervals
    cleanup_interval: Duration,
    /// Maximum age for inactive DIDs
    max_inactive_age: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(rules: RateLimitRules, penalty_config: PenaltyConfig) -> Self {
        let global_bucket = rules.global_limit.as_ref().map(|config| {
            TokenBucket::new(config, SystemTime::now())
        }).unwrap_or_else(|| {
            // Default global bucket with high limits
            let default_global = BucketConfig {
                capacity: 10000,
                refill_amount: 1000,
                refill_interval: Duration::from_secs(60),
                initial_tokens: Some(10000),
                allow_burst: false,
                burst_capacity: None,
                burst_cooldown: None,
            };
            TokenBucket::new(&default_global, SystemTime::now())
        });

        Self {
            rules,
            penalty_config,
            states: Arc::new(RwLock::new(HashMap::new())),
            global_state: Arc::new(RwLock::new(global_bucket)),
            cleanup_interval: Duration::from_secs(3600), // 1 hour
            max_inactive_age: Duration::from_secs(86400 * 7), // 7 days
        }
    }

    /// Check if a request is allowed for the given DID and endpoint
    pub async fn check_rate_limit(
        &self,
        did: &str,
        endpoint: &str,
        tokens_requested: u64,
    ) -> RateLimitResult<RateLimitDecision> {
        // Check global rate limit first
        self.check_global_limit(tokens_requested).await?;

        // Get or create DID state
        let mut did_state = self.get_or_create_did_state(did).await;

        // Check if DID has active penalty
        if let Some(penalty) = &did_state.penalty {
            if penalty.expires_at > SystemTime::now() {
                return Err(RateLimitError::PenaltyActive {
                    penalty_type: penalty.penalty_type.clone(),
                    until: penalty.expires_at,
                });
            } else {
                // Penalty has expired, clear it
                did_state.penalty = None;
            }
        }

        // Get bucket configuration for this endpoint
        let bucket_config = self.get_bucket_config(did, endpoint);

        // Get or create bucket for this endpoint
        let bucket = did_state.buckets.entry(endpoint.to_string())
            .or_insert_with(|| TokenBucket::new(&bucket_config, SystemTime::now()));

        // Refill tokens based on elapsed time
        bucket.refill(&bucket_config, SystemTime::now());

        // Check if request can be satisfied
        let decision = if bucket.can_consume(tokens_requested, &bucket_config) {
            // Consume tokens
            bucket.consume(tokens_requested, &bucket_config);
            
            // Update DID state
            did_state.total_requests += 1;
            did_state.last_activity = SystemTime::now();

            RateLimitDecision {
                allowed: true,
                tokens_remaining: bucket.tokens as u64,
                refill_time: bucket.time_until_refill(&bucket_config),
                reset_time: bucket.next_refill_time(&bucket_config),
                penalty: did_state.penalty.clone(),
                headers: self.generate_headers(&bucket, &bucket_config),
            }
        } else {
            // Record violation
            let violation = ViolationRecord {
                timestamp: SystemTime::now(),
                endpoint: endpoint.to_string(),
                severity: self.calculate_violation_severity(tokens_requested, bucket.tokens),
                tokens_requested,
                tokens_available: bucket.tokens,
            };

            did_state.violations.push(violation.clone());

            // Check if penalty should be applied
            if self.penalty_config.enabled {
                self.evaluate_penalty(&mut did_state).await;
            }

            RateLimitDecision {
                allowed: false,
                tokens_remaining: bucket.tokens as u64,
                refill_time: bucket.time_until_refill(&bucket_config),
                reset_time: bucket.next_refill_time(&bucket_config),
                penalty: did_state.penalty.clone(),
                headers: self.generate_headers(&bucket, &bucket_config),
            }
        };

        // Update state
        self.update_did_state(did, did_state).await;

        if !decision.allowed {
            warn!(
                did = did,
                endpoint = endpoint,
                tokens_requested = tokens_requested,
                tokens_available = bucket.tokens,
                "Rate limit exceeded"
            );
            
            return Err(RateLimitError::RateLimitExceeded {
                did: did.to_string(),
            });
        }

        debug!(
            did = did,
            endpoint = endpoint,
            tokens_remaining = decision.tokens_remaining,
            "Rate limit check passed"
        );

        Ok(decision)
    }

    /// Get rate limit status for a DID without consuming tokens
    pub async fn get_rate_limit_status(&self, did: &str, endpoint: &str) -> RateLimitResult<RateLimitDecision> {
        let did_state = self.get_or_create_did_state(did).await;
        let bucket_config = self.get_bucket_config(did, endpoint);
        
        let bucket = did_state.buckets.get(endpoint)
            .cloned()
            .unwrap_or_else(|| TokenBucket::new(&bucket_config, SystemTime::now()));

        let mut bucket_copy = bucket.clone();
        bucket_copy.refill(&bucket_config, SystemTime::now());

        Ok(RateLimitDecision {
            allowed: bucket_copy.can_consume(1, &bucket_config), // Check if at least 1 token available
            tokens_remaining: bucket_copy.tokens as u64,
            refill_time: bucket_copy.time_until_refill(&bucket_config),
            reset_time: bucket_copy.next_refill_time(&bucket_config),
            penalty: did_state.penalty.clone(),
            headers: self.generate_headers(&bucket_copy, &bucket_config),
        })
    }

    /// Reset rate limits for a DID (admin function)
    pub async fn reset_rate_limit(&self, did: &str, endpoint: Option<&str>) -> RateLimitResult<()> {
        let mut states = self.states.write().unwrap();
        
        if let Some(did_state) = states.get_mut(did) {
            if let Some(endpoint) = endpoint {
                // Reset specific endpoint
                if let Some(bucket) = did_state.buckets.get_mut(endpoint) {
                    let config = self.get_bucket_config(did, endpoint);
                    *bucket = TokenBucket::new(&config, SystemTime::now());
                }
            } else {
                // Reset all endpoints
                did_state.buckets.clear();
                did_state.penalty = None;
                did_state.violations.clear();
            }
            
            info!(did = did, endpoint = endpoint, "Rate limit reset");
        }

        Ok(())
    }

    /// Apply a manual penalty to a DID (admin function)
    pub async fn apply_penalty(
        &self,
        did: &str,
        penalty_type: &str,
        duration: Duration,
    ) -> RateLimitResult<()> {
        let mut did_state = self.get_or_create_did_state(did).await;
        
        let penalty = PenaltyInfo {
            penalty_type: penalty_type.to_string(),
            applied_at: SystemTime::now(),
            expires_at: SystemTime::now() + duration,
            violation_count: 0, // Manual penalty
            escalation_level: 0,
        };

        did_state.penalty = Some(penalty);
        self.update_did_state(did, did_state).await;

        warn!(
            did = did,
            penalty_type = penalty_type,
            duration = ?duration,
            "Manual penalty applied"
        );

        Ok(())
    }

    /// Get statistics for all DIDs or a specific DID
    pub async fn get_statistics(&self, did: Option<&str>) -> RateLimitResult<serde_json::Value> {
        let states = self.states.read().unwrap();
        
        if let Some(did) = did {
            // Get stats for specific DID
            if let Some(state) = states.get(did) {
                Ok(serde_json::to_value(state)?)
            } else {
                Ok(serde_json::json!({
                    "did": did,
                    "found": false
                }))
            }
        } else {
            // Get aggregate stats
            let total_dids = states.len();
            let total_requests: u64 = states.values().map(|s| s.total_requests).sum();
            let active_penalties: usize = states.values()
                .filter(|s| s.penalty.as_ref().map_or(false, |p| p.expires_at > SystemTime::now()))
                .count();
            let total_violations: usize = states.values().map(|s| s.violations.len()).sum();

            Ok(serde_json::json!({
                "total_dids": total_dids,
                "total_requests": total_requests,
                "active_penalties": active_penalties,
                "total_violations": total_violations,
                "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
            }))
        }
    }

    /// Cleanup expired states and old violations
    pub async fn cleanup(&self) -> RateLimitResult<u32> {
        let mut states = self.states.write().unwrap();
        let now = SystemTime::now();
        let mut cleaned_count = 0;

        // Remove expired penalties and old violations
        for (_, state) in states.iter_mut() {
            // Clear expired penalties
            if let Some(penalty) = &state.penalty {
                if penalty.expires_at <= now {
                    state.penalty = None;
                    cleaned_count += 1;
                }
            }

            // Remove old violations
            let violation_cutoff = now - self.penalty_config.violation_window;
            let original_len = state.violations.len();
            state.violations.retain(|v| v.timestamp > violation_cutoff);
            cleaned_count += (original_len - state.violations.len()) as u32;
        }

        // Remove inactive DIDs
        let inactive_cutoff = now - self.max_inactive_age;
        let original_count = states.len();
        states.retain(|_, state| state.last_activity > inactive_cutoff);
        cleaned_count += (original_count - states.len()) as u32;

        if cleaned_count > 0 {
            info!(cleaned_items = cleaned_count, "Rate limiter cleanup completed");
        }

        Ok(cleaned_count)
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.cleanup_interval);
        
        loop {
            interval.tick().await;
            if let Err(e) = self.cleanup().await {
                error!(error = %e, "Rate limiter cleanup failed");
            }
        }
    }

    // Private helper methods

    async fn check_global_limit(&self, tokens_requested: u64) -> RateLimitResult<()> {
        if let Some(_) = &self.rules.global_limit {
            let mut global_bucket = self.global_state.write().unwrap();
            let global_config = self.rules.global_limit.as_ref().unwrap();
            
            global_bucket.refill(global_config, SystemTime::now());
            
            if !global_bucket.can_consume(tokens_requested, global_config) {
                return Err(RateLimitError::RateLimitExceeded {
                    did: "global".to_string(),
                });
            }
            
            global_bucket.consume(tokens_requested, global_config);
        }
        
        Ok(())
    }

    async fn get_or_create_did_state(&self, did: &str) -> DidRateLimitState {
        let states = self.states.read().unwrap();
        
        if let Some(state) = states.get(did) {
            state.clone()
        } else {
            drop(states);
            
            let new_state = DidRateLimitState {
                did: did.to_string(),
                buckets: HashMap::new(),
                penalty: None,
                violations: Vec::new(),
                total_requests: 0,
                first_seen: SystemTime::now(),
                last_activity: SystemTime::now(),
            };

            let mut states = self.states.write().unwrap();
            states.insert(did.to_string(), new_state.clone());
            new_state
        }
    }

    async fn update_did_state(&self, did: &str, state: DidRateLimitState) {
        let mut states = self.states.write().unwrap();
        states.insert(did.to_string(), state);
    }

    fn get_bucket_config(&self, did: &str, endpoint: &str) -> BucketConfig {
        // Check for DID-specific override
        if let Some(config) = self.rules.did_overrides.get(did) {
            return config.clone();
        }
        
        // Check for endpoint-specific config
        if let Some(config) = self.rules.endpoints.get(endpoint) {
            return config.clone();
        }
        
        // Use default config
        self.rules.default.clone()
    }

    fn calculate_violation_severity(&self, tokens_requested: u64, tokens_available: f64) -> ViolationSeverity {
        let ratio = tokens_requested as f64 / (tokens_available.max(1.0));
        
        match ratio {
            r if r <= 2.0 => ViolationSeverity::Minor,
            r if r <= 5.0 => ViolationSeverity::Moderate,
            r if r <= 10.0 => ViolationSeverity::Severe,
            _ => ViolationSeverity::Critical,
        }
    }

    async fn evaluate_penalty(&self, did_state: &mut DidRateLimitState) {
        let now = SystemTime::now();
        let window_start = now - self.penalty_config.violation_window;
        
        // Count recent violations
        let recent_violations: Vec<_> = did_state.violations.iter()
            .filter(|v| v.timestamp > window_start)
            .collect();

        if recent_violations.len() >= self.penalty_config.violation_threshold as usize {
            // Calculate penalty duration based on escalation
            let current_level = did_state.penalty.as_ref()
                .map_or(0, |p| p.escalation_level + 1);
            
            let penalty_multiplier = self.penalty_config.escalation_multiplier.powi(current_level as i32);
            let penalty_duration = Duration::from_secs(
                (self.penalty_config.base_penalty_duration.as_secs() as f64 * penalty_multiplier) as u64
            ).min(self.penalty_config.max_penalty_duration);

            let penalty = PenaltyInfo {
                penalty_type: "rate_limit_violation".to_string(),
                applied_at: now,
                expires_at: now + penalty_duration,
                violation_count: recent_violations.len() as u32,
                escalation_level: current_level,
            };

            did_state.penalty = Some(penalty);

            warn!(
                did = did_state.did,
                violations = recent_violations.len(),
                escalation_level = current_level,
                penalty_duration = ?penalty_duration,
                "Penalty applied due to rate limit violations"
            );
        }
    }

    fn generate_headers(&self, bucket: &TokenBucket, config: &BucketConfig) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        headers.insert("X-RateLimit-Limit".to_string(), config.capacity.to_string());
        headers.insert("X-RateLimit-Remaining".to_string(), (bucket.tokens as u64).to_string());
        headers.insert("X-RateLimit-Reset".to_string(), 
            bucket.next_refill_time(config)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        );
        
        if config.allow_burst {
            if let Some(burst_capacity) = config.burst_capacity {
                headers.insert("X-RateLimit-Burst-Limit".to_string(), burst_capacity.to_string());
                headers.insert("X-RateLimit-Burst-Remaining".to_string(), 
                    burst_capacity.saturating_sub(bucket.burst_tokens_used).to_string());
            }
        }

        headers
    }
}

impl TokenBucket {
    fn new(config: &BucketConfig, now: SystemTime) -> Self {
        let initial_tokens = config.initial_tokens.unwrap_or(config.capacity);
        
        Self {
            tokens: initial_tokens as f64,
            capacity: config.capacity,
            refill_rate: config.refill_amount as f64 / config.refill_interval.as_secs_f64(),
            last_refill: now,
            burst_tokens_used: 0,
            burst_cooldown_until: None,
            request_count: 0,
            window_start: now,
        }
    }

    fn refill(&mut self, config: &BucketConfig, now: SystemTime) {
        let elapsed = now.duration_since(self.last_refill)
            .unwrap_or_default()
            .as_secs_f64();
        
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(config.capacity as f64);
        self.last_refill = now;

        // Reset burst cooldown if expired
        if let Some(cooldown_until) = self.burst_cooldown_until {
            if now >= cooldown_until {
                self.burst_tokens_used = 0;
                self.burst_cooldown_until = None;
            }
        }
    }

    fn can_consume(&self, tokens: u64, config: &BucketConfig) -> bool {
        if self.tokens >= tokens as f64 {
            return true;
        }

        // Check if burst is allowed and available
        if config.allow_burst {
            if let Some(burst_capacity) = config.burst_capacity {
                let burst_available = burst_capacity.saturating_sub(self.burst_tokens_used);
                let tokens_needed = tokens as f64 - self.tokens;
                return tokens_needed <= burst_available as f64;
            }
        }

        false
    }

    fn consume(&mut self, tokens: u64, config: &BucketConfig) {
        let tokens_f64 = tokens as f64;
        
        if self.tokens >= tokens_f64 {
            // Normal consumption
            self.tokens -= tokens_f64;
        } else if config.allow_burst {
            // Burst consumption
            let burst_needed = tokens_f64 - self.tokens;
            self.tokens = 0.0;
            self.burst_tokens_used += burst_needed as u64;
            
            // Set burst cooldown
            if let Some(cooldown_duration) = config.burst_cooldown {
                self.burst_cooldown_until = Some(SystemTime::now() + cooldown_duration);
            }
        }

        self.request_count += 1;
    }

    fn time_until_refill(&self, config: &BucketConfig) -> Duration {
        if self.tokens >= config.capacity as f64 {
            return Duration::ZERO;
        }

        let tokens_needed = config.capacity as f64 - self.tokens;
        let time_needed = tokens_needed / self.refill_rate;
        Duration::from_secs_f64(time_needed)
    }

    fn next_refill_time(&self, config: &BucketConfig) -> SystemTime {
        self.last_refill + self.time_until_refill(config)
    }
}

// Default configurations for common use cases
impl Default for BucketConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            refill_amount: 10,
            refill_interval: Duration::from_secs(60),
            initial_tokens: None,
            allow_burst: false,
            burst_capacity: None,
            burst_cooldown: None,
        }
    }
}

impl Default for PenaltyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            violation_threshold: 5,
            base_penalty_duration: Duration::from_secs(300), // 5 minutes
            escalation_multiplier: 2.0,
            max_penalty_duration: Duration::from_secs(3600 * 24), // 24 hours
            violation_window: Duration::from_secs(3600), // 1 hour
            penalty_types: HashMap::from([
                ("rate_limit_violation".to_string(), 1.0),
                ("severe_violation".to_string(), 2.0),
                ("abuse_detection".to_string(), 5.0),
            ]),
        }
    }
}

impl Default for RateLimitRules {
    fn default() -> Self {
        Self {
            default: BucketConfig::default(),
            endpoints: HashMap::new(),
            did_overrides: HashMap::new(),
            global_limit: None,
        }
    }
}
