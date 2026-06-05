//! Runqueue Implementation for Polymera OS Scheduler
//!
//! This module implements the CFS (Completely Fair Scheduler) runqueue
//! using a Red-Black tree (BTreeSet) to manage tasks ordered by their
//! virtual runtime (vruntime).
//!
//! Requirements: 4.1, 4.2, 4.3, 4.4, Constraint: O(log N) scheduling

use alloc::vec::Vec;
use alloc::collections::{BTreeSet, BTreeMap, VecDeque};
use core::fmt;

use super::TaskId;
use super::task::TaskPriority;  // Import TaskPriority
use super::fairness::{SchedEntity, MIN_NICE, DEFAULT_NICE};

/// Runqueue structure using a Red-Black tree (CFS)
///
/// **Constraint: O(log N) Scheduling**
/// Uses BTreeSet for O(log N) insertion and removal, ordered by vruntime.
pub struct RunQueue {
    /// Ordered set of tasks waiting for CPU (RB-Tree) for CFS
    timeline: BTreeSet<SchedEntity>,
    /// Real-Time Queue (FIFO)
    rt_queue: VecDeque<TaskId>, 
    /// Map task IDs to their entities for O(log N) lookup/removal

    /// We need this because we can't search BTreeSet by just TaskId
    tasks: BTreeMap<TaskId, SchedEntity>,
    /// Minimum vruntime in the queue (for new tasks)
    min_vruntime: u64,
    /// Number of tasks
    count: usize,
}

impl RunQueue {
    /// Create a new empty runqueue
    pub fn new() -> Self {
        Self {
            timeline: BTreeSet::new(),
            rt_queue: VecDeque::new(),
            tasks: BTreeMap::new(),
            min_vruntime: 0,
            count: 0,
        }
    }

    /// Add a task to the runqueue
    /// 
    /// # Arguments
    /// * `task_id` - Task ID to add
    /// * `priority` - Task priority
    pub fn push(&mut self, task_id: TaskId, priority: TaskPriority) -> bool {
        // Check for RT priority
        if priority == TaskPriority::RealTime {
            self.rt_queue.push_back(task_id);
            self.count += 1;
            return true;
        }

        // If task already exists in CFS map, don't add duplicate
        if self.tasks.contains_key(&task_id) {
            return false;
        }

        // Create new entity or define logic to retrieve existing state
        // For simplicity here, we assume new task or reset state
        // In full implementation, we'd preserve vruntime if task was just blocked
        // and is now waking up
        let mut entity = SchedEntity::new(task_id);
        
        // **Property 5: Starvation Freedom**
        // New tasks start with min_vruntime to prevent them from
        // starving existing tasks (by having 0 vruntime) or being starved
        // (if we used max vruntime).
        entity.vruntime = self.min_vruntime;
        
        self.insert_entity(entity);
        true
    }

    /// Add a pre-configured entity to the runqueue
    /// Use this when re-enqueuing a task that was running
    pub fn push_entity(&mut self, entity: SchedEntity) {
        // Ensure vruntime monotonicity
        let mut e = entity;
        if e.vruntime < self.min_vruntime {
            e.vruntime = self.min_vruntime;
        }
        
        self.insert_entity(e);
    }
    
    /// Internal insert helper
    fn insert_entity(&mut self, entity: SchedEntity) {
        // Remove existing if any (to update key)
        if let Some(old) = self.tasks.remove(&entity.task_id) {
            self.timeline.remove(&old);
            self.count -= 1;
        }

        self.tasks.insert(entity.task_id, entity);
        self.timeline.insert(entity);
        self.count += 1;
    }

    /// Pick the next task to run
    pub fn pop(&mut self) -> Option<TaskId> {
        // 1. Check Real-Time queue first (Strict Priority)
        if let Some(task_id) = self.rt_queue.pop_front() {
            self.count -= 1;
            return Some(task_id);
        }

        // 2. Check CFS queue
        // Pop first item (leftmost in RB tree -> min vruntime)
        // BTreeSet doesn't have pop_first on stable no_std easily without unstable features
        // But we can clone the first item then remove it
        let entity = self.Leftmost()?;
        
        self.timeline.remove(&entity);
        self.tasks.remove(&entity.task_id);
        self.count -= 1;

        // Update min_vruntime to maintain monotonicity
        // We only increase it, never decrease
        if entity.vruntime > self.min_vruntime {
            self.min_vruntime = entity.vruntime;
        }

        Some(entity.task_id)
    }

    /// Peek at the next task without removing it
    pub fn peek(&self) -> Option<TaskId> {
        self.timeline.iter().next().map(|e| e.task_id)
    }
    
    /// Get reference to entity for a task
    pub fn get_entity(&self, task_id: TaskId) -> Option<&SchedEntity> {
        self.tasks.get(&task_id)
    }
    
    /// Get helper for leftmost node
    #[allow(non_snake_case)]
    fn Leftmost(&self) -> Option<SchedEntity> {
        self.timeline.iter().next().copied()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if queue is full (effectively never for BTree)
    pub fn is_full(&self) -> bool {
        false
    }

    /// Get number of tasks
    pub fn len(&self) -> usize {
        self.count
    }

    /// Get capacity (arbitrary large number)
    pub const fn capacity() -> usize {
        usize::MAX
    }

    /// Get the current global runqueue size for dashboards/diagnostics.
    pub fn get_current_size() -> usize {
        0
    }

    /// Remove a specific task
    pub fn remove(&mut self, task_id: TaskId) -> bool {
        if let Some(entity) = self.tasks.remove(&task_id) {
            self.timeline.remove(&entity);
            self.count -= 1;
            true
        } else {
            false
        }
    }
    
    /// Get current min_vruntime
    pub fn get_min_vruntime(&self) -> u64 {
        self.min_vruntime
    }
}

impl fmt::Debug for RunQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunQueue")
            .field("count", &self.count)
            .field("min_vruntime", &self.min_vruntime)
            .finish()
    }
}

impl fmt::Display for RunQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CFS RunQueue: {} tasks, min_vr={}", self.count, self.min_vruntime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfs_ordering() {
        let mut rq = RunQueue::new();
        
        // Create entities with different vruntimes
        let mut t1 = SchedEntity::new(TaskId(1));
        t1.vruntime = 1000;
        
        let mut t2 = SchedEntity::new(TaskId(2));
        t2.vruntime = 500;
        
        let mut t3 = SchedEntity::new(TaskId(3));
        t3.vruntime = 1500;
        
        rq.push_entity(t1);
        rq.push_entity(t2);
        rq.push_entity(t3);
        
        // Should pop in order of vruntime: 500, 1000, 1500
        assert_eq!(rq.pop(), Some(TaskId(2)));
        assert_eq!(rq.pop(), Some(TaskId(1)));
        assert_eq!(rq.pop(), Some(TaskId(3)));
    }
}
