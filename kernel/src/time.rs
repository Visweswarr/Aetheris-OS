//! Time module stub for no_std kernel
//! Provides Duration and Instant types compatible with no_std

use core::ops::{Add, Sub};

/// Duration type for time intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Duration {
    nanos: u64,
}

impl Duration {
    pub const ZERO: Duration = Duration { nanos: 0 };
    
    pub const fn from_secs(secs: u64) -> Self {
        Self { nanos: secs * 1_000_000_000 }
    }
    
    pub const fn from_millis(millis: u64) -> Self {
        Self { nanos: millis * 1_000_000 }
    }
    
    pub const fn from_micros(micros: u64) -> Self {
        Self { nanos: micros * 1_000 }
    }
    
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }
    
    pub const fn as_secs(&self) -> u64 {
        self.nanos / 1_000_000_000
    }
    
    pub const fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }
    
    pub const fn as_micros(&self) -> u64 {
        self.nanos / 1_000
    }
    
    pub const fn as_nanos(&self) -> u64 {
        self.nanos
    }
    
    pub const fn subsec_nanos(&self) -> u32 {
        (self.nanos % 1_000_000_000) as u32
    }
}

impl Add for Duration {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { nanos: self.nanos + rhs.nanos }
    }
}

impl Sub for Duration {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { nanos: self.nanos.saturating_sub(rhs.nanos) }
    }
}

/// Instant type for timestamps
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Instant {
    nanos: u64,
}

impl Instant {
    pub fn now() -> Self {
        // Use kernel time source
        Self { nanos: crate::log::get_current_time_ms() * 1_000_000 }
    }

    /// Construct an instant from a monotonic millisecond value.
    pub const fn from_millis_since_boot(millis: u64) -> Self {
        Self { nanos: millis * 1_000_000 }
    }

    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        Duration::from_nanos(now.nanos.saturating_sub(self.nanos))
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from_nanos(self.nanos.saturating_sub(earlier.nanos))
    }

    /// Milliseconds since boot represented by this instant.
    pub const fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }

    /// Microseconds since boot represented by this instant.
    pub const fn as_micros(&self) -> u64 {
        self.nanos / 1_000
    }

    /// Nanoseconds since boot represented by this instant.
    pub const fn as_nanos(&self) -> u64 {
        self.nanos
    }
    
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.nanos.checked_add(duration.nanos).map(|nanos| Self { nanos })
    }
    
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.nanos.checked_sub(duration.nanos).map(|nanos| Self { nanos })
    }
}

impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        Self { nanos: self.nanos + rhs.nanos }
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        Self { nanos: self.nanos.saturating_sub(rhs.nanos) }
    }
}

impl Sub for Instant {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Duration {
        Duration::from_nanos(self.nanos.saturating_sub(rhs.nanos))
    }
}

/// Wall-clock time in milliseconds since boot (delegates to the log module's monotonic counter).
#[inline]
pub fn get_current_time_ms() -> u64 {
    crate::log::get_current_time_ms()
}

/// Wall-clock time in seconds since boot.
#[inline]
pub fn now_secs() -> u64 {
    get_current_time_ms() / 1_000
}

/// Wall-clock time in 90-kHz ticks (matches NTP RTP timebase used by audit log).
#[inline]
pub fn get_time_90khz() -> u64 {
    // 1 ms = 90 ticks at 90 kHz
    get_current_time_ms().saturating_mul(90)
}
