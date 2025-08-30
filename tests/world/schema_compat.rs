use crate::world::schema::{FactV1, EntityId, PredId, ValueAtom, schema_hash, SCHEMA_VERSION};

#[test]
fn test_schema_version() {
    assert_eq!(SCHEMA_VERSION, 1);
}

#[test]
fn test_schema_hash_stability() {
    let hash1 = schema_hash();
    let hash2 = schema_hash();
    assert_eq!(hash1, hash2, "Schema hash should be stable across calls");
}

#[test]
fn test_fact_serialization_roundtrip() {
    let fact = FactV1 {
        subject: EntityId(12345),
        predicate: PredId(1),
        object: ValueAtom::String("test_value".to_string()),
        ts_vclock: 1000,
        provenance: "test_provenance".to_string(),
    };
    
    let serialized = serde_cbor::to_vec(&fact).unwrap();
    let deserialized: FactV1 = serde_cbor::from_slice(&serialized).unwrap();
    
    assert_eq!(fact.subject, deserialized.subject);
    assert_eq!(fact.predicate, deserialized.predicate);
    assert_eq!(fact.ts_vclock, deserialized.ts_vclock);
    assert_eq!(fact.provenance, deserialized.provenance);
    
    match (fact.object, deserialized.object) {
        (ValueAtom::String(s1), ValueAtom::String(s2)) => assert_eq!(s1, s2),
        _ => panic!("Object type mismatch"),
    }
}

#[test]
fn test_value_atom_types() {
    let u64_val = ValueAtom::U64(42);
    let i64_val = ValueAtom::I64(-42);
    let f64_val = ValueAtom::F64(3.14);
    let bool_val = ValueAtom::Bool(true);
    let bytes_val = ValueAtom::Bytes(vec![1, 2, 3, 4]);
    let string_val = ValueAtom::String("hello".to_string());
    
    let u64_ser = serde_cbor::to_vec(&u64_val).unwrap();
    let i64_ser = serde_cbor::to_vec(&i64_val).unwrap();
    let f64_ser = serde_cbor::to_vec(&f64_val).unwrap();
    let bool_ser = serde_cbor::to_vec(&bool_val).unwrap();
    let bytes_ser = serde_cbor::to_vec(&bytes_val).unwrap();
    let string_ser = serde_cbor::to_vec(&string_val).unwrap();
    
    let u64_deser: ValueAtom = serde_cbor::from_slice(&u64_ser).unwrap();
    let i64_deser: ValueAtom = serde_cbor::from_slice(&i64_ser).unwrap();
    let f64_deser: ValueAtom = serde_cbor::from_slice(&f64_ser).unwrap();
    let bool_deser: ValueAtom = serde_cbor::from_slice(&bool_ser).unwrap();
    let bytes_deser: ValueAtom = serde_cbor::from_slice(&bytes_ser).unwrap();
    let string_deser: ValueAtom = serde_cbor::from_slice(&string_ser).unwrap();
    
    assert_eq!(u64_val, u64_deser);
    assert_eq!(i64_val, i64_deser);
    assert_eq!(f64_val, f64_deser);
    assert_eq!(bool_val, bool_deser);
    assert_eq!(bytes_val, bytes_deser);
    assert_eq!(string_val, string_deser);
}

#[test]
fn test_entity_id_equality() {
    let id1 = EntityId(12345);
    let id2 = EntityId(12345);
    let id3 = EntityId(67890);
    
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_pred_id_equality() {
    let id1 = PredId(1);
    let id2 = PredId(1);
    let id3 = PredId(2);
    
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_constants() {
    assert_eq!(crate::world::schema::ENTITY_TYPES.len(), 8);
    assert_eq!(crate::world::schema::PREDICATE_TYPES.len(), 10);
    assert_eq!(crate::world::schema::VALUE_TYPES.len(), 6);
    assert_eq!(crate::world::schema::LIMITS.len(), 6);
}
