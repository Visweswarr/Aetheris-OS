use crate::intent::schema::{EvidenceV1, IntentV1};
use serde::{Deserialize, Serialize};
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use alloc::vec;

// Stub blake3 hasher for no_std
struct Hasher {
    data: Vec<u8>,
}

impl Hasher {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn update(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    fn finalize(&self) -> HashOutput {
        // Simple hash stub - XOR-based for determinism
        let mut result = [0u8; 32];
        for (i, byte) in self.data.iter().enumerate() {
            result[i % 32] ^= byte;
        }
        HashOutput(result)
    }
}

struct HashOutput([u8; 32]);

impl From<HashOutput> for [u8; 32] {
    fn from(h: HashOutput) -> [u8; 32] {
        h.0
    }
}

pub const WHYLOG_MAX_ENTRIES: usize = 1000;
pub const WHYLOG_ENTRY_MAX_SIZE: usize = 1024;

// Dilithium signature constants (stubbed for now)
pub const DILITHIUM_SIGNATURE_SIZE: usize = 1024;
pub const DILITHIUM_PUBLIC_KEY_SIZE: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct WhyLogEntry {
    pub ts_vclock: u64,
    pub reason: String,
    pub evidence: Vec<EvidenceV1>,
    pub prev_hash: [u8; 32],
    pub entry_hash: [u8; 32],
    // New fields for CBOR and signatures
    pub seq: u64,
    pub actor: String,
    pub event: String,
    pub data_hash: [u8; 32],
    pub signature: Vec<u8>,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct WhyLogTail {
    pub tail_hash: [u8; 32],
    pub entries_count: usize,
    pub truncated: bool,
    pub last_seq: u64,
    pub chain_verified: bool,
}

pub struct WhyLog {
    ring: VecDeque<WhyLogEntry>,
    head: usize,
    tail: usize,
    tail_hash: [u8; 32],
    max_entries: usize,
    next_seq: u64,
    kernel_signing_key: [u8; 32], // Stubbed kernel key
}

impl WhyLog {
    pub fn new(max_entries: usize) -> Self {
        let mut kernel_key = [0u8; 32];
        kernel_key[0] = 0x1a; // Stubbed key identifier
        kernel_key[1] = 0x2b;
        kernel_key[2] = 0x3c;
        kernel_key[3] = 0x4d;
        
        Self {
            ring: VecDeque::with_capacity(max_entries),
            head: 0,
            tail: 0,
            tail_hash: [0u8; 32],
            max_entries,
            next_seq: 0,
            kernel_signing_key: kernel_key,
        }
    }
    
    pub fn append(&mut self, reason: String, evidence: Vec<EvidenceV1>, vclock: u64) -> [u8; 32] {
        let seq = self.next_seq;
        self.next_seq += 1;
        
        let actor = "kernel".to_string();
        let event = "intent_operation".to_string();
        
        // Compute data hash from reason and evidence
        let mut hasher = Hasher::new();
        hasher.update(reason.as_bytes());
        for ev in &evidence {
            hasher.update(ev.key.as_bytes());
            hasher.update(ev.value.as_bytes());
        }
        let data_hash = hasher.finalize();
        let data_hash_bytes: [u8; 32] = data_hash.into();
        
        // Create entry with previous hash
        let mut entry = WhyLogEntry {
            ts_vclock: vclock,
            reason,
            evidence,
            prev_hash: self.tail_hash,
            entry_hash: [0u8; 32], // Will be computed
            seq,
            actor,
            event,
            data_hash: data_hash_bytes,
            signature: Vec::new(), // Will be computed
            justification: format!("Intent operation at vclock {}", vclock),
        };
        
        // Compute entry hash
        entry.entry_hash = self.compute_entry_hash(&entry);
        
        // Sign the entry (stubbed for now)
        entry.signature = self.sign_entry(&entry);
        
        // Add to ring buffer
        if self.ring.len() >= self.max_entries {
            // Remove oldest entry
            self.ring.pop_front();
            self.head = (self.head + 1) % self.max_entries;
        }
        
        self.ring.push_back(entry.clone());
        self.tail = (self.tail + 1) % self.max_entries;
        
        // Update tail hash
        self.tail_hash = self.compute_chain_hash(&entry);
        
        self.tail_hash
    }
    
    pub fn get_tail(&self) -> WhyLogTail {
        WhyLogTail {
            tail_hash: self.tail_hash,
            entries_count: self.ring.len(),
            truncated: self.ring.len() >= self.max_entries,
            last_seq: if self.next_seq > 0 { self.next_seq - 1 } else { 0 },
            chain_verified: self.verify_chain(),
        }
    }
    
    pub fn get_entries_since(&self, since_hash: [u8; 32]) -> Vec<WhyLogEntry> {
        let mut entries = Vec::new();
        let mut found_start = false;
        
        for entry in &self.ring {
            if entry.entry_hash == since_hash {
                found_start = true;
            }
            if found_start {
                entries.push(entry.clone());
            }
        }
        
        entries
    }

    pub fn get_entries_from_seq(&self, cursor: u64) -> Vec<WhyLogEntry> {
        self.ring
            .iter()
            .filter(|entry| entry.seq >= cursor)
            .cloned()
            .collect()
    }
    
    pub fn verify_chain(&self) -> bool {
        if self.ring.is_empty() {
            return true;
        }
        
        let mut prev_hash = [0u8; 32];
        for entry in &self.ring {
            // Verify entry hash
            if entry.entry_hash != self.compute_entry_hash(entry) {
                return false;
            }
            
            // Verify chain continuity
            if entry.prev_hash != prev_hash {
                return false;
            }
            
            // Verify signature (stubbed for now)
            if !self.verify_signature(entry) {
                return false;
            }
            
            prev_hash = entry.entry_hash;
        }
        
        // Verify final hash matches
        prev_hash == self.tail_hash
    }
    
    pub fn export_digest(&self) -> Vec<u8> {
        let mut digest = Vec::new();
        digest.extend_from_slice(&self.tail_hash);
        digest.extend_from_slice(&(self.ring.len() as u64).to_le_bytes());
        digest.extend_from_slice(&(self.next_seq as u64).to_le_bytes());
        digest
    }
    
    fn compute_entry_hash(&self, entry: &WhyLogEntry) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(&entry.seq.to_le_bytes());
        hasher.update(&entry.ts_vclock.to_le_bytes());
        hasher.update(entry.actor.as_bytes());
        hasher.update(entry.event.as_bytes());
        hasher.update(&entry.data_hash);
        hasher.update(&entry.prev_hash);
        hasher.update(entry.reason.as_bytes());
        
        for ev in &entry.evidence {
            hasher.update(ev.key.as_bytes());
            hasher.update(ev.value.as_bytes());
        }
        
        let result = hasher.finalize();
        result.into()
    }
    
    fn compute_chain_hash(&self, entry: &WhyLogEntry) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(&entry.entry_hash);
        hasher.update(&entry.prev_hash);
        hasher.update(&entry.seq.to_le_bytes());
        hasher.update(&entry.ts_vclock.to_le_bytes());
        
        let result = hasher.finalize();
        result.into()
    }
    
