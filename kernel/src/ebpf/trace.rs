//! eBPF Tracing Module
//!
//! Support for attaching eBPF programs to kernel events and exporting trace data.
//!
//! Requirement: 10.4 - Probe attachment and tracing
//! Requirement: 10.6 - Trace data export

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use super::Vm;

/// Trace Event Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TraceEventType {
    SyscallEntry = 1,
    SyscallExit = 2,
    ContextSwitch = 3,
    PageFault = 4,
    IrqEntry = 5,
    IrqExit = 6,
    Custom = 100,
}

/// Trace Event Data
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub event_type: TraceEventType,
    pub timestamp: u64,
    pub pid: u64,
    pub cpu_id: u32,
    pub data: [u64; 4], // Arguments or return values
}

/// Circular Buffer for Trace Data Export
pub struct TraceBuffer {
    buffer: Vec<TraceEvent>,
    head: usize,
    capacity: usize,
}

impl TraceBuffer {
    pub const fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::new(),
            head: 0,
            capacity,
        }
    }
    
    pub fn push(&mut self, event: TraceEvent) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(event);
        } else {
            self.buffer[self.head] = event;
            self.head = (self.head + 1) % self.capacity;
        }
    }
    
    pub fn read_all(&self) -> Vec<TraceEvent> {
        // Simple export
        let mut events = Vec::new();
        // If full, start from head
        if self.buffer.len() == self.capacity {
            events.extend_from_slice(&self.buffer[self.head..]);
            events.extend_from_slice(&self.buffer[..self.head]);
        } else {
            events.extend_from_slice(&self.buffer);
        }
        events
    }
}

/// Helper struct for attached probes
pub struct Probe {
    pub prog_id: u32,
    pub enabled: bool,
}

/// Trace Manager
pub struct TraceManager {
    probes: BTreeMap<TraceEventType, Vec<Probe>>,
    buffer: Mutex<TraceBuffer>,
}

impl TraceManager {
    pub const fn new() -> Self {
        Self {
            probes: BTreeMap::new(),
            buffer: Mutex::new(TraceBuffer::new(4096)), // 4K events
        }
    }
    
    pub fn attach_probe(&mut self, event_type: TraceEventType, prog_id: u32) {
        let probes = self.probes.entry(event_type).or_insert(Vec::new());
        probes.push(Probe { prog_id, enabled: true });
        crate::kprintln!("[TRACE] Attached prog {} to event {:?}", prog_id, event_type);
    }
    
    pub fn trigger_event(&self, mut event: TraceEvent) {
        // 1. Log to buffer
        self.buffer.lock().push(event.clone());
        
        // 2. Run attached eBPF programs
        if let Some(probes) = self.probes.get(&event.event_type) {
            for probe in probes {
                if probe.enabled {
                    // In a real implementation, we'd fetch the program bytecode
                    // and run it with the event context.
                    // For now, we simulate execution.
                    // let _ret = vm.execute(program, &mut context);
                }
            }
        }
    }
}

static GLOBAL_TRACE_MANAGER: Mutex<TraceManager> = Mutex::new(TraceManager::new());

/// Public API: Attach probe
pub fn attach(event_type: TraceEventType, prog_id: u32) {
    let mut tm = GLOBAL_TRACE_MANAGER.lock();
    tm.attach_probe(event_type, prog_id);
}

/// Public API: Trigger event (called by kernel hooks)
pub fn trigger(event_type: TraceEventType, pid: u64, data: [u64; 4]) {
    let event = TraceEvent {
        event_type,
        timestamp: crate::log::get_current_time_ms(),
        pid,
        cpu_id: 0, // Should be correct CPU
        data,
    };
    
    // We shouldn't hold the lock for long, but trigger_event does minimal work
    // However, locking global TraceManager might contend. 
    // Usually we'd use Per-CPU buffers.
    // For Phase 1, global lock is acceptable for logic demonstration.
    // But we need to be careful about deadlock if trigger is called from inside a lock held by attach.
    // attach holds lock. trigger holds lock. Safe if not recursive.
    
    // Using a separate lock for buffer vs probes helps?
    // GLOBAL_TRACE_MANAGER protects both.
    
    // Since `trigger` is called from arbitrary contexts, we ideally use lock-free or Per-CPU.
    // For now, spinning mutex is fine for prototype.
    if let Some(mut tm) = GLOBAL_TRACE_MANAGER.try_lock() {
        tm.trigger_event(event);
    } else {
        // If locked (contention or recursion), drop event
        // Trace data loss is acceptable
    }
}
