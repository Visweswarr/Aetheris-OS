//! Completely Fair Scheduler (CFS) Logic
//!
//! This module implements the core logic for the CFS scheduler, including
//! virtual runtime (vruntime) calculation, nice values, and weight mapping.
//!
//! Requirements: 4.1, 4.2, 4.3

pub use crate::sched::task::{Task, TaskId};

/// Nice value range (-20 to 19)
pub const MIN_NICE: i32 = -20;
pub const MAX_NICE: i32 = 19;
pub const DEFAULT_NICE: i32 = 0;

/// Base time slice in nanoseconds
pub const BASE_TIME_SLICE_NS: u64 = 10_000_000; // 10ms

/// Standard load weight for nice 0
pub const NICE_0_LOAD: u64 = 1024;

/// Map nice values to relative weights
/// Corresponds to roughly 10% cpu capacity difference per nice level
static PRIO_TO_WEIGHT: [u64; 40] = [
    88761, 71755, 56483, 46273, 36291, // -20 .. -16
    29154, 23254, 18705, 14949, 11916, // -15 .. -11
    9548, 7620, 6100, 4904, 3906,      // -10 .. -6
    3121, 2501, 1991, 1586, 1277,      // -5  .. -1
    1024, 820, 655, 526, 423,          // 0   .. 4
    335, 272, 215, 172, 137,           // 5   .. 9
    110, 87, 70, 56, 45,               // 10  .. 14
    36, 29, 23, 18, 15,                // 15  .. 19
];

/// Scheduler entity structure tracked in the Red-Black tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedEntity {
    /// Associated task ID
    pub task_id: TaskId,
    /// Nice value (-20 to 19)
    pub nice: i32,
    /// Load weight based on nice value
    pub weight: u64,
    /// Virtual runtime (nsec) - key for RB tree
    pub vruntime: u64,
    /// Total execution time (nsec)
    pub sum_exec_runtime: u64,
    /// Previous execution time (nsec)
    pub prev_sum_exec_runtime: u64,
}

impl SchedEntity {
    /// Create a new scheduler entity with default nice 0
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            nice: DEFAULT_NICE,
            weight: NICE_0_LOAD,
            vruntime: 0,
            sum_exec_runtime: 0,
            prev_sum_exec_runtime: 0,
        }
    }

    /// Set nice value and update weight
    pub fn set_nice(&mut self, nice: i32) {
        let clamped = nice.max(MIN_NICE).min(MAX_NICE);
        self.nice = clamped;
        
        // Map nice to array index: -20 -> 0, 19 -> 39
        let index = (clamped - MIN_NICE) as usize;
        self.weight = PRIO_TO_WEIGHT[index];
    }

    /// Calculate time slice based on weight
    pub fn calc_time_slice(&self, total_weight: u64) -> u64 {
        if total_weight == 0 {
            return BASE_TIME_SLICE_NS;
        }
        
        // slice = base_slice * (weight / total_weight)
        // Scaled to avoid precision loss
        (BASE_TIME_SLICE_NS * self.weight) / total_weight
    }

    /// Update virtual runtime based on actual runtime
    ///
    /// vruntime += delta_exec * (NICE_0_LOAD / weight)
    /// High priority (high weight) tasks gain vruntime slower
    /// Low priority (low weight) tasks gain vruntime faster
    pub fn update_vruntime(&mut self, delta_exec: u64) {
        self.sum_exec_runtime += delta_exec;
        
        if self.weight == NICE_0_LOAD {
            self.vruntime += delta_exec;
        } else {
            // Apply weight scaling
            // vruntime += (delta_exec * NICE_0_LOAD) / weight
            // Use 128-bit arithmetic to hold intermediate product if needed
            // But since u64 max is huge, we'll check overflow carefully
            let weighted_delta = (delta_exec as u128 * NICE_0_LOAD as u128) / self.weight as u128;
            self.vruntime += weighted_delta as u64;
        }
    }
}

/// Calculate delta exec time
pub fn calc_delta_exec(curr_time: u64, prev_time: u64) -> u64 {
    curr_time.saturating_sub(prev_time)
}

/// Initialize the fairness subsystem (stub – currently nothing to set up).
pub fn init() {}

/// Ordering for RB tree (min vruntime first)
impl Ord for SchedEntity {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Primary sort key: vruntime (ascending)
        match self.vruntime.cmp(&other.vruntime) {
            core::cmp::Ordering::Equal => {
                // Secondary sort key: task_id (stable tie-breaker)
                self.task_id.cmp(&other.task_id)
            }
            order => order,
        }
    }
}

impl PartialOrd for SchedEntity {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_to_weight() {
        let mut entity = SchedEntity::new(TaskId(1));
        
        // Test Nice 0
        entity.set_nice(0);
        assert_eq!(entity.weight, 1024);
        
        // Test Nice -20 (Highest priority)
        entity.set_nice(-20);
        assert_eq!(entity.weight, 88761);
        
        // Test Nice 19 (Lowest priority)
        entity.set_nice(19);
        assert_eq!(entity.weight, 15);
    }

    #[test]
    fn test_vruntime_update() {
        let mut nice0 = SchedEntity::new(TaskId(1));
        nice0.set_nice(0);
        
        let mut nice_plus10 = SchedEntity::new(TaskId(2));
        nice_plus10.set_nice(10); // weight 110
        
        // Run both for 10ms
        let runtime = 10_000_000;
        
        nice0.update_vruntime(runtime);
        // Nice 0: vruntime += runtime * 1024/1024 = runtime
        assert_eq!(nice0.vruntime, runtime);
        
        nice_plus10.update_vruntime(runtime);
        // Nice 10: vruntime += runtime * 1024/110 approx 9.3 * runtime
        let expected = (runtime as u128 * 1024 / 110) as u64;
        assert_eq!(nice_plus10.vruntime, expected);
        
        // Lower priority task gained more vruntime -> moves to back of queue faster
        assert!(nice_plus10.vruntime > nice0.vruntime);
    }
}
