use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::budget::{PrivacyBudgetManager, BudgetRequest, BudgetResponse, PrivacyBudgetError};

/// HTTP request context for privacy budget tracking
#[derive(Debug, Clone)]
pub struct HttpRequestContext {
    /// Request ID
    pub request_id: String,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// User ID from authentication
    pub user_id: Option<String>,
    /// Request headers
    pub headers: std::collections::HashMap<String, String>,
    /// Request timestamp
    pub timestamp: Instant,
    /// Request metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// HTTP response context for privacy budget tracking
#[derive(Debug, Clone)]
pub struct HttpResponseContext {
    /// Response status code
    pub status_code: u16,
    /// Response headers
    pub headers: std::collections::HashMap<String, String>,
    /// Response body size
    pub body_size: usize,
    /// Response timestamp
    pub timestamp: Instant,
    /// Processing time
    pub processing_time: std::time::Duration,
}

/// Privacy budget middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// Enable privacy budget enforcement
    pub enabled: bool,
    /// Enable automatic cost calculation
    pub auto_cost_calculation: bool,
    /// Default cost for unknown endpoints
    pub default_cost: f64,
    /// Enable cost estimation based on response size
    pub size_based_cost: bool,
    /// Cost per KB of response data
    pub cost_per_kb: f64,
    /// Enable method-based cost adjustment
    pub method_based_cost: bool,
    /// Method cost multipliers
    pub method_multipliers: std::collections::HashMap<String, f64>,
    /// Enable user-based cost adjustment
    pub user_based_cost: bool,
    /// User cost multipliers
    pub user_multipliers: std::collections::HashMap<String, f64>,
    /// Enable path-based cost adjustment
    pub path_based_cost: bool,
    /// Path cost patterns
    pub path_patterns: Vec<PathCostPattern>,
    /// Enable response time cost adjustment
    pub response_time_cost: bool,
    /// Cost per millisecond of processing time
    pub cost_per_ms: f64,
    /// Enable error cost adjustment
    pub error_cost_adjustment: bool,
    /// Error cost multipliers
    pub error_multipliers: std::collections::HashMap<u16, f64>,
    /// Enable header-based cost adjustment
    pub header_based_cost: bool,
    /// Header cost patterns
    pub header_patterns: Vec<HeaderCostPattern>,
    /// Enable query parameter cost adjustment
    pub query_param_cost: bool,
    /// Query parameter cost patterns
    pub query_patterns: Vec<QueryParamCostPattern>,
    /// Enable body size cost adjustment
    pub body_size_cost: bool,
    /// Body size cost thresholds
    pub body_size_thresholds: Vec<BodySizeThreshold>,
    /// Enable rate limiting integration
    pub rate_limit_integration: bool,
    /// Enable metrics collection
    pub metrics_collection: true,
    /// Enable audit logging
    pub audit_logging: true,
    /// Enable performance monitoring
    pub performance_monitoring: true,
}

/// Path cost pattern for dynamic cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathCostPattern {
    /// Pattern to match (regex)
    pub pattern: String,
    /// Cost multiplier
    pub multiplier: f64,
    /// Base cost
    pub base_cost: f64,
    /// Priority (higher = more specific)
    pub priority: u8,
}

/// Header cost pattern for dynamic cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderCostPattern {
    /// Header name
    pub header_name: String,
    /// Header value pattern (regex)
    pub value_pattern: String,
    /// Cost adjustment
    pub cost_adjustment: f64,
    /// Multiplier
    pub multiplier: f64,
}

/// Query parameter cost pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParamCostPattern {
    /// Parameter name
    pub param_name: String,
    /// Parameter value pattern (regex)
    pub value_pattern: String,
    /// Cost adjustment
    pub cost_adjustment: f64,
    /// Multiplier
    pub multiplier: f64,
}

/// Body size threshold for cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodySizeThreshold {
    /// Size threshold in bytes
    pub size_threshold: usize,
    /// Cost multiplier
    pub multiplier: f64,
    /// Additional cost
    pub additional_cost: f64,
}

/// Privacy budget middleware
pub struct PrivacyBudgetMiddleware {
    budget_manager: Arc<PrivacyBudgetManager>,
    config: MiddlewareConfig,
    metrics: Arc<RwLock<MiddlewareMetrics>>,
}

