//! Basic quarantine system usage example
//!
//! Demonstrates how to use the quarantine system for peer management.

use polymera_polynet::{
    QuarantineSystem, QuarantineConfig, QuarantineReason, QuarantineSeverity
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting quarantine system example");

    // Create configuration
    let mut config = QuarantineConfig::default();
    config.data_file = PathBuf::from("example_quarantine.json");
    config.backup_dir = PathBuf::from("example_backups");
    
    // Set shorter durations for demo
    config.default_durations.insert(QuarantineSeverity::Low, Duration::from_secs(10));
    config.default_durations.insert(QuarantineSeverity::Medium, Duration::from_secs(30));
    config.default_durations.insert(QuarantineSeverity::High, Duration::from_secs(60));
    config.cleanup_interval = Duration::from_secs(5);

    // Create quarantine system
    let quarantine = Arc::new(QuarantineSystem::new(config, "example_authority".to_string())?);

    info!("Quarantine system initialized");

    // Example 1: Basic quarantine operations
    demo_basic_operations(&quarantine).await?;
    
    // Example 2: Automatic expiry
    demo_expiry(&quarantine).await?;
    
    // Example 3: Different severity levels
    demo_severity_levels(&quarantine).await?;
    
    // Example 4: Statistics and monitoring
    demo_statistics(&quarantine).await?;
    
    // Example 5: Background cleanup
    demo_background_cleanup(quarantine).await?;

    info!("Quarantine system example completed");
    Ok(())
}

async fn demo_basic_operations(quarantine: &QuarantineSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demo: Basic Operations ===");
    
    let malicious_peer = "peer_malicious_001";
    let spam_peer = "peer_spam_002";
    
    // Check initial state
    info!("Checking if peers are quarantined...");
    info!("Malicious peer quarantined: {}", quarantine.is_quarantined(malicious_peer)?);
    info!("Spam peer quarantined: {}", quarantine.is_quarantined(spam_peer)?);
    
    // Quarantine malicious peer
    info!("Quarantining malicious peer...");
    let entry_id = quarantine.quarantine_peer(
        malicious_peer.to_string(),
        QuarantineReason::MaliciousBehavior,
        QuarantineSeverity::High,
        Some("Detected sending corrupted consensus messages".to_string()),
        Some(HashMap::from([
            ("detection_method".to_string(), "consensus_validator".to_string()),
            ("severity_score".to_string(), "8.5".to_string()),
        ]))
    )?;
    info!("Malicious peer quarantined with entry ID: {}", entry_id);
    
    // Quarantine spam peer
    info!("Quarantining spam peer...");
    quarantine.quarantine_peer(
        spam_peer.to_string(),
        QuarantineReason::Spam,
        QuarantineSeverity::Medium,
        Some("Flooding network with excessive requests".to_string()),
        None,
    )?;
    info!("Spam peer quarantined");
    
    // Check updated state
    info!("Checking quarantine status after adding...");
    info!("Malicious peer quarantined: {}", quarantine.is_quarantined(malicious_peer)?);
    info!("Spam peer quarantined: {}", quarantine.is_quarantined(spam_peer)?);
    
    // Get detailed entry information
    if let Some(entry) = quarantine.get_quarantine_entry(malicious_peer)? {
        info!("Malicious peer entry details:");
        info!("  Reason: {:?}", entry.reason);
        info!("  Severity: {:?}", entry.severity);
        info!("  Evidence: {:?}", entry.evidence);
        info!("  Quarantined at: {:?}", entry.quarantined_at);
        info!("  Expires at: {:?}", entry.expires_at);
        info!("  Metadata: {:?}", entry.metadata);
    }
    
    // Unquarantine spam peer
    info!("Removing spam peer from quarantine...");
    let removed = quarantine.unquarantine_peer(spam_peer)?;
    info!("Spam peer removed: {}", removed);
    
    // Check final state
    info!("Final quarantine status:");
    info!("Malicious peer quarantined: {}", quarantine.is_quarantined(malicious_peer)?);
    info!("Spam peer quarantined: {}", quarantine.is_quarantined(spam_peer)?);
    
    Ok(())
}

async fn demo_expiry(quarantine: &QuarantineSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demo: Automatic Expiry ===");
    
    let temp_peer = "peer_temporary_003";
    
    // Quarantine with low severity (short duration)
    info!("Quarantining peer with low severity (10 second expiry)...");
    quarantine.quarantine_peer(
        temp_peer.to_string(),
        QuarantineReason::RateLimitViolation,
        QuarantineSeverity::Low,
        Some("Temporary rate limit violation".to_string()),
        None,
    )?;
    
    info!("Temporary peer quarantined: {}", quarantine.is_quarantined(temp_peer)?);
    
    // Wait for expiry
    info!("Waiting 12 seconds for quarantine to expire...");
    sleep(Duration::from_secs(12)).await;
    
    info!("After expiry - temporary peer quarantined: {}", quarantine.is_quarantined(temp_peer)?);
    
    // Manual cleanup to demonstrate
    let cleaned = quarantine.cleanup_expired()?;
    info!("Cleaned up {} expired entries", cleaned);
    
    Ok(())
}

