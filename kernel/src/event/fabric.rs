use alloc::string::ToString;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::topics::{TopicDesc, find_topic, get_matching_topics};
use super::queue::{Event, Lane, Inbox, InboxStats};
use crate::secman::cap_flags::*;

/// Subscription pattern type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Exact(String),
    Prefix(String),
}

impl Pattern {
    pub fn new(pattern: String) -> Result<Self, &'static str> {
        if pattern.is_empty() || pattern.len() > 64 {
            return Err("Invalid pattern length");
        }
        
        if pattern.ends_with(".*") {
            let prefix = pattern[..pattern.len() - 2].to_string();
            if prefix.is_empty() {
                return Err("Invalid prefix pattern");
            }
            Ok(Pattern::Prefix(prefix))
        } else {
            Ok(Pattern::Exact(pattern))
        }
    }
    
    pub fn matches(&self, topic_name: &str) -> bool {
        match self {
            Pattern::Exact(exact) => topic_name == exact,
            Pattern::Prefix(prefix) => topic_name.starts_with(prefix) && topic_name != prefix,
        }
    }
}

/// Active subscription
#[derive(Debug)]
pub struct Subscription {
    pub handle: u32,
    pub pattern: Pattern,
    pub lane: Lane,
    pub caps_mask: u64,
    pub task_id: u32,
}

/// Event fabric statistics
#[derive(Debug, Clone, Default)]
pub struct FabricStats {
    pub total_events_published: u64,
    pub total_events_delivered: u64,
    pub total_drops_hi: u64,
    pub total_drops_med: u64,
    pub total_drops_lo: u64,
    pub total_subscriptions: u64,
    pub total_unsubscriptions: u64,
}

/// Main event fabric that manages subscriptions and routing
pub struct EventFabric {
    subscriptions: Mutex<Vec<Subscription>>,
    task_inboxes: Mutex<BTreeMap<u32, Inbox>>,
    next_subscription_id: AtomicU64,
    stats: Mutex<FabricStats>,
}

impl EventFabric {
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(Vec::new()),
            task_inboxes: Mutex::new(BTreeMap::new()),
            next_subscription_id: AtomicU64::new(1),
            stats: Mutex::new(FabricStats::default()),
        }
    }

    /// Subscribe to events matching a pattern
    pub fn subscribe(
        &self,
        task_id: u32,
        pattern: Pattern,
        lane: Lane,
        caps_mask: u64,
    ) -> Result<u32, &'static str> {
        // Validate pattern
        if let Pattern::Prefix(ref prefix) = pattern {
            // Prefix subscriptions require elevated caps for security
            if !self.has_elevated_caps(caps_mask) {
                return Err("Prefix subscriptions require elevated capabilities");
            }
        }
        
        // Create subscription
        let handle = self.next_subscription_id.fetch_add(1, Ordering::SeqCst) as u32;
        let subscription = Subscription {
            handle,
            pattern,
            lane,
            caps_mask,
            task_id,
        };
        
        // Add to subscriptions
        self.subscriptions.lock().push(subscription);
        
        // Ensure task has an inbox
        if !self.task_inboxes.lock().contains_key(&task_id) {
            let inbox = Inbox::new(256 * 1024); // 256 KiB default budget
            self.task_inboxes.lock().insert(task_id, inbox);
        }
        
        // Update stats
        self.stats.lock().total_subscriptions += 1;
        
        Ok(handle)
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, handle: u32) -> Result<(), &'static str> {
        let mut subscriptions = self.subscriptions.lock();
        
        if let Some(index) = subscriptions.iter().position(|s| s.handle == handle) {
            subscriptions.remove(index);
            self.stats.lock().total_unsubscriptions += 1;
            Ok(())
        } else {
            Err("Subscription not found")
        }
    }

    /// Publish an event to all matching subscribers
    pub fn publish(&self, topic_name: &str, priority: u8, payload: Vec<u8>) -> Result<u64, &'static str> {
        // Find topic descriptor
        let topic = find_topic(topic_name).ok_or("Topic not found")?;
        
        // Create event
        let ts_vclock = self.get_virtual_clock();
        let event = Event::new(topic.id, ts_vclock, priority, payload);
        
        // Find all matching subscriptions
        let subscriptions = self.subscriptions.lock();
        let mut delivery_count = 0u64;
        let mut drops_hi = 0u64;
        let mut drops_med = 0u64;
        let mut drops_lo = 0u64;
        
        for subscription in subscriptions.iter() {
            if !subscription.pattern.matches(topic_name) {
                continue;
            }
            
            // Check capabilities
            if (subscription.caps_mask & topic.caps_required_sub) != topic.caps_required_sub {
                continue;
            }
            
            // Deliver to task inbox
            if let Some(inbox) = self.task_inboxes.lock().get_mut(&subscription.task_id) {
                match inbox.enqueue(event.clone()) {
                    Ok(_) => {
                        delivery_count += 1;
                    }
                    Err(_) => {
                        // Event was dropped due to queue limits
                        match priority {
                            0 => drops_hi += 1,
                            1 => drops_med += 1,
                            2 => drops_lo += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Update statistics
        {
            let mut stats = self.stats.lock();
            stats.total_events_published += 1;
            stats.total_events_delivered += delivery_count;
            stats.total_drops_hi += drops_hi;
            stats.total_drops_med += drops_med;
            stats.total_drops_lo += drops_lo;
        }
        
        Ok(delivery_count)
    }

    /// Poll events from a task's inbox
    pub fn poll(&self, task_id: u32, max_events: usize) -> Result<Vec<Event>, &'static str> {
        let mut inboxes = self.task_inboxes.lock();
        
        if let Some(inbox) = inboxes.get_mut(&task_id) {
            let events = inbox.dequeue_batch(max_events);
            Ok(events)
        } else {
            Err("Task inbox not found")
        }
    }

    /// Acknowledge events (for diagnostics)
    pub fn ack(&self, task_id: u32, event_id: u64) -> Result<(), &'static str> {
        let mut inboxes = self.task_inboxes.lock();
        
        if let Some(inbox) = inboxes.get_mut(&task_id) {
            inbox.update_seen_id(event_id);
            Ok(())
        } else {
            Err("Task inbox not found")
        }
    }

    /// Get task inbox statistics
    pub fn get_inbox_stats(&self, task_id: u32) -> Option<InboxStats> {
        let inboxes = self.task_inboxes.lock();
        inboxes.get(&task_id).map(|inbox| inbox.stats())
    }

    /// Get fabric statistics
    pub fn get_stats(&self) -> FabricStats {
        self.stats.lock().clone()
    }

    /// Remove task inbox (when task terminates)
    pub fn remove_task(&self, task_id: u32) {
        // Remove subscriptions for this task
        let mut subscriptions = self.subscriptions.lock();
        subscriptions.retain(|s| s.task_id != task_id);
        
        // Remove inbox
        self.task_inboxes.lock().remove(&task_id);
    }

    /// Check if capabilities are elevated for prefix subscriptions
    fn has_elevated_caps(&self, caps_mask: u64) -> bool {
        // For now, require INTENT_SUBMIT or WM_WRITE for prefix subscriptions
        // This can be made more configurable in the future
        (caps_mask & CAP_INTENT_SUBMIT) != 0 || (caps_mask & CAP_WM_WRITE) != 0
    }

    /// Get virtual clock timestamp
    fn get_virtual_clock(&self) -> u64 {
        // This would use the kernel's virtual clock
        // For now, use a simple counter
        static CLOCK_COUNTER: AtomicU64 = AtomicU64::new(0);
        CLOCK_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// Get all subscriptions for a task
    pub fn get_task_subscriptions(&self, task_id: u32) -> Vec<Subscription> {
        let subscriptions = self.subscriptions.lock();
        subscriptions.iter()
            .filter(|s| s.task_id == task_id)
            .cloned()
            .collect()
    }

    /// Get all active topics
    pub fn get_active_topics(&self) -> Vec<&'static TopicDesc> {
        // Return all topics for now
        // In the future, this could track which topics have active subscribers
        super::topics::TOPICS.to_vec()
    }
}

impl Clone for Subscription {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            pattern: self.pattern.clone(),
            lane: self.lane,
            caps_mask: self.caps_mask,
            task_id: self.task_id,
        }
    }
}

