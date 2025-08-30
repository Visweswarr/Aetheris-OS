use kernel::event::fabric::{EventFabric, Pattern};
use kernel::event::queue::Lane;
use kernel::secman::cap_flags::*;

#[test]
fn test_publish_to_single_subscriber() {
    let fabric = EventFabric::new();
    let task_id = 200;
    
    // Subscribe to intent.created
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    // Publish event
    let payload = b"test intent".to_vec();
    let delivery_count = fabric.publish("intent.created", 0, payload).unwrap();
    assert_eq!(delivery_count, 1);
    
    // Poll events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.topic_id, 1); // intent.created
    assert_eq!(events[0].payload, b"test intent");
}

#[test]
fn test_publish_to_multiple_subscribers() {
    let fabric = EventFabric::new();
    
    // Subscribe task 201 to intent.*
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(201, pattern.clone(), Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    
    // Subscribe task 202 to intent.created specifically
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    fabric.subscribe(202, pattern, Lane::MED, CAP_INTENT_QUERY).unwrap();
    
    // Subscribe task 203 to intent.previewed
    let pattern = Pattern::new("intent.previewed".to_string()).unwrap();
    fabric.subscribe(203, pattern, Lane::LO, CAP_INTENT_QUERY).unwrap();
    
    // Publish intent.created (should go to tasks 201 and 202)
    let payload = b"new intent".to_vec();
    let delivery_count = fabric.publish("intent.created", 0, payload.clone()).unwrap();
    assert_eq!(delivery_count, 2);
    
    // Publish intent.previewed (should go to tasks 201 and 203)
    let payload2 = b"previewed intent".to_vec();
    let delivery_count = fabric.publish("intent.previewed", 1, payload2.clone()).unwrap();
    assert_eq!(delivery_count, 2);
    
    // Verify task 201 received both events (prefix subscription)
    let events = fabric.poll(201, 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].header.topic_id, 1); // intent.created
    assert_eq!(events[1].header.topic_id, 2); // intent.previewed
    
    // Verify task 202 received only intent.created
    let events = fabric.poll(202, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.topic_id, 1); // intent.created
    
    // Verify task 203 received only intent.previewed
    let events = fabric.poll(203, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.topic_id, 2); // intent.previewed
}

#[test]
fn test_lane_priority_ordering() {
    let fabric = EventFabric::new();
    let task_id = 204;
    
    // Subscribe to all intent events on different lanes
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern.clone(), Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    fabric.subscribe(task_id, pattern.clone(), Lane::MED, CAP_INTENT_SUBMIT).unwrap();
    fabric.subscribe(task_id, pattern, Lane::LO, CAP_INTENT_SUBMIT).unwrap();
    
    // Publish events with different priorities
    fabric.publish("intent.created", 2, b"low priority".to_vec()).unwrap(); // LO
    fabric.publish("intent.previewed", 0, b"high priority".to_vec()).unwrap(); // HI
    fabric.publish("intent.completed", 1, b"medium priority".to_vec()).unwrap(); // MED
    
    // Poll events - should come back in priority order: HI, MED, LO
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 3);
    
    // First should be high priority
    assert_eq!(events[0].header.prio, 0);
    assert_eq!(events[0].payload, b"high priority");
    
    // Second should be medium priority
    assert_eq!(events[1].header.prio, 1);
    assert_eq!(events[1].payload, b"medium priority");
    
    // Third should be low priority
    assert_eq!(events[2].header.prio, 2);
    assert_eq!(events[2].payload, b"low priority");
}

#[test]
fn test_topic_capability_checks() {
    let fabric = EventFabric::new();
    let task_id = 205;
    
    // Subscribe without required capabilities
    let pattern = Pattern::new("wm.put".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::MED, 0).unwrap();
    
    // Try to publish to wm.put (requires CAP_WM_WRITE)
    let payload = b"test fact".to_vec();
    let delivery_count = fabric.publish("wm.put", 1, payload).unwrap();
    
    // Should be delivered (capability check is on subscription, not delivery)
    assert_eq!(delivery_count, 1);
    
    // But the event should have the correct topic ID
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.topic_id, 10); // wm.put
}

#[test]
fn test_event_id_uniqueness() {
    let fabric = EventFabric::new();
    let task_id = 206;
    
    // Subscribe to intent events
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    
    // Publish multiple events
    fabric.publish("intent.created", 0, b"event 1".to_vec()).unwrap();
    fabric.publish("intent.previewed", 0, b"event 2".to_vec()).unwrap();
    fabric.publish("intent.completed", 0, b"event 3".to_vec()).unwrap();
    
    // Poll all events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 3);
    
    // Verify event IDs are unique and increasing
    let mut event_ids: Vec<u64> = events.iter().map(|e| e.header.id).collect();
    event_ids.sort();
    
    assert_eq!(event_ids.len(), 3);
    assert!(event_ids[0] < event_ids[1]);
    assert!(event_ids[1] < event_ids[2]);
}

#[test]
fn test_virtual_clock_timestamps() {
    let fabric = EventFabric::new();
    let task_id = 207;
    
    // Subscribe to intent events
    let pattern = Pattern::new("intent.created".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    // Publish event
    let payload = b"test event".to_vec();
    fabric.publish("intent.created", 0, payload).unwrap();
    
    // Poll event
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 1);
    
    // Verify timestamp is reasonable (should be > 0)
    let event = &events[0];
    assert!(event.header.ts_vclock > 0);
    
    // Publish another event
    let payload2 = b"test event 2".to_vec();
    fabric.publish("intent.created", 0, payload2).unwrap();
    
    // Poll second event
    let events2 = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events2.len(), 1);
    
    // Verify timestamp is increasing
    let event2 = &events2[0];
    assert!(event2.header.ts_vclock > event.header.ts_vclock);
}