/// Middleware metrics
#[derive(Debug, Clone, Serialize)]
pub struct MiddlewareMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests with budget enforcement
    pub budget_enforced: u64,
    /// Requests without budget enforcement
    pub budget_bypassed: u64,
    /// Average cost per request
    pub avg_cost_per_request: f64,
    /// Total cost consumed
    pub total_cost_consumed: f64,
    /// Cost distribution by endpoint
    pub cost_by_endpoint: std::collections::HashMap<String, f64>,
    /// Cost distribution by user
    pub cost_by_user: std::collections::HashMap<String, f64>,
    /// Cost distribution by method
    pub cost_by_method: std::collections::HashMap<String, f64>,
    /// Response time distribution
    pub response_time_distribution: std::collections::HashMap<String, u64>,
    /// Error rate by endpoint
    pub error_rate_by_endpoint: std::collections::HashMap<String, f64>,
    /// Last metrics update
    pub last_update: std::time::SystemTime,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        let mut method_multipliers = std::collections::HashMap::new();
        method_multipliers.insert("GET".to_string(), 1.0);
        method_multipliers.insert("POST".to_string(), 2.0);
        method_multipliers.insert("PUT".to_string(), 1.5);
        method_multipliers.insert("DELETE".to_string(), 1.5);
        method_multipliers.insert("PATCH".to_string(), 1.2);

        let mut error_multipliers = std::collections::HashMap::new();
        error_multipliers.insert(400, 1.0); // Bad Request
        error_multipliers.insert(401, 1.0); // Unauthorized
        error_multipliers.insert(403, 1.0); // Forbidden
        error_multipliers.insert(404, 1.0); // Not Found
        error_multipliers.insert(429, 1.5); // Too Many Requests
        error_multipliers.insert(500, 2.0); // Internal Server Error

        Self {
            enabled: true,
            auto_cost_calculation: true,
            default_cost: 1.0,
            size_based_cost: true,
            cost_per_kb: 0.1,
            method_based_cost: true,
            method_multipliers,
            user_based_cost: true,
            user_multipliers: std::collections::HashMap::new(),
            path_based_cost: true,
            path_patterns: vec![],
            response_time_cost: true,
            cost_per_ms: 0.001,
            error_cost_adjustment: true,
            error_multipliers,
            header_based_cost: true,
            header_patterns: vec![],
            query_param_cost: true,
            query_patterns: vec![],
            body_size_cost: true,
            body_size_thresholds: vec![],
            rate_limit_integration: true,
            metrics_collection: true,
            audit_logging: true,
            performance_monitoring: true,
        }
    }
}

impl PrivacyBudgetMiddleware {
    /// Create new privacy budget middleware
    pub fn new(budget_manager: Arc<PrivacyBudgetManager>, config: MiddlewareConfig) -> Self {
        Self {
            budget_manager,
            config,
            metrics: Arc::new(RwLock::new(MiddlewareMetrics::new())),
        }
    }

    /// Process incoming HTTP request
    pub async fn process_request(&self, context: &HttpRequestContext) -> Result<BudgetResponse, PrivacyBudgetError> {
        if !self.config.enabled {
            return self.create_bypass_response(context).await;
        }

        // Calculate request cost
        let cost = self.calculate_request_cost(context).await;

        // Create budget request
        let budget_request = BudgetRequest {
            request_id: context.request_id.clone(),
            api_path: context.path.clone(),
            method: context.method.clone(),
            user_id: context.user_id.clone(),
            timestamp: context.timestamp,
            cost,
            metadata: context.metadata.clone(),
        };

        // Check budget
        let response = self.budget_manager.check_budget(&budget_request).await?;

        // Update metrics
        self.update_request_metrics(context, &response, cost).await;

        Ok(response)
    }

    /// Process outgoing HTTP response
    pub async fn process_response(&self, request_context: &HttpRequestContext, response_context: &HttpResponseContext) {
        if !self.config.enabled {
            return;
        }

        // Update response metrics
        self.update_response_metrics(request_context, response_context).await;

        // Log audit information if enabled
        if self.config.audit_logging {
            self.log_audit_info(request_context, response_context).await;
        }
    }

    /// Calculate request cost based on various factors
    async fn calculate_request_cost(&self, context: &HttpRequestContext) -> f64 {
        let mut cost = self.config.default_cost;

        // Method-based cost adjustment
        if self.config.method_based_cost {
            if let Some(multiplier) = self.config.method_multipliers.get(&context.method) {
                cost *= multiplier;
            }
        }

        // User-based cost adjustment
        if self.config.user_based_cost {
            if let Some(user_id) = &context.user_id {
                if let Some(multiplier) = self.config.user_multipliers.get(user_id) {
                    cost *= multiplier;
                }
            }
        }

        // Path-based cost adjustment
        if self.config.path_based_cost {
            cost = self.apply_path_cost_patterns(cost, &context.path);
        }

        // Header-based cost adjustment
        if self.config.header_based_cost {
            cost = self.apply_header_cost_patterns(cost, &context.headers);
        }

        // Query parameter cost adjustment
        if self.config.query_param_cost {
            cost = self.apply_query_param_cost_patterns(cost, &context.path);
        }

        cost
    }

