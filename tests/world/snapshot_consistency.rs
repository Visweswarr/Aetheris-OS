use crate::world::{WorldModelKernel, schema::{FactV1, EntityId, PredId, ValueAtom}};

#[test]
fn test_snapshot_consistency() {
    let world = WorldModelKernel::new();
    
    let facts1 = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("cap1".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
    ];
    
    let (_, snapshot1) = world.put_facts(&facts1).unwrap();
    
    let facts2 = vec![
        FactV1 {
            subject: EntityId(2),
            predicate: PredId(1),
            object: ValueAtom::String("cap2".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
    ];
    
    let (_, snapshot2) = world.put_facts(&facts2).unwrap();
    
    let view1 = world.snapshot_open(snapshot1).unwrap();
    let view2 = world.snapshot_open(snapshot2).unwrap();
    
    assert_ne!(snapshot1.0, snapshot2.0);
    
    let pattern = crate::world::query::Pattern::new().with_predicate(PredId(1));
    let range = crate::world::query::Range::new();
    
    let query1 = crate::world::query::query(&view1, &pattern, &range, 10, 0);
    let query2 = crate::world::query::query(&view2, &pattern, &range, 10, 0);
    
    assert_eq!(query1.facts.len(), 1);
    assert_eq!(query2.facts.len(), 2);
}

#[test]
fn test_snapshot_isolation() {
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
    
    let (_, snapshot_id) = world.put_facts(&facts).unwrap();
    
    let view = world.snapshot_open(snapshot_id).unwrap();
    
    let new_facts = vec![
        FactV1 {
            subject: EntityId(2),
            predicate: PredId(1),
            object: ValueAtom::String("cap2".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&new_facts).unwrap();
    
    let pattern = crate::world::query::Pattern::new().with_predicate(PredId(1));
    let range = crate::world::query::Range::new();
    let query = crate::world::query::query(&view, &pattern, &range, 10, 0);
    
    assert_eq!(query.facts.len(), 1);
}

#[test]
fn test_snapshot_export() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("cap1".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(2),
            predicate: PredId(1),
            object: ValueAtom::String("cap2".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
    ];
    
    let (_, snapshot_id) = world.put_facts(&facts).unwrap();
    
    let export_result = world.export_snapshot(snapshot_id, None);
    assert!(export_result.is_ok());
    
    let cbor_data = export_result.unwrap();
    assert!(!cbor_data.is_empty());
    
    let deserialized: Vec<FactV1> = serde_cbor::from_slice(&cbor_data).unwrap();
    assert_eq!(deserialized.len(), 2);
}

#[test]
fn test_snapshot_export_with_filter() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("cap1".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(2),
            predicate: PredId(1),
            object: ValueAtom::String("cap2".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
    ];
    
    let (_, snapshot_id) = world.put_facts(&facts).unwrap();
    
    let export_result = world.export_snapshot(snapshot_id, Some(EntityId(1)));
    assert!(export_result.is_ok());
    
    let cbor_data = export_result.unwrap();
    let deserialized: Vec<FactV1> = serde_cbor::from_slice(&cbor_data).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert_eq!(deserialized[0].subject, EntityId(1));
}

#[test]
fn test_snapshot_not_found() {
    let world = WorldModelKernel::new();
    
    let snapshot_id = crate::world::schema::SnapshotId(999);
    let view = world.snapshot_open(snapshot_id);
    
    assert!(view.is_none());
}

#[test]
fn test_snapshot_creation() {
    let world = WorldModelKernel::new();
    
    let snapshot_id = world.snapshot_create();
    assert_eq!(snapshot_id.0, 0);
    
    let view = world.snapshot_open(snapshot_id);
    assert!(view.is_some());
    
    let stats = world.get_stats();
    assert_eq!(stats.snapshots, 1);
}
