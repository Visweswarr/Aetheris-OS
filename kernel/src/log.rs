/// Enhanced Logging System for Polymera OS
/// 
/// Provides structured logging with module tags, rate limiting, and configurable levels.
/// Supports production debugging and monitoring with minimal performance overhead.

use core::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use core::hash::{Hash, Hasher};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

/// Log level configuration
static LEVEL: AtomicU8 = AtomicU8::new(2); // 0=TRACE,1=INFO,2=WARN,3=ERR

/// Global timestamp counter for rate limiting
static TIMESTAMP_MS: AtomicU64 = AtomicU64::new(0);

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    TRACE = 0,
    INFO = 1,
    WARN = 2,
    ERR = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::TRACE => "TRACE",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERR => "ERR",
        }
    }
    
    pub fn as_color_code(&self) -> &'static str {
        match self {
            LogLevel::TRACE => "\x1b[90m",    // Gray
            LogLevel::INFO => "\x1b[36m",     // Cyan
            LogLevel::WARN => "\x1b[33m",     // Yellow
            LogLevel::ERR => "\x1b[31m",      // Red
        }
    }
}

/// Common module tags
pub mod tags {
    pub const BOOT: &str = "BOOT";
    pub const SCHED: &str = "SCHED";
    pub const IPC: &str = "IPC";
    pub const MM: &str = "MM";
    pub const HAL: &str = "HAL";
    pub const SECURITY: &str = "SECURITY";
    pub const SECMAN: &str = "SECMAN";
    pub const SYSCALL: &str = "SYSCALL";
    pub const TIMER: &str = "TIMER";
    pub const IDT: &str = "IDT";
    pub const DEMO: &str = "DEMO";
    pub const TEST: &str = "TEST";
    pub const VALIDATOR: &str = "VALIDATOR";
    pub const AUDIT: &str = "AUDIT";
    pub const PANIC: &str = "PANIC";
    pub const SERIAL: &str = "SERIAL";
    pub const ASSERT: &str = "ASSERT";
    pub const FAULT_INJECTION: &str = "FAULT_INJECTION";
}

/// Rate limiting configuration
const RATE_LIMIT_THRESHOLD: u32 = 5; // Suppress after N identical messages
const RATE_LIMIT_WINDOW_MS: u64 = 1000; // Within 1 second window
const MAX_RATE_LIMIT_ENTRIES: usize = 256; // Maximum tracked unique messages

/// Simple hash function for log messages
struct SimpleHasher {
    hash: u64,
}

impl SimpleHasher {
    fn new() -> Self {
        Self { hash: 0 }
    }
}

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash = self.hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
    }
}

/// Rate limiting entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// First occurrence timestamp
    first_seen_ms: u64,
    
    /// Last occurrence timestamp
    last_seen_ms: u64,
    
    /// Number of occurrences
    count: u32,
    
    /// Number of suppressed messages shown
    suppressions_shown: u32,
    
    /// Sample message for suppression reporting
    sample_message: String,
}

impl RateLimitEntry {
    fn new(timestamp_ms: u64, message: String) -> Self {
        Self {
            first_seen_ms: timestamp_ms,
            last_seen_ms: timestamp_ms,
            count: 1,
            suppressions_shown: 0,
            sample_message: message,
        }
    }
    
    fn update(&mut self, timestamp_ms: u64) {
        self.last_seen_ms = timestamp_ms;
        self.count += 1;
    }
    
    fn should_suppress(&self, timestamp_ms: u64) -> bool {
        // Check if we're within the rate limit window
        if timestamp_ms.saturating_sub(self.first_seen_ms) <= RATE_LIMIT_WINDOW_MS {
            self.count > RATE_LIMIT_THRESHOLD
        } else {
            // Outside window, reset suppression
            false
        }
    }
    
