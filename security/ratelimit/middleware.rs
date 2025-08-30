//! Middleware integrations for popular Rust web frameworks
//!
//! Provides ready-to-use middleware for Axum, Actix Web, and other frameworks
//! to easily integrate rate limiting into existing services.

use crate::{RateLimiter, RateLimitError, RateLimitResult};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Trait for extracting DID from requests
pub trait DidExtractor<Request> {
    fn extract_did(&self, request: &Request) -> Option<String>;
}

/// Default DID extractor that looks for common headers
#[derive(Debug, Clone)]
pub struct DefaultDidExtractor;

impl DefaultDidExtractor {
    pub fn new() -> Self {
        Self
    }
}

/// Trait for extracting endpoint/route from requests
pub trait EndpointExtractor<Request> {
    fn extract_endpoint(&self, request: &Request) -> String;
}

/// Default endpoint extractor that uses the request path
#[derive(Debug, Clone)]
pub struct DefaultEndpointExtractor;

impl DefaultEndpointExtractor {
    pub fn new() -> Self {
        Self
    }
}

/// Configuration for rate limiting middleware
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    /// DID extractor implementation
    pub did_extractor: Arc<dyn DidExtractor<hyper::Request<hyper::body::Incoming>> + Send + Sync>,
    /// Endpoint extractor implementation
    pub endpoint_extractor: Arc<dyn EndpointExtractor<hyper::Request<hyper::body::Incoming>> + Send + Sync>,
    /// Number of tokens to consume per request
    pub tokens_per_request: u64,
    /// Whether to include rate limit headers in response
    pub include_headers: bool,
    /// Custom header prefix (default: "X-RateLimit")
    pub header_prefix: String,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            did_extractor: Arc::new(DefaultDidExtractor),
            endpoint_extractor: Arc::new(DefaultEndpointExtractor),
            tokens_per_request: 1,
            include_headers: true,
            header_prefix: "X-RateLimit".to_string(),
        }
    }
}

#[cfg(feature = "axum-integration")]
pub mod axum {
    use super::*;
    use axum::{
        extract::{Request, State},
        http::{HeaderMap, HeaderValue, StatusCode},
        middleware::Next,
        response::{IntoResponse, Response},
    };
    use std::collections::HashMap;

    /// Axum middleware state
    #[derive(Clone)]
    pub struct RateLimitState {
        pub limiter: Arc<RateLimiter>,
        pub config: MiddlewareConfig,
    }

    impl RateLimitState {
        pub fn new(limiter: Arc<RateLimiter>, config: MiddlewareConfig) -> Self {
            Self { limiter, config }
        }
    }

