use crate::intent::planner::{Planner, EchoPlanner};
use crate::intent::schema::{IntentV1, PlanPreviewV1, PreviewHandle, IntentStatusV1, WhyRecordV1};
use crate::intent::whylog::WhyLog;
use serde_cbor;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod schema;
pub mod whylog;
pub mod planner;

#[derive(Debug)]
pub struct IntentCounters {
    pub intent_submit_ok: u64,
    pub preview_ok: u64,
    pub schema_fail: u64,
    pub policy_deny_sim: u64,
    pub intent_cancelled: u64,
    pub intent_status_queries: u64,
    pub whylog_streams: u64,
}

impl Default for IntentCounters {
    fn default() -> Self {
        Self {
            intent_submit_ok: 0,
            preview_ok: 0,
            schema_fail: 0,
            policy_deny_sim: 0,
            intent_cancelled: 0,
            intent_status_queries: 0,
            whylog_streams: 0,
        }
    }
}

pub struct IntentKernel {
    planner: Box<dyn Planner>,
    why_log: Mutex<WhyLog>,
    counters: Mutex<IntentCounters>,
    intents: Mutex<HashMap<u128, IntentV1>>,
    intent_states: Mutex<HashMap<u128, u8>>, // Intent state tracking
    virtual_clock: Mutex<u64>,
}

impl IntentKernel {
    pub fn new() -> Self {
        Self {
            planner: Box::new(EchoPlanner::new()),
            why_log: Mutex::new(WhyLog::new(1000)),
            counters: Mutex::new(IntentCounters::default()),
            intents: Mutex::new(HashMap::new()),
            intent_states: Mutex::new(HashMap::new()),
            virtual_clock: Mutex::new(0),
        }
    }
    
    pub fn with_planner(mut self, planner: Box<dyn Planner>) -> Self {
        self.planner = planner;
        self
    }
    
    pub fn submit(&self, bytes: &[u8]) -> Result<PreviewHandle, Errno> {
        // Parse CBOR intent
        let intent = match IntentV1::from_cbor(bytes) {
            Ok(intent) => intent,
            Err(_) => {
                self.increment_counter(|c| c.schema_fail += 1);
                return Err(Errno::EINVAL);
            }
        };
        
        // Validate intent
        if !self.validate_intent(&intent) {
            self.increment_counter(|c| c.schema_fail += 1);
            return Err(Errno::EINVAL);
        }
        
        // Check policy
        if !self.check_policy_sim(&intent) {
            self.increment_counter(|c| c.policy_deny_sim += 1);
            return Err(Errno::EPERM);
        }
        
        // Store intent
        {
            let mut intents = self.intents.lock().unwrap();
            intents.insert(intent.id, intent.clone());
        }
        
        // Set initial state
        {
            let mut states = self.intent_states.lock().unwrap();
            states.insert(intent.id, crate::intent::schema::INTENT_STATE_SUBMITTED);
        }
        
        // Generate preview
        let preview = self.generate_preview(&intent)?;
        
        // Log to why-log
        {
            let mut why_log = self.why_log.lock().unwrap();
            let vclock = self.get_virtual_clock();
            why_log.log_intent_submitted(&intent, vclock);
        }
        
        // Update counters
        self.increment_counter(|c| c.intent_submit_ok += 1);
        
        // Create preview handle
        let handle = PreviewHandle {
            intent_id: intent.id,
            preview_hash: self.compute_preview_hash(&preview),
            timestamp: self.get_virtual_clock(),
        };
        
        Ok(handle)
    }
    
    pub fn preview(&self, bytes: &[u8]) -> Result<PlanPreviewV1, Errno> {
        // Parse CBOR intent
        let intent = match IntentV1::from_cbor(bytes) {
            Ok(intent) => intent,
            Err(_) => {
                self.increment_counter(|c| c.schema_fail += 1);
                return Err(Errno::EINVAL);
            }
        };
        
        // Validate intent
        if !self.validate_intent(&intent) {
            self.increment_counter(|c| c.schema_fail += 1);
            return Err(Errno::EINVAL);
        }
        
        // Generate preview
        let preview = self.generate_preview(&intent)?;
        
        // Update counters
        self.increment_counter(|c| c.preview_ok += 1);
        
        Ok(preview)
    }
    