    fn should_show_suppression_summary(&self, timestamp_ms: u64) -> bool {
        // Show suppression summary if:
        // 1. We have suppressions to report
        // 2. We're past the rate limit window OR haven't shown in a while
        self.count > RATE_LIMIT_THRESHOLD && (
            timestamp_ms.saturating_sub(self.first_seen_ms) > RATE_LIMIT_WINDOW_MS ||
            timestamp_ms.saturating_sub(self.last_seen_ms) > 500 // Show every 500ms during active suppression
        )
    }
    
    fn get_suppressed_count(&self) -> u32 {
        self.count.saturating_sub(RATE_LIMIT_THRESHOLD)
    }
}

/// Rate limiting state
static RATE_LIMITER: Mutex<BTreeMap<u64, RateLimitEntry>> = Mutex::new(BTreeMap::new());

/// Hash a log message for rate limiting
fn hash_message(level: LogLevel, tag: &str, message: &str) -> u64 {
    let mut hasher = SimpleHasher::new();
    level.hash(&mut hasher);
    tag.hash(&mut hasher);
    message.hash(&mut hasher);
    hasher.finish()
}

/// Get current timestamp in milliseconds
pub fn get_current_time_ms() -> u64 {
    #[cfg(feature = "deterministic")]
    {
        if crate::determinism::is_determinism_enabled() {
            return crate::determinism::get_global_time_ms();
        }
    }
    
    TIMESTAMP_MS.load(Ordering::Relaxed)
}

/// Update current timestamp
pub fn update_timestamp_ms(timestamp_ms: u64) {
    TIMESTAMP_MS.store(timestamp_ms, Ordering::Relaxed);
}

/// Check if a log message should be rate limited
fn should_rate_limit(level: LogLevel, tag: &str, message: &str) -> (bool, Option<String>) {
    let current_time = get_current_time_ms();
    let hash = hash_message(level, tag, message);
    
    let mut limiter = RATE_LIMITER.lock();
    
    // Clean up old entries if we have too many
    if limiter.len() >= MAX_RATE_LIMIT_ENTRIES {
        let cutoff_time = current_time.saturating_sub(RATE_LIMIT_WINDOW_MS * 2);
        limiter.retain(|_, entry| entry.last_seen_ms >= cutoff_time);
    }
    
    match limiter.get_mut(&hash) {
        Some(entry) => {
            let should_suppress = entry.should_suppress(current_time);
            let suppression_summary = if entry.should_show_suppression_summary(current_time) {
                let suppressed_count = entry.get_suppressed_count();
                entry.suppressions_shown += 1;
                Some(alloc::format!(
                    "{}[{}] {} … ×{} suppressed (last {}ms ago)", 
                    level.as_color_code(),
                    tag,
                    level.as_str(),
                    suppressed_count,
                    current_time.saturating_sub(entry.first_seen_ms)
                ))
            } else {
                None
            };
            
            entry.update(current_time);
            
            // If we're outside the window, reset the entry
            if current_time.saturating_sub(entry.first_seen_ms) > RATE_LIMIT_WINDOW_MS {
                entry.first_seen_ms = current_time;
                entry.count = 1;
                entry.suppressions_shown = 0;
                (false, suppression_summary)
            } else {
                (should_suppress, suppression_summary)
            }
        }
        None => {
            // First occurrence of this message
            limiter.insert(hash, RateLimitEntry::new(current_time, String::from(message)));
            (false, None)
        }
    }
}

/// Core logging function with rate limiting and tagging
pub fn log_with_rate_limit(level: LogLevel, tag: &str, message: &str) {
    // Check log level first
    if (level as u8) < lvl() {
        return;
    }
    
    // Check rate limiting
    let (should_suppress, suppression_summary) = should_rate_limit(level, tag, message);
    
    // Show suppression summary if available
    if let Some(summary) = suppression_summary {
        crate::serial::kprintln!("{}\x1b[0m", summary);
    }
    
    // Skip the actual message if suppressed
    if should_suppress {
        return;
    }
    
    // Format and output the log message
    let timestamp = get_current_time_ms();
    crate::serial::kprintln!(
        "{}[{:>6}] [{}] {} {}\x1b[0m", 
        level.as_color_code(),
        timestamp,
        tag,
        level.as_str(),
        message
    );
}

