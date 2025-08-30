//! ABI (Application Binary Interface) Module
//! 
//! This module provides ABI-related functionality including feature flags,
//! versioning, and compatibility management.

pub mod features;

// Re-export commonly used items
pub use features::{
    FeatureManager,
    FeatureInfo,
    AbiNegotiationHeader,
    FeatureNegotiationResult,
    negotiate_abi_features,
    get_features,
    is_feature_enabled,
    check_required_features,
    get_missing_features,
    init_feature_manager,
    get_feature_manager,
    get_feature_manager_mut,
};

// Feature flag constants for easy access
pub use features::{
    FEATURE_PQC_CAPS_V2,
    FEATURE_APIC_TIMER,
    FEATURE_HPET_FALLBACK,
    FEATURE_CAP_REPLAY_WIN,
    FEATURE_MMU_AUDIT,
    FEATURE_CRASH_ANALYSIS,
    FEATURE_SOAK_CHAOS,
    FEATURE_SIDE_CHANNEL_GUARD,
    FEATURE_STREAM_AUTH,
    FEATURE_DID_RESOLVER,
    FEATURE_SECURE_BOOT,
    FEATURE_ABI_HARDENING,
    MAX_FEATURES,
};