    pub fn get_intent_status(&self, intent_id: u128) -> Result<IntentStatusV1, Errno> {
        // Get intent
        let intent = {
            let intents = self.intents.lock().unwrap();
            intents.get(&intent_id).cloned()
        };
        
        if intent.is_none() {
            return Err(Errno::ESRCH);
        }
        
        let intent = intent.unwrap();
        
        // Get state
        let state = {
            let states = self.intent_states.lock().unwrap();
            *states.get(&intent_id).unwrap_or(&crate::intent::schema::INTENT_STATE_SUBMITTED)
        };
        
        // Get why-log tail hash
        let why_log_tail = {
            let why_log = self.why_log.lock().unwrap();
            why_log.get_tail()
        };
        
        // Update counters
        self.increment_counter(|c| c.intent_status_queries += 1);
        
        Ok(IntentStatusV1 {
            intent_id,
            state,
            last_why_log_hash: why_log_tail.tail_hash,
            progress: 0, // Not implemented in v0
            last_update: self.get_virtual_clock(),
            error_code: 0,
        })
    }
    
    pub fn cancel_intent(&self, intent_id: u128) -> Result<u8, Errno> {
        // Check if intent exists
        let intent = {
            let intents = self.intents.lock().unwrap();
            intents.get(&intent_id).cloned()
        };
        
        if intent.is_none() {
            return Err(Errno::ESRCH);
        }
        
        // Check current state
        let current_state = {
            let states = self.intent_states.lock().unwrap();
            *states.get(&intent_id).unwrap_or(&crate::intent::schema::INTENT_STATE_SUBMITTED)
        };
        
        // Can only cancel if not running or completed
        if current_state == crate::intent::schema::INTENT_STATE_RUNNING ||
           current_state == crate::intent::schema::INTENT_STATE_COMPLETED {
            return Err(Errno::EBUSY);
        }
        
        // Update state to cancelled
        {
            let mut states = self.intent_states.lock().unwrap();
            states.insert(intent_id, crate::intent::schema::INTENT_STATE_CANCELLED);
        }
        
        // Log to why-log
        {
            let mut why_log = self.why_log.lock().unwrap();
            let vclock = self.get_virtual_clock();
            why_log.log_policy_decision("cancelled", "User requested cancellation", vclock);
        }
        
        // Update counters
        self.increment_counter(|c| c.intent_cancelled += 1);
        
        Ok(crate::intent::schema::INTENT_STATE_CANCELLED)
    }
    
    pub fn get_whylog_stream(&self, cursor: u64, max_records: u32) -> Result<Vec<u8>, Errno> {
        let why_log = self.why_log.lock().unwrap();
        
        // Get entries since cursor
        let entries = if cursor == 0 {
            // Get all entries
            why_log.get_entries_since([0u8; 32])
        } else {
            // Find entry with sequence number >= cursor
            let mut found_entries = Vec::new();
            for entry in &why_log.ring {
                if entry.seq >= cursor {
                    found_entries.push(entry.clone());
                }
            }
            found_entries
        };
        
        // Limit to max_records
        let limited_entries = if entries.len() > max_records as usize {
            entries[..max_records as usize].to_vec()
        } else {
            entries
        };
        
        // Convert to CBOR stream
        let stream_data = match serde_cbor::to_vec(&limited_entries) {
            Ok(data) => data,
            Err(_) => return Err(Errno::ENOMEM),
        };
        
        // Update counters
        self.increment_counter(|c| c.whylog_streams += 1);
        
        Ok(stream_data)
    }
    
    pub fn get_whylog_tail(&self) -> crate::intent::whylog::WhyLogTail {
        let why_log = self.why_log.lock().unwrap();
        why_log.get_tail()
    }
    
    pub fn get_counters(&self) -> IntentCounters {
        let counters = self.counters.lock().unwrap();
        counters.clone()
    }
    
    pub fn reset_counters(&self) {
        let mut counters = self.counters.lock().unwrap();
        *counters = IntentCounters::default();
    }
    