/// Initialize logging subsystem
pub fn init_levels() { 
    LEVEL.store(1, Ordering::Relaxed);
    update_timestamp_ms(1000); // Start at 1 second
    
    // Clear any existing rate limit state
    RATE_LIMITER.lock().clear();
    
    crate::serial::kprintln!("[LOG] Enhanced logging system initialized with rate limiting");
}

/// Get current log level
pub fn lvl() -> u8 { 
    LEVEL.load(Ordering::Relaxed) 
}

/// Set log level
pub fn set_level(level: LogLevel) {
    LEVEL.store(level as u8, Ordering::Relaxed);
    log_with_rate_limit(LogLevel::INFO, tags::BOOT, &alloc::format!("Log level set to {}", level.as_str()));
}

/// Get rate limiting statistics
pub fn get_rate_limit_stats() -> (usize, u32) {
    let limiter = RATE_LIMITER.lock();
    let active_entries = limiter.len();
    let total_suppressions: u32 = limiter.values()
        .map(|entry| entry.get_suppressed_count())
        .sum();
    (active_entries, total_suppressions)
}

/// Clear rate limiting state
pub fn clear_rate_limit_state() {
    RATE_LIMITER.lock().clear();
    log_with_rate_limit(LogLevel::INFO, tags::BOOT, "Rate limiting state cleared");
}

/// Print rate limiting statistics
pub fn print_rate_limit_stats() {
    let (active_entries, total_suppressions) = get_rate_limit_stats();
    crate::serial::kprintln!("");
    crate::serial::kprintln!("=== RATE LIMITING STATISTICS ===");
    crate::serial::kprintln!("Active tracked messages: {}", active_entries);
    crate::serial::kprintln!("Total suppressions: {}", total_suppressions);
    crate::serial::kprintln!("Rate limit threshold: {} messages per {}ms", RATE_LIMIT_THRESHOLD, RATE_LIMIT_WINDOW_MS);
    crate::serial::kprintln!("=== END RATE LIMITING STATISTICS ===");
    crate::serial::kprintln!("");
}

/// Test rate limiting functionality
pub fn test_rate_limiting() {
    crate::serial::kprintln!("");
    crate::serial::kprintln!("=== RATE LIMITING TEST ===");
    
    // Test repeated messages
    for i in 0..10 {
        log_with_rate_limit(LogLevel::INFO, tags::TEST, "Repeated test message");
        update_timestamp_ms(get_current_time_ms() + 50); // Advance time by 50ms
    }
    
    // Advance time past window
    update_timestamp_ms(get_current_time_ms() + 2000);
    
    // Should not be suppressed after window
    log_with_rate_limit(LogLevel::INFO, tags::TEST, "Repeated test message");
    
    print_rate_limit_stats();
    
    crate::serial::kprintln!("=== RATE LIMITING TEST COMPLETE ===");
    crate::serial::kprintln!("");
}

#[macro_export]
macro_rules! klog {
    // New format: klog!(LEVEL, [TAG], "message", args...)
    (TRACE, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 0 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::TRACE, $tag, &message);
        }
    };
    (INFO, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 1 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::INFO, $tag, &message);
        }
    };
    (WARN, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 2 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::WARN, $tag, &message);
        }
    };
    (ERR, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 3 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::ERR, $tag, &message);
        }
    };
    
    // Legacy format: klog!(LEVEL, "message", args...) - defaults to no tag
    (TRACE, $($t:tt)*) => {
        if crate::log::lvl() <= 0 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::TRACE, "KERN", &message);
        }
    };
    (INFO, $($t:tt)*) => {
        if crate::log::lvl() <= 1 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::INFO, "KERN", &message);
        }
    };
    (WARN, $($t:tt)*) => {
        if crate::log::lvl() <= 2 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::WARN, "KERN", &message);
        }
    };
    (ERR, $($t:tt)*) => {
        if crate::log::lvl() <= 3 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::ERR, "KERN", &message);
        }
    };
}

