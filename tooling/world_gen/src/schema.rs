use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSchema {
    pub version: u16,
    pub name: String,
    pub description: String,
    pub field_tags: BTreeMap<String, u8>,
    pub entity_types: BTreeMap<String, u64>,
    pub predicate_types: BTreeMap<String, u64>,
    pub value_types: BTreeMap<String, u64>,
    pub limits: Limits,
    pub cbor: CborOptions,
    pub validation: Validation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub fact_max_size: u32,
    pub snapshot_max_count: u32,
    pub segment_max_size: u32,
    pub budget_mib: u32,
    pub query_max_limit: u16,
    pub query_max_offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CborOptions {
    pub version_field: bool,
    pub field_tags: bool,
    pub deterministic: bool,
    pub sorted_fields: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub entity_id_range: String,
    pub pred_id_range: String,
    pub ts_vclock_range: String,
    pub provenance_max_len: u32,
    pub value_atom_max_size: u32,
}

impl WorldSchema {
    pub fn get_field_tag(&self, field: &str) -> Option<u8> {
        self.field_tags.get(field).copied()
    }
    
    pub fn get_entity_type(&self, name: &str) -> Option<u64> {
        self.entity_types.get(name).copied()
    }
    
    pub fn get_predicate_type(&self, name: &str) -> Option<u64> {
        self.predicate_types.get(name).copied()
    }
    
    pub fn get_value_type(&self, name: &str) -> Option<u64> {
        self.value_types.get(name).copied()
    }
}