    pub fn get_intent(&self, id: u128) -> Option<IntentV1> {
        let intents = self.intents.lock().unwrap();
        intents.get(&id).cloned()
    }
    
    pub fn list_intents(&self) -> Vec<u128> {
        let intents = self.intents.lock().unwrap();
        intents.keys().cloned().collect()
    }
    
    pub fn clear_intents(&self) {
        let mut intents = self.intents.lock().unwrap();
        let mut states = self.intent_states.lock().unwrap();
        intents.clear();
        states.clear();
    }
    
    fn generate_preview(&self, intent: &IntentV1) -> Result<PlanPreviewV1, Errno> {
        let vclock = self.get_virtual_clock();
        
        // Get available capabilities (stubbed for now)
        let available_caps = vec![1u64, 2u64, 3u64]; // Example caps
        
        // Generate preview using planner
        let mut why_log = self.why_log.lock().unwrap();
        let preview = self.planner.preview(intent, &available_caps, &mut why_log, vclock);
        
        Ok(preview)
    }
    
    fn check_policy_sim(&self, intent: &IntentV1) -> bool {
        // Stubbed policy simulation
        // In a real implementation, this would call the OPA→WASM policy engine
        
        // Deny reserved intent types
        if intent.intent_type == 0 {
            return false;
        }
        
        // Deny high priorities without proper caps
        if intent.priority > 9 {
            return false;
        }
        
        // Deny very short deadlines
        if intent.deadline_ms < 10 {
            return false;
        }
        
        // Basic capability check (stubbed)
        if intent.requested_caps.is_empty() {
            return false;
        }
        
        true
    }
    
    fn validate_intent(&self, intent: &IntentV1) -> bool {
        // Check schema version
        if intent.version != crate::intent::schema::SCHEMA_VERSION {
            return false;
        }
        
        // Check size limits
        if intent.serialized_size().unwrap_or(0) > crate::intent::schema::INTENT_MAX_SIZE {
            return false;
        }
        
        // Check metadata limits
        if intent.metadata.len() > crate::intent::schema::METADATA_MAX_ENTRIES {
            return false;
        }
        
        // Check constraints limits
        if intent.constraints.len() > crate::intent::schema::CONSTRAINTS_MAX_COUNT {
            return false;
        }
        
        // Check capability limits
        if intent.requested_caps.len() > 16 { // Arbitrary limit
            return false;
        }
        
        true
    }
    
    fn compute_preview_hash(&self, preview: &PlanPreviewV1) -> [u8; 32] {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        hasher.update(&preview.plan.intent_id.to_le_bytes());
        hasher.update(&preview.plan.total_cost.to_le_bytes());
        hasher.update(&preview.plan.total_time.to_le_bytes());
        hasher.update(&preview.confidence.to_le_bytes());
        
        let result = hasher.finalize();
        result.into()
    }
    
    fn get_virtual_clock(&self) -> u64 {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock += 1;
        *clock
    }
    
    pub fn set_virtual_clock(&self, value: u64) {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock = value;
    }
    
    fn increment_counter<F>(&self, f: F)
    where
        F: FnOnce(&mut IntentCounters),
    {
        let mut counters = self.counters.lock().unwrap();
        f(&mut counters);
    }
}

impl Default for IntentKernel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Errno {
    EINVAL = 22,   // Invalid argument
    E2BIG = 7,     // Argument list too long
    EPERM = 1,     // Operation not permitted
    ENOMEM = 12,   // Cannot allocate memory
    EAGAIN = 11,   // Resource temporarily unavailable
    ESRCH = 3,     // No such process
    EBUSY = 16,    // Device or resource busy
}

impl std::fmt::Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errno::EINVAL => write!(f, "Invalid argument"),
            Errno::E2BIG => write!(f, "Argument list too long"),
            Errno::EPERM => write!(f, "Operation not permitted"),
            Errno::ENOMEM => write!(f, "Cannot allocate memory"),
            Errno::EAGAIN => write!(f, "Resource temporarily unavailable"),
            Errno::ESRCH => write!(f, "No such process"),
            Errno::EBUSY => write!(f, "Device or resource busy"),
        }
    }
}

impl std::error::Error for Errno {}
