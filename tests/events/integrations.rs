use kernel::event::fabric::{EventFabric, Pattern};
use kernel::event::queue::Lane;
use kernel::event::EventKernel;
use kernel::secman::cap_flags::*;

#[test]
fn test_intent_event_integration() {
    let fabric = EventFabric::new();
    let task_id = 700;
    
    // Subscribe to intent events
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    // Publish intent events via kernel interface
    let intent_id = 12345;
    EventKernel::publish_intent_event("created", intent_id, "test intent").unwrap();
    EventKernel::publish_intent_event("previewed", intent_id, "preview complete").unwrap();
    EventKernel::publish_intent_event("completed", intent_id, "execution done").unwrap();
    
    // Poll events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 3);
    
    // Verify event types
    let event_types: Vec<&str> = events.iter()
        .map(|e| match e.header.topic_id {
            1 => "intent.created",
            2 => "intent.previewed",
            3 => "intent.completed",
            _ => "unknown",
        })
        .collect();
    
    assert!(event_types.contains(&"intent.created"));
    assert!(event_types.contains(&"intent.previewed"));
    assert!(event_types.contains(&"intent.completed"));
    
    // Verify payloads contain intent ID
    for event in &events {
        let payload_str = String::from_utf8_lossy(&event.payload);
        assert!(payload_str.contains(&intent_id.to_string()));
    }
}

