/// Runqueue Implementation for Polymera OS Scheduler
/// 
/// This module implements a circular buffer-based runqueue for managing
/// ready-to-run tasks in the scheduler.

use super::TaskId;
use core::fmt;

/// Maximum number of tasks that can be queued
pub const RUNQUEUE_CAPACITY: usize = 256;

/// Runqueue structure using a circular buffer
/// 
/// The runqueue maintains a FIFO (First In, First Out) queue of ready tasks
/// using a circular buffer implementation for O(1) enqueue and dequeue operations.
pub struct RunQueue {
    /// Circular buffer to store task IDs
    buf: [Option<TaskId>; RUNQUEUE_CAPACITY],
    
    /// Head index (points to next task to dequeue)
    head: usize,
    
    /// Tail index (points to next slot for enqueue)
    tail: usize,
    
    /// Number of tasks currently in the queue
    count: usize,
}

impl RunQueue {
    /// Create a new empty runqueue
    /// 
    /// # Returns
    /// A new RunQueue instance with no tasks
    pub const fn new() -> Self {
        Self {
            buf: [None; RUNQUEUE_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }
    
    /// Add a task to the back of the runqueue
    /// 
    /// # Arguments
    /// * `task_id` - The TaskId to add to the runqueue
    /// 
    /// # Returns
    /// `true` if the task was successfully added, `false` if the queue is full
    pub fn push(&mut self, task_id: TaskId) -> bool {
        if self.is_full() {
            return false;
        }
        
        self.buf[self.tail] = Some(task_id);
        self.tail = (self.tail + 1) % RUNQUEUE_CAPACITY;
        self.count += 1;
        
        true
    }
    
    /// Remove and return the task from the front of the runqueue
    /// 
    /// # Returns
    /// `Some(TaskId)` if a task was available, `None` if the queue is empty
    pub fn pop(&mut self) -> Option<TaskId> {
        if self.is_empty() {
            return None;
        }
        
        let task_id = self.buf[self.head].take();
        self.head = (self.head + 1) % RUNQUEUE_CAPACITY;
        self.count -= 1;
        
        task_id
    }
    
    /// Peek at the next task without removing it
    /// 
    /// # Returns
    /// `Some(TaskId)` if a task is available, `None` if the queue is empty
    pub fn peek(&self) -> Option<TaskId> {
        if self.is_empty() {
            None
        } else {
            self.buf[self.head]
        }
    }
    
    /// Check if the runqueue is empty
    /// 
    /// # Returns
    /// `true` if no tasks are in the queue, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Check if the runqueue is full
    /// 
    /// # Returns
    /// `true` if the queue cannot accept more tasks, `false` otherwise
    pub fn is_full(&self) -> bool {
        self.count >= RUNQUEUE_CAPACITY
    }
    
    /// Get the number of tasks currently in the runqueue
    /// 
    /// # Returns
    /// The number of tasks in the queue
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Get the maximum capacity of the runqueue
    /// 
    /// # Returns
    /// The maximum number of tasks the queue can hold
    pub const fn capacity() -> usize {
        RUNQUEUE_CAPACITY
    }
    
    /// Clear all tasks from the runqueue
    pub fn clear(&mut self) {
        self.buf = [None; RUNQUEUE_CAPACITY];
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
    
    /// Remove a specific task from the runqueue
    /// 
    /// This is an O(n) operation used for removing tasks that become blocked
    /// or are terminated.
    /// 
    /// # Arguments
    /// * `task_id` - The TaskId to remove from the queue
    /// 
    /// # Returns
    /// `true` if the task was found and removed, `false` otherwise
    pub fn remove(&mut self, task_id: TaskId) -> bool {
        if self.is_empty() {
            return false;
        }
        
        // Search for the task in the queue
        let mut current = self.head;
        for _ in 0..self.count {
            if self.buf[current] == Some(task_id) {
                // Found the task, remove it by shifting elements
                self.remove_at_index(current);
                return true;
            }
            current = (current + 1) % RUNQUEUE_CAPACITY;
        }
        
        false
    }
    
    /// Remove task at a specific index (internal helper)
    fn remove_at_index(&mut self, index: usize) {
        // If removing the head, just advance it
        if index == self.head {
            self.pop();
            return;
        }
        
        // Shift all elements after the removed index toward the head
        let mut current = index;
        loop {
            let next = (current + 1) % RUNQUEUE_CAPACITY;
            if next == self.tail {
                break;
            }
            
            self.buf[current] = self.buf[next];
            current = next;
        }
        
        // Clear the last element and update tail
        self.buf[current] = None;
        self.tail = if self.tail == 0 { 
            RUNQUEUE_CAPACITY - 1 
        } else { 
            self.tail - 1 
        };
        self.count -= 1;
    }
    
    /// Check if a specific task is in the runqueue
    /// 
    /// # Arguments
    /// * `task_id` - The TaskId to search for
    /// 
    /// # Returns
    /// `true` if the task is in the queue, `false` otherwise
    pub fn contains(&self, task_id: TaskId) -> bool {
        if self.is_empty() {
            return false;
        }
        
        let mut current = self.head;
        for _ in 0..self.count {
            if self.buf[current] == Some(task_id) {
                return true;
            }
            current = (current + 1) % RUNQUEUE_CAPACITY;
        }
        
        false
    }
    
    /// Get an iterator over the tasks in the runqueue
    /// 
    /// # Returns
    /// An iterator that yields TaskId values in queue order
    pub fn iter(&self) -> RunQueueIter {
        RunQueueIter {
            queue: self,
            current: self.head,
            remaining: self.count,
        }
    }
    
    /// Get runqueue statistics
    /// 
    /// # Returns
    /// RunQueueStats structure with current queue information
    pub fn stats(&self) -> RunQueueStats {
        RunQueueStats {
            length: self.count,
            capacity: RUNQUEUE_CAPACITY,
            is_empty: self.is_empty(),
            is_full: self.is_full(),
            utilization: (self.count as f32 / RUNQUEUE_CAPACITY as f32) * 100.0,
        }
    }
}

impl fmt::Debug for RunQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunQueue")
            .field("count", &self.count)
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("is_empty", &self.is_empty())
            .field("is_full", &self.is_full())
            .finish()
    }
}

impl fmt::Display for RunQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RunQueue[{}/{}]", self.count, RUNQUEUE_CAPACITY)?;
        
