use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;

/// Privacy budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBudgetConfig {
    /// Global privacy budget settings
    pub global: GlobalBudgetConfig,
    /// API-specific budget configurations
    pub apis: HashMap<String, ApiBudgetConfig>,
    /// User-specific budget configurations
    pub users: HashMap<String, UserBudgetConfig>,
    /// Privacy budget refresh intervals
    pub refresh: RefreshConfig,
    /// Differential privacy parameters
    pub differential_privacy: DifferentialPrivacyConfig,
}

/// Global budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalBudgetConfig {
    /// Total global privacy budget per time window
    pub total_budget: f64,
    /// Time window for global budget (in seconds)
    pub time_window: u64,
    /// Maximum budget per request
    pub max_per_request: f64,
    /// Minimum budget per request
    pub min_per_request: f64,
    /// Enable strict mode (reject requests when budget exhausted)
    pub strict_mode: bool,
}

/// API-specific budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBudgetConfig {
    /// API endpoint path
    pub path: String,
    /// HTTP methods allowed
    pub methods: Vec<String>,
    /// Privacy budget per time window
    pub budget: f64,
    /// Time window for this API (in seconds)
    pub time_window: u64,
    /// Cost per request
    pub cost_per_request: f64,
    /// Maximum requests per time window
    pub max_requests: Option<u32>,
    /// Enable rate limiting
    pub rate_limit: bool,
    /// Rate limit burst size
    pub burst_size: Option<u32>,
}

/// User-specific budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBudgetConfig {
    /// User identifier
    pub user_id: String,
    /// Personal privacy budget per time window
    pub personal_budget: f64,
    /// Time window for personal budget (in seconds)
    pub time_window: u64,
    /// Cost per request for this user
    pub cost_per_request: f64,
    /// Maximum requests per time window
    pub max_requests: Option<u32>,
    /// Priority level (higher = more budget)
    pub priority: u8,
}

/// Refresh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshConfig {
    /// Enable automatic budget refresh
    pub auto_refresh: bool,
    /// Refresh interval (in seconds)
    pub refresh_interval: u64,
    /// Enable partial refresh (gradual budget restoration)
    pub partial_refresh: bool,
    /// Partial refresh rate (budget restored per second)
    pub partial_refresh_rate: f64,
}

/// Differential privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialPrivacyConfig {
    /// Epsilon parameter for differential privacy
    pub epsilon: f64,
    /// Delta parameter for differential privacy
    pub delta: f64,
    /// Sensitivity parameter
    pub sensitivity: f64,
    /// Noise distribution type
    pub noise_type: NoiseType,
    /// Enable adaptive noise scaling
    pub adaptive_noise: bool,
}

/// Noise distribution types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseType {
    Laplace,
    Gaussian,
    Exponential,
}

/// Privacy budget manager
#[derive(Debug)]
pub struct PrivacyBudgetManager {
    config: PrivacyBudgetConfig,
    global_budget: Arc<RwLock<TokenBucket>>,
    api_budgets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    user_budgets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    metrics: Arc<RwLock<PrivacyMetrics>>,
    last_refresh: Arc<RwLock<Instant>>,
}

/// Token bucket implementation for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current token count
    tokens: f64,
    /// Maximum token capacity
    capacity: f64,
    /// Token refill rate (tokens per second)
    refill_rate: f64,
    /// Last refill timestamp
    last_refill: Instant,
    /// Time window for this bucket
    time_window: Duration,
}

