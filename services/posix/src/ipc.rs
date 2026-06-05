use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueue {
    pub id: String,
    pub name: String,
    pub max_messages: usize,
    pub max_message_size: usize,
    pub messages: VecDeque<IPCMessage>,
    pub capabilities: Vec<String>,
    pub created_at: u64,
    pub owner_cap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCMessage {
    pub id: u64,
    pub sender_cap: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Data,
    Control,
    Signal,
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryRegion {
    pub id: String,
    pub name: String,
    pub size: usize,
    pub permissions: MemoryPermissions,
    pub capabilities: Vec<String>,
    pub owner_cap: String,
    pub mapped_processes: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeEndpoint {
    pub id: String,
    pub pipe_id: String,
    pub endpoint_type: PipeType,
    pub capabilities: Vec<String>,
    pub buffer: VecDeque<u8>,
    pub max_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipeType {
    Read,
    Write,
}

pub struct IPCSubsystem {
    message_queues: Arc<Mutex<HashMap<String, MessageQueue>>>,
    shared_memory: Arc<Mutex<HashMap<String, SharedMemoryRegion>>>,
    pipes: Arc<Mutex<HashMap<String, PipeEndpoint>>>,
    message_counter: Arc<Mutex<u64>>,
    virtual_clock: Arc<Mutex<u64>>,
    audit_log: Arc<Mutex<Vec<String>>>,
}

impl IPCSubsystem {
    pub fn new() -> Self {
        Self {
            message_queues: Arc::new(Mutex::new(HashMap::new())),
            shared_memory: Arc::new(Mutex::new(HashMap::new())),
            pipes: Arc::new(Mutex::new(HashMap::new())),
            message_counter: Arc::new(Mutex::new(1)),
            virtual_clock: Arc::new(Mutex::new(0)),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Message Queue Operations
    pub fn create_message_queue(&self, name: &str, owner_cap: &str, capabilities: Vec<String>) -> Result<String, String> {
        if !capabilities.contains(&"ipc:create".to_string()) {
            return Err("Insufficient IPC creation capability".to_string());
        }

        let queue_id = format!("mq_{}", self.generate_id());
        let queue = MessageQueue {
            id: queue_id.clone(),
            name: name.to_string(),
            max_messages: 100,
            max_message_size: 8192,
            messages: VecDeque::new(),
            capabilities: capabilities.clone(),
            created_at: self.get_virtual_time(),
            owner_cap: owner_cap.to_string(),
        };

        self.message_queues.lock().unwrap().insert(queue_id.clone(), queue);
        self.log_audit(&format!("MQ_CREATED: {} by {}", queue_id, owner_cap));
        
        Ok(queue_id)
    }

    pub fn send_message(&self, queue_id: &str, sender_cap: &str, data: Vec<u8>, message_type: MessageType) -> Result<u64, String> {
        let mut queues = self.message_queues.lock().unwrap();
        let queue = queues.get_mut(queue_id)
            .ok_or_else(|| "Message queue not found".to_string())?;

        // Validate send capability
        if !queue.capabilities.contains(&"ipc:send".to_string()) && queue.owner_cap != sender_cap {
            return Err("Insufficient send capability".to_string());
        }

        // Check queue limits
        if queue.messages.len() >= queue.max_messages {
            return Err("Message queue full".to_string());
        }

        if data.len() > queue.max_message_size {
            return Err("Message too large".to_string());
        }

        let message_id = self.generate_message_id();
        let message = IPCMessage {
            id: message_id,
            sender_cap: sender_cap.to_string(),
            data,
            timestamp: self.get_virtual_time(),
            message_type,
            priority: 0,
        };

        queue.messages.push_back(message);
        self.log_audit(&format!("MSG_SENT: {} -> {} (size: {})", sender_cap, queue_id, queue.messages.back().unwrap().data.len()));

        Ok(message_id)
    }

    pub fn receive_message(&self, queue_id: &str, receiver_cap: &str) -> Result<Option<IPCMessage>, String> {
        let mut queues = self.message_queues.lock().unwrap();
        let queue = queues.get_mut(queue_id)
            .ok_or_else(|| "Message queue not found".to_string())?;

        // Validate receive capability
        if !queue.capabilities.contains(&"ipc:receive".to_string()) && queue.owner_cap != receiver_cap {
            return Err("Insufficient receive capability".to_string());
        }

        if let Some(message) = queue.messages.pop_front() {
            self.log_audit(&format!("MSG_RECEIVED: {} <- {} (id: {})", receiver_cap, queue_id, message.id));
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    // Shared Memory Operations
    pub fn create_shared_memory(&self, name: &str, size: usize, owner_cap: &str, capabilities: Vec<String>) -> Result<String, String> {
        if !capabilities.contains(&"memory:create".to_string()) {
            return Err("Insufficient memory creation capability".to_string());
        }

        if size > 1024 * 1024 * 16 { // 16MB limit
            return Err("Shared memory region too large".to_string());
        }

        let region_id = format!("shm_{}", self.generate_id());
        let region = SharedMemoryRegion {
            id: region_id.clone(),
            name: name.to_string(),
            size,
            permissions: MemoryPermissions {
                read: true,
                write: true,
                execute: false,
            },
            capabilities: capabilities.clone(),
            owner_cap: owner_cap.to_string(),
            mapped_processes: vec![owner_cap.to_string()],
            created_at: self.get_virtual_time(),
        };

        self.shared_memory.lock().unwrap().insert(region_id.clone(), region);
        self.log_audit(&format!("SHM_CREATED: {} by {} (size: {})", region_id, owner_cap, size));

        Ok(region_id)
    }

    pub fn map_shared_memory(&self, region_id: &str, process_cap: &str) -> Result<(), String> {
        let mut regions = self.shared_memory.lock().unwrap();
        let region = regions.get_mut(region_id)
            .ok_or_else(|| "Shared memory region not found".to_string())?;

        // Validate mapping capability
        if !region.capabilities.contains(&"memory:map".to_string()) && region.owner_cap != process_cap {
            return Err("Insufficient mapping capability".to_string());
        }

        if !region.mapped_processes.contains(&process_cap.to_string()) {
            region.mapped_processes.push(process_cap.to_string());
        }

        self.log_audit(&format!("SHM_MAPPED: {} -> {}", region_id, process_cap));
        Ok(())
    }

    pub fn unmap_shared_memory(&self, region_id: &str, process_cap: &str) -> Result<(), String> {
        let mut regions = self.shared_memory.lock().unwrap();
        let region = regions.get_mut(region_id)
            .ok_or_else(|| "Shared memory region not found".to_string())?;

        region.mapped_processes.retain(|cap| cap != process_cap);
        self.log_audit(&format!("SHM_UNMAPPED: {} -> {}", region_id, process_cap));

        Ok(())
    }

    // Pipe Operations
    pub fn create_pipe(&self, name: &str, owner_cap: &str, capabilities: Vec<String>) -> Result<(String, String), String> {
        if !capabilities.contains(&"ipc:pipe".to_string()) {
            return Err("Insufficient pipe capability".to_string());
        }

        let pipe_id = format!("pipe_{}", self.generate_id());
        let read_id = format!("{}_r", pipe_id);
        let write_id = format!("{}_w", pipe_id);

        let read_endpoint = PipeEndpoint {
            id: read_id.clone(),
            pipe_id: pipe_id.clone(),
            endpoint_type: PipeType::Read,
            capabilities: capabilities.clone(),
            buffer: VecDeque::new(),
            max_buffer_size: 65536,
        };

        let write_endpoint = PipeEndpoint {
            id: write_id.clone(),
            pipe_id: pipe_id.clone(),
            endpoint_type: PipeType::Write,
            capabilities: capabilities.clone(),
            buffer: VecDeque::new(),
            max_buffer_size: 65536,
        };

        let mut pipes = self.pipes.lock().unwrap();
        pipes.insert(read_id.clone(), read_endpoint);
        pipes.insert(write_id.clone(), write_endpoint);

        self.log_audit(&format!("PIPE_CREATED: {} by {} (r:{}, w:{})", pipe_id, owner_cap, read_id, write_id));

        Ok((read_id, write_id))
    }

    pub fn write_pipe(&self, write_id: &str, writer_cap: &str, data: &[u8]) -> Result<usize, String> {
        let mut pipes = self.pipes.lock().unwrap();
        let write_endpoint = pipes.get_mut(write_id)
            .ok_or_else(|| "Pipe write endpoint not found".to_string())?;

        if !matches!(write_endpoint.endpoint_type, PipeType::Write) {
            return Err("Invalid pipe endpoint for writing".to_string());
        }

        // Check buffer space
        let available_space = write_endpoint.max_buffer_size - write_endpoint.buffer.len();
        let write_size = data.len().min(available_space);

        for &byte in &data[..write_size] {
            write_endpoint.buffer.push_back(byte);
        }

        // Find corresponding read endpoint and transfer data
        let pipe_id = write_endpoint.pipe_id.clone();
        let read_id = format!("{}_r", pipe_id.trim_end_matches("_w"));
        
        if let Some(read_endpoint) = pipes.get_mut(&read_id) {
            // Transfer data from write buffer to read buffer
            while let Some(byte) = write_endpoint.buffer.pop_front() {
                if read_endpoint.buffer.len() < read_endpoint.max_buffer_size {
                    read_endpoint.buffer.push_back(byte);
                } else {
                    write_endpoint.buffer.push_front(byte);
                    break;
                }
            }
        }

        self.log_audit(&format!("PIPE_WRITE: {} -> {} (size: {})", writer_cap, write_id, write_size));
        Ok(write_size)
    }

    pub fn read_pipe(&self, read_id: &str, reader_cap: &str, buffer: &mut [u8]) -> Result<usize, String> {
        let mut pipes = self.pipes.lock().unwrap();
        let read_endpoint = pipes.get_mut(read_id)
            .ok_or_else(|| "Pipe read endpoint not found".to_string())?;

        if !matches!(read_endpoint.endpoint_type, PipeType::Read) {
            return Err("Invalid pipe endpoint for reading".to_string());
        }

        let mut read_size = 0;
        for i in 0..buffer.len() {
            if let Some(byte) = read_endpoint.buffer.pop_front() {
                buffer[i] = byte;
                read_size += 1;
            } else {
                break;
            }
        }

        self.log_audit(&format!("PIPE_READ: {} <- {} (size: {})", reader_cap, read_id, read_size));
        Ok(read_size)
    }

    // Utility functions
    pub fn list_message_queues(&self, requester_cap: &str) -> Result<Vec<String>, String> {
        let queues = self.message_queues.lock().unwrap();
        let queue_ids: Vec<String> = queues.iter()
            .filter(|(_, queue)| {
                queue.owner_cap == requester_cap || 
                queue.capabilities.contains(&"ipc:list".to_string())
            })
            .map(|(id, _)| id.clone())
            .collect();

        Ok(queue_ids)
    }

    pub fn get_queue_info(&self, queue_id: &str) -> Result<MessageQueue, String> {
        self.message_queues.lock().unwrap()
            .get(queue_id)
            .cloned()
            .ok_or_else(|| "Message queue not found".to_string())
    }

    pub fn cleanup_process_ipc(&self, process_cap: &str) {
        // Clean up message queues owned by the process
        let mut queues = self.message_queues.lock().unwrap();
        let queue_count = queues.len();
        queues.retain(|_, queue| queue.owner_cap != process_cap);

        // Clean up shared memory mappings
        let mut regions = self.shared_memory.lock().unwrap();
        let mut unmapped_count = 0;
        for region in regions.values_mut() {
            let before_count = region.mapped_processes.len();
            region.mapped_processes.retain(|cap| cap != process_cap);
            unmapped_count += before_count - region.mapped_processes.len();
        }
        let region_count = regions.len();
        regions.retain(|_, region| region.owner_cap != process_cap);

        // Clean up pipes
        let mut pipes = self.pipes.lock().unwrap();
        let pipe_count = pipes.len();
        pipes.retain(|_, pipe| !pipe.capabilities.iter().any(|cap| cap.contains(process_cap)));

        self.log_audit(&format!("IPC_CLEANUP: {} (queues: {}, regions: {}, pipes: {}, unmapped: {})", 
            process_cap, queue_count, region_count, pipe_count, unmapped_count));
    }

    pub fn get_ipc_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        let queues = self.message_queues.lock().unwrap();
        let regions = self.shared_memory.lock().unwrap();
        let pipes = self.pipes.lock().unwrap();
        
        stats.insert("message_queues".to_string(), queues.len());
        stats.insert("shared_memory_regions".to_string(), regions.len());
        stats.insert("pipes".to_string(), pipes.len());
        
        // Calculate total messages across all queues
        let total_messages: usize = queues.values().map(|q| q.messages.len()).sum();
        stats.insert("total_messages".to_string(), total_messages);
        
        // Calculate total shared memory size
        let total_shm_size: usize = regions.values().map(|r| r.size).sum();
        stats.insert("total_shm_size".to_string(), total_shm_size);
        
        // Calculate total pipe buffer usage
        let total_pipe_buffers: usize = pipes.values().map(|p| p.buffer.len()).sum();
        stats.insert("total_pipe_buffers".to_string(), total_pipe_buffers);
        
        stats
    }

    pub fn list_ipc_objects(&self, requester_cap: &str) -> Result<Vec<IPCInfo>, String> {
        if !self.check_ipc_list_capability(requester_cap) {
            return Err("Insufficient IPC list capability".to_string());
        }

        let mut objects = Vec::new();
        
        // Add message queues
        let queues = self.message_queues.lock().unwrap();
        for queue in queues.values() {
            objects.push(IPCInfo {
                type: "message_queue".to_string(),
                id: queue.id.clone(),
                name: queue.name.clone(),
                messages: Some(queue.messages.len()),
                attached_processes: None,
            });
        }
        
        // Add shared memory regions
        let regions = self.shared_memory.lock().unwrap();
        for region in regions.values() {
            objects.push(IPCInfo {
                type: "shared_memory".to_string(),
                id: region.id.clone(),
                name: region.name.clone(),
                size: Some(region.size),
                attached_processes: Some(region.mapped_processes.len()),
            });
        }
        
        // Add pipes
        let pipes = self.pipes.lock().unwrap();
        for pipe in pipes.values() {
            objects.push(IPCInfo {
                type: "pipe".to_string(),
                id: pipe.id.clone(),
                name: format!("pipe_{}", pipe.pipe_id),
                size: Some(pipe.buffer.len()),
                attached_processes: None,
            });
        }
        
        Ok(objects)
    }

    pub fn get_message_queue_stats(&self, queue_id: &str) -> Result<HashMap<String, usize>, String> {
        let queues = self.message_queues.lock().unwrap();
        let queue = queues.get(queue_id)
            .ok_or_else(|| "Message queue not found".to_string())?;
        
        let mut stats = HashMap::new();
        stats.insert("message_count".to_string(), queue.messages.len());
        stats.insert("max_messages".to_string(), queue.max_messages);
        stats.insert("max_message_size".to_string(), queue.max_message_size);
        
        Ok(stats)
    }

    pub fn get_shared_memory_stats(&self, region_id: &str) -> Result<HashMap<String, usize>, String> {
        let regions = self.shared_memory.lock().unwrap();
        let region = regions.get(region_id)
            .ok_or_else(|| "Shared memory region not found".to_string())?;
        
        let mut stats = HashMap::new();
        stats.insert("size".to_string(), region.size);
        stats.insert("mapped_processes".to_string(), region.mapped_processes.len());
        
        Ok(stats)
    }

    pub fn get_pipe_stats(&self, pipe_id: &str) -> Result<HashMap<String, usize>, String> {
        let pipes = self.pipes.lock().unwrap();
        let pipe = pipes.get(pipe_id)
            .ok_or_else(|| "Pipe not found".to_string())?;
        
        let mut stats = HashMap::new();
        stats.insert("buffer_size".to_string(), pipe.buffer.len());
        stats.insert("max_buffer_size".to_string(), pipe.max_buffer_size);
        
        Ok(stats)
    }

    pub fn flush_message_queue(&self, queue_id: &str, requester_cap: &str) -> Result<usize, String> {
        let mut queues = self.message_queues.lock().unwrap();
        let queue = queues.get_mut(queue_id)
            .ok_or_else(|| "Message queue not found".to_string())?;

        if queue.owner_cap != requester_cap && !self.check_ipc_admin_capability(requester_cap) {
            return Err("Insufficient capability to flush queue".to_string());
        }

        let message_count = queue.messages.len();
        queue.messages.clear();
        
        self.log_audit(&format!("QUEUE_FLUSHED: {} by {} ({} messages)", 
            queue_id, requester_cap, message_count));
        
        Ok(message_count)
    }

    pub fn resize_shared_memory(&self, region_id: &str, new_size: usize, requester_cap: &str) -> Result<(), String> {
        let mut regions = self.shared_memory.lock().unwrap();
        let region = regions.get_mut(region_id)
            .ok_or_else(|| "Shared memory region not found".to_string())?;

        if region.owner_cap != requester_cap && !self.check_ipc_admin_capability(requester_cap) {
            return Err("Insufficient capability to resize shared memory".to_string());
        }

        if new_size > 1024 * 1024 * 64 { // 64MB limit
            return Err("New size exceeds maximum allowed".to_string());
        }

        let old_size = region.size;
        region.size = new_size;
        
        self.log_audit(&format!("SHM_RESIZED: {} by {} ({} -> {} bytes)", 
            region_id, requester_cap, old_size, new_size));
        
        Ok(())
    }

    fn generate_id(&self) -> u64 {
        let mut counter = self.message_counter.lock().unwrap();
        *counter += 1;
        *counter
    }

    fn generate_message_id(&self) -> u64 {
        self.generate_id()
    }

    fn get_virtual_time(&self) -> u64 {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock += 1;
        *clock
    }

    fn log_audit(&self, message: &str) {
        let mut audit_log = self.audit_log.lock().unwrap();
        audit_log.push(format!("[{}] {}", self.get_virtual_time(), message));
        if audit_log.len() > 1000 {
            audit_log.remove(0);
        }
    }

    fn check_ipc_list_capability(&self, requester_cap: &str) -> bool {
        // In real implementation, would check actual capabilities
        requester_cap.contains("ipc:list") || requester_cap.contains("all")
    }

    fn check_ipc_admin_capability(&self, requester_cap: &str) -> bool {
        // In real implementation, would check actual capabilities
        requester_cap.contains("ipc:admin") || requester_cap.contains("all")
    }

    pub fn get_audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("message_queues".to_string(), self.message_queues.lock().unwrap().len());
        stats.insert("shared_memory_regions".to_string(), self.shared_memory.lock().unwrap().len());
        stats.insert("pipes".to_string(), self.pipes.lock().unwrap().len());
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_subsystem_creation() {
        let ipc = IPCSubsystem::new();
        let stats = ipc.get_stats();
        assert_eq!(stats["message_queues"], 0);
        assert_eq!(stats["shared_memory_regions"], 0);
        assert_eq!(stats["pipes"], 0);
    }

    #[test]
    fn test_message_queue_operations() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["ipc:create".to_string(), "ipc:send".to_string(), "ipc:receive".to_string()];
        
        let queue_id = ipc.create_message_queue("test_queue", "cap_proc_001", capabilities).unwrap();
        
        let message_data = b"Hello, IPC!".to_vec();
        let message_id = ipc.send_message(&queue_id, "cap_proc_001", message_data.clone(), MessageType::Data).unwrap();
        
        let received = ipc.receive_message(&queue_id, "cap_proc_001").unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().data, message_data);
    }

    #[test]
    fn test_shared_memory_operations() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["memory:create".to_string(), "memory:map".to_string()];
        
        let region_id = ipc.create_shared_memory("test_shm", 4096, "cap_proc_001", capabilities).unwrap();
        
        let result = ipc.map_shared_memory(&region_id, "cap_proc_002");
        assert!(result.is_ok());
        
        let result = ipc.unmap_shared_memory(&region_id, "cap_proc_002");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipe_operations() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["ipc:pipe".to_string()];
        
        let (read_id, write_id) = ipc.create_pipe("test_pipe", "cap_proc_001", capabilities).unwrap();
        
        let test_data = b"Hello, pipe!";
        let written = ipc.write_pipe(&write_id, "cap_proc_001", test_data).unwrap();
        assert_eq!(written, test_data.len());
        
        let mut buffer = [0u8; 64];
        let read_size = ipc.read_pipe(&read_id, "cap_proc_001", &mut buffer).unwrap();
        assert_eq!(read_size, test_data.len());
        assert_eq!(&buffer[..read_size], test_data);
    }

    #[test]
    fn test_ipc_statistics() {
        let ipc = IPCSubsystem::new();
        
        // Create some IPC objects
        let capabilities = vec!["ipc:create".to_string()];
        ipc.create_message_queue("test_queue", "cap_proc_001", capabilities.clone()).unwrap();
        ipc.create_shared_memory("test_shm", 4096, "cap_proc_001", capabilities.clone()).unwrap();
        ipc.create_pipe("test_pipe", "cap_proc_001", capabilities).unwrap();
        
        let stats = ipc.get_ipc_statistics();
        assert_eq!(stats["message_queues"], 1);
        assert_eq!(stats["shared_memory_regions"], 1);
        assert_eq!(stats["pipes"], 2); // read and write endpoints
    }

    #[test]
    fn test_list_ipc_objects() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["ipc:create".to_string(), "ipc:list".to_string()];
        
        ipc.create_message_queue("test_queue", "cap_proc_001", capabilities.clone()).unwrap();
        ipc.create_shared_memory("test_shm", 4096, "cap_proc_001", capabilities.clone()).unwrap();
        
        let objects = ipc.list_ipc_objects("cap_proc_001").unwrap();
        assert_eq!(objects.len(), 3); // 1 queue + 1 shm + 0 pipes (not created in this test)
        
        let queue_objects: Vec<_> = objects.iter().filter(|o| o.type == "message_queue").collect();
        assert_eq!(queue_objects.len(), 1);
        
        let shm_objects: Vec<_> = objects.iter().filter(|o| o.type == "shared_memory").collect();
        assert_eq!(shm_objects.len(), 1);
    }

    #[test]
    fn test_message_queue_stats() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["ipc:create".to_string(), "ipc:send".to_string()];
        
        let queue_id = ipc.create_message_queue("test_queue", "cap_proc_001", capabilities.clone()).unwrap();
        ipc.send_message(&queue_id, "cap_proc_001", b"test".to_vec(), MessageType::Data).unwrap();
        
        let stats = ipc.get_message_queue_stats(&queue_id).unwrap();
        assert_eq!(stats["message_count"], 1);
        assert_eq!(stats["max_messages"], 100);
        assert_eq!(stats["max_message_size"], 8192);
    }

    #[test]
    fn test_shared_memory_stats() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["memory:create".to_string(), "memory:map".to_string()];
        
        let region_id = ipc.create_shared_memory("test_shm", 4096, "cap_proc_001", capabilities.clone()).unwrap();
        ipc.map_shared_memory(&region_id, "cap_proc_002").unwrap();
        
        let stats = ipc.get_shared_memory_stats(&region_id).unwrap();
        assert_eq!(stats["size"], 4096);
        assert_eq!(stats["mapped_processes"], 2); // owner + mapped process
    }

    #[test]
    fn test_flush_message_queue() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["ipc:create".to_string(), "ipc:send".to_string()];
        
        let queue_id = ipc.create_message_queue("test_queue", "cap_proc_001", capabilities.clone()).unwrap();
        ipc.send_message(&queue_id, "cap_proc_001", b"test1".to_vec(), MessageType::Data).unwrap();
        ipc.send_message(&queue_id, "cap_proc_001", b"test2".to_vec(), MessageType::Data).unwrap();
        
        let flushed_count = ipc.flush_message_queue(&queue_id, "cap_proc_001").unwrap();
        assert_eq!(flushed_count, 2);
        
        let stats = ipc.get_message_queue_stats(&queue_id).unwrap();
        assert_eq!(stats["message_count"], 0);
    }

    #[test]
    fn test_resize_shared_memory() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["memory:create".to_string()];
        
        let region_id = ipc.create_shared_memory("test_shm", 4096, "cap_proc_001", capabilities).unwrap();
        
        let result = ipc.resize_shared_memory(&region_id, 8192, "cap_proc_001");
        assert!(result.is_ok());
        
        let stats = ipc.get_shared_memory_stats(&region_id).unwrap();
        assert_eq!(stats["size"], 8192);
    }

    #[test]
    fn test_cleanup_process_ipc() {
        let ipc = IPCSubsystem::new();
        let capabilities = vec!["ipc:create".to_string(), "memory:create".to_string()];
        
        // Create IPC objects
        ipc.create_message_queue("test_queue", "cap_proc_001", capabilities.clone()).unwrap();
        ipc.create_shared_memory("test_shm", 4096, "cap_proc_001", capabilities.clone()).unwrap();
        ipc.create_pipe("test_pipe", "cap_proc_001", capabilities).unwrap();
        
        // Clean up
        ipc.cleanup_process_ipc("cap_proc_001");
        
        let stats = ipc.get_ipc_statistics();
        assert_eq!(stats["message_queues"], 0);
        assert_eq!(stats["shared_memory_regions"], 0);
        assert_eq!(stats["pipes"], 0);
    }
}