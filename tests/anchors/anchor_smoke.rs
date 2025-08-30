use ngfs::anchor::{AnchorService, AnchorRequest, AnchorPriority, AnchorMetadata, ChainConfig};

#[tokio::test]
async fn test_anchor_service_creation() {
    let config = ChainConfig::default();
    let service = AnchorService::new(config);
    
    // Basic test that service can be created
    assert!(true);
}

#[tokio::test]
async fn test_anchor_metadata_default() {
    let metadata = AnchorMetadata::default();
    
    assert_eq!(metadata.description, None);
    assert_eq!(metadata.tags.len(), 0);
    assert_eq!(metadata.priority, AnchorPriority::Normal);
    assert_eq!(metadata.batch_id, None);
    assert_eq!(metadata.expires_at, None);
    assert_eq!(metadata.custom_fields.len(), 0);
}

#[tokio::test]
async fn test_anchor_priority_display() {
    assert_eq!(AnchorPriority::Low.to_string(), "low");
    assert_eq!(AnchorPriority::Normal.to_string(), "normal");
    assert_eq!(AnchorPriority::High.to_string(), "high");
    assert_eq!(AnchorPriority::Critical.to_string(), "critical");
}