/// Global event fabric instance
pub static EVENT_FABRIC: EventFabric = EventFabric::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let exact = Pattern::new("intent.created".to_string()).unwrap();
        let prefix = Pattern::new("intent.*".to_string()).unwrap();
        
        assert!(exact.matches("intent.created"));
        assert!(!exact.matches("intent.previewed"));
        
        assert!(prefix.matches("intent.created"));
        assert!(prefix.matches("intent.previewed"));
        assert!(!prefix.matches("wm.put"));
    }

    #[test]
    fn test_subscription_lifecycle() {
        let fabric = EventFabric::new();
        let task_id = 123;
        let pattern = Pattern::new("intent.*".to_string()).unwrap();
        
        // Subscribe
        let handle = fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT).unwrap();
        assert_eq!(fabric.get_task_subscriptions(task_id).len(), 1);
        
        // Unsubscribe
        fabric.unsubscribe(handle).unwrap();
        assert_eq!(fabric.get_task_subscriptions(task_id).len(), 0);
    }

    #[test]
    fn test_event_publishing() {
        let fabric = EventFabric::new();
        let task_id = 123;
        let pattern = Pattern::new("intent.created".to_string()).unwrap();
        
        // Subscribe
        fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_QUERY).unwrap();
        
        // Publish event
        let payload = b"test event".to_vec();
        let delivery_count = fabric.publish("intent.created", 0, payload).unwrap();
        assert_eq!(delivery_count, 1);
        
        // Poll events
        let events = fabric.poll(task_id, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].header.topic_id, 1); // intent.created
    }

    #[test]
    fn test_prefix_subscription_caps() {
        let fabric = EventFabric::new();
        let task_id = 123;
        let pattern = Pattern::new("intent.*".to_string()).unwrap();
        
        // Should fail without elevated caps
        let result = fabric.subscribe(task_id, pattern.clone(), Lane::HI, 0);
        assert!(result.is_err());
        
        // Should succeed with elevated caps
        let result = fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT);
        assert!(result.is_ok());
    }
}
