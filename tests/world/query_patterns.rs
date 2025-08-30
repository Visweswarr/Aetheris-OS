use crate::world::{WorldModelKernel, schema::{FactV1, EntityId, PredId, ValueAtom}};
use crate::world::query::{Pattern, Range};

#[test]
fn test_query_exact_match() {
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
    
    world.put_facts(&facts).unwrap();
    
    let pattern = Pattern::new()
        .with_subject(EntityId(1))
        .with_predicate(PredId(1));
    
    let range = Range::new();
    let result = world.query(&pattern, &range, 10, 0);
    
    assert!(result.is_ok());
    let rows = result.unwrap();
    assert_eq!(rows.facts.len(), 1);
    assert_eq!(rows.facts[0].subject, EntityId(1));
}

#[test]
fn test_query_wildcard() {
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
    
    world.put_facts(&facts).unwrap();
    
    let pattern = Pattern::new().with_predicate(PredId(1));
    let range = Range::new();
    let result = world.query(&pattern, &range, 10, 0);
    
    assert!(result.is_ok());
    let rows = result.unwrap();
    assert_eq!(rows.facts.len(), 2);
}

#[test]
fn test_query_with_range() {
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
            ts_vclock: 2000,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let pattern = Pattern::new().with_predicate(PredId(1));
    let range = Range::new().with_min(1500);
    let result = world.query(&pattern, &range, 10, 0);
    
    assert!(result.is_ok());
    let rows = result.unwrap();
    assert_eq!(rows.facts.len(), 1);
    assert_eq!(rows.facts[0].ts_vclock, 2000);
}

#[test]
fn test_query_limit_and_offset() {
    let world = WorldModelKernel::new();
    
    let mut facts = Vec::new();
    for i in 0..10 {
        facts.push(FactV1 {
            subject: EntityId(i),
            predicate: PredId(1),
            object: ValueAtom::String(format!("cap{}", i)),
            ts_vclock: 1000 + i as u64,
            provenance: "test".to_string(),
        });
    }
    
    world.put_facts(&facts).unwrap();
    
    let pattern = Pattern::new().with_predicate(PredId(1));
    let range = Range::new();
    
    let result1 = world.query(&pattern, &range, 5, 0);
    assert!(result1.is_ok());
    let rows1 = result1.unwrap();
    assert_eq!(rows1.facts.len(), 5);
    assert_eq!(rows1.next_offset, Some(5));
    
    let result2 = world.query(&pattern, &range, 5, 5);
    assert!(result2.is_ok());
    let rows2 = result2.unwrap();
    assert_eq!(rows2.facts.len(), 5);
    assert_eq!(rows2.next_offset, None);
}

#[test]
fn test_query_ordering() {
    let world = WorldModelKernel::new();
    
    let facts = vec![
        FactV1 {
            subject: EntityId(2),
            predicate: PredId(1),
            object: ValueAtom::String("cap2".to_string()),
            ts_vclock: 2000,
            provenance: "test".to_string(),
        },
        FactV1 {
            subject: EntityId(1),
            predicate: PredId(1),
            object: ValueAtom::String("cap1".to_string()),
            ts_vclock: 1000,
            provenance: "test".to_string(),
        },
    ];
    
    world.put_facts(&facts).unwrap();
    
    let pattern = Pattern::new().with_predicate(PredId(1));
    let range = Range::new();
    let result = world.query(&pattern, &range, 10, 0);
    
    assert!(result.is_ok());
    let rows = result.unwrap();
    
    assert_eq!(rows.facts[0].subject, EntityId(1));
    assert_eq!(rows.facts[1].subject, EntityId(2));
}

#[test]
fn test_query_empty_result() {
    let world = WorldModelKernel::new();
    
    let pattern = Pattern::new().with_subject(EntityId(999));
    let range = Range::new();
    let result = world.query(&pattern, &range, 10, 0);
    
    assert!(result.is_ok());
    let rows = result.unwrap();
    assert_eq!(rows.facts.len(), 0);
    assert_eq!(rows.next_offset, None);
}
