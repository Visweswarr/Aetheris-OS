use kernel::event::fabric::{EventFabric, Pattern};
use kernel::event::queue::Lane;
use kernel::secman::cap_flags::*;

#[test]
fn test_hi_lane_lossless() {
    let fabric = EventFabric::new();
    let task_id = 300;
    
    // Subscribe to intent events on HI lane
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    
    // Fill HI lane to capacity (256 events)
    for i in 0..256 {
        let payload = format!("event {}", i).into_bytes();
        let delivery_count = fabric.publish("intent.created", 0, payload).unwrap();
        assert_eq!(delivery_count, 1);
    }
    
    // Try to publish one more event
    let payload = b"overflow event".to_vec();
    let delivery_count = fabric.publish("intent.created", 0, payload).unwrap();
    assert_eq!(delivery_count, 1);
    
    // Poll all events - should get 257 events (no drops on HI lane)
    let events = fabric.poll(task_id, 300).unwrap();
    assert_eq!(events.len(), 257);
    
    // Verify no drops occurred
    let stats = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(stats.hi.drops, 0);
}

#[test]
fn test_med_lane_drop_oldest() {
    let fabric = EventFabric::new();
    let task_id = 301;
    
    // Subscribe to intent events on MED lane
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::MED, CAP_INTENT_SUBMIT).unwrap();
    
    // Fill MED lane to capacity (512 events)
    for i in 0..512 {
        let payload = format!("event {}", i).into_bytes();
        let delivery_count = fabric.publish("intent.created", 1, payload).unwrap();
        assert_eq!(delivery_count, 1);
    }
    
    // Try to publish one more event
    let payload = b"overflow event".to_vec();
    let delivery_count = fabric.publish("intent.created", 1, payload).unwrap();
    assert_eq!(delivery_count, 1);
    
    // Poll all events - should get 512 events (one was dropped)
    let events = fabric.poll(task_id, 600).unwrap();
    assert_eq!(events.len(), 512);
    
    // Verify drop occurred
    let stats = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(stats.med.drops, 1);
    
    // Verify the oldest event was dropped (event 0 should be missing)
    let event_0_found = events.iter().any(|e| e.payload == b"event 0");
    assert!(!event_0_found);
    
    // Verify the newest event is present
    let overflow_found = events.iter().any(|e| e.payload == b"overflow event");
    assert!(overflow_found);
}

#[test]
fn test_lo_lane_drop_oldest() {
    let fabric = EventFabric::new();
    let task_id = 302;
    
    // Subscribe to intent events on LO lane
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::LO, CAP_INTENT_SUBMIT).unwrap();
    
    // Fill LO lane to capacity (1024 events)
    for i in 0..1024 {
        let payload = format!("event {}", i).into_bytes();
        let delivery_count = fabric.publish("intent.created", 2, payload).unwrap();
        assert_eq!(delivery_count, 1);
    }
    
    // Try to publish one more event
    let payload = b"overflow event".to_vec();
    let delivery_count = fabric.publish("intent.created", 2, payload).unwrap();
    assert_eq!(delivery_count, 1);
    
    // Poll all events - should get 1024 events (one was dropped)
    let events = fabric.poll(task_id, 1100).unwrap();
    assert_eq!(events.len(), 1024);
    
    // Verify drop occurred
    let stats = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(stats.lo.drops, 1);
    
    // Verify the oldest event was dropped (event 0 should be missing)
    let event_0_found = events.iter().any(|e| e.payload == b"event 0");
    assert!(!event_0_found);
    
    // Verify the newest event is present
    let overflow_found = events.iter().any(|e| e.payload == b"overflow event");
    assert!(overflow_found);
}