        if !self.is_empty() {
            write!(f, " tasks: [")?;
            let mut first = true;
            for task_id in self.iter() {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", task_id.0)?;
                first = false;
            }
            write!(f, "]")?;
        }
        
        Ok(())
    }
}

/// Iterator over runqueue tasks
pub struct RunQueueIter<'a> {
    queue: &'a RunQueue,
    current: usize,
    remaining: usize,
}

impl<'a> Iterator for RunQueueIter<'a> {
    type Item = TaskId;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        
        let task_id = self.queue.buf[self.current];
        self.current = (self.current + 1) % RUNQUEUE_CAPACITY;
        self.remaining -= 1;
        
        task_id
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for RunQueueIter<'a> {
    fn len(&self) -> usize {
        self.remaining
    }
}

/// Runqueue statistics for monitoring and debugging
#[derive(Debug, Clone, Copy)]
pub struct RunQueueStats {
    /// Current number of tasks in the queue
    pub length: usize,
    
    /// Maximum capacity of the queue
    pub capacity: usize,
    
    /// Whether the queue is empty
    pub is_empty: bool,
    
    /// Whether the queue is full
    pub is_full: bool,
    
    /// Queue utilization as a percentage (0.0 to 100.0)
    pub utilization: f32,
}

impl fmt::Display for RunQueueStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RunQueue Stats: {}/{} tasks ({:.1}% full){}{}",
            self.length,
            self.capacity,
            self.utilization,
            if self.is_empty { " EMPTY" } else { "" },
            if self.is_full { " FULL" } else { "" }
        )
    }
}

/// Priority-based runqueue (future enhancement)
/// 
/// This is a placeholder for a more sophisticated runqueue that can handle
/// multiple priority levels. In Phase 2, this would replace the simple
/// FIFO runqueue for better scheduling performance.
#[allow(dead_code)]
pub struct PriorityRunQueue {
    /// High priority queue
    high: RunQueue,
    
    /// Normal priority queue  
    normal: RunQueue,
    
    /// Low priority queue
    low: RunQueue,
}

