//! Event Fabric v0 - Deterministic, low-latency event system
//! 
//! This module provides a real-time event fabric for exchanging signals between
//! kernel subsystems, skills, and user tasks with priority-aware delivery and
//! strict capability checks.

pub mod topics;
pub mod queue;
pub mod fabric;

use crate::secman::cap_flags::*;
use crate::event::fabric::EVENT_FABRIC;
use crate::event::queue::{Event, Lane};
use alloc::string::ToString;
use alloc::format;
use alloc::vec::Vec;

/// Event Fabric kernel interface
pub struct EventKernel;

impl EventKernel {
    /// Publish an event from kernel context
    pub fn publish_kernel_event(topic_name: &str, priority: Lane, payload: Vec<u8>) -> Result<u64, &'static str> {
        EVENT_FABRIC.publish(topic_name, priority as u8, payload)
    }
    
    /// Publish intent-related events
    pub fn publish_intent_event(event_type: &str, intent_id: u128, metadata: &str) -> Result<u64, &'static str> {
        let topic = format!("intent.{}", event_type);
        let payload = format!("{{\"intent_id\":\"{}\",\"metadata\":\"{}\"}}", intent_id, metadata);
        Self::publish_kernel_event(&topic, Lane::HI, payload.into_bytes())
    }
    
    /// Publish world model events
    pub fn publish_wm_event(event_type: &str, entity_id: u128, count: u32) -> Result<u64, &'static str> {
        let topic = format!("wm.{}", event_type);
        let payload = format!("{{\"entity_id\":\"{}\",\"count\":{}}}", entity_id, count);
        Self::publish_kernel_event(&topic, Lane::MED, payload.into_bytes())
    }
    
    /// Publish skill events
    pub fn publish_skill_event(event_type: &str, skill_id: u32, metadata: &str) -> Result<u64, &'static str> {
        let topic = format!("skill.{}", event_type);
        let payload = format!("{{\"skill_id\":{},\"metadata\":\"{}\"}}", skill_id, metadata);
        Self::publish_kernel_event(&topic, Lane::MED, payload.into_bytes())
    }
    
    /// Publish system events
    pub fn publish_sys_event(event_type: &str, data: &str) -> Result<u64, &'static str> {
        let topic = format!("sys.{}", event_type);
        Self::publish_kernel_event(&topic, Lane::LO, data.as_bytes().to_vec())
    }
    
    /// Get event fabric statistics
    pub fn get_stats() -> crate::event::fabric::FabricStats {
        EVENT_FABRIC.get_stats()
    }
    
    /// Get task inbox statistics
    pub fn get_task_stats(task_id: u32) -> Option<crate::event::queue::InboxStats> {
        EVENT_FABRIC.get_inbox_stats(task_id)
    }
    
    /// Remove task from event fabric (when task terminates)
    pub fn remove_task(task_id: u32) {
        EVENT_FABRIC.remove_task(task_id);
    }
}

/// Event Fabric integration helpers
pub mod integration {
    use super::*;
    
    /// Initialize event fabric for a new task
    pub fn init_task_events(task_id: u32) -> Result<(), &'static str> {
        // This would be called when a new task is created
        // For now, just ensure the task has an inbox by subscribing to a dummy topic
        let pattern = crate::event::fabric::Pattern::new("sys.timer".to_string())?;
        EVENT_FABRIC.subscribe(task_id, pattern, Lane::LO, 0)?;
        Ok(())
    }
    
    /// Clean up event fabric for a terminating task
    pub fn cleanup_task_events(task_id: u32) {
        EVENT_FABRIC.remove_task(task_id);
    }
    
    /// Check if event fabric is available
    pub fn is_available() -> bool {
        // Check if EVENT_FABRIC_V0 feature bit is set
        crate::abi::features::has_feature(crate::abi::features::KernelFeature::EVENT_FABRIC_V0)
    }
}

/// Event Fabric constants
pub mod constants {
    /// Maximum event payload size
    pub const MAX_EVENT_PAYLOAD: usize = 4096;
    
    /// Maximum events per poll batch
    pub const MAX_EVENTS_PER_POLL: usize = 64;
    
    /// Default task inbox budget (256 KiB)
    pub const DEFAULT_INBOX_BUDGET: usize = 256 * 1024;
    
    /// High priority lane capacity
    pub const HI_LANE_CAPACITY: usize = 256;
    
    /// Medium priority lane capacity
    pub const MED_LANE_CAPACITY: usize = 512;
    
    /// Low priority lane capacity
    pub const LO_LANE_CAPACITY: usize = 1024;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_event_publishing() {
        // Test kernel event publishing
        let result = EventKernel::publish_sys_event("test", "test data");
        assert!(result.is_ok());
    }

    #[test]
    fn test_intent_event_publishing() {
        // Test intent event publishing
        let result = EventKernel::publish_intent_event("created", 123, "test intent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_wm_event_publishing() {
        // Test world model event publishing
        let result = EventKernel::publish_wm_event("put", 456, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_skill_event_publishing() {
        // Test skill event publishing
        let result = EventKernel::publish_skill_event("loaded", 789, "test skill");
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration_helpers() {
        // Test integration helpers
        assert!(integration::is_available());
        
        let result = integration::init_task_events(999);
        assert!(result.is_ok());
        
        integration::cleanup_task_events(999);
    }
}
