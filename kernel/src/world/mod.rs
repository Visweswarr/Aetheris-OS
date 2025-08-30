use crate::{kprintln, klog};
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::schema::{FactV1, SnapshotId, EntityId, ValueAtom};
use super::store::{WorldModel, ReadView};
use super::query::{Pattern, Range, Rows, get_entity_caps, list_devices, last_seen};

pub mod schema;
pub mod store;
pub mod query;

pub struct WorldModelKernel {
    pub world: Mutex<WorldModel>,
    pub counters: WorldCounters,
}

impl WorldModelKernel {
    pub fn new() -> Self {
        Self {
            world: Mutex::new(WorldModel::new()),
            counters: WorldCounters::new(),
        }
    }
    
    pub fn put_facts(&self, facts: &[FactV1]) -> Result<(u32, SnapshotId), ()> {
        let result = self.world.lock().put_facts(facts);
        
        match result {
            Ok((count, snapshot_id)) => {
                self.counters.put_ok.fetch_add(count as u64, Ordering::SeqCst);
                Ok((count, snapshot_id))
            }
            Err(()) => {
                self.counters.put_enospc.fetch_add(1, Ordering::SeqCst);
                Err(())
            }
        }
    }
    
    pub fn query(&self, pattern: &Pattern, range: &Range, limit: u16, offset: u32) -> Result<Rows, ()> {
        let view = {
            let world = self.world.lock();
            let current_snapshot = world.snapshot_create();
            world.snapshot_open(current_snapshot).ok_or(())?
        };
        
        let rows = query::query(&view, pattern, range, limit, offset);
        self.counters.query_ok.fetch_add(1, Ordering::SeqCst);
        
        Ok(rows)
    }
    
    pub fn snapshot_create(&self) -> SnapshotId {
        let snapshot_id = self.world.lock().snapshot_create();
        self.counters.snapshots.fetch_add(1, Ordering::SeqCst);
        snapshot_id
    }
    
    pub fn snapshot_open(&self, id: SnapshotId) -> Option<ReadView> {
        self.world.lock().snapshot_open(id)
    }
    
    pub fn export_snapshot(&self, id: SnapshotId, subject_filter: Option<EntityId>) -> Result<Vec<u8>, ()> {
        let view = self.world.lock().snapshot_open(id).ok_or(())?;
        
        let mut facts = Vec::new();
        for segment in &view.segments {
            for fact in &segment.facts {
                if let Some(filter_subject) = subject_filter {
                    if fact.subject == filter_subject {
                        facts.push(fact.clone());
                    }
                } else {
                    facts.push(fact.clone());
                }
            }
        }
        
        let cbor_data = serde_cbor::to_vec(&facts).map_err(|_| ())?;
        self.counters.export_ok.fetch_add(1, Ordering::SeqCst);
        
        Ok(cbor_data)
    }
    
    pub fn get_entity_caps(&self, entity_id: &EntityId) -> Vec<FactV1> {
        let view = {
            let world = self.world.lock();
            let current_snapshot = world.snapshot_create();
            world.snapshot_open(current_snapshot).unwrap_or_else(|| {
                let empty_view = ReadView::new(current_snapshot, Vec::new(), query::Index::new());
                empty_view
            })
        };
        
        get_entity_caps(&view, entity_id)
    }
    
    pub fn list_devices(&self) -> Vec<EntityId> {
        let view = {
            let world = self.world.lock();
            let current_snapshot = world.snapshot_create();
            world.snapshot_open(current_snapshot).unwrap_or_else(|| {
                let empty_view = ReadView::new(current_snapshot, Vec::new(), query::Index::new());
                empty_view
            })
        };
        
        list_devices(&view)
    }
    
    pub fn last_seen(&self, entity_id: &EntityId) -> Option<u64> {
        let view = {
            let world = self.world.lock();
            let current_snapshot = world.snapshot_create();
            world.snapshot_open(current_snapshot).unwrap_or_else(|| {
                let empty_view = ReadView::new(current_snapshot, Vec::new(), query::Index::new());
                empty_view
            })
        };
        
        last_seen(&view, entity_id)
    }
    
    pub fn compact_for_tests(&self) {
        self.world.lock().compact_for_tests();
    }
    
    pub fn get_stats(&self) -> WorldStats {
        WorldStats {
            put_ok: self.counters.put_ok.load(Ordering::SeqCst),
            put_enospc: self.counters.put_enospc.load(Ordering::SeqCst),
            query_ok: self.counters.query_ok.load(Ordering::SeqCst),
            snapshots: self.counters.snapshots.load(Ordering::SeqCst),
            export_ok: self.counters.export_ok.load(Ordering::SeqCst),
        }
    }
}

pub struct WorldCounters {
    pub put_ok: AtomicU64,
    pub put_enospc: AtomicU64,
    pub query_ok: AtomicU64,
    pub snapshots: AtomicU64,
    pub export_ok: AtomicU64,
}

impl WorldCounters {
    pub fn new() -> Self {
        Self {
            put_ok: AtomicU64::new(0),
            put_enospc: AtomicU64::new(0),
            query_ok: AtomicU64::new(0),
            snapshots: AtomicU64::new(0),
            export_ok: AtomicU64::new(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorldStats {
    pub put_ok: u64,
    pub put_enospc: u64,
    pub query_ok: u64,
    pub snapshots: u64,
    pub export_ok: u64,
}

impl WorldStats {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"test":"wm_stats","put_ok":{},"put_enospc":{},"query_ok":{},"snapshots":{},"export_ok":{}}}"#,
            self.put_ok, self.put_enospc, self.query_ok, self.snapshots, self.export_ok
        )
    }
}
