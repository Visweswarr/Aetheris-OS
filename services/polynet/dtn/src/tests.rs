use super::*;
use std::time::Duration;
use tokio::time::sleep;

/// Test suite for DTN envelope functionality
#[cfg(test)]
mod dtn_envelope_tests {
    use super::*;
    
    /// Test that expired envelopes are properly dropped
    #[tokio::test]
    async fn test_expired_envelope_dropped() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Create envelope with very short TTL
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            1, // 1 second TTL
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope
        let envelope_id = envelope.id.clone();
        service.send_envelope(envelope).await.unwrap();
        
        // Verify envelope is stored
        let stored_envelope = service.query_envelope(&envelope_id).await.unwrap();
        assert!(stored_envelope.is_some());
        
        // Wait for expiration
        sleep(Duration::from_secs(2)).await;
        
        // Cleanup expired envelopes
        let cleaned = service.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
        
        // Verify envelope is no longer available
        let expired_envelope = service.query_envelope(&envelope_id).await.unwrap();
        assert!(expired_envelope.is_none());
        
        // Check statistics
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_expired, 1);
        assert_eq!(stats.store.total_envelopes, 0);
    }
    
    /// Test that expired envelopes are automatically filtered during retrieval
    #[tokio::test]
    async fn test_expired_envelope_filtered_on_retrieval() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Create envelope with short TTL
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            1, // 1 second TTL
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope
        let envelope_id = envelope.id.clone();
        service.send_envelope(envelope).await.unwrap();
        
        // Wait for expiration
        sleep(Duration::from_secs(2)).await;
        
        // Try to receive envelopes - should not get expired ones
        let received = service.receive_envelopes("did:polynet:dest", 10).await.unwrap();
        assert_eq!(received.len(), 0);
        
        // Verify envelope is still in storage but marked as expired
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_envelopes, 1);
    }
    
    /// Test acknowledgment path functionality
    #[tokio::test]
    async fn test_acknowledgment_path_works() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Create envelope with acknowledgment requirements
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Enable acknowledgments
        envelope.enable_acknowledgments(AckType::Delivery, 300);
        envelope.add_required_ack_node("did:polynet:dest".to_string());
        envelope.add_required_ack_node("did:polynet:intermediate".to_string());
        
        // Send envelope
        let envelope_id = envelope.id.clone();
        service.send_envelope(envelope).await.unwrap();
        
        // Initially no acknowledgments received
        let envelope_before = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert!(!envelope_before.all_acks_received());
        assert_eq!(envelope_before.ack_requirements.ack_records.len(), 0);
        
        // Send acknowledgment from destination
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:dest",
            AckType::Delivery,
            AckStatus::Success,
            "Delivered successfully",
        ).await.unwrap();
        
        // Send acknowledgment from intermediate node
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:intermediate",
            AckType::Forward,
            AckStatus::Success,
            "Forwarded successfully",
        ).await.unwrap();
        
        // Verify all acknowledgments received
        let envelope_after = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert!(envelope_after.all_acks_received());
        assert_eq!(envelope_after.ack_requirements.ack_records.len(), 2);
        
        // Check acknowledgment details
        let dest_ack = envelope_after.ack_requirements.ack_records.iter()
            .find(|ack| ack.ack_node == "did:polynet:dest")
            .unwrap();
        assert_eq!(dest_ack.ack_type, AckType::Delivery);
        assert_eq!(dest_ack.status, AckStatus::Success);
        
        let intermediate_ack = envelope_after.ack_requirements.ack_records.iter()
            .find(|ack| ack.ack_node == "did:polynet:intermediate")
            .unwrap();
        assert_eq!(intermediate_ack.ack_type, AckType::Forward);
        assert_eq!(intermediate_ack.status, AckStatus::Success);
    }
    
    /// Test partial acknowledgment scenarios
    #[tokio::test]
    async fn test_partial_acknowledgment_scenarios() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Create envelope with multiple required acknowledgments
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        envelope.enable_acknowledgments(AckType::Delivery, 300);
        envelope.add_required_ack_node("did:polynet:dest".to_string());
        envelope.add_required_ack_node("did:polynet:intermediate1".to_string());
        envelope.add_required_ack_node("did:polynet:intermediate2".to_string());
        
        // Send envelope
        let envelope_id = envelope.id.clone();
        service.send_envelope(envelope).await.unwrap();
        
        // Send partial acknowledgments
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:dest",
            AckType::Delivery,
            AckStatus::Success,
            "Delivered",
        ).await.unwrap();
        
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:intermediate1",
            AckType::Forward,
            AckStatus::Failure,
            "Forwarding failed",
        ).await.unwrap();
        
        // Verify partial acknowledgment status
        let envelope_partial = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert!(!envelope_partial.all_acks_received());
        assert_eq!(envelope_partial.ack_requirements.ack_records.len(), 2);
        
        // Check failure acknowledgment
        let failure_ack = envelope_partial.ack_requirements.ack_records.iter()
            .find(|ack| ack.ack_node == "did:polynet:intermediate1")
            .unwrap();
        assert_eq!(failure_ack.status, AckStatus::Failure);
        
        // Send final acknowledgment
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:intermediate2",
            AckType::Forward,
            AckStatus::Success,
            "Forwarded",
        ).await.unwrap();
        
        // Now all acknowledgments received
        let envelope_complete = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert!(envelope_complete.all_acks_received());
        assert_eq!(envelope_complete.ack_requirements.ack_records.len(), 3);
    }
    
    /// Test acknowledgment timeout handling
    #[tokio::test]
    async fn test_acknowledgment_timeout() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Create envelope with very short acknowledgment timeout
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        envelope.enable_acknowledgments(AckType::Delivery, 1); // 1 second timeout
        envelope.add_required_ack_node("did:polynet:dest".to_string());
        
        // Send envelope
        let envelope_id = envelope.id.clone();
        service.send_envelope(envelope).await.unwrap();
        
        // Wait for acknowledgment timeout
        sleep(Duration::from_secs(2)).await;
        
        // Send acknowledgment after timeout
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:dest",
            AckType::Delivery,
            AckStatus::Timeout,
            "Acknowledgment timeout",
        ).await.unwrap();
        
        // Verify timeout acknowledgment
        let envelope_timeout = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        let timeout_ack = envelope_timeout.ack_requirements.ack_records.iter()
            .find(|ack| ack.ack_node == "did:polynet:dest")
            .unwrap();
        assert_eq!(timeout_ack.status, AckStatus::Timeout);
    }
    
    /// Test duplicate envelope detection
    #[tokio::test]
    async fn test_duplicate_envelope_detection() {
        let mut config = StoreConfig::default();
        config.detect_duplicates = true;
        let service = DtnService::new(config).unwrap();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope first time
        let envelope_id = envelope.id.clone();
        service.send_envelope(envelope.clone()).await.unwrap();
        
        // Try to send same envelope again
        let result = service.send_envelope(envelope).await;
        assert!(result.is_err());
        
        // Verify only one envelope stored
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_envelopes, 1);
        assert_eq!(stats.store.total_received, 1);
    }
    
    /// Test routing with different strategies
    #[tokio::test]
    async fn test_routing_strategies() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Add routing nodes
        let node1 = RoutingNode {
            node_id: "did:polynet:node1".to_string(),
            capabilities: std::collections::HashSet::new(),
            location: None,
            interfaces: Vec::new(),
            last_seen: std::time::SystemTime::now(),
            storage_capacity: 1024 * 1024,
            available_storage: 512 * 1024,
        };
        
        let node2 = RoutingNode {
            node_id: "did:polynet:node2".to_string(),
            capabilities: std::collections::HashSet::new(),
            location: None,
            interfaces: Vec::new(),
            last_seen: std::time::SystemTime::now(),
            storage_capacity: 1024 * 1024,
            available_storage: 256 * 1024,
        };
        
        service.add_routing_node(node1).await.unwrap();
        service.add_routing_node(node2).await.unwrap();
        
        // Test flood routing
        let mut flood_envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "flood_message".to_string(),
            b"flood payload".to_vec(),
            100,
        );
        flood_envelope.set_routing_strategy(RoutingStrategy::Flood);
        
        let flood_id = service.send_envelope(flood_envelope).await.unwrap();
        assert!(!flood_id.is_empty());
        
        // Test directed routing
        let mut directed_envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "directed_message".to_string(),
            b"directed payload".to_vec(),
            100,
        );
        directed_envelope.set_routing_strategy(RoutingStrategy::Directed);
        
        let directed_id = service.send_envelope(directed_envelope).await.unwrap();
        assert!(!directed_id.is_empty());
        
        // Verify both envelopes stored
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_envelopes, 2);
    }
    
    /// Test security operations
    #[tokio::test]
    async fn test_security_operations() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Generate signing key
        let public_key = service.generate_signing_key().await.unwrap();
        assert_eq!(public_key.len(), 32);
        
        // Create envelope
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "secure_message".to_string(),
            b"secure payload".to_vec(),
            100,
        );
        
        // Send envelope (should be signed automatically)
        let envelope_id = service.send_envelope(envelope).await.unwrap();
        
        // Verify signature
        let retrieved_envelope = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        let is_valid = service.verify_envelope_signature(&retrieved_envelope).await.unwrap();
        assert!(is_valid);
        
        // Check security level
        let security = service.security.read().await;
        let level = security.calculate_security_level(&retrieved_envelope);
        assert_eq!(level, SecurityLevel::Medium);
    }
    
    /// Test envelope metadata and filtering
    #[tokio::test]
    async fn test_envelope_metadata_and_filtering() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Create envelope with metadata
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "metadata_message".to_string(),
            b"metadata payload".to_vec(),
            100,
        );
        
        envelope.add_metadata("priority".to_string(), "high".to_string());
        envelope.add_metadata("category".to_string(), "test".to_string());
        envelope.add_metadata("version".to_string(), "1.0".to_string());
        
        // Send envelope
        let envelope_id = service.send_envelope(envelope).await.unwrap();
        
        // Retrieve and check metadata
        let retrieved_envelope = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert_eq!(retrieved_envelope.get_metadata("priority"), Some(&"high".to_string()));
        assert_eq!(retrieved_envelope.get_metadata("category"), Some(&"test".to_string()));
        assert_eq!(retrieved_envelope.get_metadata("version"), Some(&"1.0".to_string()));
        assert_eq!(retrieved_envelope.get_metadata("nonexistent"), None);
    }
    
    /// Test envelope priority handling
    #[tokio::test]
    async fn test_envelope_priority_handling() {
        let mut config = StoreConfig::default();
        config.respect_priority = true;
        config.eviction_policy = EvictionPolicy::Priority;
        config.max_envelopes = 2;
        let service = DtnService::new(config).unwrap();
        
        // Send low priority envelope
        let low_priority_envelope = DtnEnvelope::new(
            "did:polynet:source1".to_string(),
            "did:polynet:dest1".to_string(),
            3600,
            "low_priority".to_string(),
            b"low priority payload".to_vec(),
            50, // Low priority
        );
        service.send_envelope(low_priority_envelope).await.unwrap();
        
        // Send high priority envelope
        let high_priority_envelope = DtnEnvelope::new(
            "did:polynet:source2".to_string(),
            "did:polynet:dest2".to_string(),
            3600,
            "high_priority".to_string(),
            b"high priority payload".to_vec(),
            200, // High priority
        );
        service.send_envelope(high_priority_envelope).await.unwrap();
        
        // Send medium priority envelope (should trigger eviction of low priority)
        let medium_priority_envelope = DtnEnvelope::new(
            "did:polynet:source3".to_string(),
            "did:polynet:dest3".to_string(),
            3600,
            "medium_priority".to_string(),
            b"medium priority payload".to_vec(),
            100, // Medium priority
        );
        service.send_envelope(medium_priority_envelope).await.unwrap();
        
        // Check that low priority envelope was evicted
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_envelopes, 2);
        assert_eq!(stats.store.total_dropped, 1);
    }
    
    /// Test comprehensive envelope lifecycle
    #[tokio::test]
    async fn test_comprehensive_envelope_lifecycle() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Generate security key
        service.generate_signing_key().await.unwrap();
        
        // Add routing nodes
        let node = RoutingNode {
            node_id: "did:polynet:intermediate".to_string(),
            capabilities: std::collections::HashSet::new(),
            location: None,
            interfaces: Vec::new(),
            last_seen: std::time::SystemTime::now(),
            storage_capacity: 1024 * 1024,
            available_storage: 512 * 1024,
        };
        service.add_routing_node(node).await.unwrap();
        
        // Create comprehensive envelope
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "lifecycle_test".to_string(),
            b"lifecycle payload".to_vec(),
            150,
        );
        
        // Configure routing
        envelope.set_routing_strategy(RoutingStrategy::Directed);
        envelope.add_intermediate_node("did:polynet:intermediate".to_string());
        
        // Configure acknowledgments
        envelope.enable_acknowledgments(AckType::Delivery, 300);
        envelope.add_required_ack_node("did:polynet:dest".to_string());
        envelope.add_required_ack_node("did:polynet:intermediate".to_string());
        
        // Add metadata
        envelope.add_metadata("test_id".to_string(), "lifecycle_001".to_string());
        envelope.add_metadata("environment".to_string(), "test".to_string());
        
        // Send envelope
        let envelope_id = service.send_envelope(envelope).await.unwrap();
        
        // Verify envelope stored
        let stored_envelope = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert_eq!(stored_envelope.id, envelope_id);
        assert_eq!(stored_envelope.get_metadata("test_id"), Some(&"lifecycle_001".to_string()));
        
        // Send acknowledgments
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:intermediate",
            AckType::Forward,
            AckStatus::Success,
            "Forwarded",
        ).await.unwrap();
        
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:dest",
            AckType::Delivery,
            AckStatus::Success,
            "Delivered",
        ).await.unwrap();
        
        // Verify all acknowledgments received
        let final_envelope = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert!(final_envelope.all_acks_received());
        
        // Verify signature
        let is_valid = service.verify_envelope_signature(&final_envelope).await.unwrap();
        assert!(is_valid);
        
        // Check final statistics
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_envelopes, 1);
        assert_eq!(stats.store.total_received, 1);
        assert_eq!(stats.routing.total_nodes, 1);
    }
}

