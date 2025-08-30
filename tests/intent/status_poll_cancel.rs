use crate::intent::wire::{
    IntentHeaderV1, IntentLane, IntentState, IntentPollFilter
};
use crate::intent::{Intent, IntentManager, IntentError};
use crate::process::ProcessId;

#[test]
fn test_intent_status_lifecycle() {
    let mut manager = IntentManager::new();
    
    // Submit an intent
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Check initial status
    let status = manager.get_status(123, 1).unwrap();
    assert_eq!(status.state, IntentState::Submitted as u8);
    assert_eq!(status.progress, 0);
    assert_eq!(status.error, 0);
    
    // Process queue to move to Running state
    manager.process_queue();
    
    // Check updated status
    let status = manager.get_status(123, 1).unwrap();
    assert_eq!(status.state, IntentState::Running as u8);
    assert_eq!(status.progress, 0);
}

#[test]
fn test_intent_poll_events() {
    let mut manager = IntentManager::new();
    
    // Submit multiple intents
    for i in 1..=5 {
        let mut header = IntentHeaderV1::default();
        header.intent_id = i as u128;
        header.lane = IntentLane::HIGH as u8;
        header.ttl_ms = 1000;
        header.text_len = 100;
        header.scope_flags = 0x0000_0000_0000_0001;
        
        let text_data = vec![0u8; 100];
        
        let result = manager.submit_intent(
            123,
            header,
            text_data,
            vec![],
            vec![],
            vec![],
        );
        
        assert!(result.is_ok());
    }
    
    // Create poll filter
    let filter = IntentPollFilter {
        lane_mask: 0x02, // HIGH lane only
        since_ts: 0,
        max_items: 10,
        state_mask: 0xFF, // All states
    };
    
    // Poll for events
    let events = manager.poll_events(123, &filter, 10);
    
    // Should get 5 events for submitted intents
    assert_eq!(events.len(), 5);
    
    // Verify event properties
    for event in &events {
        assert!(event.intent_id >= 1 && event.intent_id <= 5);
        assert_eq!(event.event_type, IntentState::Submitted as u8);
        assert_eq!(event.data_len, 0);
    }
}

#[test]
fn test_intent_poll_filtering() {
    let mut manager = IntentManager::new();
    
    // Submit intents to different lanes
    let submissions = vec![
        (IntentLane::RT as u8, 1),
        (IntentLane::HIGH as u8, 2),
        (IntentLane::BEST as u8, 3),
    ];
    
    for (lane, intent_id) in submissions {
        let mut header = IntentHeaderV1::default();
        header.intent_id = intent_id as u128;
        header.lane = lane;
        header.ttl_ms = 1000;
        header.text_len = 100;
        
        if lane == IntentLane::RT as u8 {
            header.scope_flags = 0x0000_0000_0000_0001;
        }
        
        let text_data = vec![0u8; 100];
        
        let result = manager.submit_intent(
            123,
            header,
            text_data,
            vec![],
            vec![],
            vec![],
        );
        
        assert!(result.is_ok());
    }
    
    // Filter by RT lane only
    let filter = IntentPollFilter {
        lane_mask: 0x01, // RT lane only
        since_ts: 0,
        max_items: 10,
        state_mask: 0xFF,
    };
    
    let events = manager.poll_events(123, &filter, 10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].intent_id, 1);
    
    // Filter by HIGH lane only
    let filter = IntentPollFilter {
        lane_mask: 0x02, // HIGH lane only
        since_ts: 0,
        max_items: 10,
        state_mask: 0xFF,
    };
    
    let events = manager.poll_events(123, &filter, 10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].intent_id, 2);
    
    // Filter by BEST lane only
    let filter = IntentPollFilter {
        lane_mask: 0x04, // BEST lane only
        since_ts: 0,
        max_items: 10,
        state_mask: 0xFF,
    };
    
    let events = manager.poll_events(123, &filter, 10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].intent_id, 3);
}

