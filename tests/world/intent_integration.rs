use crate::world::{WorldModelKernel, schema::{FactV1, EntityId, PredId, ValueAtom}};

#[test]
fn test_get_entity_caps() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("CAP_FS_READ".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("CAP_NET_OUT".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(2),
            predicate: PredId(1),
            object: ValueAtom::String("CAP_DEVICE_USE".to_string()),
            ts_vclock: 1002,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let caps = world.get_entity_caps(&EntityId(1));
    assert_eq!(caps.len(), 2);
    
    let cap_names: Vec<String> = caps.iter()
        .map(|cap| {
            if let ValueAtom::String(s) = &cap.object {
                s.clone()
            } else {
                String::new()
            }
        })
        .collect();
    
    assert!(cap_names.contains(&"CAP_FS_READ".to_string()));
    assert!(cap_names.contains(&"CAP_NET_OUT".to_string()));
}

#[test]
fn test_list_devices() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(100),
            predicate: PredId(2),
            object: ValueAtom::String("device1".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(101),
            predicate: PredId(2),
            object: ValueAtom::String("device2".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(102),
            predicate: PredId(1),
            object: ValueAtom::String("cap1".to_string()),
            ts_vclock: 1002,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let devices = world.list_devices();
    assert_eq!(devices.len(), 2);
    assert!(devices.contains(&EntityId(100)));
    assert!(devices.contains(&EntityId(101)));
    assert!(!devices.contains(&EntityId(102)));
}

#[test]
fn test_last_seen() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(7),
            object: ValueAtom::U64(1000),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(7),
            object: ValueAtom::U64(2000),
            ts_vclock: 2000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(7),
            object: ValueAtom::U64(1500),
            ts_vclock: 1500,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let last_seen = world.last_seen(&EntityId(1));
    assert_eq!(last_seen, Some(2000));
}

#[test]
fn test_entity_not_found() {
    let world = WorldModelKernel::new();
    
    let caps = world.get_entity_caps(&EntityId(999));
    assert_eq!(caps.len(), 0);
    
    let devices = world.list_devices();
    assert_eq!(devices.len(), 0);
    
    let last_seen = world.last_seen(&EntityId(999));
    assert_eq!(last_seen, None);
}

#[test]
fn test_world_model_stats() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("cap1".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let stats = world.get_stats();
    assert_eq!(stats.put_ok, 1);
    assert_eq!(stats.put_enospc, 0);
    assert_eq!(stats.query_ok, 0);
    assert_eq!(stats.snapshots, 1);
    assert_eq!(stats.export_ok, 0);
    
    let json_stats = stats.to_json();
    assert!(json_stats.contains("wm_stats"));
    assert!(json_stats.contains("\"put_ok\":1"));
}

#[test]
fn test_world_model_helper_functions() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("CAP_INTENT_SUBMIT".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(2),
            object: ValueAtom::String("location1".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let caps = world.get_entity_caps(&EntityId(1));
    assert_eq!(caps.len(), 1);
    
    let devices = world.list_devices();
    assert_eq!(devices.len(), 1);
    
    let last_seen = world.last_seen(&EntityId(1));
    assert_eq!(last_seen, Some(1001));
}
