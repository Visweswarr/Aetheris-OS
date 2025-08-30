use crate::{kprintln, klog};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;
use serde::{Serialize, Deserialize};

use super::schema::{FactV1, SnapshotId, SegmentId, IndexEntry, EntityId, PredId};

pub const MAX_SEGMENTS: usize = 1000;
pub const MAX_SNAPSHOTS: usize = 100;
pub const BUDGET_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Segment {
    pub id: SegmentId,
    pub facts: Vec<FactV1>,
    pub bytes: u32,
}

impl Segment {
    pub fn new(id: u64) -> Self {
        Self {
            id: SegmentId(id),
            facts: Vec::new(),
            bytes: 0,
        }
    }
    
    pub fn add_fact(&mut self, fact: FactV1) -> u32 {
        let fact_size = self.estimate_fact_size(&fact);
        self.facts.push(fact);
        self.bytes += fact_size;
        fact_size
    }
    
    fn estimate_fact_size(&self, fact: &FactV1) -> u32 {
        let mut size = 8 + 8 + 8 + 64;
        match &fact.object {
            super::schema::ValueAtom::U64(_) | super::schema::ValueAtom::I64(_) => size += 8,
            super::schema::ValueAtom::F64(_) => size += 8,
            super::schema::ValueAtom::Bool(_) => size += 1,
            super::schema::ValueAtom::Bytes(bytes) => size += 4 + bytes.len() as u32,
            super::schema::ValueAtom::String(s) => size += 4 + s.len() as u32,
        }
        size
    }
}

#[derive(Debug)]
pub struct Index {
    pub sp: BTreeMap<(EntityId, PredId), Vec<IndexEntry>>,
    pub ps: BTreeMap<PredId, Vec<EntityId>>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            sp: BTreeMap::new(),
            ps: BTreeMap::new(),
        }
    }
    
    pub fn add_fact(&mut self, fact: &FactV1, segment_id: SegmentId, offset: u32) {
        let entry = IndexEntry { segment_id, offset };
        
        let sp_key = (fact.subject.clone(), fact.predicate.clone());
        self.sp.entry(sp_key).or_insert_with(Vec::new).push(entry.clone());
        
        self.ps.entry(fact.predicate.clone()).or_insert_with(Vec::new).push(fact.subject.clone());
    }
    
    pub fn find_subject_predicate(&self, subject: &EntityId, predicate: &PredId) -> Vec<IndexEntry> {
        self.sp.get(&(subject.clone(), predicate.clone())).cloned().unwrap_or_default()
    }
    
    pub fn find_predicate(&self, predicate: &PredId) -> Vec<EntityId> {
        self.ps.get(predicate).cloned().unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct Budget {
    pub total_bytes: usize,
    pub used_bytes: usize,
}

impl Budget {
    pub fn new() -> Self {
        Self {
            total_bytes: BUDGET_BYTES,
            used_bytes: 0,
        }
    }
    
    pub fn can_allocate(&self, bytes: usize) -> bool {
        self.used_bytes + bytes <= self.total_bytes
    }
    
    pub fn allocate(&mut self, bytes: usize) -> Result<(), ()> {
        if self.can_allocate(bytes) {
            self.used_bytes += bytes;
            Ok(())
        } else {
            Err(())
        }
    }
    
    pub fn free(&mut self, bytes: usize) {
        if bytes <= self.used_bytes {
            self.used_bytes -= bytes;
        }
    }
}

#[derive(Debug)]
pub struct ReadView {
    pub snapshot_id: SnapshotId,
    pub segments: Vec<Segment>,
    pub index: Index,
}

impl ReadView {
    pub fn new(snapshot_id: SnapshotId, segments: Vec<Segment>, index: Index) -> Self {
        Self {
            snapshot_id,
            segments,
            index,
        }
    }
}

pub struct WorldModel {
    pub segments: Vec<Segment>,
    pub index: RwLock<Index>,
    pub budget: RwLock<Budget>,
    pub epoch: AtomicU64,
    pub snapshots: RwLock<Vec<SnapshotId>>,
}

impl WorldModel {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            index: RwLock::new(Index::new()),
            budget: RwLock::new(Budget::new()),
            epoch: AtomicU64::new(0),
            snapshots: RwLock::new(Vec::new()),
        }
    }
    
    pub fn put_facts(&mut self, facts: &[FactV1]) -> Result<(u32, SnapshotId), ()> {
        let mut total_size = 0;
        let mut ok_count = 0;
        
        for fact in facts {
            let fact_size = self.estimate_fact_size(fact);
            if !self.budget.read().can_allocate(fact_size) {
                return Err(());
            }
            total_size += fact_size;
        }
        
        self.budget.write().allocate(total_size)?;
        
        let current_epoch = self.epoch.fetch_add(1, Ordering::SeqCst);
        let snapshot_id = SnapshotId(current_epoch);
        
        for fact in facts {
            if let Some(segment) = self.segments.last_mut() {
                if segment.facts.len() >= 1000 {
                    self.segments.push(Segment::new(self.segments.len() as u64));
                }
            } else {
                self.segments.push(Segment::new(0));
            }
            
            let segment = self.segments.last_mut().unwrap();
            let offset = segment.facts.len() as u32;
            let fact_size = segment.add_fact(fact.clone());
            
            self.index.write().add_fact(fact, segment.id.clone(), offset);
            ok_count += 1;
        }
        
        self.snapshots.write().push(snapshot_id);
        
        Ok((ok_count, snapshot_id))
    }
    
    pub fn snapshot_create(&self) -> SnapshotId {
        let current_epoch = self.epoch.fetch_add(1, Ordering::SeqCst);
        let snapshot_id = SnapshotId(current_epoch);
        
        self.snapshots.write().push(snapshot_id);
        snapshot_id
    }
    
    pub fn snapshot_open(&self, id: SnapshotId) -> Option<ReadView> {
        if !self.snapshots.read().contains(&id) {
            return None;
        }
        
        let segments = self.segments.clone();
        let index = self.index.read().clone();
        
        Some(ReadView::new(id, segments, index))
    }
    
    fn estimate_fact_size(&self, fact: &FactV1) -> usize {
        let mut size = 8 + 8 + 8 + 64;
        match &fact.object {
            super::schema::ValueAtom::U64(_) | super::schema::ValueAtom::I64(_) => size += 8,
            super::schema::ValueAtom::F64(_) => size += 8,
            super::schema::ValueAtom::Bool(_) => size += 1,
            super::schema::ValueAtom::Bytes(bytes) => size += 4 + bytes.len(),
            super::schema::ValueAtom::String(s) => size += 4 + s.len(),
        }
        size
    }
    
    pub fn compact_for_tests(&mut self) {
        let mut new_segments = Vec::new();
        let mut new_index = Index::new();
        
        for segment in &self.segments {
            if !segment.facts.is_empty() {
                let mut new_segment = Segment::new(segment.id.0);
                for (offset, fact) in segment.facts.iter().enumerate() {
                    new_segment.add_fact(fact.clone());
                    new_index.add_fact(fact, new_segment.id.clone(), offset as u32);
                }
                new_segments.push(new_segment);
            }
        }
        
        self.segments = new_segments;
        *self.index.write() = new_index;
    }
}