#[allow(dead_code)]
impl PriorityRunQueue {
    /// Create a new priority-based runqueue
    pub const fn new() -> Self {
        Self {
            high: RunQueue::new(),
            normal: RunQueue::new(),
            low: RunQueue::new(),
        }
    }
    
    /// Add a task to the appropriate priority queue
    pub fn push(&mut self, task_id: TaskId, priority: crate::sched::task::TaskPriority) -> bool {
        use crate::sched::task::TaskPriority;
        
        match priority {
            TaskPriority::RealTime | TaskPriority::High => self.high.push(task_id),
            TaskPriority::Normal => self.normal.push(task_id),
            TaskPriority::Low => self.low.push(task_id),
        }
    }
    
    /// Get the next task from the highest priority non-empty queue
    pub fn pop(&mut self) -> Option<TaskId> {
        // Try high priority first
        if let Some(task) = self.high.pop() {
            return Some(task);
        }
        
        // Then normal priority
        if let Some(task) = self.normal.pop() {
            return Some(task);
        }
        
        // Finally low priority
        self.low.pop()
    }
    
    /// Check if all queues are empty
    pub fn is_empty(&self) -> bool {
        self.high.is_empty() && self.normal.is_empty() && self.low.is_empty()
    }
    
    /// Get total number of tasks across all priority levels
    pub fn len(&self) -> usize {
        self.high.len() + self.normal.len() + self.low.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runqueue_basic_operations() {
        let mut rq = RunQueue::new();
        
        assert!(rq.is_empty());
        assert!(!rq.is_full());
        assert_eq!(rq.len(), 0);
        
        // Add some tasks
        assert!(rq.push(TaskId(1)));
        assert!(rq.push(TaskId(2)));
        assert!(rq.push(TaskId(3)));
        
        assert!(!rq.is_empty());
        assert_eq!(rq.len(), 3);
        
        // Check peek
        assert_eq!(rq.peek(), Some(TaskId(1)));
        assert_eq!(rq.len(), 3); // Peek shouldn't change length
        
        // Pop tasks in FIFO order
        assert_eq!(rq.pop(), Some(TaskId(1)));
        assert_eq!(rq.pop(), Some(TaskId(2)));
        assert_eq!(rq.pop(), Some(TaskId(3)));
        assert_eq!(rq.pop(), None);
        
        assert!(rq.is_empty());
    }
    
    #[test]
    fn test_runqueue_circular_buffer() {
        let mut rq = RunQueue::new();
        
        // Fill the queue to capacity
        for i in 0..RUNQUEUE_CAPACITY {
            assert!(rq.push(TaskId(i as u64)));
        }
        
        assert!(rq.is_full());
        assert!(!rq.push(TaskId(999))); // Should fail when full
        
        // Remove half the tasks
        for i in 0..RUNQUEUE_CAPACITY / 2 {
            assert_eq!(rq.pop(), Some(TaskId(i as u64)));
        }
        
        // Add new tasks (tests circular buffer wrapping)
        for i in 0..RUNQUEUE_CAPACITY / 2 {
            assert!(rq.push(TaskId((RUNQUEUE_CAPACITY + i) as u64)));
        }
        
        assert!(rq.is_full());
    }
    
    #[test]
    fn test_runqueue_remove() {
        let mut rq = RunQueue::new();
        
        // Add tasks
        rq.push(TaskId(1));
        rq.push(TaskId(2));
        rq.push(TaskId(3));
        
        // Remove middle task
        assert!(rq.remove(TaskId(2)));
        assert_eq!(rq.len(), 2);
        
        // Verify remaining tasks are in correct order
        assert_eq!(rq.pop(), Some(TaskId(1)));
        assert_eq!(rq.pop(), Some(TaskId(3)));
        
        // Try to remove non-existent task
        assert!(!rq.remove(TaskId(999)));
    }
    
    #[test]
    fn test_runqueue_iterator() {
        let mut rq = RunQueue::new();
        
        rq.push(TaskId(1));
        rq.push(TaskId(2));
        rq.push(TaskId(3));
        
        let tasks: Vec<TaskId> = rq.iter().collect();
        assert_eq!(tasks, vec![TaskId(1), TaskId(2), TaskId(3)]);
        
        // Iterator shouldn't modify the queue
        assert_eq!(rq.len(), 3);
    }
}
