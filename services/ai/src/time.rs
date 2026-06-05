//! Time utilities for 90kHz timestamp base used across Phase 5 components

/// Return current timestamp in 90kHz timebase as u64
pub fn get_time_90khz() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    d.as_secs()
        .saturating_mul(90_000)
        .saturating_add((d.subsec_nanos() as u64).saturating_mul(90_000) / 1_000_000_000)
}