#[test]
fn test_mixed_lane_backpressure() {
    let fabric = EventFabric::new();
    let task_id = 303;
    
    // Subscribe to intent events on all lanes
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern.clone(), Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    fabric.subscribe(task_id, pattern.clone(), Lane::MED, CAP_INTENT_SUBMIT).unwrap();
    fabric.subscribe(task_id, pattern, Lane::LO, CAP_INTENT_SUBMIT).unwrap();
    
    // Fill all lanes
    for i in 0..256 {
        let payload = format!("hi_event {}", i).into_bytes();
        fabric.publish("intent.created", 0, payload).unwrap();
    }
    
    for i in 0..512 {
        let payload = format!("med_event {}", i).into_bytes();
        fabric.publish("intent.previewed", 1, payload).unwrap();
    }
    
    for i in 0..1024 {
        let payload = format!("lo_event {}", i).into_bytes();
        fabric.publish("intent.completed", 2, payload).unwrap();
    }
    
    // Try to overflow each lane
    fabric.publish("intent.created", 0, b"hi_overflow".to_vec()).unwrap();
    fabric.publish("intent.previewed", 1, b"med_overflow".to_vec()).unwrap();
    fabric.publish("intent.completed", 2, b"lo_overflow".to_vec()).unwrap();
    
    // Poll all events
    let events = fabric.poll(task_id, 2000).unwrap();
    
    // Verify counts
    let hi_events = events.iter().filter(|e| e.header.prio == 0).count();
    let med_events = events.iter().filter(|e| e.header.prio == 1).count();
    let lo_events = events.iter().filter(|e| e.header.prio == 2).count();
    
    assert_eq!(hi_events, 257); // HI lane: no drops
    assert_eq!(med_events, 512); // MED lane: one drop
    assert_eq!(lo_events, 1024); // LO lane: one drop
    
    // Verify drops in stats
    let stats = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(stats.hi.drops, 0);
    assert_eq!(stats.med.drops, 1);
    assert_eq!(stats.lo.drops, 1);
}

#[test]
fn test_byte_budget_enforcement() {
    let fabric = EventFabric::new();
    let task_id = 304;
    
    // Subscribe to intent events on MED lane
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::MED, CAP_INTENT_SUBMIT).unwrap();
    
    // Create large payloads to test byte budget
    let large_payload = vec![0u8; 1000]; // 1 KiB per event
    
    // Fill up to byte budget (128 KiB)
    for i in 0..128 {
        let payload = format!("event_{}", i).into_bytes();
        fabric.publish("intent.created", 1, payload).unwrap();
    }
    
    // Try to add one more large event
    let overflow_payload = vec![0u8; 1000];
    fabric.publish("intent.created", 1, overflow_payload).unwrap();
    
    // Poll events
    let events = fabric.poll(task_id, 200).unwrap();
    
    // Should have 128 events (one was dropped due to byte budget)
    assert_eq!(events.len(), 128);
    
    // Verify drop occurred
    let stats = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(stats.med.drops, 1);
}

#[test]
fn test_concurrent_backpressure() {
    let fabric = EventFabric::new();
    
    // Create multiple tasks with different lane subscriptions
    let task_ids = [400, 401, 402, 403, 404];
    
    for (i, &task_id) in task_ids.iter().enumerate() {
        let lane = match i % 3 {
            0 => Lane::HI,
            1 => Lane::MED,
            _ => Lane::LO,
        };
        
        let pattern = Pattern::new("intent.*".to_string()).unwrap();
        fabric.subscribe(task_id, pattern, lane, CAP_INTENT_SUBMIT).unwrap();
    }
    
    // Publish many events to all tasks
    for i in 0..1000 {
        let payload = format!("event_{}", i).into_bytes();
        fabric.publish("intent.created", i as u8 % 3, payload).unwrap();
    }
    
    // Check stats for each task
    for &task_id in &task_ids {
        let stats = fabric.get_inbox_stats(task_id).unwrap();
        
        // Verify total events received
        let total_events = stats.hi.length + stats.med.length + stats.lo.length;
        assert!(total_events > 0);
        
        // Verify drops occurred on MED/LO lanes
        if stats.med.length > 0 || stats.lo.length > 0 {
            assert!(stats.med.drops > 0 || stats.lo.drops > 0);
        }
    }
    
    // Verify fabric-level stats
    let fabric_stats = fabric.get_stats();
    assert!(fabric_stats.total_events_published > 0);
    assert!(fabric_stats.total_events_delivered > 0);
    assert!(fabric_stats.total_drops_med > 0 || fabric_stats.total_drops_lo > 0);
}