/// Privacy budget request
#[derive(Debug, Clone)]
pub struct BudgetRequest {
    /// Unique request identifier
    pub request_id: String,
    /// API endpoint path
    pub api_path: String,
    /// HTTP method
    pub method: String,
    /// User identifier (if authenticated)
    pub user_id: Option<String>,
    /// Request timestamp
    pub timestamp: Instant,
    /// Request cost (privacy budget units)
    pub cost: f64,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Privacy budget response
#[derive(Debug, Clone)]
pub struct BudgetResponse {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining budget for the API
    pub remaining_api_budget: f64,
    /// Remaining global budget
    pub remaining_global_budget: f64,
    /// Remaining user budget (if applicable)
    pub remaining_user_budget: Option<f64>,
    /// Rate limit headers
    pub rate_limit_headers: HashMap<String, String>,
    /// Privacy budget headers
    pub privacy_headers: HashMap<String, String>,
    /// Error message if not allowed
    pub error_message: Option<String>,
}

/// Privacy metrics
#[derive(Debug, Clone, Serialize)]
pub struct PrivacyMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests allowed
    pub requests_allowed: u64,
    /// Requests denied
    pub requests_denied: u64,
    /// Total budget consumed
    pub total_budget_consumed: f64,
    /// Current global budget usage
    pub global_budget_usage: f64,
    /// API-specific budget usage
    pub api_budget_usage: HashMap<String, f64>,
    /// User-specific budget usage
    pub user_budget_usage: HashMap<String, f64>,
    /// Rate limiting statistics
    pub rate_limit_stats: RateLimitStats,
    /// Privacy budget violations
    pub budget_violations: u64,
    /// Last metrics update
    pub last_update: SystemTime,
}

/// Rate limiting statistics
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitStats {
    /// Total rate limit hits
    pub total_hits: u64,
    /// Rate limit hits by API
    pub hits_by_api: HashMap<String, u64>,
    /// Rate limit hits by user
    pub hits_by_user: HashMap<String, u64>,
    /// Average response time for rate-limited requests
    pub avg_response_time: Duration,
}

/// Privacy budget errors
#[derive(Error, Debug)]
pub enum PrivacyBudgetError {
    #[error("Insufficient privacy budget: {message}")]
    InsufficientBudget { message: String },
    
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("API not found: {api_path}")]
    ApiNotFound { api_path: String },
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },
    
    #[error("Invalid budget request: {message}")]
    InvalidRequest { message: String },
    
    #[error("Differential privacy error: {message}")]
    DifferentialPrivacyError { message: String },
}

impl PrivacyBudgetManager {
    /// Create a new privacy budget manager
    pub fn new(config: PrivacyBudgetConfig) -> Result<Self, PrivacyBudgetError> {
        // Validate configuration
        Self::validate_config(&config)?;
        
        let now = Instant::now();
        
        // Initialize global budget
        let global_budget = Arc::new(RwLock::new(TokenBucket::new(
            config.global.total_budget,
            config.global.total_budget,
            0.0, // Global budget doesn't auto-refill
            Duration::from_secs(config.global.time_window),
        )));
        
        // Initialize API budgets
        let mut api_budgets = HashMap::new();
        for (api_name, api_config) in &config.apis {
            let bucket = TokenBucket::new(
                api_config.budget,
                api_config.budget,
                if api_config.rate_limit {
                    api_config.budget / api_config.time_window as f64
                } else {
                    0.0
                },
                Duration::from_secs(api_config.time_window),
            );
            api_budgets.insert(api_name.clone(), bucket);
        }
        let api_budgets = Arc::new(RwLock::new(api_budgets));
        
        // Initialize user budgets
        let mut user_budgets = HashMap::new();
        for (user_id, user_config) in &config.users {
            let bucket = TokenBucket::new(
                user_config.personal_budget,
                user_config.personal_budget,
                if config.refresh.partial_refresh {
                    config.refresh.partial_refresh_rate
                } else {
                    0.0
                },
                Duration::from_secs(user_config.time_window),
            );
            user_budgets.insert(user_id.clone(), bucket);
        }
        let user_budgets = Arc::new(RwLock::new(user_budgets));
        
        // Initialize metrics
        let metrics = Arc::new(RwLock::new(PrivacyMetrics::new()));
        let last_refresh = Arc::new(RwLock::new(now));
        
        Ok(Self {
            config,
            global_budget,
            api_budgets,
            user_budgets,
            metrics,
            last_refresh,
        })
    }
    
