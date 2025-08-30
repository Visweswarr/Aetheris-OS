//! Privacy Budget System for Polymera OS
//! 
//! This module provides differential privacy protection through token bucket rate limiting
//! and privacy budget management, ensuring APIs respect privacy constraints and prevent
//! data leakage through excessive queries.

pub mod budget;
pub mod middleware;

// Re-export main types
pub use budget::{
    PrivacyBudgetManager,
    PrivacyBudgetConfig,
    BudgetRequest,
    BudgetResponse,
    PrivacyBudgetError,
    PrivacyMetrics,
    GlobalBudgetConfig,
    ApiBudgetConfig,
    UserBudgetConfig,
    RefreshConfig,
    DifferentialPrivacyConfig,
    NoiseType,
};

pub use middleware::{
    PrivacyBudgetMiddleware,
    MiddlewareConfig,
    HttpRequestContext,
    HttpResponseContext,
    MiddlewareMetrics,
    PathCostPattern,
    HeaderCostPattern,
    QueryParamCostPattern,
    BodySizeThreshold,
    AuditLog,
};

// Re-export integrations
pub use middleware::integrations;

/// Privacy budget system version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Privacy budget system description
pub const DESCRIPTION: &str = "Differential privacy protection through token bucket rate limiting";

/// Privacy budget system author
pub const AUTHOR: &str = "Polymera OS Team";

/// Privacy budget system license
pub const LICENSE: &str = "MIT OR Apache-2.0";

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "privacy/config.yaml";

/// Default privacy budget refresh interval (1 hour)
pub const DEFAULT_REFRESH_INTERVAL: u64 = 3600;

/// Default differential privacy epsilon
pub const DEFAULT_EPSILON: f64 = 1.0;

/// Default differential privacy delta
pub const DEFAULT_DELTA: f64 = 1e-5;

/// Default sensitivity parameter
pub const DEFAULT_SENSITIVITY: f64 = 1.0;

/// Maximum privacy budget per request
pub const MAX_BUDGET_PER_REQUEST: f64 = 1000.0;

/// Minimum privacy budget per request
pub const MIN_BUDGET_PER_REQUEST: f64 = 0.1;

/// Privacy budget violation HTTP status code
pub const BUDGET_VIOLATION_STATUS: u16 = 429; // Too Many Requests

/// Privacy budget headers
pub mod headers {
    /// Global privacy budget header
    pub const GLOBAL_BUDGET: &str = "X-Privacy-Budget-Global";
    
    /// API privacy budget header
    pub const API_BUDGET: &str = "X-Privacy-Budget-API";
    
    /// User privacy budget header
    pub const USER_BUDGET: &str = "X-Privacy-Budget-User";
    
    /// Consumed budget header
    pub const CONSUMED_BUDGET: &str = "X-Privacy-Budget-Consumed";
    
    /// Budget error header
    pub const BUDGET_ERROR: &str = "X-Privacy-Budget-Error";
    
    /// Budget bypassed header
    pub const BUDGET_BYPASSED: &str = "X-Privacy-Budget-Bypassed";
    
    /// Rate limit headers
    pub const RATE_LIMIT_LIMIT: &str = "X-RateLimit-Limit";
    pub const RATE_LIMIT_REMAINING: &str = "X-RateLimit-Remaining";
    pub const RATE_LIMIT_RESET: &str = "X-RateLimit-Reset";
    pub const RETRY_AFTER: &str = "Retry-After";
}

/// Privacy budget error codes
pub mod error_codes {
    /// Insufficient budget error
    pub const INSUFFICIENT_BUDGET: &str = "INSUFFICIENT_BUDGET";
    
    /// Rate limit exceeded error
    pub const RATE_LIMIT_EXCEEDED: &str = "RATE_LIMIT_EXCEEDED";
    
    /// Configuration error
    pub const CONFIGURATION_ERROR: &str = "CONFIGURATION_ERROR";
    
    /// API not found error
    pub const API_NOT_FOUND: &str = "API_NOT_FOUND";
    
    /// User not found error
    pub const USER_NOT_FOUND: &str = "USER_NOT_FOUND";
    
    /// Invalid request error
    pub const INVALID_REQUEST: &str = "INVALID_REQUEST";
    
    /// Differential privacy error
    pub const DIFFERENTIAL_PRIVACY_ERROR: &str = "DIFFERENTIAL_PRIVACY_ERROR";
}

/// Privacy budget metrics names
pub mod metrics {
    /// Total requests metric
    pub const TOTAL_REQUESTS: &str = "privacy_budget_total_requests";
    
    /// Requests allowed metric
    pub const REQUESTS_ALLOWED: &str = "privacy_budget_requests_allowed";
    
    /// Requests denied metric
    pub const REQUESTS_DENIED: &str = "privacy_budget_requests_denied";
    
    /// Budget consumed metric
    pub const BUDGET_CONSUMED: &str = "privacy_budget_consumed";
    
