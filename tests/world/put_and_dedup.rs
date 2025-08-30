use crate::world::{WorldModelKernel, schema::{FactV1, EntityId, PredId, ValueAtom}};

#[test]
fn test_put_facts() {
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
            subject: EntityId(1),
            predicate: PredId(2),
            object: ValueAtom::String("location1".to_string()),
            ts_vclock: 1001,
            provenance: "test".to_string(),
        },
    ];
    
    let result = world.put_facts(&facts);
    assert!(result.is_ok());
    
    let (count, snapshot_id) = result.unwrap();
    assert_eq!(count, 2);
    assert_eq!(snapshot_id.0, 0);
}

#[test]
fn test_put_duplicates() {
    let world = WorldModelKernel::new();
    
    let fact = FactV1 {
        subject: EntityId(1),
        predicate: PredId(1),
        object: ValueAtom::String("cap1".to_string()),
        ts_vclock: 1000,
        provenance: "test".to_string(),
    };
    
    let facts = vec![fact.clone(), fact.clone()];
    
    let result = world.put_facts(&facts);
    assert!(result.is_ok());
    
    let (count, snapshot_id) = result.unwrap();
    assert_eq!(count, 2);
    
    let stats = world.get_stats();
    assert_eq!(stats.put_ok, 2);
}

#[test]
fn test_budget_enforcement() {
    let world = WorldModelKernel::new();
    
    let mut facts = Vec::new();
    for i in 0..10000 {
        facts.push(FactV1 {
            subject: EntityId(i),
            predicate: PredId(1),
            object: ValueAtom::String(format!("value_{}", i)),
            ts_vclock: i as u64,
            provenance: "test".to_string(),
        });
    }
    
    let result = world.put_facts(&facts);
    assert!(result.is_err());
    
    let stats = world.get_stats();
    assert_eq!(stats.put_enospc, 1);
}

#[test]
fn test_snapshot_creation() {
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
    
    let snapshot = world.snapshot_open(snapshot_id);
    assert!(snapshot.is_some());
    
    let stats = world.get_stats();
    assert_eq!(stats.snapshots, 1);
}

#[test]
fn test_fact_size_estimation() {
    let world = WorldModelKernel::new();
    
    let small_fact = FactV1 {
        subject: EntityId(1),
        predicate: PredId(1),
        object: ValueAtom::U64(42),
        ts_vclock: 1000,
        provenance: "test".to_string(),
    };
    
    let large_fact = FactV1 {
        subject: EntityId(2),
        predicate: PredId(2),
        object: ValueAtom::String("a".repeat(1000)),
        ts_vclock: 1001,
        provenance: "test".to_string(),
    };
    
    let facts = vec![small_fact, large_fact];
    let result = world.put_facts(&facts);
    
    if result.is_ok() {
        let (count, _) = result.unwrap();
        assert_eq!(count, 2);
    } else {
        let stats = world.get_stats();
        assert_eq!(stats.put_enospc, 1);
    }
}
