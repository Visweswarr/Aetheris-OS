//! Kernel Feature Flags System
//! 
//! This module implements a feature flags system that allows the kernel to
//! advertise available features and enables user applications to adapt their
//! behavior accordingly. Features are represented as a bitset for efficient
//! storage and comparison.

use core::fmt;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::hal::x86_64::timer::{get_timer_kind, TimerKind};
use crate::sched::tick::{get_jitter_budget, is_jitter_budget_active};

/// Kernel feature flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelFeature {
    /// Post-quantum cryptography capabilities v2
    PQC_CAPS_V2 = 0,
    /// Local APIC timer support
    APIC_TIMER = 1,
    /// HPET fallback timer support
    HPET_FALLBACK = 2,
    /// Capability replay window protection
    CAP_REPLAY_WIN = 3,
    /// DID anchor trust management
    DID_ANCHORS = 4,
    /// Comprehensive audit codes
    AUDIT_CODES = 5,
    /// Jitter budget management
    JITTER_BUDGET = 6,
    /// Timer selector (APIC↔HPET)
    TIMER_SELECTOR = 7,
    /// High-precision timing
    HIGH_PRECISION_TIMING = 8,
    /// Real-time task scheduling
    RT_SCHEDULING = 9,
    /// Performance monitoring
    PERF_MONITORING = 10,
    /// Intent Bus ABI v1 (from P2-AI1)
    INTENT_BUS_V1 = 11,
    /// Intent Kernel v0 (from P2-J2)
    INTENT_KERNEL_V0 = 12,
    /// World Model v0 (from P2-J3)
    WORLD_MODEL_V0 = 13,
    /// Skill Runtime v0 (from P2-J4)
    SKILL_RUNTIME_V0 = 14,
            /// Event Fabric v0 (from P2-J5)
        EVENT_FABRIC_V0 = 15,
            /// Policy Guardrail v0 (from P2-J6)
    POLICY_GUARDRAIL_V0 = 16,
    /// LLM Adapter v0 (from P2-J7)
    LLM_ADAPTER_V0 = 17,
    /// NGFS v1 Read-Only (from P3-01-A1)
    NGFS_V1_RO = 18,
    /// NGFS IPFS Compatibility (from P3-01-A1)
    NGFS_IPFS_MAP = 19,
    /// NGFS DID Encryption (from P3-01-A1)
    NGFS_DID_ENCRYPTION = 20,
}

impl KernelFeature {
    /// Get the bit position for this feature
    pub fn bit_position(&self) -> u32 {
        *self as u32
    }
    
    /// Get the bit mask for this feature
    pub fn bit_mask(&self) -> u64 {
        1u64 << self.bit_position()
    }
    
    /// Check if this feature is enabled
    pub fn is_enabled(&self) -> bool {
        let features = get_kernel_features();
        (features & self.bit_mask()) != 0
    }
}

/// Get kernel feature flags
pub fn get_kernel_features() -> u64 {
    let mut features = 0u64;
    
    // Check APIC timer support
    if let Some(timer_kind) = get_timer_kind() {
        match timer_kind {
            TimerKind::APIC => {
                features |= KernelFeature::APIC_TIMER.bit_mask();
                features |= KernelFeature::TIMER_SELECTOR.bit_mask();
                features |= KernelFeature::HIGH_PRECISION_TIMING.bit_mask();
            }
            TimerKind::HPET => {
                features |= KernelFeature::HPET_FALLBACK.bit_mask();
                features |= KernelFeature::TIMER_SELECTOR.bit_mask();
                features |= KernelFeature::HIGH_PRECISION_TIMING.bit_mask();
            }
        }
    }
    
    // Check jitter budget support
    features |= KernelFeature::JITTER_BUDGET.bit_mask();
    
    // Check RT scheduling support
    if is_jitter_budget_active() {
        features |= KernelFeature::RT_SCHEDULING.bit_mask();
    }
    
    // Check performance monitoring
    if let Some(timer_kind) = get_timer_kind() {
        if let Some(jitter_stats) = get_jitter_stats() {
            features |= KernelFeature::PERF_MONITORING.bit_mask();
        }
    }
    
    // Always enabled features (core functionality)
    features |= KernelFeature::PQC_CAPS_V2.bit_mask();
    features |= KernelFeature::CAP_REPLAY_WIN.bit_mask();
    features |= KernelFeature::DID_ANCHORS.bit_mask();
    features |= KernelFeature::AUDIT_CODES.bit_mask();
    
    // Intent-related features
    features |= KernelFeature::INTENT_BUS_V1.bit_mask();
    features |= KernelFeature::INTENT_KERNEL_V0.bit_mask();
    
    // World Model features
    features |= KernelFeature::WORLD_MODEL_V0.bit_mask();
    
    // Skill Runtime features
    features |= KernelFeature::SKILL_RUNTIME_V0.bit_mask();
    
    // Event Fabric features
    features |= KernelFeature::EVENT_FABRIC_V0.bit_mask();
    features |= KernelFeature::POLICY_GUARDRAIL_V0.bit_mask();
    features |= KernelFeature::LLM_ADAPTER_V0.bit_mask();
    
    // NGFS features
    features |= KernelFeature::NGFS_V1_RO.bit_mask();
    features |= KernelFeature::NGFS_IPFS_MAP.bit_mask();
    features |= KernelFeature::NGFS_DID_ENCRYPTION.bit_mask();
    
    features
}