    /// Global budget usage metric
    pub const GLOBAL_BUDGET_USAGE: &str = "privacy_budget_global_usage";
    
    /// API budget usage metric
    pub const API_BUDGET_USAGE: &str = "privacy_budget_api_usage";
    
    /// User budget usage metric
    pub const USER_BUDGET_USAGE: &str = "privacy_budget_user_usage";
    
    /// Budget violations metric
    pub const BUDGET_VIOLATIONS: &str = "privacy_budget_violations";
    
    /// Rate limit hits metric
    pub const RATE_LIMIT_HITS: &str = "privacy_budget_rate_limit_hits";
    
    /// Response time metric
    pub const RESPONSE_TIME: &str = "privacy_budget_response_time";
}

/// Privacy budget configuration validation
pub mod validation {
    use super::*;
    
    /// Validate privacy budget configuration
    pub fn validate_config(config: &PrivacyBudgetConfig) -> Result<(), PrivacyBudgetError> {
        // Global budget validation
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
        
        if config.global.max_per_request <= 0.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Maximum budget per request must be positive".to_string(),
            });
        }
        
        if config.global.min_per_request < 0.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Minimum budget per request must be non-negative".to_string(),
            });
        }
        
        // API budget validation
        for (api_name, api_config) in &config.apis {
            if api_config.budget <= 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("API {} budget must be positive", api_name),
                });
            }
            
            if api_config.time_window == 0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("API {} time window must be positive", api_name),
                });
            }
            
            if api_config.cost_per_request < 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("API {} cost per request must be non-negative", api_name),
                });
            }
        }
        
        // User budget validation
        for (user_id, user_config) in &config.users {
            if user_config.personal_budget <= 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("User {} budget must be positive", user_id),
                });
            }
            
            if user_config.time_window == 0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("User {} time window must be positive", user_id),
                });
            }
            
            if user_config.cost_per_request < 0.0 {
                return Err(PrivacyBudgetError::ConfigurationError {
                    message: format!("User {} cost per request must be non-negative", user_id),
                });
            }
        }
        
        // Differential privacy validation
        if config.differential_privacy.epsilon <= 0.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Epsilon must be positive".to_string(),
            });
        }
        
        if config.differential_privacy.delta <= 0.0 || config.differential_privacy.delta >= 1.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Delta must be between 0 and 1".to_string(),
            });
        }
        
        if config.differential_privacy.sensitivity <= 0.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Sensitivity must be positive".to_string(),
            });
        }
        
        // Refresh configuration validation
        if config.refresh.refresh_interval == 0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Refresh interval must be positive".to_string(),
            });
        }
        
        if config.refresh.partial_refresh_rate < 0.0 {
            return Err(PrivacyBudgetError::ConfigurationError {
                message: "Partial refresh rate must be non-negative".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Privacy budget utilities
pub mod utils {
    use super::*;
    
    /// Calculate privacy budget cost based on request characteristics
    pub fn calculate_request_cost(
        method: &str,
        path: &str,
        user_id: Option<&str>,
        body_size: usize,
        query_params: &[(&str, &str)],
        headers: &std::collections::HashMap<String, String>,
    ) -> f64 {
        let mut cost = 1.0; // Base cost
        
        // Method-based cost
        cost *= match method {
            "GET" => 1.0,
            "POST" => 2.0,
            "PUT" => 1.5,
            "DELETE" => 1.5,
            "PATCH" => 1.2,
            _ => 1.0,
        };
        
        // Path-based cost
        if path.contains("/analytics") || path.contains("/ml") {
            cost *= 2.0; // Higher cost for data-intensive endpoints
        }
        
        if path.contains("/health") || path.contains("/status") {
            cost *= 0.1; // Lower cost for health checks
        }
        
        // Body size cost
        if body_size > 0 {
            cost += (body_size as f64 / 1024.0) * 0.1; // 0.1 per KB
        }
        
        // Query parameter cost
        cost += query_params.len() as f64 * 0.1;
        
        // Header-based cost adjustments
        if let Some(auth_header) = headers.get("Authorization") {
            if auth_header.starts_with("Bearer ") {
                cost *= 0.8; // Authenticated requests get discount
            }
        }
        
        cost.max(MIN_BUDGET_PER_REQUEST).min(MAX_BUDGET_PER_REQUEST)
    }
    
    /// Format privacy budget headers
    pub fn format_privacy_headers(
        global_budget: f64,
        api_budget: f64,
        user_budget: Option<f64>,
        consumed: f64,
    ) -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();
        
        headers.insert(headers::GLOBAL_BUDGET.to_string(), global_budget.to_string());
        headers.insert(headers::API_BUDGET.to_string(), api_budget.to_string());
        if let Some(user_budget) = user_budget {
            headers.insert(headers::USER_BUDGET.to_string(), user_budget.to_string());
        }
        headers.insert(headers::CONSUMED_BUDGET.to_string(), consumed.to_string());
        
        headers
    }
    
    /// Parse privacy budget headers from response
    pub fn parse_privacy_headers(
        headers: &std::collections::HashMap<String, String>,
    ) -> std::collections::HashMap<String, f64> {
        let mut budget_info = std::collections::HashMap::new();
        
        for (key, value) in headers {
            if key.starts_with("X-Privacy-Budget-") {
                if let Ok(parsed_value) = value.parse::<f64>() {
                    budget_info.insert(key.clone(), parsed_value);
                }
            }
        }
        
        budget_info
    }
    
    /// Check if privacy budget is exhausted
    pub fn is_budget_exhausted(remaining_budget: f64, threshold: f64) -> bool {
        remaining_budget <= threshold
    }
    
    /// Calculate budget usage percentage
    pub fn calculate_budget_usage_percentage(used: f64, total: f64) -> f64 {
        if total == 0.0 {
            0.0
        } else {
            (used / total) * 100.0
        }
    }
    
    /// Estimate remaining requests based on budget
    pub fn estimate_remaining_requests(remaining_budget: f64, cost_per_request: f64) -> u32 {
        if cost_per_request <= 0.0 {
            0
        } else {
            (remaining_budget / cost_per_request) as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!DESCRIPTION.is_empty());
        assert!(!AUTHOR.is_empty());
        assert!(!LICENSE.is_empty());
    }
    
    #[test]
    fn test_default_constants() {
        assert!(DEFAULT_REFRESH_INTERVAL > 0);
        assert!(DEFAULT_EPSILON > 0.0);
        assert!(DEFAULT_DELTA > 0.0 && DEFAULT_DELTA < 1.0);
        assert!(DEFAULT_SENSITIVITY > 0.0);
        assert!(MAX_BUDGET_PER_REQUEST > MIN_BUDGET_PER_REQUEST);
    }
    
    #[test]
    fn test_headers_constants() {
        assert!(!headers::GLOBAL_BUDGET.is_empty());
        assert!(!headers::API_BUDGET.is_empty());
        assert!(!headers::USER_BUDGET.is_empty());
        assert!(!headers::CONSUMED_BUDGET.is_empty());
    }
    
    #[test]
    fn test_error_codes_constants() {
        assert!(!error_codes::INSUFFICIENT_BUDGET.is_empty());
        assert!(!error_codes::RATE_LIMIT_EXCEEDED.is_empty());
        assert!(!error_codes::CONFIGURATION_ERROR.is_empty());
    }
    
    #[test]
    fn test_metrics_constants() {
        assert!(!metrics::TOTAL_REQUESTS.is_empty());
        assert!(!metrics::REQUESTS_ALLOWED.is_empty());
        assert!(!metrics::REQUESTS_DENIED.is_empty());
        assert!(!metrics::BUDGET_CONSUMED.is_empty());
    }
    
    #[test]
    fn test_validation() {
        let config = PrivacyBudgetConfig::default();
        assert!(validation::validate_config(&config).is_ok());
    }
    
    #[test]
    fn test_utils() {
        let cost = utils::calculate_request_cost("GET", "/api/test", None, 0, &[], &std::collections::HashMap::new());
        assert!(cost >= MIN_BUDGET_PER_REQUEST);
        assert!(cost <= MAX_BUDGET_PER_REQUEST);
        
        let headers = utils::format_privacy_headers(100.0, 50.0, Some(25.0), 5.0);
        assert_eq!(headers.get(headers::GLOBAL_BUDGET), Some(&"100".to_string()));
        assert_eq!(headers.get(headers::API_BUDGET), Some(&"50".to_string()));
        assert_eq!(headers.get(headers::USER_BUDGET), Some(&"25".to_string()));
        assert_eq!(headers.get(headers::CONSUMED_BUDGET), Some(&"5".to_string()));
        
        let budget_info = utils::parse_privacy_headers(&headers);
        assert_eq!(budget_info.get(headers::GLOBAL_BUDGET), Some(&100.0));
        assert_eq!(budget_info.get(headers::API_BUDGET), Some(&50.0));
        assert_eq!(budget_info.get(headers::USER_BUDGET), Some(&25.0));
        assert_eq!(budget_info.get(headers::CONSUMED_BUDGET), Some(&5.0));
        
        assert!(utils::is_budget_exhausted(0.5, 1.0));
        assert!(!utils::is_budget_exhausted(1.5, 1.0));
        
        assert_eq!(utils::calculate_budget_usage_percentage(50.0, 100.0), 50.0);
        assert_eq!(utils::calculate_budget_usage_percentage(0.0, 100.0), 0.0);
        
        assert_eq!(utils::estimate_remaining_requests(100.0, 10.0), 10);
        assert_eq!(utils::estimate_remaining_requests(100.0, 0.0), 0);
    }
}