    /// Apply path-based cost patterns
    fn apply_path_cost_patterns(&self, base_cost: f64, path: &str) -> f64 {
        let mut cost = base_cost;
        let mut best_priority = 0;

        for pattern in &self.config.path_patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(path) && pattern.priority >= best_priority {
                    cost = pattern.base_cost + (cost * pattern.multiplier);
                    best_priority = pattern.priority;
                }
            }
        }

        cost
    }

    /// Apply header-based cost patterns
    fn apply_header_cost_patterns(&self, base_cost: f64, headers: &std::collections::HashMap<String, String>) -> f64 {
        let mut cost = base_cost;

        for pattern in &self.config.header_patterns {
            if let Some(header_value) = headers.get(&pattern.header_name) {
                if let Ok(regex) = regex::Regex::new(&pattern.value_pattern) {
                    if regex.is_match(header_value) {
                        cost += pattern.cost_adjustment;
                        cost *= pattern.multiplier;
                    }
                }
            }
        }

        cost
    }

    /// Apply query parameter cost patterns
    fn apply_query_param_cost_patterns(&self, base_cost: f64, path: &str) -> f64 {
        let mut cost = base_cost;

        if let Some(query_part) = path.split('?').nth(1) {
            for pattern in &self.config.query_patterns {
                if let Ok(regex) = regex::Regex::new(&pattern.value_pattern) {
                    if regex.is_match(query_part) {
                        cost += pattern.cost_adjustment;
                        cost *= pattern.multiplier;
                    }
                }
            }
        }

        cost
    }

    /// Create bypass response when middleware is disabled
    async fn create_bypass_response(&self, context: &HttpRequestContext) -> BudgetResponse {
        BudgetResponse {
            allowed: true,
            remaining_api_budget: f64::INFINITY,
            remaining_global_budget: f64::INFINITY,
            remaining_user_budget: None,
            rate_limit_headers: std::collections::HashMap::new(),
            privacy_headers: {
                let mut headers = std::collections::HashMap::new();
                headers.insert("X-Privacy-Budget-Bypassed".to_string(), "true".to_string());
                headers
            },
            error_message: None,
        }
    }

    /// Update request metrics
    async fn update_request_metrics(&self, context: &HttpRequestContext, response: &BudgetResponse, cost: f64) {
        if !self.config.metrics_collection {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        if response.allowed {
            metrics.budget_enforced += 1;
        } else {
            metrics.budget_bypassed += 1;
        }

        // Update cost metrics
        metrics.total_cost_consumed += cost;
        metrics.avg_cost_per_request = metrics.total_cost_consumed / metrics.total_requests as f64;

        // Update endpoint cost distribution
        *metrics.cost_by_endpoint.entry(context.path.clone()).or_insert(0.0) += cost;

        // Update user cost distribution
        if let Some(user_id) = &context.user_id {
            *metrics.cost_by_user.entry(user_id.clone()).or_insert(0.0) += cost;
        }

        // Update method cost distribution
        *metrics.cost_by_method.entry(context.method.clone()).or_insert(0.0) += cost;

        metrics.last_update = std::time::SystemTime::now();
    }

    /// Update response metrics
    async fn update_response_metrics(&self, request_context: &HttpRequestContext, response_context: &HttpResponseContext) {
        if !self.config.metrics_collection {
            return;
        }

        let mut metrics = self.metrics.write().await;

        // Update response time distribution
        let response_time_bucket = match response_context.processing_time.as_millis() {
            0..=10 => "0-10ms",
            11..=50 => "11-50ms",
            51..=100 => "51-100ms",
            101..=500 => "101-500ms",
            _ => "500ms+",
        };
        *metrics.response_time_distribution.entry(response_time_bucket.to_string()).or_insert(0) += 1;

        // Update error rate by endpoint
        if response_context.status_code >= 400 {
            let error_count = metrics.error_rate_by_endpoint.entry(request_context.path.clone()).or_insert(0.0);
            *error_count += 1.0;
        }

        metrics.last_update = std::time::SystemTime::now();
    }

    /// Log audit information
    async fn log_audit_info(&self, request_context: &HttpRequestContext, response_context: &HttpResponseContext) {
        let audit_log = AuditLog {
            timestamp: std::time::SystemTime::now(),
            request_id: request_context.request_id.clone(),
            user_id: request_context.user_id.clone(),
            method: request_context.method.clone(),
            path: request_context.path.clone(),
            status_code: response_context.status_code,
            processing_time: response_context.processing_time,
            headers: request_context.headers.clone(),
            metadata: request_context.metadata.clone(),
        };

        // In a real implementation, you'd send this to an audit log system
        log::info!("AUDIT: {:?}", audit_log);
    }

    /// Get middleware metrics
    pub async fn get_metrics(&self) -> MiddlewareMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset middleware metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = MiddlewareMetrics::new();
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: MiddlewareConfig) {
        // In a real implementation, you'd use proper locking
        // For now, we'll just log the change
        log::info!("Updating middleware configuration: {:?}", new_config);
    }

    /// Health check for middleware
    pub async fn health_check(&self) -> bool {
        // Check if budget manager is accessible
        match self.budget_manager.get_metrics().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize)]