#[test]
fn test_intent_cancel_before_running() {
    let mut manager = IntentManager::new();
    
    // Submit an intent
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Cancel before it starts running
    let result = manager.cancel_intent(123, 1);
    assert!(result.is_ok());
    
    // Check status
    let status = manager.get_status(123, 1).unwrap();
    assert_eq!(status.state, IntentState::Cancelled as u8);
}

#[test]
fn test_intent_cancel_after_running() {
    let mut manager = IntentManager::new();
    
    // Submit an intent
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Process queue to move to Running state
    manager.process_queue();
    
    // Try to cancel after it's running
    let result = manager.cancel_intent(123, 1);
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::AlreadyRunning => {},
        _ => panic!("Expected AlreadyRunning error"),
    }
    
    // Status should still be Running
    let status = manager.get_status(123, 1).unwrap();
    assert_eq!(status.state, IntentState::Running as u8);
}

#[test]
fn test_intent_cancel_not_found() {
    let mut manager = IntentManager::new();
    
    // Try to cancel non-existent intent
    let result = manager.cancel_intent(123, 999);
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::NotFound => {},
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_intent_cancel_wrong_owner() {
    let mut manager = IntentManager::new();
    
    // Submit intent with PID 123
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Try to cancel with different PID
    let result = manager.cancel_intent(456, 1);
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::PermissionDenied => {},
        _ => panic!("Expected PermissionDenied error"),
    }
}

#[test]
fn test_intent_status_wrong_owner() {
    let mut manager = IntentManager::new();
    
    // Submit intent with PID 123
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Try to get status with different PID
    let result = manager.get_status(456, 1);
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::PermissionDenied => {},
        _ => panic!("Expected PermissionDenied error"),
    }
}

#[test]
fn test_intent_poll_max_items() {
    let mut manager = IntentManager::new();
    
    // Submit 10 intents
    for i in 1..=10 {
        let mut header = IntentHeaderV1::default();
        header.intent_id = i as u128;
        header.lane = IntentLane::HIGH as u8;
        header.ttl_ms = 1000;
        header.text_len = 100;
        header.scope_flags = 0x0000_0000_0000_0001;
        
        let text_data = vec![0u8; 100];
        
        let result = manager.submit_intent(
            123,
            header,
            text_data,
            vec![],
            vec![],
            vec![],
        );
        
        assert!(result.is_ok());
    }
    
    // Poll with max_items = 5
    let filter = IntentPollFilter {
        lane_mask: 0x02,
        since_ts: 0,
        max_items: 5,
        state_mask: 0xFF,
    };
    
    let events = manager.poll_events(123, &filter, 5);
    assert_eq!(events.len(), 5);
}

#[test]
fn test_intent_poll_since_timestamp() {
    let mut manager = IntentManager::new();
    
    // Submit an intent
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Get current timestamp
    let current_ts = crate::time::MonotonicTime::now().ticks();
    
    // Poll with since_timestamp = current_ts (should get no events)
    let filter = IntentPollFilter {
        lane_mask: 0x02,
        since_ts: current_ts,
        max_items: 10,
        state_mask: 0xFF,
    };
    
    let events = manager.poll_events(123, &filter, 10);
    assert_eq!(events.len(), 0);
}

#[test]
fn test_intent_state_transitions() {
    let mut manager = IntentManager::new();
    
    // Submit an intent
    let mut header = IntentHeaderV1::default();
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_ok());
    
    // Check Submitted state
    let status = manager.get_status(123, 1).unwrap();
    assert_eq!(status.state, IntentState::Submitted as u8);
    
    // Process queue to move to Running
    manager.process_queue();
    
    let status = manager.get_status(123, 1).unwrap();
    assert_eq!(status.state, IntentState::Running as u8);
    
    // In a real implementation, we'd simulate completion
    // For now, we just verify the state transitions work
}
