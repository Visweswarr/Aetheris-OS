use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};

/// Event priority lanes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    HI = 0,   // High priority - lossless until bound
    MED = 1,  // Medium priority - drop oldest when full
    LO = 2,   // Low priority - drop oldest when full
}

impl Lane {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Lane::HI),
            1 => Some(Lane::MED),
            2 => Some(Lane::LO),
            _ => None,
        }
    }
}

/// Event header with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHeader {
    pub topic_id: u32,
    pub ts_vclock: u64,
    pub prio: u8,
    pub size: u16,
    pub flags: u8,
    pub id: u64,
}

/// Event with header and payload
#[derive(Debug, Clone)]
pub struct Event {
    pub header: EventHeader,
    pub payload: Vec<u8>,
}

impl Event {
    pub fn new(topic_id: u32, ts_vclock: u64, prio: u8, payload: Vec<u8>) -> Self {
        let size = payload.len() as u16;
        let id = EventIdGenerator::next();
        
        Self {
            header: EventHeader {
                topic_id,
                ts_vclock,
                prio,
                size,
                flags: 0,
                id,
            },
            payload,
        }
    }

    pub fn total_size(&self) -> usize {
        core::mem::size_of::<EventHeader>() + self.payload.len()
    }
}

/// Bounded queue for a priority lane
pub struct BoundedQueue {
    queue: VecDeque<Event>,
    max_size: usize,
    max_bytes: usize,
    current_bytes: usize,
    drops: AtomicU64,
}

impl BoundedQueue {
    pub fn new(max_size: usize, max_bytes: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max_size,
            max_bytes,
            current_bytes: 0,
            drops: AtomicU64::new(0),
        }
    }

    /// Enqueue an event, potentially dropping oldest if full
    pub fn enqueue(&mut self, event: Event) -> Result<(), Event> {
        let event_size = event.total_size();
        
        // Check if we can fit this event
        if self.queue.len() >= self.max_size || self.current_bytes + event_size > self.max_bytes {
            // Queue is full, need to drop oldest
            if let Some(oldest) = self.queue.pop_front() {
                self.current_bytes -= oldest.total_size();
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }
        
        // Now enqueue the new event
        self.current_bytes += event_size;
        self.queue.push_back(event);
        Ok(())
    }

    /// Dequeue the next event
    pub fn dequeue(&mut self) -> Option<Event> {
        if let Some(event) = self.queue.pop_front() {
            self.current_bytes -= event.total_size();
            Some(event)
        } else {
            None
        }
    }

    /// Peek at the next event without removing it
    pub fn peek(&self) -> Option<&Event> {
        self.queue.front()
    }

    /// Get current queue statistics
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            length: self.queue.len(),
            bytes: self.current_bytes,
            drops: self.drops.load(Ordering::SeqCst),
            max_size: self.max_size,
            max_bytes: self.max_bytes,
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.max_size || self.current_bytes >= self.max_bytes
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.queue.clear();
        self.current_bytes = 0;
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub length: usize,
    pub bytes: usize,
    pub drops: u64,
    pub max_size: usize,
    pub max_bytes: usize,
}

/// Per-task inbox with priority lanes
pub struct Inbox {
    pub lanes: [BoundedQueue; 3],
    pub budget_bytes: usize,
    pub seen_id: u64,
}

impl Inbox {
    pub fn new(budget_bytes: usize) -> Self {
        // Default lane configurations
        let hi_queue = BoundedQueue::new(256, 64 * 1024);   // 256 events, 64 KiB
        let med_queue = BoundedQueue::new(512, 128 * 1024); // 512 events, 128 KiB
        let lo_queue = BoundedQueue::new(1024, 64 * 1024);  // 1024 events, 64 KiB
        
        Self {
            lanes: [hi_queue, med_queue, lo_queue],
            budget_bytes,
            seen_id: 0,
        }
    }

    /// Enqueue event to appropriate lane
    pub fn enqueue(&mut self, event: Event) -> Result<(), Event> {
        let lane = Lane::from_u8(event.header.prio).unwrap_or(Lane::LO);
        let lane_idx = lane as usize;
        
        self.lanes[lane_idx].enqueue(event)
    }