async fn demo_severity_levels(quarantine: &QuarantineSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demo: Different Severity Levels ===");
    
    let peers = [
        ("peer_low_004", QuarantineSeverity::Low, "Minor protocol deviation"),
        ("peer_medium_005", QuarantineSeverity::Medium, "Moderate violations"),
        ("peer_high_006", QuarantineSeverity::High, "Serious security issue"),
        ("peer_critical_007", QuarantineSeverity::Critical, "Critical system threat"),
        ("peer_permanent_008", QuarantineSeverity::Permanent, "Known malicious actor"),
    ];
    
    for (peer_id, severity, evidence) in peers {
        info!("Quarantining {} with {:?} severity", peer_id, severity);
        quarantine.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::Custom(format!("Demo {:?} violation", severity)),
            severity,
            Some(evidence.to_string()),
            None,
        )?;
        
        if let Some(entry) = quarantine.get_quarantine_entry(peer_id)? {
            match entry.expires_at {
                Some(expires) => info!("  Expires at: {:?}", expires),
                None => info!("  Permanent quarantine"),
            }
        }
    }
    
    Ok(())
}

async fn demo_statistics(quarantine: &QuarantineSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demo: Statistics and Monitoring ===");
    
    let stats = quarantine.get_statistics()?;
    
    info!("Quarantine Statistics:");
    info!("  Total entries: {}", stats.total_entries);
    info!("  Active entries: {}", stats.active_entries);
    info!("  Expired entries: {}", stats.expired_entries);
    info!("  File size: {} bytes", stats.file_size);
    
    info!("Entries by reason:");
    for (reason, count) in &stats.entries_by_reason {
        info!("  {}: {}", reason, count);
    }
    
    info!("Entries by severity:");
    for (severity, count) in &stats.entries_by_severity {
        info!("  {}: {}", severity, count);
    }
    
    // List all quarantined peers
    info!("All quarantined peers:");
    let entries = quarantine.list_quarantined_peers(false)?;
    for entry in entries {
        info!("  {} - {:?} - {:?}", entry.peer_id, entry.reason, entry.severity);
    }
    
    Ok(())
}

async fn demo_background_cleanup(quarantine: Arc<QuarantineSystem>) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demo: Background Cleanup ===");
    
    // Add a peer with very short expiry for demo
    quarantine.quarantine_peer(
        "peer_auto_cleanup_009".to_string(),
        QuarantineReason::Spam,
        QuarantineSeverity::Low,
        Some("Will be auto-cleaned".to_string()),
        None,
    )?;
    
    info!("Added peer for auto-cleanup demo");
    
    // Start background cleanup task
    let quarantine_clone = Arc::clone(&quarantine);
    let cleanup_task = tokio::spawn(async move {
        quarantine_clone.start_cleanup_task().await;
    });
    
    // Wait and observe automatic cleanup
    info!("Waiting for automatic cleanup...");
    for i in 1..=15 {
        sleep(Duration::from_secs(1)).await;
        let is_quarantined = quarantine.is_quarantined("peer_auto_cleanup_009")?;
        if i % 5 == 0 {
            info!("After {} seconds: peer still quarantined: {}", i, is_quarantined);
        }
        if !is_quarantined && i > 10 {
            info!("Peer automatically removed from quarantine after {} seconds", i);
            break;
        }
    }
    
    // Stop background task
    cleanup_task.abort();
    
    // Final statistics
    let final_stats = quarantine.get_statistics()?;
    info!("Final statistics:");
    info!("  Total entries: {}", final_stats.total_entries);
    info!("  Active entries: {}", final_stats.active_entries);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_quarantine() -> (QuarantineSystem, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = QuarantineConfig::default();
        config.data_file = temp_dir.path().join("test_quarantine.json");
        config.backup_dir = temp_dir.path().join("backups");
        
        let quarantine = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        (quarantine, temp_dir)
    }

    #[tokio::test]
    async fn test_example_operations() {
        let (quarantine, _temp_dir) = create_test_quarantine();
        
        // Test basic operations
        assert!(demo_basic_operations(&quarantine).await.is_ok());
        
        // Test statistics
        assert!(demo_statistics(&quarantine).await.is_ok());
    }
}