    /// Axum rate limiting middleware function
    pub async fn rate_limit_middleware(
        State(state): State<RateLimitState>,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract DID from request
        let did = state.config.did_extractor.extract_did(&request)
            .unwrap_or_else(|| "anonymous".to_string());

        // Extract endpoint from request
        let endpoint = state.config.endpoint_extractor.extract_endpoint(&request);

        // Check rate limit
        match state.limiter.check_rate_limit(&did, &endpoint, state.config.tokens_per_request).await {
            Ok(decision) => {
                // Request allowed, proceed to next middleware/handler
                let mut response = next.run(request).await;

                // Add rate limit headers if configured
                if state.config.include_headers {
                    add_rate_limit_headers(response.headers_mut(), &decision.headers, &state.config.header_prefix);
                }

                Ok(response)
            }
            Err(RateLimitError::RateLimitExceeded { .. }) => {
                // Rate limit exceeded, return 429
                let mut headers = HeaderMap::new();
                
                // Get current status for headers
                if let Ok(status) = state.limiter.get_rate_limit_status(&did, &endpoint).await {
                    if state.config.include_headers {
                        add_rate_limit_headers(&mut headers, &status.headers, &state.config.header_prefix);
                    }
                }

                Err(StatusCode::TOO_MANY_REQUESTS)
            }
            Err(RateLimitError::PenaltyActive { penalty_type, until }) => {
                // Penalty active, return 429 with penalty info
                let mut headers = HeaderMap::new();
                
                if state.config.include_headers {
                    headers.insert(
                        format!("{}-Penalty-Type", state.config.header_prefix).parse().unwrap(),
                        HeaderValue::from_str(&penalty_type).unwrap(),
                    );
                    headers.insert(
                        format!("{}-Penalty-Until", state.config.header_prefix).parse().unwrap(),
                        HeaderValue::from_str(&format!("{:?}", until)).unwrap(),
                    );
                }

                Err(StatusCode::TOO_MANY_REQUESTS)
            }
            Err(_) => {
                // Other errors, return 500
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Helper function to add rate limit headers
    fn add_rate_limit_headers(
        response_headers: &mut HeaderMap,
        rate_limit_headers: &HashMap<String, String>,
        prefix: &str,
    ) {
        for (key, value) in rate_limit_headers {
            if let Ok(header_name) = key.parse() {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    response_headers.insert(header_name, header_value);
                }
            }
        }
    }

    /// DID extractor for Axum requests
    impl DidExtractor<Request> for DefaultDidExtractor {
        fn extract_did(&self, request: &Request) -> Option<String> {
            // Check common headers for DID
            let headers = request.headers();
            
            // Try Authorization header (Bearer token with DID)
            if let Some(auth_header) = headers.get("authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(did) = extract_did_from_bearer_token(auth_str) {
                        return Some(did);
                    }
                }
            }

            // Try X-DID header
            if let Some(did_header) = headers.get("x-did") {
                if let Ok(did_str) = did_header.to_str() {
                    return Some(did_str.to_string());
                }
            }

            // Try X-User-ID header as fallback
            if let Some(user_header) = headers.get("x-user-id") {
                if let Ok(user_str) = user_header.to_str() {
                    return Some(format!("did:user:{}", user_str));
                }
            }

            None
        }
    }

    /// Endpoint extractor for Axum requests
    impl EndpointExtractor<Request> for DefaultEndpointExtractor {
        fn extract_endpoint(&self, request: &Request) -> String {
            // Use the request path as endpoint identifier
            let path = request.uri().path();
            
            // Normalize path (remove query parameters, trailing slashes)
            let normalized_path = path.trim_end_matches('/');
            
            // Group similar endpoints (replace IDs with placeholders)
            normalize_endpoint_path(normalized_path)
        }
    }

    /// Example of how to set up Axum with rate limiting
    pub fn example_axum_setup() -> axum::Router {
        use axum::{routing::get, Router};
        
        // Create rate limiter
        let rules = crate::RateLimitRules::default();
        let penalty_config = crate::PenaltyConfig::default();
        let limiter = Arc::new(RateLimiter::new(rules, penalty_config));
        
        // Create middleware state
        let rate_limit_state = RateLimitState::new(limiter, MiddlewareConfig::default());
        
        // Build router with rate limiting middleware
        Router::new()
            .route("/api/test", get(|| async { "Hello, World!" }))
            .route("/api/users/:id", get(|| async { "User details" }))
            .layer(axum::middleware::from_fn_with_state(
                rate_limit_state.clone(),
                rate_limit_middleware,
            ))
            .with_state(rate_limit_state)
    }
}

#[cfg(feature = "actix-integration")]
pub mod actix {
    use super::*;
    use actix_web::{
        dev::{Service, ServiceRequest, ServiceResponse, Transform},
        Error, HttpMessage, HttpResponse,
    };
    use futures::future::{ok, Ready};
    use std::rc::Rc;

    /// Actix Web rate limiting middleware
    pub struct RateLimitMiddleware {
        limiter: Arc<RateLimiter>,
        config: MiddlewareConfig,
    }