/// Performance benchmarks for DTN operations
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    /// Benchmark envelope creation
    #[test]
    fn benchmark_envelope_creation() {
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _envelope = DtnEnvelope::new(
                "did:polynet:source".to_string(),
                "did:polynet:dest".to_string(),
                3600,
                "benchmark_message".to_string(),
                b"benchmark payload".to_vec(),
                100,
            );
        }
        
        let duration = start.elapsed();
        println!("Created 1000 envelopes in {:?}", duration);
        assert!(duration.as_millis() < 100); // Should be very fast
    }
    
    /// Benchmark envelope signing
    #[test]
    fn benchmark_envelope_signing() {
        let mut security = DtnSecurity::new();
        security.generate_signing_key().unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "benchmark_message".to_string(),
            b"benchmark payload".to_vec(),
            100,
        );
        
        let start = Instant::now();
        
        for _ in 0..100 {
            security.sign_envelope(&mut envelope).unwrap();
        }
        
        let duration = start.elapsed();
        println!("Signed 100 envelopes in {:?}", duration);
        assert!(duration.as_millis() < 1000); // Should be reasonably fast
    }
    
    /// Benchmark envelope storage operations
    #[tokio::test]
    async fn benchmark_envelope_storage() {
        let config = StoreConfig::default();
        let store = DtnStore::new(config).unwrap();
        
        let start = Instant::now();
        
        for i in 0..100 {
            let envelope = DtnEnvelope::new(
                format!("did:polynet:source{}", i),
                format!("did:polynet:dest{}", i),
                3600,
                "benchmark_message".to_string(),
                b"benchmark payload".to_vec(),
                100,
            );
            
            store.store_envelope(envelope).await.unwrap();
        }
        
        let duration = start.elapsed();
        println!("Stored 100 envelopes in {:?}", duration);
        assert!(duration.as_millis() < 5000); // Should be reasonably fast
        
        // Verify all stored
        let stats = store.get_stats().await;
        assert_eq!(stats.total_envelopes, 100);
    }
}