    /// Dequeue events from all lanes in priority order
    pub fn dequeue_batch(&mut self, max_events: usize) -> Vec<Event> {
        let mut events = Vec::new();
        let mut remaining = max_events;
        
        // Process lanes in priority order: HI, MED, LO
        for lane_idx in 0..3 {
            if remaining == 0 {
                break;
            }
            
            let lane = &mut self.lanes[lane_idx];
            while !lane.is_empty() && remaining > 0 {
                if let Some(event) = lane.dequeue() {
                    events.push(event);
                    remaining -= 1;
                }
            }
        }
        
        events
    }

    /// Get inbox statistics
    pub fn stats(&self) -> InboxStats {
        InboxStats {
            hi: self.lanes[0].stats(),
            med: self.lanes[1].stats(),
            lo: self.lanes[2].stats(),
            total_bytes: self.lanes.iter().map(|l| l.current_bytes).sum(),
            budget_bytes: self.budget_bytes,
            seen_id: self.seen_id,
        }
    }

    /// Update seen ID (for ACK tracking)
    pub fn update_seen_id(&mut self, event_id: u64) {
        if event_id > self.seen_id {
            self.seen_id = event_id;
        }
    }

    /// Check if inbox is over budget
    pub fn is_over_budget(&self) -> bool {
        let total_bytes: usize = self.lanes.iter().map(|l| l.current_bytes).sum();
        total_bytes > self.budget_bytes
    }
}

/// Inbox statistics
#[derive(Debug, Clone)]
pub struct InboxStats {
    pub hi: QueueStats,
    pub med: QueueStats,
    pub lo: QueueStats,
    pub total_bytes: usize,
    pub budget_bytes: usize,
    pub seen_id: u64,
}

/// Global event ID generator
struct EventIdGenerator;

impl EventIdGenerator {
    static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
    
    pub fn next() -> u64 {
        EventIdGenerator::ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lane_ordering() {
        assert!(Lane::HI < Lane::MED);
        assert!(Lane::MED < Lane::LO);
        assert_eq!(Lane::from_u8(0), Some(Lane::HI));
        assert_eq!(Lane::from_u8(1), Some(Lane::MED));
        assert_eq!(Lane::from_u8(2), Some(Lane::LO));
        assert_eq!(Lane::from_u8(3), None);
    }

    #[test]
    fn test_bounded_queue_enqueue() {
        let mut queue = BoundedQueue::new(2, 100);
        
        let event1 = Event::new(1, 100, 0, vec![1, 2, 3]);
        let event2 = Event::new(2, 101, 0, vec![4, 5, 6]);
        let event3 = Event::new(3, 102, 0, vec![7, 8, 9]);
        
        assert!(queue.enqueue(event1).is_ok());
        assert!(queue.enqueue(event2).is_ok());
        assert!(queue.enqueue(event3).is_ok()); // Should drop oldest
        
        assert_eq!(queue.queue.len(), 2);
        assert_eq!(queue.stats().drops, 1);
    }

    #[test]
    fn test_inbox_priority_ordering() {
        let mut inbox = Inbox::new(1024);
        
        // Add events in mixed priority order
        let hi_event = Event::new(1, 100, 0, vec![1]); // HI
        let med_event = Event::new(2, 101, 1, vec![2]); // MED
        let lo_event = Event::new(3, 102, 2, vec![3]); // LO
        
        inbox.enqueue(lo_event).unwrap();
        inbox.enqueue(hi_event).unwrap();
        inbox.enqueue(med_event).unwrap();
        
        // Dequeue should return in priority order: HI, MED, LO
        let batch = inbox.dequeue_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].header.topic_id, 1); // HI first
        assert_eq!(batch[1].header.topic_id, 2); // MED second
        assert_eq!(batch[2].header.topic_id, 3); // LO third
    }

    #[test]
    fn test_event_id_generation() {
        let id1 = EventIdGenerator::next();
        let id2 = EventIdGenerator::next();
        assert_eq!(id1 + 1, id2);
    }
}