/// Backward-compatible helper that returns the current feature bitset.
pub fn get_features() -> u64 {
    get_kernel_features()
}

/// Get feature information as a string
pub fn get_feature_string() -> &'static str {
    let features = get_kernel_features();
    
    if features & KernelFeature::APIC_TIMER.bit_mask() != 0 {
        "APIC_TIMER"
    } else if features & KernelFeature::HPET_FALLBACK.bit_mask() != 0 {
        "HPET_FALLBACK"
    } else {
        "NO_TIMER"
    }
}

/// Check if a specific feature is available
pub fn has_feature(feature: KernelFeature) -> bool {
    feature.is_enabled()
}

/// Get timer-specific features
pub fn get_timer_features() -> u64 {
    let mut timer_features = 0u64;
    
    if let Some(timer_kind) = get_timer_kind() {
        match timer_kind {
            TimerKind::APIC => {
                timer_features |= KernelFeature::APIC_TIMER.bit_mask();
                timer_features |= KernelFeature::HIGH_PRECISION_TIMING.bit_mask();
            }
            TimerKind::HPET => {
                timer_features |= KernelFeature::HPET_FALLBACK.bit_mask();
                timer_features |= KernelFeature::HIGH_PRECISION_TIMING.bit_mask();
            }
        }
    }
    
    timer_features
}

/// Get scheduling features
pub fn get_scheduling_features() -> u64 {
    let mut sched_features = 0u64;
    
    sched_features |= KernelFeature::JITTER_BUDGET.bit_mask();
    
    if is_jitter_budget_active() {
        sched_features |= KernelFeature::RT_SCHEDULING.bit_mask();
    }
    
    sched_features
}

/// Get security features
pub fn get_security_features() -> u64 {
    let mut security_features = 0u64;
    
    // Core security features
    security_features |= KernelFeature::PQC_CAPS_V2.bit_mask();
    security_features |= KernelFeature::CAP_REPLAY_WIN.bit_mask();
    security_features |= KernelFeature::DID_ANCHORS.bit_mask();
    security_features |= KernelFeature::AUDIT_CODES.bit_mask();
    
    security_features
}

/// Print feature status
pub fn print_feature_status() {
    let features = get_kernel_features();
    
    crate::kprintln!("=== Kernel Features ===");
    crate::kprintln!("Feature Flags: 0x{:016x}", features);
    crate::kprintln!("");
    
    // Timer features
    crate::kprintln!("Timer Features:");
    if features & KernelFeature::APIC_TIMER.bit_mask() != 0 {
        crate::kprintln!("  ✓ APIC Timer");
    }
    if features & KernelFeature::HPET_FALLBACK.bit_mask() != 0 {
        crate::kprintln!("  ✓ HPET Fallback");
    }
    if features & KernelFeature::TIMER_SELECTOR.bit_mask() != 0 {
        crate::kprintln!("  ✓ Timer Selector");
    }
    if features & KernelFeature::HIGH_PRECISION_TIMING.bit_mask() != 0 {
        crate::kprintln!("  ✓ High Precision Timing");
    }
    
    // Scheduling features
    crate::kprintln!("Scheduling Features:");
    if features & KernelFeature::JITTER_BUDGET.bit_mask() != 0 {
        crate::kprintln!("  ✓ Jitter Budget");
    }
    if features & KernelFeature::RT_SCHEDULING.bit_mask() != 0 {
        crate::kprintln!("  ✓ Real-Time Scheduling");
    }
    if features & KernelFeature::PERF_MONITORING.bit_mask() != 0 {
        crate::kprintln!("  ✓ Performance Monitoring");
    }
    
    // Security features
    crate::kprintln!("Security Features:");
    if features & KernelFeature::PQC_CAPS_V2.bit_mask() != 0 {
        crate::kprintln!("  ✓ PQC Capabilities v2");
    }
    if features & KernelFeature::CAP_REPLAY_WIN.bit_mask() != 0 {
        crate::kprintln!("  ✓ Capability Replay Protection");
    }
    if features & KernelFeature::DID_ANCHORS.bit_mask() != 0 {
        crate::kprintln!("  ✓ DID Trust Anchors");
    }
    if features & KernelFeature::AUDIT_CODES.bit_mask() != 0 {
        crate::kprintln!("  ✓ Audit Codes");
    }
    
    // Intent features
    crate::kprintln!("Intent Features:");
    if features & KernelFeature::INTENT_BUS_V1.bit_mask() != 0 {
        crate::kprintln!("  ✓ Intent Bus ABI v1");
    }
    if features & KernelFeature::INTENT_KERNEL_V0.bit_mask() != 0 {
        crate::kprintln!("  ✓ Intent Kernel v0");
    }
    
    // NGFS features
    crate::kprintln!("NGFS Features:");
    if features & KernelFeature::NGFS_V1_RO.bit_mask() != 0 {
        crate::kprintln!("  ✓ NGFS v1 Read-Only");
    }
    if features & KernelFeature::NGFS_IPFS_MAP.bit_mask() != 0 {
        crate::kprintln!("  ✓ NGFS IPFS Compatibility");
    }
    if features & KernelFeature::NGFS_DID_ENCRYPTION.bit_mask() != 0 {
        crate::kprintln!("  ✓ NGFS DID Encryption");
    }
    
    crate::kprintln!("=====================");
}

