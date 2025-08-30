//! Rate Limiting Module for Polymera OS
//!
//! Provides comprehensive token bucket rate limiting with DID-based tracking,
//! burst allowances, penalty mechanisms, and middleware integrations.

// Re-export main types
pub use lib::{
    RateLimiter, RateLimitResult, RateLimitError, RateLimitDecision,
    BucketConfig, RateLimitRules, PenaltyConfig, PenaltyInfo,
    TokenBucket, DidRateLimitState, ViolationRecord, ViolationSeverity,
};

// Module structure
pub mod lib;

#[cfg(test)]
pub mod tests;

// Middleware integrations
#[cfg(feature = "axum-integration")]
pub mod middleware;

#[cfg(feature = "axum-integration")]
pub use middleware::axum::*;

#[cfg(feature = "actix-integration")]
pub use middleware::actix::*;

// Metrics integration
#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "metrics")]
pub use metrics::*;