    /// Validate configuration
    fn validate_config(config: &PrivacyBudgetConfig) -> Result<(), PrivacyBudgetError> {
        if config.global.total_budget <= 0.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Global budget must be positive".to_string(),
            });
        }
        
        if config.global.time_window == 0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Global time window must be positive".to_string(),
            });
        }
        
        for (api_name, api_config) in &config.apis {
            if api_config.budget <= 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("API {} budget must be positive", api_name),
                });
            }
            
            if api_config.cost_per_request < 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("API {} cost per request must be non-negative", api_name),
                });
            }
        }
        
        for (user_id, user_config) in &config.users {
            if user_config.personal_budget <= 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("User {} budget must be positive", user_id),
                });
            }
            
            if user_config.cost_per_request < 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("User {} cost per request must be non-negative", user_id),
                });
            }
        }
        
        Ok(())
    }
    
    /// Check if a request is allowed based on privacy budgets
    pub async fn check_budget(&self, request: &BudgetRequest) -> Result<BudgetResponse, PrivacyBudgetError> {
        let start_time = Instant::now();
        
        // Update metrics
        self.update_metrics(request).await;
        
        // Check global budget
        let global_allowed = self.check_global_budget(request.cost).await?;
        if !global_allowed {
            return self.create_denied_response(request, "Global privacy budget exhausted").await;
        }
        
        // Check API budget
        let api_allowed = self.check_api_budget(&request.api_path, request.cost).await?;
        if !api_allowed {
            return self.create_denied_response(request, "API privacy budget exhausted").await;
        }
        
        // Check user budget if applicable
        if let Some(user_id) = &request.user_id {
            let user_allowed = self.check_user_budget(user_id, request.cost).await?;
            if !user_allowed {
                return self.create_denied_response(request, "User privacy budget exhausted").await;
            }
        }
        
        // Check rate limits
        let rate_limit_allowed = self.check_rate_limits(request).await?;
        if !rate_limit_allowed {
            return self.create_denied_response(request, "Rate limit exceeded").await;
        }
        
        // Consume budgets
        self.consume_budgets(request).await?;
        
        // Create allowed response
        let response = self.create_allowed_response(request).await;
        
        // Update metrics with response time
        let response_time = start_time.elapsed();
        self.update_response_time_metrics(response_time).await;
        
        Ok(response)
    }
    
    /// Check global budget
    async fn check_global_budget(&self, cost: f64) -> Result<bool, PrivacyBudgetError> {
        let mut global_budget = self.global_budget.write().await;
        global_budget.refill();
        
        if global_budget.tokens >= cost {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Check API budget
    async fn check_api_budget(&self, api_path: &str, cost: f64) -> Result<bool, PrivacyBudgetError> {
        let mut api_budgets = self.api_budgets.write().await;
        
        // Find matching API configuration
        let api_config = self.find_api_config(api_path)?;
        let api_name = self.get_api_name_from_path(api_path);
        
        if let Some(bucket) = api_budgets.get_mut(&api_name) {
            bucket.refill();
            
            if bucket.tokens >= cost {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            // Use default budget if no specific configuration
            Ok(true)
        }
    }
    
    /// Check user budget
    async fn check_user_budget(&self, user_id: &str, cost: f64) -> Result<bool, PrivacyBudgetError> {
        let mut user_budgets = self.user_budgets.write().await;
        
        if let Some(bucket) = user_budgets.get_mut(user_id) {
            bucket.refill();
            
            if bucket.tokens >= cost {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            // User not found, use default behavior
            Ok(true)
        }
    }
    
    /// Check rate limits
    async fn check_rate_limits(&self, request: &BudgetRequest) -> Result<bool, PrivacyBudgetError> {
        let api_config = self.find_api_config(&request.api_path)?;
        
        if !api_config.rate_limit {
            return Ok(true);
        }
        
        // Simple rate limiting based on request count
        // In a production system, you'd use a more sophisticated rate limiter
        let mut metrics = self.metrics.write().await;
        
        let api_requests = metrics.api_budget_usage.get(&request.api_path).unwrap_or(&0.0);
        if let Some(max_requests) = api_config.max_requests {
            if *api_requests >= max_requests as f64 {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Consume budgets for an allowed request
    async fn consume_budgets(&self, request: &BudgetRequest) -> Result<(), PrivacyBudgetError> {
        // Consume global budget
        {
            let mut global_budget = self.global_budget.write().await;
            global_budget.tokens -= request.cost;
        }
        
        // Consume API budget
        {
            let mut api_budgets = self.api_budgets.write().await;
            let api_name = self.get_api_name_from_path(&request.api_path);
            if let Some(bucket) = api_budgets.get_mut(&api_name) {
                bucket.tokens -= request.cost;
            }
        }
        
        // Consume user budget if applicable
        if let Some(user_id) = &request.user_id {
            let mut user_budgets = self.user_budgets.write().await;
            if let Some(bucket) = user_budgets.get_mut(user_id) {
                bucket.tokens -= request.cost;
            }
        }
        
        Ok(())
    }
    
    /// Find API configuration for a given path
    fn find_api_config(&self, api_path: &str) -> Result<&ApiBudgetConfig, PrivacyBudgetError> {
        // Find the most specific matching API configuration
        let mut best_match = None;
        let mut best_length = 0;
        
        for (_, api_config) in &self.config.apis {
            if api_path.starts_with(&api_config.path) && api_config.path.len() > best_length {
                best_match = Some(api_config);
                best_length = api_config.path.len();
            }
        }
        
        best_match.ok_or_else(|| PrivacyBudgetError::ApiNotFound {
            api_path: api_path.to_string(),
        })
    }
    
    /// Get API name from path
    fn get_api_name_from_path(&self, api_path: &str) -> String {
        // Extract API name from path (e.g., "/api/v1/users" -> "users")
        api_path.split('/').filter(|s| !s.is_empty()).last().unwrap_or("default").to_string()
    }
    
    /// Create allowed response
    async fn create_allowed_response(&self, request: &BudgetRequest) -> BudgetResponse {
        let mut rate_limit_headers = HashMap::new();
        let mut privacy_headers = HashMap::new();
        
        // Get remaining budgets
        let remaining_global = {
            let global_budget = self.global_budget.read().await;
            global_budget.tokens
        };
        
        let remaining_api = {
            let api_budgets = self.api_budgets.read().await;
            let api_name = self.get_api_name_from_path(&request.api_path);
            api_budgets.get(&api_name).map(|b| b.tokens).unwrap_or(f64::INFINITY)
        };
        
        let remaining_user = if let Some(user_id) = &request.user_id {
            let user_budgets = self.user_budgets.read().await;
            user_budgets.get(user_id).map(|b| b.tokens)
        } else {
            None
        };
        
        // Set rate limit headers
        rate_limit_headers.insert("X-RateLimit-Limit".to_string(), "1000".to_string());
        rate_limit_headers.insert("X-RateLimit-Remaining".to_string(), "999".to_string());
        rate_limit_headers.insert("X-RateLimit-Reset".to_string(), 
            (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600).to_string());
        
        // Set privacy budget headers
        privacy_headers.insert("X-Privacy-Budget-Global".to_string(), remaining_global.to_string());
        privacy_headers.insert("X-Privacy-Budget-API".to_string(), remaining_api.to_string());
        if let Some(user_budget) = remaining_user {
            privacy_headers.insert("X-Privacy-Budget-User".to_string(), user_budget.to_string());
        }
        privacy_headers.insert("X-Privacy-Budget-Consumed".to_string(), request.cost.to_string());
        
        BudgetResponse {
            allowed: true,
            remaining_api_budget: remaining_api,
            remaining_global_budget: remaining_global,
            remaining_user_budget: remaining_user,
            rate_limit_headers,
            privacy_headers,
            error_message: None,
        }
    }
    
    /// Create denied response
    async fn create_denied_response(&self, request: &BudgetRequest, reason: &str) -> BudgetResponse {
        let mut rate_limit_headers = HashMap::new();
        let mut privacy_headers = HashMap::new();
        
        // Set error headers
        rate_limit_headers.insert("X-RateLimit-Limit".to_string(), "0".to_string());
        rate_limit_headers.insert("X-RateLimit-Remaining".to_string(), "0".to_string());
        rate_limit_headers.insert("Retry-After".to_string(), "3600".to_string());
        
        privacy_headers.insert("X-Privacy-Budget-Error".to_string(), reason.to_string());
        
        BudgetResponse {
            allowed: false,
            remaining_api_budget: 0.0,
            remaining_global_budget: 0.0,
            remaining_user_budget: None,
            rate_limit_headers,
            privacy_headers,
            error_message: Some(reason.to_string()),
        }
    }
    
    /// Update metrics
    async fn update_metrics(&self, request: &BudgetRequest) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_requests += 1;
        metrics.last_update = SystemTime::now();
        
        // Update API usage
        let api_name = self.get_api_name_from_path(&request.api_path);
        *metrics.api_budget_usage.entry(api_name).or_insert(0.0) += request.cost;
        
        // Update user usage if applicable
        if let Some(user_id) = &request.user_id {
            *metrics.user_budget_usage.entry(user_id.clone()).or_insert(0.0) += request.cost;
        }
    }
    
    /// Update response time metrics
    async fn update_response_time_metrics(&self, response_time: Duration) {
        let mut metrics = self.metrics.write().await;
        
        // Simple moving average for response time
        let current_avg = metrics.rate_limit_stats.avg_response_time;
        let new_avg = Duration::from_millis(
            (current_avg.as_millis() + response_time.as_millis()) / 2
        );
        metrics.rate_limit_stats.avg_response_time = new_avg;
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> PrivacyMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Refresh budgets (called periodically)
    pub async fn refresh_budgets(&self) -> Result<(), PrivacyBudgetError> {
        if !self.config.refresh.auto_refresh {
            return Ok(());
        }
        
        let now = Instant::now();
        let mut last_refresh = self.last_refresh.write().await;
        
        if now.duration_since(*last_refresh).as_secs() < self.config.refresh.refresh_interval {
            return Ok(());
        }
        
        // Refresh global budget
        {
            let mut global_budget = self.global_budget.write().await;
            global_budget.tokens = self.config.global.total_budget;
        }
        
        // Refresh API budgets
        {
            let mut api_budgets = self.api_budgets.write().await;
            for (api_name, bucket) in api_budgets.iter_mut() {
                if let Some(api_config) = self.config.apis.get(api_name) {
                    bucket.tokens = api_config.budget;
                }
            }
        }
        
        // Refresh user budgets
        {
            let mut user_budgets = self.user_budgets.write().await;
            for (user_id, bucket) in user_budgets.iter_mut() {
                if let Some(user_config) = self.config.users.get(user_id) {
                    bucket.tokens = user_config.personal_budget;
                }
            }
        }
        
        *last_refresh = now;
        Ok(())
    }
    
    /// Add noise for differential privacy
    pub fn add_differential_privacy_noise(&self, value: f64) -> Result<f64, PrivacyBudgetError> {
        let dp_config = &self.config.differential_privacy;
        
        match dp_config.noise_type {
            NoiseType::Laplace => {
                let scale = dp_config.sensitivity / dp_config.epsilon;
                let noise = self.sample_laplace(scale)?;
                Ok(value + noise)
            }
            NoiseType::Gaussian => {
                let scale = (dp_config.sensitivity * (2.0 * dp_config.delta.ln()).sqrt()) / dp_config.epsilon;
                let noise = self.sample_gaussian(0.0, scale)?;
                Ok(value + noise)
            }
            NoiseType::Exponential => {
                let scale = dp_config.sensitivity / dp_config.epsilon;
                let noise = self.sample_exponential(scale)?;
                Ok(value + noise)
            }
        }
    }
    
    /// Sample from Laplace distribution
    fn sample_laplace(&self, scale: f64) -> Result<f64, PrivacyBudgetError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let u = rng.gen_range(-0.5..0.5);
        let noise = -scale * (1.0 - 2.0 * u.abs()).ln() * u.signum();
        
        Ok(noise)
    }
    
    /// Sample from Gaussian distribution
    fn sample_gaussian(&self, mean: f64, std_dev: f64) -> Result<f64, PrivacyBudgetError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Box-Muller transform
        let u1 = rng.gen_range(0.0..1.0);
        let u2 = rng.gen_range(0.0..1.0);
        
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let noise = mean + std_dev * z0;
        
        Ok(noise)
    }
    
    /// Sample from Exponential distribution
    fn sample_exponential(&self, scale: f64) -> Result<f64, PrivacyBudgetError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let u = rng.gen_range(0.0..1.0);
        let noise = -scale * u.ln();
        
        Ok(noise)
    }
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(capacity: f64, initial_tokens: f64, refill_rate: f64, time_window: Duration) -> Self {
        Self {
            tokens: initial_tokens,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
            time_window,
        }
    }
    
    /// Refill tokens based on time elapsed
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        
        if elapsed >= self.time_window {
            // Full refill
            self.tokens = self.capacity;
        } else {
            // Partial refill
            let elapsed_secs = elapsed.as_secs_f64();
            let refill_amount = self.refill_rate * elapsed_secs;
            self.tokens = (self.tokens + refill_amount).min(self.capacity);
        }
        
        self.last_refill = now;
    }
}

impl PrivacyMetrics {
    /// Create new privacy metrics
    fn new() -> Self {
        Self {
            total_requests: 0,
            requests_allowed: 0,
            requests_denied: 0,
            total_budget_consumed: 0.0,
            global_budget_usage: 0.0,
            api_budget_usage: HashMap::new(),
            user_budget_usage: HashMap::new(),
            rate_limit_stats: RateLimitStats::new(),
            budget_violations: 0,
            last_update: SystemTime::now(),
        }
    }
}

impl RateLimitStats {
    /// Create new rate limit statistics
    fn new() -> Self {
        Self {
            total_hits: 0,
            hits_by_api: HashMap::new(),
            hits_by_user: HashMap::new(),
            avg_response_time: Duration::from_millis(0),
        }
    }
}

impl Default for PrivacyBudgetConfig {
    fn default() -> Self {
        Self {
            global: GlobalBudgetConfig {
                total_budget: 1000.0,
                time_window: 3600, // 1 hour
                max_per_request: 100.0,
                min_per_request: 0.1,
                strict_mode: true,
            },
            apis: HashMap::new(),
            users: HashMap::new(),
            refresh: RefreshConfig {
                auto_refresh: true,
                refresh_interval: 3600, // 1 hour
                partial_refresh: true,
                partial_refresh_rate: 0.1, // 0.1 tokens per second
            },
            differential_privacy: DifferentialPrivacyConfig {
                epsilon: 1.0,
                delta: 1e-5,
                sensitivity: 1.0,
                noise_type: NoiseType::Laplace,
                adaptive_noise: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_privacy_budget_manager_creation() {
        let config = PrivacyBudgetConfig::default();
        let manager = PrivacyBudgetManager::new(config).unwrap();
        
        assert!(manager.config.global.total_budget > 0.0);
        assert!(manager.config.global.time_window > 0);
    }
    
    #[tokio::test]
    async fn test_budget_request_allowed() {
        let config = PrivacyBudgetConfig::default();
        let manager = PrivacyBudgetManager::new(config).unwrap();
        
        let request = BudgetRequest {
            request_id: Uuid::new_v4().to_string(),
            api_path: "/api/test".to_string(),
            method: "GET".to_string(),
            user_id: None,
            timestamp: Instant::now(),
            cost: 1.0,
            metadata: HashMap::new(),
        };
        
        let response = manager.check_budget(&request).await.unwrap();
        assert!(response.allowed);
        assert!(response.remaining_global_budget > 0.0);
    }
    
    #[tokio::test]
    async fn test_budget_request_denied() {
        let mut config = PrivacyBudgetConfig::default();
        config.global.total_budget = 0.1; // Very small budget
        
        let manager = PrivacyBudgetManager::new(config).unwrap();
        
        let request = BudgetRequest {
            request_id: Uuid::new_v4().to_string(),
            api_path: "/api/test".to_string(),
            method: "GET".to_string(),
            user_id: None,
            timestamp: Instant::now(),
            cost: 1.0, // Cost exceeds budget
            metadata: HashMap::new(),
        };
        
        let response = manager.check_budget(&request).await.unwrap();
        assert!(!response.allowed);
        assert!(response.error_message.is_some());
    }
    
    #[tokio::test]
    async fn test_differential_privacy_noise() {
        let config = PrivacyBudgetConfig::default();
        let manager = PrivacyBudgetManager::new(config).unwrap();
        
        let original_value = 100.0;
        let noisy_value = manager.add_differential_privacy_noise(original_value).unwrap();
        
        // Noise should be added (value should be different)
        assert_ne!(original_value, noisy_value);
        
        // But should be reasonably close
        let difference = (original_value - noisy_value).abs();
        assert!(difference < 100.0); // Reasonable noise level
    }
    
    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(100.0, 50.0, 10.0, Duration::from_secs(1));
        
        // Initial state
        assert_eq!(bucket.tokens, 50.0);
        
        // Consume some tokens
        bucket.tokens -= 20.0;
        assert_eq!(bucket.tokens, 30.0);
        
        // Wait and refill
        std::thread::sleep(Duration::from_millis(100));
        bucket.refill();
        
        // Should have refilled some tokens
        assert!(bucket.tokens > 30.0);
    }
}