pub struct AuditLog {
    /// Timestamp of the event
    pub timestamp: std::time::SystemTime,
    /// Request ID
    pub request_id: String,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Response status code
    pub status_code: u16,
    /// Processing time
    pub processing_time: std::time::Duration,
    /// Request headers
    pub headers: std::collections::HashMap<String, String>,
    /// Request metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl MiddlewareMetrics {
    /// Create new middleware metrics
    fn new() -> Self {
        Self {
            total_requests: 0,
            budget_enforced: 0,
            budget_bypassed: 0,
            avg_cost_per_request: 0.0,
            total_cost_consumed: 0.0,
            cost_by_endpoint: std::collections::HashMap::new(),
            cost_by_user: std::collections::HashMap::new(),
            cost_by_method: std::collections::HashMap::new(),
            response_time_distribution: std::collections::HashMap::new(),
            error_rate_by_endpoint: std::collections::HashMap::new(),
            last_update: std::time::SystemTime::now(),
        }
    }
}

/// Integration with common web frameworks
pub mod integrations {
    use super::*;

    /// Actix-web integration
    #[cfg(feature = "actix-web")]
    pub mod actix_web {
        use super::*;
        use actix_web::{
            dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
            Error, HttpMessage,
        };
        use std::future::{ready, Ready, Future};
        use std::pin::Pin;

        pub struct PrivacyBudgetMiddlewareTransform {
            middleware: Arc<PrivacyBudgetMiddleware>,
        }

        impl PrivacyBudgetMiddlewareTransform {
            pub fn new(middleware: Arc<PrivacyBudgetMiddleware>) -> Self {
                Self { middleware }
            }
        }

        impl<S, B> Transform<S, ServiceRequest> for PrivacyBudgetMiddlewareTransform
        where
            S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
            S::Future: 'static,
            B: 'static,
        {
            type Response = ServiceResponse<B>;
            type Error = Error;
            type InitError = ();
            type Transform = PrivacyBudgetMiddlewareService<S>;
            type Future = Ready<Result<Self::Transform, Self::InitError>>;

            fn new_transform(&self, service: S) -> Self::Future {
                ready(Ok(PrivacyBudgetMiddlewareService {
                    service,
                    middleware: self.middleware.clone(),
                }))
            }
        }

        pub struct PrivacyBudgetMiddlewareService<S> {
            service: S,
            middleware: Arc<PrivacyBudgetMiddleware>,
        }

        impl<S, B> Service<ServiceRequest> for PrivacyBudgetMiddlewareService<S>
        where
            S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
            S::Future: 'static,
            B: 'static,
        {
            type Response = ServiceResponse<B>;
            type Error = Error;
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

            forward_ready!(service);

            fn call(&self, req: ServiceRequest) -> Self::Future {
                let middleware = self.middleware.clone();
                let start_time = std::time::Instant::now();

                Box::pin(async move {
                    // Extract request context
                    let context = HttpRequestContext {
                        request_id: Uuid::new_v4().to_string(),
                        method: req.method().to_string(),
                        path: req.path().to_string(),
                        user_id: req.extensions().get::<String>().cloned(),
                        headers: req
                            .headers()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect(),
                        timestamp: std::time::Instant::now(),
                        metadata: std::collections::HashMap::new(),
                    };

                    // Process request through privacy budget middleware
                    let budget_response = middleware.process_request(&context).await?;

                    if !budget_response.allowed {
                        // Return 429 Too Many Requests
                        return Err(actix_web::error::ErrorTooManyRequests("Privacy budget exceeded"));
                    }

                    // Add privacy budget headers
                    let mut req = req;
                    for (key, value) in budget_response.privacy_headers {
                        req.headers_mut().insert(key, value.parse().unwrap());
                    }

                    // Call the inner service
                    let res = self.service.call(req).await?;

                    // Process response
                    let response_context = HttpResponseContext {
                        status_code: res.status().as_u16(),
                        headers: res
                            .headers()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect(),
                        body_size: 0, // Would need to be calculated from response body
                        timestamp: std::time::Instant::now(),
                        processing_time: start_time.elapsed(),
                    };

                    middleware.process_response(&context, &response_context).await;

                    Ok(res)
                })
            }
        }
    }

