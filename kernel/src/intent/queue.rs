use crate::intent::wire::IntentLane;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct IntentQueue {
    lane: IntentLane,
    capacity: usize,
    queue: VecDeque<u128>,
    size: AtomicUsize,
}

impl IntentQueue {
    pub fn new(lane: IntentLane) -> Self {
        let capacity = lane.queue_capacity();
        
        Self {
            lane,
            capacity,
            queue: VecDeque::with_capacity(capacity),
            size: AtomicUsize::new(0),
        }
    }
    
    pub fn enqueue(&mut self, intent_id: u128) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("Queue is full");
        }
        
        self.queue.push_back(intent_id);
        self.size.fetch_add(1, Ordering::SeqCst);
        
        Ok(())
    }
    
    pub fn dequeue(&mut self) -> Option<u128> {
        let intent_id = self.queue.pop_front()?;
        self.size.fetch_sub(1, Ordering::SeqCst);
        Some(intent_id)
    }
    
    pub fn peek(&self) -> Option<&u128> {
        self.queue.front()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.capacity
    }
    
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn clear(&mut self) {
        self.queue.clear();
        self.size.store(0, Ordering::SeqCst);
    }
    
    pub fn contains(&self, intent_id: u128) -> bool {
        self.queue.iter().any(|&id| id == intent_id)
    }
    
    pub fn remove(&mut self, intent_id: u128) -> bool {
        if let Some(pos) = self.queue.iter().position(|&id| id == intent_id) {
            self.queue.remove(pos);
            self.size.fetch_sub(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
    
    pub fn get_lane(&self) -> IntentLane {
        self.lane
    }
    
    pub fn get_utilization(&self) -> f64 {
        self.len() as f64 / self.capacity as f64
    }
    
    pub fn get_wait_time_estimate(&self) -> u64 {
        match self.lane {
            IntentLane::RT => 0, // RT lane processes immediately
            IntentLane::HIGH => self.len() as u64 * 100, // 100μs per intent
            IntentLane::BEST => self.len() as u64 * 1000, // 1ms per intent
        }
    }
}

impl Default for IntentQueue {
    fn default() -> Self {
        Self::new(IntentLane::BEST)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_queue_creation() {
        let queue = IntentQueue::new(IntentLane::RT);
        assert_eq!(queue.lane, IntentLane::RT);
        assert_eq!(queue.capacity(), IntentLane::RT.queue_capacity());
        assert!(queue.is_empty());
        assert!(!queue.is_full());
    }
    
    #[test]
    fn test_queue_enqueue_dequeue() {
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        
        assert!(queue.enqueue(1).is_ok());
        assert!(queue.enqueue(2).is_ok());
        assert_eq!(queue.len(), 2);
        
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), None);
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_queue_capacity() {
        let mut queue = IntentQueue::new(IntentLane::BEST);
        let capacity = queue.capacity();
        
        for i in 0..capacity {
            assert!(queue.enqueue(i as u128).is_ok());
        }
        
        assert!(queue.is_full());
        assert!(queue.enqueue(capacity as u128).is_err());
    }
    
    #[test]
    fn test_queue_peek() {
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        
        assert_eq!(queue.peek(), None);
        
        queue.enqueue(42).unwrap();
        assert_eq!(queue.peek(), Some(&42));
        assert_eq!(queue.len(), 1);
    }
    
    #[test]
    fn test_queue_contains() {
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        
        queue.enqueue(1).unwrap();
        queue.enqueue(2).unwrap();
        
        assert!(queue.contains(1));
        assert!(queue.contains(2));
        assert!(!queue.contains(3));
    }
    
    #[test]
    fn test_queue_remove() {
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        
        queue.enqueue(1).unwrap();
        queue.enqueue(2).unwrap();
        queue.enqueue(3).unwrap();
        
        assert!(queue.remove(2));
        assert_eq!(queue.len(), 2);
        assert!(!queue.contains(2));
        
        assert!(!queue.remove(4));
        assert_eq!(queue.len(), 2);
    }
    
    #[test]
    fn test_queue_clear() {
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        
        queue.enqueue(1).unwrap();
        queue.enqueue(2).unwrap();
        assert_eq!(queue.len(), 2);
        
        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }
    
    #[test]
    fn test_queue_utilization() {
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        
        assert_eq!(queue.get_utilization(), 0.0);
        
        queue.enqueue(1).unwrap();
        assert_eq!(queue.get_utilization(), 1.0 / queue.capacity() as f64);
        
        queue.enqueue(2).unwrap();
        assert_eq!(queue.get_utilization(), 2.0 / queue.capacity() as f64);
    }
    
    #[test]
    fn test_queue_wait_time_estimate() {
        let mut queue = IntentQueue::new(IntentLane::RT);
        assert_eq!(queue.get_wait_time_estimate(), 0);
        
        let mut queue = IntentQueue::new(IntentLane::HIGH);
        queue.enqueue(1).unwrap();
        queue.enqueue(2).unwrap();
        assert_eq!(queue.get_wait_time_estimate(), 200);
        
        let mut queue = IntentQueue::new(IntentLane::BEST);
        queue.enqueue(1).unwrap();
        queue.enqueue(2).unwrap();
        assert_eq!(queue.get_wait_time_estimate(), 2000);
    }
}
