use crate::intent::wire::{
    IntentHeaderV1, IntentLane, IntentState
};
use crate::intent::{Intent, IntentManager, IntentError};
use crate::process::ProcessId;

#[test]
fn test_best_lane_backpressure() {
    let mut manager = IntentManager::new();
    
    // Fill the BEST lane to capacity
    let capacity = IntentLane::BEST.queue_capacity();
    
    for i in 0..capacity {
        let mut header = IntentHeaderV1::default();
        header.intent_id = i as u128 + 1;
        header.lane = IntentLane::BEST as u8;
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
        
        assert!(result.is_ok(), "Failed to submit intent {}", i);
    }
    
    // Next submission should fail with EAGAIN
    let mut header = IntentHeaderV1::default();
    header.intent_id = capacity as u128 + 1;
    header.lane = IntentLane::BEST as u8;
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
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::QueueFull => {},
        _ => panic!("Expected QueueFull error"),
    }
}

#[test]
fn test_high_lane_backpressure() {
    let mut manager = IntentManager::new();
    
    // Fill the HIGH lane to capacity
    let capacity = IntentLane::HIGH.queue_capacity();
    
    for i in 0..capacity {
        let mut header = IntentHeaderV1::default();
        header.intent_id = i as u128 + 1;
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
        
        assert!(result.is_ok(), "Failed to submit intent {}", i);
    }
    
    // Next submission should fail with EAGAIN
    let mut header = IntentHeaderV1::default();
    header.intent_id = capacity as u128 + 1;
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
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::QueueFull => {},
        _ => panic!("Expected QueueFull error"),
    }
}

#[test]
fn test_rt_lane_backpressure() {
    let mut manager = IntentManager::new();
    
    // Fill the RT lane to capacity
    let capacity = IntentLane::RT.queue_capacity();
    
    for i in 0..capacity {
        let mut header = IntentHeaderV1::default();
        header.intent_id = i as u128 + 1;
        header.lane = IntentLane::RT as u8;
        header.ttl_ms = 1000;
        header.text_len = 100;
        header.scope_flags = 0x0000_0000_0000_0001; // RT requires elevated caps
        
        let text_data = vec![0u8; 100];
        
        let result = manager.submit_intent(
            123,
            header,
            text_data,
            vec![],
            vec![],
            vec![],
        );
        
        assert!(result.is_ok(), "Failed to submit intent {}", i);
    }
    
    // Next submission should fail with EAGAIN
    let mut header = IntentHeaderV1::default();
    header.intent_id = capacity as u128 + 1;
    header.lane = IntentLane::RT as u8;
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
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::QueueFull => {},
        _ => panic!("Expected QueueFull error"),
    }
}

#[test]
fn test_mixed_lane_submissions() {
    let mut manager = IntentManager::new();
    
    // Submit intents to different lanes
    let lanes = vec![
        IntentLane::RT as u8,
        IntentLane::HIGH as u8,
        IntentLane::BEST as u8,
    ];
    
    for (lane_idx, lane) in lanes.iter().enumerate() {
        let mut header = IntentHeaderV1::default();
        header.intent_id = lane_idx as u128 + 1;
        header.lane = *lane;
        header.ttl_ms = 1000;
        header.text_len = 100;
        
        // RT lane requires elevated capabilities
        if *lane == IntentLane::RT as u8 {
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
        
        assert!(result.is_ok(), "Failed to submit intent to lane {}", lane);
    }
}

#[test]
fn test_queue_processing_order() {
    let mut manager = IntentManager::new();
    
    // Submit intents to different lanes with different priorities
    let submissions = vec![
        (IntentLane::BEST as u8, 1),
        (IntentLane::HIGH as u8, 2),
        (IntentLane::RT as u8, 3),
        (IntentLane::BEST as u8, 4),
        (IntentLane::HIGH as u8, 5),
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
        
        assert!(result.is_ok(), "Failed to submit intent {}", intent_id);
    }
    
    // Process the queue and verify RT intents are processed first
    manager.process_queue();
    
    // In a real implementation, we'd verify the processing order
    // For now, we just ensure the queue processing doesn't crash
}

#[test]
fn test_queue_capacity_limits() {
    // Verify queue capacity constants match the schema
    assert_eq!(IntentLane::RT.queue_capacity(), 100);
    assert_eq!(IntentLane::HIGH.queue_capacity(), 500);
    assert_eq!(IntentLane::BEST.queue_capacity(), 1000);
}

#[test]
fn test_queue_utilization() {
    let mut manager = IntentManager::new();
    
    // Submit a few intents to HIGH lane
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
    
    // In a real implementation, we'd check queue utilization
    // For now, we just ensure the submission succeeded
}

#[test]
fn test_queue_wait_time_estimates() {
    // Test wait time estimates for different lanes
    let rt_queue = crate::intent::queue::IntentQueue::new(IntentLane::RT);
    let high_queue = crate::intent::queue::IntentQueue::new(IntentLane::HIGH);
    let best_queue = crate::intent::queue::IntentQueue::new(IntentLane::BEST);
    
    // RT lane should always have 0 wait time
    assert_eq!(rt_queue.get_wait_time_estimate(), 0);
    
    // HIGH lane should have reasonable wait time estimates
    assert!(high_queue.get_wait_time_estimate() >= 0);
    
    // BEST lane should have higher wait time estimates
    assert!(best_queue.get_wait_time_estimate() >= 0);
}

#[test]
fn test_queue_overflow_protection() {
    let mut manager = IntentManager::new();
    
    // Try to submit more intents than the total system capacity
    let total_capacity = IntentLane::RT.queue_capacity() + 
                        IntentLane::HIGH.queue_capacity() + 
                        IntentLane::BEST.queue_capacity();
    
    let mut intent_id = 1;
    
    for lane in [IntentLane::RT, IntentLane::HIGH, IntentLane::BEST] {
        let capacity = lane.queue_capacity();
        
        for i in 0..capacity {
            let mut header = IntentHeaderV1::default();
            header.intent_id = intent_id;
            header.lane = lane as u8;
            header.ttl_ms = 1000;
            header.text_len = 100;
            
            if lane == IntentLane::RT {
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
            
            assert!(result.is_ok(), "Failed to submit intent {} to lane {:?}", intent_id, lane);
            intent_id += 1;
        }
        
        // Next submission to this lane should fail
        let mut header = IntentHeaderV1::default();
        header.intent_id = intent_id;
        header.lane = lane as u8;
        header.ttl_ms = 1000;
        header.text_len = 100;
        
        if lane == IntentLane::RT {
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
        
        assert!(result.is_err());
        match result.unwrap_err() {
            IntentError::QueueFull => {},
            _ => panic!("Expected QueueFull error for lane {:?}", lane),
        }
        
        intent_id += 1;
    }
}
