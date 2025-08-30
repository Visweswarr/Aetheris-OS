use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use crate::llm::schema::{
    PromptV1, CompletionChunkV1, AdapterConfigV1, RedactionRuleV1,
    QuotaLimitsV1, validate_prompt_size, validate_chunk_size
};
use crate::llm::backend::{LlmBackend, BackendFactory};

/// Chunk sink for streaming completion chunks
pub trait ChunkSink {
    /// Push a completion chunk to the sink
    fn push(&mut self, chunk: CompletionChunkV1) -> Result<(), &'static str>;
    
    /// Get the number of chunks in the sink
    fn len(&self) -> usize;
    
    /// Check if sink is empty
    fn is_empty(&self) -> bool;
}

/// Ring buffer for completion chunks
pub struct ChunkRing {
    /// Buffer capacity
    capacity: usize,
    /// Chunks in the buffer
    chunks: Vec<CompletionChunkV1>,
    /// Head index
    head: usize,
    /// Tail index
    tail: usize,
    /// Number of chunks
    count: usize,
}

impl ChunkRing {
    /// Create a new chunk ring buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            chunks: Vec::with_capacity(capacity),
            head: 0,
            tail: 0,
            count: 0,
        }
    }
    
    /// Push a chunk to the ring buffer
    pub fn push(&mut self, chunk: CompletionChunkV1) -> Result<(), &'static str> {
        if self.count >= self.capacity {
            return Err("Ring buffer full");
        }
        
        // Validate chunk size
        validate_chunk_size(&chunk)?;
        
        if self.chunks.len() < self.capacity {
            self.chunks.push(chunk);
        } else {
            self.chunks[self.tail] = chunk;
        }
        
        self.tail = (self.tail + 1) % self.capacity;
        self.count += 1;
        
        Ok(())
    }
    
    /// Pop a chunk from the ring buffer
    pub fn pop(&mut self) -> Option<CompletionChunkV1> {
        if self.count == 0 {
            return None;
        }
        
        let chunk = self.chunks[self.head].clone();
        self.head = (self.head + 1) % self.capacity;
        self.count -= 1;
        
        Some(chunk)
    }
    
    /// Peek at the next chunk without removing it
    pub fn peek(&self) -> Option<&CompletionChunkV1> {
        if self.count == 0 {
            None
        } else {
            Some(&self.chunks[self.head])
        }
    }
    
    /// Get all chunks in order
    pub fn get_all(&self) -> Vec<CompletionChunkV1> {
        let mut result = Vec::new();
        let mut idx = self.head;
        
        for _ in 0..self.count {
            result.push(self.chunks[idx].clone());
            idx = (idx + 1) % self.capacity;
        }
        
        result
    }
    
    /// Clear the ring buffer
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
    
    /// Get the number of chunks
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.count >= self.capacity
    }
    
    /// Get available space
    pub fn available(&self) -> usize {
        self.capacity - self.count
    }
}

impl ChunkSink for ChunkRing {
    fn push(&mut self, chunk: CompletionChunkV1) -> Result<(), &'static str> {
        self.push(chunk)
    }
    
    fn len(&self) -> usize {
        self.len()
    }
    
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

/// Quota state for tracking usage
pub struct QuotaState {
    /// Tokens per minute limit
    tpm_limit: u32,
    /// Bytes per minute limit
    bpm_limit: u32,
    /// Time slice limit (milliseconds)
    ts_limit_ms: u32,
    
    /// Current tokens this minute
    current_tpm: AtomicU64,
    /// Current bytes this minute
    current_bpm: AtomicU64,
    /// Last reset time (monotonic ticks)
    last_reset: AtomicU64,
    /// Current virtual clock reference
    virtual_clock: AtomicU64,
}

impl QuotaState {
    /// Create a new quota state
    pub fn new(limits: &QuotaLimitsV1) -> Self {
        Self {
            tpm_limit: limits.tpm,
            bpm_limit: limits.bpm,
            ts_limit_ms: limits.ts_ms,
            current_tpm: AtomicU64::new(0),
            current_bpm: AtomicU64::new(0),
            last_reset: AtomicU64::new(0),
            virtual_clock: AtomicU64::new(0),
        }
    }
    