    impl RateLimitMiddleware {
        pub fn new(limiter: Arc<RateLimiter>, config: MiddlewareConfig) -> Self {
            Self { limiter, config }
        }
    }

    impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Transform = RateLimitMiddlewareService<S>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ok(RateLimitMiddlewareService {
                service: Rc::new(service),
                limiter: Arc::clone(&self.limiter),
                config: self.config.clone(),
            })
        }
    }

    pub struct RateLimitMiddlewareService<S> {
        service: Rc<S>,
        limiter: Arc<RateLimiter>,
        config: MiddlewareConfig,
    }

    impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

        fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.service.poll_ready(cx)
        }

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let service = Rc::clone(&self.service);
            let limiter = Arc::clone(&self.limiter);
            let config = self.config.clone();

            Box::pin(async move {
                // Extract DID and endpoint (simplified for Actix)
                let did = req.headers()
                    .get("x-did")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("anonymous")
                    .to_string();

                let endpoint = req.path().to_string();

                // Check rate limit
                match limiter.check_rate_limit(&did, &endpoint, config.tokens_per_request).await {
                    Ok(_) => {
                        // Proceed with request
                        service.call(req).await
                    }
                    Err(RateLimitError::RateLimitExceeded { .. }) => {
                        Ok(req.into_response(
                            HttpResponse::TooManyRequests()
                                .json(serde_json::json!({
                                    "error": "Rate limit exceeded"
                                }))
                                .into_body(),
                        ))
                    }
                    Err(RateLimitError::PenaltyActive { penalty_type, .. }) => {
                        Ok(req.into_response(
                            HttpResponse::TooManyRequests()
                                .json(serde_json::json!({
                                    "error": "Penalty active",
                                    "penalty_type": penalty_type
                                }))
                                .into_body(),
                        ))
                    }
                    Err(_) => {
                        Ok(req.into_response(
                            HttpResponse::InternalServerError()
                                .json(serde_json::json!({
                                    "error": "Internal server error"
                                }))
                                .into_body(),
                        ))
                    }
                }
            })
        }
    }
}

// Helper functions

/// Extract DID from Bearer token
fn extract_did_from_bearer_token(auth_header: &str) -> Option<String> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        // This is a simplified example - in practice, you'd decode JWT or validate token
        // and extract DID from claims
        if token.starts_with("did:") {
            Some(token.to_string())
        } else {
            // Could be a JWT containing DID
            None
        }
    } else {
        None
    }
}

/// Normalize endpoint path for rate limiting grouping
fn normalize_endpoint_path(path: &str) -> String {
    // Replace common ID patterns with placeholders
    let normalized = path
        .split('/')
        .map(|segment| {
            // Replace UUIDs
            if segment.len() == 36 && segment.contains('-') {
                ":id"
            }
            // Replace numeric IDs
            else if segment.chars().all(|c| c.is_ascii_digit()) {
                ":id"
            }
            // Replace DID-like segments
            else if segment.starts_with("did:") {
                ":did"
            }
            else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    
    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_endpoint_path() {
        assert_eq!(normalize_endpoint_path("/api/users/123"), "/api/users/:id");
        assert_eq!(normalize_endpoint_path("/api/users/550e8400-e29b-41d4-a716-446655440000"), "/api/users/:id");
        assert_eq!(normalize_endpoint_path("/api/dids/did:web:example.com"), "/api/dids/:did");
        assert_eq!(normalize_endpoint_path("/api/static"), "/api/static");
        assert_eq!(normalize_endpoint_path(""), "/");
    }

    #[test]
    fn test_extract_did_from_bearer_token() {
        assert_eq!(
            extract_did_from_bearer_token("Bearer did:web:example.com"),
            Some("did:web:example.com".to_string())
        );
        assert_eq!(extract_did_from_bearer_token("Bearer jwt_token_here"), None);
        assert_eq!(extract_did_from_bearer_token("Basic base64_here"), None);
    }
}