/// Global timestamp counter for rate limiting
static TIMESTAMP_MS: AtomicU64 = AtomicU64::new(0);

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    TRACE = 0,
    INFO = 1,
    WARN = 2,
    ERR = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::TRACE => "TRACE",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERR => "ERR",
        }
    }
    
    pub fn as_color_code(&self) -> &'static str {
        match self {
            LogLevel::TRACE => "\x1b[90m",    // Gray
            LogLevel::INFO => "\x1b[36m",     // Cyan
            LogLevel::WARN => "\x1b[33m",     // Yellow
            LogLevel::ERR => "\x1b[31m",      // Red
        }
    }
}

/// Common module tags
pub mod tags {
    pub const BOOT: &str = "BOOT";
    pub const SCHED: &str = "SCHED";
    pub const IPC: &str = "IPC";
    pub const MM: &str = "MM";
    pub const HAL: &str = "HAL";
    pub const SECURITY: &str = "SECURITY";
    pub const SECMAN: &str = "SECMAN";
    pub const SYSCALL: &str = "SYSCALL";
    pub const TIMER: &str = "TIMER";
    pub const IDT: &str = "IDT";
    pub const DEMO: &str = "DEMO";
    pub const TEST: &str = "TEST";
    pub const VALIDATOR: &str = "VALIDATOR";
    pub const AUDIT: &str = "AUDIT";
    pub const PANIC: &str = "PANIC";
    pub const SERIAL: &str = "SERIAL";
}

/// Rate limiting configuration
const RATE_LIMIT_THRESHOLD: u32 = 5; // Suppress after N identical messages
const RATE_LIMIT_WINDOW_MS: u64 = 1000; // Within 1 second window
const MAX_RATE_LIMIT_ENTRIES: usize = 256; // Maximum tracked unique messages

/// Simple hash function for log messages
struct SimpleHasher {
    hash: u64,
}

impl SimpleHasher {
    fn new() -> Self {
        Self { hash: 0 }
    }
}

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash = self.hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
    }
}

/// Rate limiting entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// First occurrence timestamp
    first_seen_ms: u64,
    
    /// Last occurrence timestamp
    last_seen_ms: u64,
    
    /// Number of occurrences
    count: u32,
    
    /// Number of suppressed messages shown
    suppressions_shown: u32,
    
    /// Sample message for suppression reporting
    sample_message: String,
}

impl RateLimitEntry {
    fn new(timestamp_ms: u64, message: String) -> Self {
        Self {
            first_seen_ms: timestamp_ms,
            last_seen_ms: timestamp_ms,
            count: 1,
            suppressions_shown: 0,
            sample_message: message,
        }
    }
    
    fn update(&mut self, timestamp_ms: u64) {
        self.last_seen_ms = timestamp_ms;
        self.count += 1;
    }
    
    fn should_suppress(&self, timestamp_ms: u64) -> bool {
        // Check if we're within the rate limit window
        if timestamp_ms.saturating_sub(self.first_seen_ms) <= RATE_LIMIT_WINDOW_MS {
            self.count > RATE_LIMIT_THRESHOLD
        } else {
            // Outside window, reset suppression
            false
        }
    }
    
    fn should_show_suppression_summary(&self, timestamp_ms: u64) -> bool {
        // Show suppression summary if:
        // 1. We have suppressions to report
        // 2. We're past the rate limit window OR haven't shown in a while
        self.count > RATE_LIMIT_THRESHOLD && (
            timestamp_ms.saturating_sub(self.first_seen_ms) > RATE_LIMIT_WINDOW_MS ||
            timestamp_ms.saturating_sub(self.last_seen_ms) > 500 // Show every 500ms during active suppression
        )
    }
    