    /// Update virtual clock reference
    pub fn update_clock(&self, clock: u64) {
        self.virtual_clock.store(clock, Ordering::SeqCst);
        
        // Reset counters if a minute has passed
        let last = self.last_reset.load(Ordering::SeqCst);
        if clock >= last + 60_000 { // 60 seconds in milliseconds
            self.current_tpm.store(0, Ordering::SeqCst);
            self.current_bpm.store(0, Ordering::SeqCst);
            self.last_reset.store(clock, Ordering::SeqCst);
        }
    }
    
    /// Check if operation is allowed within quotas
    pub fn check_quotas(&self, estimated_tokens: u32, estimated_bytes: u32) -> Result<(), &'static str> {
        let current_tpm = self.current_tpm.load(Ordering::SeqCst) as u32;
        let current_bpm = self.current_bpm.load(Ordering::SeqCst) as u32;
        
        if current_tpm + estimated_tokens > self.tpm_limit {
            return Err("Token per minute quota exceeded");
        }
        
        if current_bpm + estimated_bytes > self.bpm_limit {
            return Err("Bytes per minute quota exceeded");
        }
        
        Ok(())
    }
    
    /// Consume quotas for an operation
    pub fn consume_quotas(&self, tokens: u32, bytes: u32) {
        self.current_tpm.fetch_add(tokens as u64, Ordering::SeqCst);
        self.current_bpm.fetch_add(bytes as u64, Ordering::SeqCst);
    }
    
    /// Get current usage
    pub fn get_usage(&self) -> (u32, u32) {
        (
            self.current_tpm.load(Ordering::SeqCst) as u32,
            self.current_bpm.load(Ordering::SeqCst) as u32
        )
    }
    
    /// Get quota limits
    pub fn get_limits(&self) -> (u32, u32, u32) {
        (self.tpm_limit, self.bpm_limit, self.ts_limit_ms)
    }
    
    /// Reset quotas (for testing)
    pub fn reset(&self) {
        self.current_tpm.store(0, Ordering::SeqCst);
        self.current_bpm.store(0, Ordering::SeqCst);
        self.last_reset.store(0, Ordering::SeqCst);
    }
}

/// LLM session statistics
#[derive(Debug, Clone, Default)]
pub struct LlmStats {
    /// Total sessions opened
    pub sessions_opened: u64,
    /// Total sessions closed
    pub sessions_closed: u64,
    /// Total prompts sent
    pub prompts_sent: u64,
    /// Total tokens generated
    pub tokens_generated: u64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Quota violations
    pub quota_violations: u64,
    /// Redaction rules applied
    pub redactions_applied: u64,
}

/// LLM session
pub struct LlmSession {
    /// Session ID
    pub id: u64,
    /// Session configuration
    pub config: AdapterConfigV1,
    /// Chunk ring buffer
    pub ring: ChunkRing,
    /// Quota state
    pub quotas: QuotaState,
    /// Session statistics
    pub stats: LlmStats,
    /// Backend instance
    pub backend: Box<dyn LlmBackend>,
    /// Whether session is active
    pub active: bool,
}

impl LlmSession {
    /// Create a new LLM session
    pub fn new(id: u64, config: AdapterConfigV1) -> Result<Self, &'static str> {
        // Validate prompt size
        if let Some(tools) = &config.tools {
            for tool in tools {
                if tool.name.len() > 1024 || tool.description.len() > 4096 {
                    return Err("Tool definition too large");
                }
            }
        }
        
        // Create backend
        let backend = BackendFactory::create_backend(config.backend)?;
        
        // Create ring buffer (8 MiB limit as specified)
        let ring = ChunkRing::new(8 * 1024 * 1024 / 1024); // Approximate chunk count
        
        // Create quota state
        let quotas = QuotaState::new(&config.quotas);
        
