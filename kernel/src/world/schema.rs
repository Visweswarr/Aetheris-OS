use serde::{Serialize, Deserialize};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

pub const SCHEMA_VERSION: u16 = 1;
pub const SCHEMA_HASH: &[u8; 32] = &[/* TODO: computed at build time */];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct EntityId(pub u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct PredId(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub enum ValueAtom {
    #[serde(rename = "u64")]
    U64(u64),
    #[serde(rename = "i64")]
    I64(i64),
    #[serde(rename = "f64")]
    F64(f64),
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "bytes")]
    Bytes(Vec<u8>),
    #[serde(rename = "string")]
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct FactV1 {
    #[serde(rename = "s")]
    pub subject: EntityId,
    #[serde(rename = "p")]
    pub predicate: PredId,
    #[serde(rename = "o")]
    pub object: ValueAtom,
    #[serde(rename = "ts")]
    pub ts_vclock: u64,
    #[serde(rename = "prov")]
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct SnapshotId(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct SegmentId(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct IndexEntry {
    pub segment_id: SegmentId,
    pub offset: u32,
}

pub const ENTITY_TYPES: &[(u64, &str)] = &[
    (1, "USER"),
    (2, "DEVICE"),
    (3, "SERVICE"),
    (4, "RESOURCE"),
    (5, "POLICY"),
    (6, "INTENT"),
    (7, "ACTION"),
    (8, "RELATION"),
];

pub const PREDICATE_TYPES: &[(u64, &str)] = &[
    (1, "HAS_CAPABILITY"),
    (2, "LOCATED_AT"),
    (3, "OWNS"),
    (4, "ACCESSES"),
    (5, "DEPENDS_ON"),
    (6, "TRUSTS"),
    (7, "LAST_SEEN"),
    (8, "STATUS"),
    (9, "METADATA"),
    (10, "INTENT_STATE"),
];

pub const VALUE_TYPES: &[(u64, &str)] = &[
    (1, "ATOM_U64"),
    (2, "ATOM_I64"),
    (3, "ATOM_F64"),
    (4, "ATOM_BOOL"),
    (5, "ATOM_BYTES"),
    (6, "ATOM_STRING"),
];

pub const LIMITS: &[(&str, u64)] = &[
    ("fact_max_size", 1024),
    ("snapshot_max_count", 100),
    ("segment_max_size", 65536),
    ("budget_mib", 8),
    ("query_max_limit", 1024),
    ("query_max_offset", 1048576),
];

pub fn schema_hash() -> [u8; 32] {
    *SCHEMA_HASH
}