    /// Axum integration
    #[cfg(feature = "axum")]
    pub mod axum {
        use super::*;
        use axum::{
            extract::Request,
            http::{Response, StatusCode},
            middleware::Next,
        };
        use std::sync::Arc;

        pub async fn privacy_budget_middleware(
            req: Request,
            next: Next,
            middleware: Arc<PrivacyBudgetMiddleware>,
        ) -> Result<Response<axum::body::Body>, StatusCode> {
            let start_time = std::time::Instant::now();

            // Extract request context
            let context = HttpRequestContext {
                request_id: Uuid::new_v4().to_string(),
                method: req.method().to_string(),
                path: req.uri().path().to_string(),
                user_id: None, // Would need to be extracted from authentication
                headers: req
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect(),
                timestamp: std::time::Instant::now(),
                metadata: std::collections::HashMap::new(),
            };

            // Process request through privacy budget middleware
            let budget_response = middleware.process_request(&context).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if !budget_response.allowed {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }

            // Add privacy budget headers
            let mut req = req;
            for (key, value) in budget_response.privacy_headers {
                req.headers_mut().insert(key, value.parse().unwrap());
            }

            // Call the next middleware/handler
            let res = next.run(req).await;

            // Process response
            let response_context = HttpResponseContext {
                status_code: res.status().as_u16(),
                headers: res
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect(),
                body_size: 0, // Would need to be calculated from response body
                timestamp: std::time::Instant::now(),
                processing_time: start_time.elapsed(),
            };

            middleware.process_response(&context, &response_context).await;

            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::PrivacyBudgetConfig;

    #[tokio::test]
    async fn test_middleware_creation() {
        let config = PrivacyBudgetConfig::default();
        let budget_manager = Arc::new(PrivacyBudgetManager::new(config).unwrap());
        let middleware_config = MiddlewareConfig::default();
        let middleware = PrivacyBudgetMiddleware::new(budget_manager, middleware_config);

        assert!(middleware.config.enabled);
        assert!(middleware.config.auto_cost_calculation);
    }

    #[tokio::test]
    async fn test_request_cost_calculation() {
        let config = PrivacyBudgetConfig::default();
        let budget_manager = Arc::new(PrivacyBudgetManager::new(config).unwrap());
        let middleware_config = MiddlewareConfig::default();
        let middleware = PrivacyBudgetMiddleware::new(budget_manager, middleware_config);

        let context = HttpRequestContext {
            request_id: Uuid::new_v4().to_string(),
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            user_id: None,
            headers: std::collections::HashMap::new(),
            timestamp: std::time::Instant::now(),
            metadata: std::collections::HashMap::new(),
        };

        let cost = middleware.calculate_request_cost(&context).await;
        assert!(cost > 0.0);
    }

    #[tokio::test]
    async fn test_middleware_metrics() {
        let config = PrivacyBudgetConfig::default();
        let budget_manager = Arc::new(PrivacyBudgetManager::new(config).unwrap());
        let middleware_config = MiddlewareConfig::default();
        let middleware = PrivacyBudgetMiddleware::new(budget_manager, middleware_config);

        let metrics = middleware.get_metrics().await;
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.budget_enforced, 0);
        assert_eq!(metrics.budget_bypassed, 0);
    }

    #[tokio::test]
    async fn test_middleware_health_check() {
        let config = PrivacyBudgetConfig::default();
        let budget_manager = Arc::new(PrivacyBudgetManager::new(config).unwrap());
        let middleware_config = MiddlewareConfig::default();
        let middleware = PrivacyBudgetMiddleware::new(budget_manager, middleware_config);

        let health = middleware.health_check().await;
        assert!(health);
    }
}