#[test]
fn test_world_model_event_integration() {
    let fabric = EventFabric::new();
    let task_id = 701;
    
    // Subscribe to world model events
    let pattern = Pattern::new("wm.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::MED, CAP_WM_READ).unwrap();
    
    // Publish world model events via kernel interface
    let entity_id = 67890;
    EventKernel::publish_wm_event("put", entity_id, 5).unwrap();
    EventKernel::publish_wm_event("snapshot.new", entity_id, 1).unwrap();
    EventKernel::publish_wm_event("query", entity_id, 10).unwrap();
    
    // Poll events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 3);
    
    // Verify event types
    let event_types: Vec<&str> = events.iter()
        .map(|e| match e.header.topic_id {
            10 => "wm.put",
            11 => "wm.snapshot.new",
            12 => "wm.query",
            _ => "unknown",
        })
        .collect();
    
    assert!(event_types.contains(&"wm.put"));
    assert!(event_types.contains(&"wm.snapshot.new"));
    assert!(event_types.contains(&"wm.query"));
    
    // Verify payloads contain entity ID
    for event in &events {
        let payload_str = String::from_utf8_lossy(&event.payload);
        assert!(payload_str.contains(&entity_id.to_string()));
    }
    
    // Verify MED priority
    for event in &events {
        assert_eq!(event.header.prio, 1); // MED priority
    }
}

#[test]
fn test_skill_event_integration() {
    let fabric = EventFabric::new();
    let task_id = 702;
    
    // Subscribe to skill events
    let pattern = Pattern::new("skill.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::MED, CAP_SKILL_QUERY).unwrap();
    
    // Publish skill events via kernel interface
    let skill_id = 111;
    EventKernel::publish_skill_event("loaded", skill_id, "test skill loaded").unwrap();
    EventKernel::publish_skill_event("invoked", skill_id, "skill invoked").unwrap();
    EventKernel::publish_skill_event("preview", skill_id, "preview result").unwrap();
    EventKernel::publish_skill_event("unloaded", skill_id, "skill unloaded").unwrap();
    
    // Poll events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 4);
    
    // Verify event types
    let event_types: Vec<&str> = events.iter()
        .map(|e| match e.header.topic_id {
            20 => "skill.loaded",
            21 => "skill.invoked",
            22 => "skill.preview",
            23 => "skill.unloaded",
            _ => "unknown",
        })
        .collect();
    
    assert!(event_types.contains(&"skill.loaded"));
    assert!(event_types.contains(&"skill.invoked"));
    assert!(event_types.contains(&"skill.preview"));
    assert!(event_types.contains(&"skill.unloaded"));
    
    // Verify payloads contain skill ID
    for event in &events {
        let payload_str = String::from_utf8_lossy(&event.payload);
        assert!(payload_str.contains(&skill_id.to_string()));
    }
}

#[test]
fn test_system_event_integration() {
    let fabric = EventFabric::new();
    let task_id = 703;
    
    // Subscribe to system events
    let pattern = Pattern::new("sys.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::LO, 0).unwrap();
    
    // Publish system events via kernel interface
    EventKernel::publish_sys_event("timer", "APIC timer tick").unwrap();
    EventKernel::publish_sys_event("audit", "audit event logged").unwrap();
    
    // Poll events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 2);
    
    // Verify event types
    let event_types: Vec<&str> = events.iter()
        .map(|e| match e.header.topic_id {
            30 => "sys.timer",
            31 => "sys.audit",
            _ => "unknown",
        })
        .collect();
    
    assert!(event_types.contains(&"sys.timer"));
    assert!(event_types.contains(&"sys.audit"));
    
    // Verify LO priority
    for event in &events {
        assert_eq!(event.header.prio, 2); // LO priority
    }
}

#[test]
fn test_mixed_event_integration() {
    let fabric = EventFabric::new();
    let task_id = 704;
    
    // Subscribe to multiple event types
    let intent_pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, intent_pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    let wm_pattern = Pattern::new("wm.*".to_string()).unwrap();
    fabric.subscribe(task_id, wm_pattern, Lane::MED, CAP_WM_READ).unwrap();
    
    let skill_pattern = Pattern::new("skill.*".to_string()).unwrap();
    fabric.subscribe(task_id, skill_pattern, Lane::LO, CAP_SKILL_QUERY).unwrap();
    
    // Publish events of different types
    EventKernel::publish_intent_event("created", 999, "mixed test").unwrap();
    EventKernel::publish_wm_event("put", 888, 3).unwrap();
    EventKernel::publish_skill_event("loaded", 777, "mixed test").unwrap();
    
    // Poll events
    let events = fabric.poll(task_id, 10).unwrap();
    assert_eq!(events.len(), 3);
    
    // Verify priority ordering: HI first, then MED, then LO
    assert_eq!(events[0].header.prio, 0); // HI
    assert_eq!(events[1].header.prio, 1); // MED
    assert_eq!(events[2].header.prio, 2); // LO
    
    // Verify topic IDs
    assert_eq!(events[0].header.topic_id, 1); // intent.created
    assert_eq!(events[1].header.topic_id, 10); // wm.put
    assert_eq!(events[2].header.topic_id, 20); // skill.loaded
}

#[test]
fn test_event_fabric_availability() {
    // Test that event fabric is available
    assert!(kernel::event::integration::is_available());
    
    // Test task initialization
    let task_id = 705;
    let result = kernel::event::integration::init_task_events(task_id);
    assert!(result.is_ok());
    
    // Verify task has inbox
    let fabric = EventFabric::new();
    let stats = fabric.get_inbox_stats(task_id);
    assert!(stats.is_some());
    
    // Clean up
    kernel::event::integration::cleanup_task_events(task_id);
    
    // Verify task removed
    let stats = fabric.get_inbox_stats(task_id);
    assert!(stats.is_none());
}

#[test]
fn test_event_fabric_constants() {
    use kernel::event::constants::*;
    
    // Verify constants are reasonable
    assert_eq!(MAX_EVENT_PAYLOAD, 4096);
    assert_eq!(MAX_EVENTS_PER_POLL, 64);
    assert_eq!(DEFAULT_INBOX_BUDGET, 256 * 1024);
    assert_eq!(HI_LANE_CAPACITY, 256);
    assert_eq!(MED_LANE_CAPACITY, 512);
    assert_eq!(LO_LANE_CAPACITY, 1024);
    
    // Verify relationships
    assert!(HI_LANE_CAPACITY < MED_LANE_CAPACITY);
    assert!(MED_LANE_CAPACITY < LO_LANE_CAPACITY);
    assert!(MAX_EVENT_PAYLOAD <= DEFAULT_INBOX_BUDGET / 64); // Reasonable ratio
}

#[test]
fn test_event_fabric_statistics() {
    let fabric = EventFabric::new();
    let task_id = 706;
    
    // Subscribe to events
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
    
    // Publish some events
    for i in 0..100 {
        let payload = format!("stat_test_{}", i).into_bytes();
        fabric.publish("intent.created", 0, payload).unwrap();
    }
    
    // Get fabric statistics
    let fabric_stats = fabric.get_stats();
    assert_eq!(fabric_stats.total_events_published, 100);
    assert_eq!(fabric_stats.total_events_delivered, 100);
    assert_eq!(fabric_stats.total_subscriptions, 1);
    
    // Get task statistics
    let task_stats = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(task_stats.hi.length, 100);
    assert_eq!(task_stats.total_bytes, 100 * (core::mem::size_of::<kernel::event::queue::EventHeader>() + 15)); // Approximate
    
    // Poll events
    let events = fabric.poll(task_id, 100).unwrap();
    assert_eq!(events.len(), 100);
    
    // Verify statistics updated
    let task_stats_after = fabric.get_inbox_stats(task_id).unwrap();
    assert_eq!(task_stats_after.hi.length, 0); // All events polled
    assert_eq!(task_stats_after.total_bytes, 0); // No events in inbox
}
