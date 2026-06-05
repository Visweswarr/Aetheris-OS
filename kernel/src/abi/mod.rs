//! ABI (Application Binary Interface) Module
//! 
//! This module provides ABI-related functionality including feature flags,
//! versioning, and compatibility management.

pub mod features;

/// Initialize the kernel feature manager (stub).
pub fn init_feature_manager() {
    let _ = features::get_kernel_features();
}

// Re-export commonly used feature helpers
pub use features::{
    KernelFeature,
    FeatureCompatibility,
    get_kernel_features,
    get_feature_string,
    has_feature,
    get_timer_features,
    get_scheduling_features,
    get_timer_compatibility,
    get_jitter_compatibility,
    print_feature_status,
};
