use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use super::schema::{FactV1, EntityId, PredId, ValueAtom};
use super::store::{ReadView, IndexEntry};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub subject: Option<EntityId>,
    pub predicate: Option<PredId>,
    pub object: Option<ValueAtom>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Range {
    pub ts_min: Option<u64>,
    pub ts_max: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rows {
    pub facts: Vec<FactV1>,
    pub next_offset: Option<u32>,
}

impl Pattern {
    pub fn new() -> Self {
        Self {
            subject: None,
            predicate: None,
            object: None,
        }
    }
    
    pub fn with_subject(mut self, subject: EntityId) -> Self {
        self.subject = Some(subject);
        self
    }
    
    pub fn with_predicate(mut self, predicate: PredId) -> Self {
        self.predicate = Some(predicate);
        self
    }
    
    pub fn with_object(mut self, object: ValueAtom) -> Self {
        self.object = Some(object);
        self
    }
    
    pub fn is_exact(&self) -> bool {
        self.subject.is_some() && self.predicate.is_some() && self.object.is_some()
    }
    
    pub fn is_wildcard(&self) -> bool {
        self.subject.is_none() && self.predicate.is_none() && self.object.is_none()
    }
}

impl Range {
    pub fn new() -> Self {
        Self {
            ts_min: None,
            ts_max: None,
        }
    }
    
    pub fn with_min(mut self, ts_min: u64) -> Self {
        self.ts_min = Some(ts_min);
        self
    }
    
    pub fn with_max(mut self, ts_max: u64) -> Self {
        self.ts_max = Some(ts_max);
        self
    }
    
    pub fn contains(&self, ts: u64) -> bool {
        if let Some(min) = self.ts_min {
            if ts < min {
                return false;
            }
        }
        if let Some(max) = self.ts_max {
            if ts > max {
                return false;
            }
        }
        true
    }
}

pub fn query(
    view: &ReadView,
    pattern: &Pattern,
    range: &Range,
    limit: u16,
    offset: u32,
) -> Rows {
    let mut results = Vec::new();
    let mut current_offset = 0;
    let mut found_count = 0;
    
    let limit = limit.min(1024) as usize;
    
    for segment in &view.segments {
        for fact in &segment.facts {
            if !range.contains(fact.ts_vclock) {
                continue;
            }
            
            if !pattern_matches(fact, pattern) {
                continue;
            }
            
            if current_offset < offset {
                current_offset += 1;
                continue;
            }
            
            if found_count >= limit {
                let next_offset = Some(offset + found_count as u32);
                return Rows { facts: results, next_offset };
            }
            
            results.push(fact.clone());
            found_count += 1;
        }
    }
    
    Rows {
        facts: results,
        next_offset: None,
    }
}

fn pattern_matches(fact: &FactV1, pattern: &Pattern) -> bool {
    if let Some(subject) = &pattern.subject {
        if fact.subject != *subject {
            return false;
        }
    }
    
    if let Some(predicate) = &pattern.predicate {
        if fact.predicate != *predicate {
            return false;
        }
    }
    
    if let Some(object) = &pattern.object {
        if !value_matches(&fact.object, object) {
            return false;
        }
    }
    
    true
}

fn value_matches(fact_value: &ValueAtom, pattern_value: &ValueAtom) -> bool {
    match (fact_value, pattern_value) {
        (ValueAtom::U64(a), ValueAtom::U64(b)) => a == b,
        (ValueAtom::I64(a), ValueAtom::I64(b)) => a == b,
        (ValueAtom::F64(a), ValueAtom::F64(b)) => (a - b).abs() < f64::EPSILON,
        (ValueAtom::Bool(a), ValueAtom::Bool(b)) => a == b,
        (ValueAtom::Bytes(a), ValueAtom::Bytes(b)) => a == b,
        (ValueAtom::String(a), ValueAtom::String(b)) => a == b,
        _ => false,
    }
}

pub fn get_entity_caps(view: &ReadView, entity_id: &EntityId) -> Vec<FactV1> {
    let mut caps = Vec::new();
    
    for segment in &view.segments {
        for fact in &segment.facts {
            if fact.subject == *entity_id && fact.predicate.0 == 1 {
                caps.push(fact.clone());
            }
        }
    }
    
    caps
}

pub fn list_devices(view: &ReadView) -> Vec<EntityId> {
    let mut devices = Vec::new();
    
    for segment in &view.segments {
        for fact in &segment.facts {
            if fact.predicate.0 == 2 {
                devices.push(fact.subject.clone());
            }
        }
    }
    
    devices
}

pub fn last_seen(view: &ReadView, entity_id: &EntityId) -> Option<u64> {
    let mut last_ts = None;
    
    for segment in &view.segments {
        for fact in &segment.facts {
            if fact.subject == *entity_id && fact.predicate.0 == 7 {
                if let Some(current_ts) = last_ts {
                    if fact.ts_vclock > current_ts {
                        last_ts = Some(fact.ts_vclock);
                    }
                } else {
                    last_ts = Some(fact.ts_vclock);
                }
            }
        }
    }
    
    last_ts
}