    fn sign_entry(&self, entry: &WhyLogEntry) -> Vec<u8> {
        // Stubbed Dilithium signature
        // In a real implementation, this would use the kernel's signing key
        let mut signature = Vec::with_capacity(DILITHIUM_SIGNATURE_SIZE);
        
        // Create a deterministic signature based on entry content
        let mut hasher = Hasher::new();
        hasher.update(&entry.entry_hash);
        hasher.update(&self.kernel_signing_key);
        hasher.update(&entry.seq.to_le_bytes());
        
        let hash_result = hasher.finalize();
        let hash_bytes: [u8; 32] = hash_result.into();
        
        // Fill signature with deterministic bytes
        signature.extend_from_slice(&hash_bytes);
        signature.extend_from_slice(&[0u8; DILITHIUM_SIGNATURE_SIZE - 32]);
        
        signature
    }
    
    fn verify_signature(&self, entry: &WhyLogEntry) -> bool {
        // Stubbed signature verification
        // In a real implementation, this would verify the Dilithium signature
        if entry.signature.len() != DILITHIUM_SIGNATURE_SIZE {
            return false;
        }
        
        // For now, just verify the signature is deterministic
        let expected_signature = self.sign_entry(entry);
        entry.signature == expected_signature
    }
    
    // Helper methods for common logging patterns
    pub fn log_intent_submitted(&mut self, intent: &IntentV1, vclock: u64) -> [u8; 32] {
        let evidence = vec![
            EvidenceV1 {
                key: "intent_id".to_string(),
                value: intent.id.to_string(),
            },
            EvidenceV1 {
                key: "intent_type".to_string(),
                value: intent.intent_type.to_string(),
            },
            EvidenceV1 {
                key: "priority".to_string(),
                value: intent.priority.to_string(),
            },
        ];
        
        self.append(
            "Intent submitted".to_string(),
            evidence,
            vclock,
        )
    }
    
    pub fn log_constraint_applied(&mut self, constraint: &crate::intent::schema::ConstraintV1, vclock: u64) -> [u8; 32] {
        let evidence = vec![
            EvidenceV1 {
                key: "constraint_kind".to_string(),
                value: constraint.kind.to_string(),
            },
        ];
        
        self.append(
            "Constraint applied".to_string(),
            evidence,
            vclock,
        )
    }
    
    pub fn log_action_selected(&mut self, action: &crate::intent::schema::ActionV1, vclock: u64) -> [u8; 32] {
        let evidence = vec![
            EvidenceV1 {
                key: "action_kind".to_string(),
                value: action.kind.to_string(),
            },
            EvidenceV1 {
                key: "cost_estimate".to_string(),
                value: action.cost_estimate.to_string(),
            },
            EvidenceV1 {
                key: "time_estimate".to_string(),
                value: action.time_estimate.to_string(),
            },
        ];
        
        self.append(
            "Action selected".to_string(),
            evidence,
            vclock,
        )
    }
    
