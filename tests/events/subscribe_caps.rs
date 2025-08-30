use kernel::event::fabric::{EventFabric, Pattern};
use kernel::event::queue::Lane;
use kernel::secman::cap_flags::*;

#[test]
fn test_subscribe_with_valid_caps() {
    let fabric = EventFabric::new();
    let task_id = 123;
    
    // Test exact pattern subscription with valid caps
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY);
    assert!(result.is_ok());
    
    // Test prefix pattern subscription with elevated caps
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::MED, CAP_INTENT_SUBMIT);
    assert!(result.is_ok());
}

#[test]
fn test_subscribe_without_caps() {
    let fabric = EventFabric::new();
    let task_id = 124;
    
    // Test subscription without required caps
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::HI, 0);
    assert!(result.is_ok()); // Basic subscription doesn't require caps
    
    // Test prefix subscription without elevated caps (should fail)
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::MED, 0);
    assert!(result.is_err());
}

#[test]
fn test_prefix_subscription_elevated_caps() {
    let fabric = EventFabric::new();
    let task_id = 125;
    
    // Test prefix subscription with INTENT_SUBMIT (elevated)
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT);
    assert!(result.is_ok());
    
    // Test prefix subscription with WM_WRITE (elevated)
    let pattern = Pattern::new("wm.*".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::MED, CAP_WM_WRITE);
    assert!(result.is_ok());
    
    // Test prefix subscription without elevated caps (should fail)
    let pattern = Pattern::new("skill.*".to_string()).unwrap();
    let result = fabric.subscribe(task_id, pattern, Lane::LO, CAP_SKILL_QUERY);
    assert!(result.is_err());
}

#[test]
fn test_subscription_lifecycle() {
    let fabric = EventFabric::new();
    let task_id = 126;
    
    // Subscribe
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    let handle = fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    // Verify subscription exists
    let subscriptions = fabric.get_task_subscriptions(task_id);
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].handle, handle);
    
    // Unsubscribe
    let result = fabric.unsubscribe(handle);
    assert!(result.is_ok());
    
    // Verify subscription removed
    let subscriptions = fabric.get_task_subscriptions(task_id);
    assert_eq!(subscriptions.len(), 0);
}

#[test]
fn test_multiple_subscriptions() {
    let fabric = EventFabric::new();
    let task_id = 127;
    
    // Subscribe to multiple topics
    let pattern1 = Pattern::new("intent.created".to_string()).unwrap();
    let handle1 = fabric.subscribe(task_id, pattern1, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    let pattern2 = Pattern::new("wm.put".to_string()).unwrap();
    let handle2 = fabric.subscribe(task_id, pattern2, Lane::MED, CAP_WM_READ).unwrap();
    
    let pattern3 = Pattern::new("skill.loaded".to_string()).unwrap();
    let handle3 = fabric.subscribe(task_id, pattern3, Lane::LO, CAP_SKILL_QUERY).unwrap();
    
    // Verify all subscriptions exist
    let subscriptions = fabric.get_task_subscriptions(task_id);
    assert_eq!(subscriptions.len(), 3);
    
    // Verify handles are unique
    let handles: Vec<u32> = subscriptions.iter().map(|s| s.handle).collect();
    assert_eq!(handles.len(), 3);
    assert!(handles.contains(&handle1));
    assert!(handles.contains(&handle2));
    assert!(handles.contains(&handle3));
}

#[test]
fn test_invalid_patterns() {
    let fabric = EventFabric::new();
    let task_id = 128;
    
    // Test empty pattern
    let result = Pattern::new("".to_string());
    assert!(result.is_err());
    
    // Test pattern that's just ".*"
    let result = Pattern::new(".*".to_string());
    assert!(result.is_err());
    
    // Test pattern with invalid characters
    let result = Pattern::new("intent.created!".to_string());
    assert!(result.is_ok()); // This is actually valid for now
}

#[test]
fn test_task_inbox_creation() {
    let fabric = EventFabric::new();
    let task_id = 129;
    
    // Initially no inbox
    let stats = fabric.get_inbox_stats(task_id);
    assert!(stats.is_none());
    
    // Subscribe creates inbox
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    // Now inbox exists
    let stats = fabric.get_inbox_stats(task_id);
    assert!(stats.is_some());
    
    // Verify inbox has default configuration
    let stats = stats.unwrap();
    assert_eq!(stats.budget_bytes, 256 * 1024); // 256 KiB
    assert_eq!(stats.hi.max_size, 256);
    assert_eq!(stats.med.max_size, 512);
    assert_eq!(stats.lo.max_size, 1024);
}