/// Get jitter statistics for performance monitoring
fn get_jitter_stats() -> Option<JitterStats> {
    let budget = get_jitter_budget();
    let stats = budget.get_stats();
    Some(JitterStats {
        mean_us: stats.jitter_mean_us,
        p95_us: stats.jitter_p95_us,
        total_samples: stats.jitter_samples,
        consecutive_overruns: stats.consecutive_overruns,
        boost_active: stats.boost_active,
    })
}

/// Jitter statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct JitterStats {
    pub mean_us: u32,
    pub p95_us: u32,
    pub total_samples: u32,
    pub consecutive_overruns: u32,
    pub boost_active: bool,
}

/// Feature compatibility matrix
pub struct FeatureCompatibility {
    pub required_features: Vec<KernelFeature>,
    pub optional_features: Vec<KernelFeature>,
    pub incompatible_features: Vec<KernelFeature>,
}

impl FeatureCompatibility {
    /// Check if required features are available
    pub fn check_required(&self) -> bool {
        self.required_features.iter().all(|f| f.is_enabled())
    }
    
    /// Check if optional features are available
    pub fn get_optional_count(&self) -> usize {
        self.optional_features.iter().filter(|f| f.is_enabled()).count()
    }
    
    /// Check for incompatible features
    pub fn has_incompatibilities(&self) -> bool {
        self.incompatible_features.iter().any(|f| f.is_enabled())
    }
}

/// Get feature compatibility for timer selection
pub fn get_timer_compatibility() -> FeatureCompatibility {
    FeatureCompatibility {
        required_features: vec![
            KernelFeature::TIMER_SELECTOR,
        ],
        optional_features: vec![
            KernelFeature::APIC_TIMER,
            KernelFeature::HPET_FALLBACK,
            KernelFeature::HIGH_PRECISION_TIMING,
        ],
        incompatible_features: vec![], // No incompatibilities
    }
}

/// Get feature compatibility for jitter budgeting
pub fn get_jitter_compatibility() -> FeatureCompatibility {
    FeatureCompatibility {
        required_features: vec![
            KernelFeature::JITTER_BUDGET,
        ],
        optional_features: vec![
            KernelFeature::RT_SCHEDULING,
            KernelFeature::PERF_MONITORING,
        ],
        incompatible_features: vec![], // No incompatibilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_feature_enum() {
        assert_eq!(KernelFeature::PQC_CAPS_V2.bit_position(), 0);
        assert_eq!(KernelFeature::APIC_TIMER.bit_position(), 1);
        assert_eq!(KernelFeature::HPET_FALLBACK.bit_position(), 2);
        
        assert_eq!(KernelFeature::PQC_CAPS_V2.bit_mask(), 1);
        assert_eq!(KernelFeature::APIC_TIMER.bit_mask(), 2);
        assert_eq!(KernelFeature::HPET_FALLBACK.bit_mask(), 4);
        
        // Test NGFS features
        assert_eq!(KernelFeature::NGFS_V1_RO.bit_position(), 18);
        assert_eq!(KernelFeature::NGFS_IPFS_MAP.bit_position(), 19);
        assert_eq!(KernelFeature::NGFS_DID_ENCRYPTION.bit_position(), 20);
        
        assert_eq!(KernelFeature::NGFS_V1_RO.bit_mask(), 1 << 18);
        assert_eq!(KernelFeature::NGFS_IPFS_MAP.bit_mask(), 1 << 19);
        assert_eq!(KernelFeature::NGFS_DID_ENCRYPTION.bit_mask(), 1 << 20);
    }

    #[test]
    fn test_feature_compatibility() {
        let timer_compat = get_timer_compatibility();
        assert_eq!(timer_compat.required_features.len(), 1);
        assert_eq!(timer_compat.optional_features.len(), 3);
        assert_eq!(timer_compat.incompatible_features.len(), 0);
        
        let jitter_compat = get_jitter_compatibility();
        assert_eq!(jitter_compat.required_features.len(), 1);
        assert_eq!(jitter_compat.optional_features.len(), 2);
        assert_eq!(jitter_compat.incompatible_features.len(), 0);
    }

    #[test]
    fn test_feature_strings() {
        // These tests depend on runtime state, so they're basic validation
        let feature_string = get_feature_string();
        assert!(!feature_string.is_empty());
        assert!(feature_string.len() < 100); // Reasonable length
    }
}