    pub fn log_policy_decision(&mut self, decision: &str, reason: &str, vclock: u64) -> [u8; 32] {
        let evidence = vec![
            EvidenceV1 {
                key: "decision".to_string(),
                value: decision.to_string(),
            },
            EvidenceV1 {
                key: "reason".to_string(),
                value: reason.to_string(),
            },
        ];
        
        self.append(
            "Policy decision".to_string(),
            evidence,
            vclock,
        )
    }
    
    // Utility methods
    pub fn len(&self) -> usize {
        self.ring.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }
    
    pub fn capacity(&self) -> usize {
        self.max_entries
    }
    
    pub fn clear(&mut self) {
        self.ring.clear();
        self.head = 0;
        self.tail = 0;
        self.tail_hash = [0u8; 32];
        self.next_seq = 0;
    }
    
    pub fn get_sequence_number(&self) -> u64 {
        self.next_seq
    }
    
    pub fn get_kernel_key(&self) -> [u8; 32] {
        self.kernel_signing_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::schema::{IntentV1, EvidenceV1};

    #[test]
    fn test_whylog_creation() {
        let whylog = WhyLog::new(100);
        assert_eq!(whylog.len(), 0);
        assert_eq!(whylog.capacity(), 100);
        assert!(whylog.is_empty());
    }

    #[test]
    fn test_whylog_append() {
        let mut whylog = WhyLog::new(5);
        
        let hash1 = whylog.append(
            "First entry".to_string(),
            vec![EvidenceV1::new("key1".to_string(), "value1".to_string())],
            1000,
        );

        let hash2 = whylog.append(
            "Second entry".to_string(),
            vec![EvidenceV1::new("key2".to_string(), "value2".to_string())],
            1001,
        );

        assert_eq!(whylog.len(), 2);
        assert_eq!(whylog.get_tail().entries_count, 2);
        assert_eq!(whylog.get_tail().tail_hash, hash2);
        assert!(!whylog.get_tail().truncated);
    }

    #[test]
    fn test_whylog_ring_buffer() {
        let mut whylog = WhyLog::new(3);
        
        // Add 4 entries to trigger ring buffer behavior
        for i in 0..4 {
            whylog.append(
                format!("Entry {}", i),
                vec![EvidenceV1::new("index".to_string(), i.to_string())],
                1000 + i as u64,
            );
        }

        assert_eq!(whylog.len(), 3); // Should be capped at 3
        assert!(whylog.get_tail().truncated);
        assert_eq!(whylog.get_tail().entries_count, 3);
    }

    #[test]
    fn test_whylog_chain_verification() {
        let mut whylog = WhyLog::new(10);
        
        for i in 0..5 {
            whylog.append(
                format!("Entry {}", i),
                vec![EvidenceV1::new("index".to_string(), i.to_string())],
                1000 + i as u64,
            );
        }

        assert!(whylog.verify_chain());
    }

    #[test]
    fn test_whylog_entries_since() {
        let mut whylog = WhyLog::new(10);
        let mut hashes = Vec::new();
        
        for i in 0..5 {
            let hash = whylog.append(
                format!("Entry {}", i),
                vec![EvidenceV1::new("index".to_string(), i.to_string())],
                1000 + i as u64,
            );
            hashes.push(hash);
        }

        let entries = whylog.get_entries_since(hashes[1]);
        assert_eq!(entries.len(), 3); // Should get entries 2, 3, 4
        assert_eq!(entries[0].reason, "Entry 2");
        assert_eq!(entries[2].reason, "Entry 4");
    }

    #[test]
    fn test_whylog_helper_functions() {
        let mut whylog = WhyLog::new(10);
        let intent = IntentV1::new(123, "test intent".to_string(), 1);
        
        let hash1 = whylog.log_intent_submitted(&intent, 1000);
        let hash2 = whylog.log_constraint_applied(1, "max_cost=100", 1001);
        let hash3 = whylog.log_policy_decision("allow", "capabilities_sufficient", 1002);
        let hash4 = whylog.log_action_selected(1, 50, 1003);

        assert_eq!(whylog.len(), 4);
        assert_eq!(whylog.get_tail().tail_hash, hash4);
    }

    #[test]
    fn test_whylog_export_digest() {
        let mut whylog = WhyLog::new(10);
        
        whylog.append(
            "Test entry".to_string(),
            vec![EvidenceV1::new("test".to_string(), "value".to_string())],
            1000,
        );

        let digest = whylog.export_digest();
        assert_eq!(digest.len(), 32); // Blake3 hash size
    }

    #[test]
    fn test_whylog_clear() {
        let mut whylog = WhyLog::new(10);
        
        whylog.append(
            "Test entry".to_string(),
            vec![EvidenceV1::new("test".to_string(), "value".to_string())],
            1000,
        );

        assert_eq!(whylog.len(), 1);
        whylog.clear();
        assert_eq!(whylog.len(), 0);
        assert!(whylog.is_empty());
    }

    #[test]
    fn test_whylog_default() {
        let whylog = WhyLog::default();
        assert_eq!(whylog.capacity(), WHYLOG_MAX_ENTRIES);
        assert_eq!(whylog.len(), 0);
    }
}