    fn get_suppressed_count(&self) -> u32 {
        self.count.saturating_sub(RATE_LIMIT_THRESHOLD)
    }
}

/// Rate limiting state
static RATE_LIMITER: Mutex<BTreeMap<u64, RateLimitEntry>> = Mutex::new(BTreeMap::new());

/// Hash a log message for rate limiting
fn hash_message(level: LogLevel, tag: &str, message: &str) -> u64 {
    let mut hasher = SimpleHasher::new();
    level.hash(&mut hasher);
    tag.hash(&mut hasher);
    message.hash(&mut hasher);
    hasher.finish()
}

/// Get current timestamp in milliseconds
pub fn get_current_time_ms() -> u64 {
    TIMESTAMP_MS.load(Ordering::Relaxed)
}

/// Update current timestamp
pub fn update_timestamp_ms(timestamp_ms: u64) {
    TIMESTAMP_MS.store(timestamp_ms, Ordering::Relaxed);
}

/// Check if a log message should be rate limited
fn should_rate_limit(level: LogLevel, tag: &str, message: &str) -> (bool, Option<String>) {
    let current_time = get_current_time_ms();
    let hash = hash_message(level, tag, message);
    
    let mut limiter = RATE_LIMITER.lock();
    
    // Clean up old entries if we have too many
    if limiter.len() >= MAX_RATE_LIMIT_ENTRIES {
        let cutoff_time = current_time.saturating_sub(RATE_LIMIT_WINDOW_MS * 2);
        limiter.retain(|_, entry| entry.last_seen_ms >= cutoff_time);
    }
    
    match limiter.get_mut(&hash) {
        Some(entry) => {
            let should_suppress = entry.should_suppress(current_time);
            let suppression_summary = if entry.should_show_suppression_summary(current_time) {
                let suppressed_count = entry.get_suppressed_count();
                entry.suppressions_shown += 1;
                Some(alloc::format!(
                    "{}[{}] {} … ×{} suppressed (last {}ms ago)", 
                    level.as_color_code(),
                    tag,
                    level.as_str(),
                    suppressed_count,
                    current_time.saturating_sub(entry.first_seen_ms)
                ))
            } else {
                None
            };
            
            entry.update(current_time);
            
            // If we're outside the window, reset the entry
            if current_time.saturating_sub(entry.first_seen_ms) > RATE_LIMIT_WINDOW_MS {
                entry.first_seen_ms = current_time;
                entry.count = 1;
                entry.suppressions_shown = 0;
                (false, suppression_summary)
            } else {
                (should_suppress, suppression_summary)
            }
        }
        None => {
            // First occurrence of this message
            limiter.insert(hash, RateLimitEntry::new(current_time, String::from(message)));
            (false, None)
        }
    }
}

/// Core logging function with rate limiting and tagging
pub fn log_with_rate_limit(level: LogLevel, tag: &str, message: &str) {
    // Check log level first
    if (level as u8) < lvl() {
        return;
    }
    
    // Check rate limiting
    let (should_suppress, suppression_summary) = should_rate_limit(level, tag, message);
    
    // Show suppression summary if available
    if let Some(summary) = suppression_summary {
        crate::serial::kprintln!("{}\x1b[0m", summary);
    }
    
    // Skip the actual message if suppressed
    if should_suppress {
        return;
    }
    
    // Format and output the log message
    let timestamp = get_current_time_ms();
    crate::serial::kprintln!(
        "{}[{:>6}] [{}] {} {}\x1b[0m", 
        level.as_color_code(),
        timestamp,
        tag,
        level.as_str(),
        message
    );
}

/// Initialize logging subsystem
pub fn init_levels() { 
    LEVEL.store(1, Ordering::Relaxed);
    update_timestamp_ms(1000); // Start at 1 second
    
    // Clear any existing rate limit state
    RATE_LIMITER.lock().clear();
    
    crate::serial::kprintln!("[LOG] Enhanced logging system initialized with rate limiting");
}