        Ok(Self {
            id,
            config,
            ring,
            quotas,
            stats: LlmStats::default(),
            backend,
            active: true,
        })
    }
    
    /// Apply redaction rules to a prompt
    pub fn apply_redactions(&mut self, prompt: &mut PromptV1) -> Result<(), &'static str> {
        if self.config.redactions.is_empty() {
            return Ok(());
        }
        
        // Validate prompt size first
        validate_prompt_size(prompt)?;
        
        // Apply redaction rules
        for rule in &self.config.redactions {
            for message in &mut prompt.messages {
                // Simple pattern matching (in real implementation, use regex)
                if rule.pattern.contains("email") && message.content.contains("@") {
                    message.content = message.content.replace("@", &rule.replacement);
                    self.stats.redactions_applied += 1;
                }
                if rule.pattern.contains("phone") && message.content.contains("+1-") {
                    message.content = message.content.replace("+1-", &rule.replacement);
                    self.stats.redactions_applied += 1;
                }
                if rule.pattern.contains("key") && message.content.contains("sk-") {
                    message.content = message.content.replace("sk-", &rule.replacement);
                    self.stats.redactions_applied += 1;
                }
            }
        }
        
        Ok(())
    }
    
    /// Enforce quotas for an operation
    pub fn enforce_quotas(&mut self, prompt_bytes: usize, estimated_tokens: u32) -> Result<(), &'static str> {
        // Update virtual clock
        let clock = self.get_virtual_clock();
        self.quotas.update_clock(clock);
        
        // Check quotas
        self.quotas.check_quotas(estimated_tokens, prompt_bytes as u32)?;
        
        Ok(())
    }
    
    /// Send a prompt and generate completion
    pub fn send_prompt(&mut self, prompt: &mut PromptV1) -> Result<(), &'static str> {
        if !self.active {
            return Err("Session not active");
        }
        
        // Apply redactions
        self.apply_redactions(prompt)?;
        
        // Estimate tokens (rough approximation: 1 token ≈ 4 characters)
        let estimated_tokens = (prompt.messages.iter()
            .map(|m| m.content.len())
            .sum::<usize>() / 4) as u32;
        
        // Enforce quotas
        let prompt_bytes = prompt.messages.iter()
            .map(|m| m.content.len())
            .sum();
        self.enforce_quotas(prompt_bytes, estimated_tokens)?;
        
        // Send to backend
        let mut sink = &mut self.ring;
        let metrics = self.backend.send(prompt, &mut sink)?;
        
        // Consume quotas
        self.quotas.consume_quotas(metrics.tokens, metrics.bytes);
        
        // Update stats
        self.stats.prompts_sent += 1;
        self.stats.tokens_generated += metrics.tokens as u64;
        self.stats.bytes_processed += metrics.bytes as u64;
        
        Ok(())
    }
    
    /// Receive completion chunks
    pub fn receive_chunks(&mut self, max_chunks: usize) -> Vec<CompletionChunkV1> {
        let mut chunks = Vec::new();
        let mut count = 0;
        
        while count < max_chunks {
            if let Some(chunk) = self.ring.pop() {
                chunks.push(chunk);
                count += 1;
            } else {
                break;
            }
        }
        
        chunks
    }
    
    /// Close the session
    pub fn close(&mut self) {
        self.active = false;
        self.ring.clear();
        self.stats.sessions_closed += 1;
    }
    
    /// Get virtual clock (monotonic ticks)
    fn get_virtual_clock(&self) -> u64 {
        // In a real implementation, this would get the kernel's virtual clock
        // For now, we'll use a simple counter
        self.quotas.virtual_clock.load(Ordering::SeqCst)
    }
    
    /// Get session info
    pub fn get_info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id,
            backend_type: self.backend.get_backend_type(),
            active: self.active,
            chunks_available: self.ring.len(),
            ring_capacity: self.ring.capacity,
            quotas: self.quotas.get_limits(),
            usage: self.quotas.get_usage(),
        }
    }
}

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session ID
    pub id: u64,
    /// Backend type
    pub backend_type: u8,
    /// Whether session is active
    pub active: bool,
    /// Number of chunks available
    pub chunks_available: usize,
    /// Ring buffer capacity
    pub ring_capacity: usize,
    /// Quota limits
    pub quotas: (u32, u32, u32),
    /// Current usage
    pub usage: (u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::schema::{
        PromptV1, MessageV1, CompletionChunkV1, AdapterConfigV1, QuotaLimitsV1,
        RedactionRuleV1, ROLE_USER, BACKEND_NULL
    };

    #[test]
    fn test_chunk_ring_creation() {
        let ring = ChunkRing::new(10);
        assert_eq!(ring.capacity, 10);
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
    }

    #[test]
    fn test_chunk_ring_push_pop() {
        let mut ring = ChunkRing::new(3);
        
        let chunk1 = CompletionChunkV1 {
            seq: 1,
            token: Some("Hello".to_string()),
            tool: None,
            finish: None,
        };
        
        let chunk2 = CompletionChunkV1 {
            seq: 2,
            token: Some("world".to_string()),
            tool: None,
            finish: None,
        };
        
        ring.push(chunk1.clone()).unwrap();
        ring.push(chunk2.clone()).unwrap();
        
        assert_eq!(ring.len(), 2);
        assert!(!ring.is_empty());
        
        let popped1 = ring.pop().unwrap();
        assert_eq!(popped1, chunk1);
        
        let popped2 = ring.pop().unwrap();
        assert_eq!(popped2, chunk2);
        
        assert!(ring.is_empty());
    }

    #[test]
    fn test_chunk_ring_full() {
        let mut ring = ChunkRing::new(2);
        
        let chunk = CompletionChunkV1 {
            seq: 1,
            token: Some("Hello".to_string()),
            tool: None,
            finish: None,
        };
        
        ring.push(chunk.clone()).unwrap();
        ring.push(chunk.clone()).unwrap();
        
        // Third push should fail
        let result = ring.push(chunk);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Ring buffer full");
    }

    #[test]
    fn test_quota_state_creation() {
        let limits = QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        };
        
        let quotas = QuotaState::new(&limits);
        let (tpm, bpm, ts_ms) = quotas.get_limits();
        
        assert_eq!(tpm, 1000);
        assert_eq!(bpm, 10000);
        assert_eq!(ts_ms, 2000);
    }

    #[test]
    fn test_quota_checking() {
        let limits = QuotaLimitsV1 {
            tpm: 100,
            bpm: 1000,
            ts_ms: 2000,
        };
        
        let quotas = QuotaState::new(&limits);
        
        // Should pass
        assert!(quotas.check_quotas(50, 500).is_ok());
        
        // Should fail
        assert!(quotas.check_quotas(150, 500).is_err());
        assert!(quotas.check_quotas(50, 1500).is_err());
    }

    #[test]
    fn test_session_creation() {
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![],
        };
        
        let session = LlmSession::new(1, config).unwrap();
        assert_eq!(session.id, 1);
        assert!(session.active);
        assert_eq!(session.ring.capacity, 8 * 1024 * 1024 / 1024);
    }

    #[test]
    fn test_session_redaction() {
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![
                RedactionRuleV1 {
                    name: "email".to_string(),
                    pattern: "email".to_string(),
                    replacement: "[EMAIL]".to_string(),
                },
            ],
        };
        
        let mut session = LlmSession::new(1, config).unwrap();
        
        let mut prompt = PromptV1 {
            messages: vec![
                MessageV1 { 
                    role: ROLE_USER, 
                    content: "My email is user@example.com".to_string() 
                },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };
        
        session.apply_redactions(&mut prompt).unwrap();
        
        assert!(prompt.messages[0].content.contains("[EMAIL]"));
        assert!(!prompt.messages[0].content.contains("@"));
        assert_eq!(session.stats.redactions_applied, 1);
    }

    #[test]
    fn test_session_info() {
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![],
        };
        
        let session = LlmSession::new(1, config).unwrap();
        let info = session.get_info();
        
        assert_eq!(info.id, 1);
        assert_eq!(info.backend_type, BACKEND_NULL);
        assert!(info.active);
        assert_eq!(info.chunks_available, 0);
    }
}
