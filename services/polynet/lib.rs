//! Polymera OS Network Services
//!
//! Provides comprehensive networking capabilities including peer management,
//! quarantine systems, mesh networking, and protocol implementations.

pub mod quarantine;

// Re-export main types
pub use quarantine::{
    QuarantineSystem, QuarantineConfig, QuarantineEntry, QuarantineList,
    QuarantineReason, QuarantineSeverity, QuarantineStats, QuarantineError,
    QuarantineResult,
};

// Optional libp2p integration
#[cfg(feature = "libp2p-support")]
pub mod p2p;

#[cfg(feature = "libp2p-support")]
pub use p2p::*;

// Optional QUIC support
#[cfg(feature = "quic-support")]
pub mod quic;

#[cfg(feature = "quic-support")]
pub use quic::*;

// Optional metrics
#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "metrics")]
pub use self::metrics::*;