/// Get current log level
pub fn lvl() -> u8 { 
    LEVEL.load(Ordering::Relaxed) 
}

/// Set log level
pub fn set_level(level: LogLevel) {
    LEVEL.store(level as u8, Ordering::Relaxed);
    log_with_rate_limit(LogLevel::INFO, tags::BOOT, &alloc::format!("Log level set to {}", level.as_str()));
}

/// Get rate limiting statistics
pub fn get_rate_limit_stats() -> (usize, u32) {
    let limiter = RATE_LIMITER.lock();
    let active_entries = limiter.len();
    let total_suppressions: u32 = limiter.values()
        .map(|entry| entry.get_suppressed_count())
        .sum();
    (active_entries, total_suppressions)
}

/// Clear rate limiting state
pub fn clear_rate_limit_state() {
    RATE_LIMITER.lock().clear();
    log_with_rate_limit(LogLevel::INFO, tags::BOOT, "Rate limiting state cleared");
}

/// Print rate limiting statistics
pub fn print_rate_limit_stats() {
    let (active_entries, total_suppressions) = get_rate_limit_stats();
    crate::serial::kprintln!("");
    crate::serial::kprintln!("=== RATE LIMITING STATISTICS ===");
    crate::serial::kprintln!("Active tracked messages: {}", active_entries);
    crate::serial::kprintln!("Total suppressions: {}", total_suppressions);
    crate::serial::kprintln!("Rate limit threshold: {} messages per {}ms", RATE_LIMIT_THRESHOLD, RATE_LIMIT_WINDOW_MS);
    crate::serial::kprintln!("=== END RATE LIMITING STATISTICS ===");
    crate::serial::kprintln!("");
}

/// Test rate limiting functionality
pub fn test_rate_limiting() {
    crate::serial::kprintln!("");
    crate::serial::kprintln!("=== RATE LIMITING TEST ===");
    
    // Test repeated messages
    for i in 0..10 {
        log_with_rate_limit(LogLevel::INFO, tags::TEST, "Repeated test message");
        update_timestamp_ms(get_current_time_ms() + 50); // Advance time by 50ms
    }
    
    // Advance time past window
    update_timestamp_ms(get_current_time_ms() + 2000);
    
    // Should not be suppressed after window
    log_with_rate_limit(LogLevel::INFO, tags::TEST, "Repeated test message");
    
    print_rate_limit_stats();
    
    crate::serial::kprintln!("=== RATE LIMITING TEST COMPLETE ===");
    crate::serial::kprintln!("");
}

#[macro_export]
macro_rules! klog {
    // New format: klog!(LEVEL, [TAG], "message", args...)
    (TRACE, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 0 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::TRACE, $tag, &message);
        }
    };
    (INFO, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 1 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::INFO, $tag, &message);
        }
    };
    (WARN, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 2 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::WARN, $tag, &message);
        }
    };
    (ERR, [$tag:expr], $($t:tt)*) => {
        if crate::log::lvl() <= 3 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::ERR, $tag, &message);
        }
    };
    
    // Legacy format: klog!(LEVEL, "message", args...) - defaults to no tag
    (TRACE, $($t:tt)*) => {
        if crate::log::lvl() <= 0 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::TRACE, "KERN", &message);
        }
    };
    (INFO, $($t:tt)*) => {
        if crate::log::lvl() <= 1 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::INFO, "KERN", &message);
        }
    };
    (WARN, $($t:tt)*) => {
        if crate::log::lvl() <= 2 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::WARN, "KERN", &message);
        }
    };
    (ERR, $($t:tt)*) => {
        if crate::log::lvl() <= 3 {
            let message = alloc::format!($($t)*);
            crate::log::log_with_rate_limit(crate::log::LogLevel::ERR, "KERN", &message);
        }
    };
}
pub fn lvl() -> u8 { LEVEL.load(core::sync::atomic::Ordering::Relaxed) }