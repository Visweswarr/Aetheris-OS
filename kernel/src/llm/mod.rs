use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::llm::schema::{
    PromptV1, CompletionChunkV1, AdapterConfigV1, SessionHandle,
    serialize_prompt, deserialize_prompt, serialize_chunk, deserialize_chunk,
    serialize_config, deserialize_config
};
use crate::llm::session::{LlmSession, SessionInfo, LlmStats};
use crate::llm::backend::{BackendFactory, BackendRegistry};

/// LLM service for managing sessions and operations
pub struct LlmService {
    /// Next session ID
    next_session_id: AtomicU64,
    /// Active sessions
    sessions: Mutex<BTreeMap<u64, LlmSession>>,
    /// Backend registry
    backend_registry: BackendRegistry,
    /// Service statistics
    stats: Mutex<LlmStats>,
}

impl LlmService {
    /// Create a new LLM service
    pub fn new() -> Self {
        Self {
            next_session_id: AtomicU64::new(1),
            sessions: Mutex::new(BTreeMap::new()),
            backend_registry: BackendRegistry::new(),
            stats: Mutex::new(LlmStats::default()),
        }
    }
    
    /// Open a new LLM session
    pub fn open_session(&self, config: AdapterConfigV1) -> Result<SessionHandle, &'static str> {
        // Check if backend is supported
        if !BackendFactory::is_backend_supported(config.backend) {
            return Err("Unsupported backend type");
        }
        
        // Check if backend is available
        if !self.backend_registry.is_backend_available(config.backend) {
            return Err("Backend not available");
        }
        
        // Generate session ID
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        
        // Create session
        let session = LlmSession::new(session_id, config)?;
        
        // Store session
        {
            let mut sessions = self.sessions.lock();
            sessions.insert(session_id, session);
        }
        
        // Update stats
        {
            let mut stats = self.stats.lock();
            stats.sessions_opened += 1;
        }
        
        // Publish event
        self.publish_session_event("llm.session.open", session_id, &config)?;
        
        Ok(SessionHandle(session_id))
    }
    
    /// Send a prompt to a session
    pub fn send_prompt(&self, session_handle: SessionHandle, prompt: &mut PromptV1) -> Result<(), &'static str> {
        let session_id = session_handle.0;
        
        // Get session
        let mut sessions = self.sessions.lock();
        let session = sessions.get_mut(&session_id)
            .ok_or("Session not found")?;
        
        if !session.active {
            return Err("Session not active");
        }
        
        // Send prompt
        session.send_prompt(prompt)?;
        
        // Publish token events
        self.publish_token_events(session_id, &session.ring.get_all())?;
        
        Ok(())
    }
    
    /// Receive completion chunks from a session
    pub fn receive_chunks(&self, session_handle: SessionHandle, max_chunks: usize) -> Result<Vec<CompletionChunkV1>, &'static str> {
        let session_id = session_handle.0;
        
        // Get session
        let mut sessions = self.sessions.lock();
        let session = sessions.get_mut(&session_id)
            .ok_or("Session not found")?;
        
        if !session.active {
            return Err("Session not active");
        }
        
        // Receive chunks
        let chunks = session.receive_chunks(max_chunks);
        
        // Publish finish event if we have a finish chunk
        if let Some(last_chunk) = chunks.last() {
            if last_chunk.finish.is_some() {
                self.publish_finish_event(session_id, last_chunk)?;
            }
        }
        
        Ok(chunks)
    }
    
    /// Close a session
    pub fn close_session(&self, session_handle: SessionHandle) -> Result<(), &'static str> {
        let session_id = session_handle.0;
        
        // Get session
        let mut sessions = self.sessions.lock();
        let session = sessions.get_mut(&session_id)
            .ok_or("Session not found")?;
        
        if !session.active {
            return Err("Session already closed");
        }
        
        // Close session
        session.close();
        
        // Remove from active sessions
        sessions.remove(&session_id);
        
        // Update stats
        {
            let mut stats = self.stats.lock();
            stats.sessions_closed += 1;
        }
        
        // Publish event
        self.publish_session_event("llm.session.close", session_id, &String::new())?;
        
        Ok(())
    }
    
    /// Get session information
    pub fn get_session_info(&self, session_handle: SessionHandle) -> Result<SessionInfo, &'static str> {
        let session_id = session_handle.0;
        
        // Get session
        let sessions = self.sessions.lock();
        let session = sessions.get(&session_id)
            .ok_or("Session not found")?;
        
        Ok(session.get_info())
    }
    
    /// Get service statistics
    pub fn get_stats(&self) -> LlmStats {
        let stats = self.stats.lock();
        stats.clone()
    }
    
    /// Check if service is available
    pub fn is_available(&self) -> bool {
        !self.backend_registry.get_available_backends().is_empty()
    }
    
    /// Get available backend types
    pub fn get_available_backends(&self) -> Vec<u8> {
        self.backend_registry.get_available_backends()
    }
    
    /// Publish session event to Event Fabric
    fn publish_session_event(&self, topic: &str, session_id: u64, payload: &str) -> Result<(), &'static str> {
        // In a real implementation, this would publish to Event Fabric
        // For now, we'll just log it
        crate::logging::info!("LLM Event: {} session={} payload={}", topic, session_id, payload);
        Ok(())
    }
    
    /// Publish token events to Event Fabric
    fn publish_token_events(&self, session_id: u64, chunks: &[CompletionChunkV1]) -> Result<(), &'static str> {
        for chunk in chunks {
            if let Some(token) = &chunk.token {
                let payload = format!("{{\"sid\":{},\"seq\":{},\"text\":\"{}\"}}", 
                    session_id, chunk.seq, token);
                self.publish_session_event("llm.token", session_id, &payload)?;
            }
            
            if let Some(tool) = &chunk.tool {
                let payload = format!("{{\"sid\":{},\"seq\":{},\"tool\":\"{}\"}}", 
                    session_id, chunk.seq, tool.name);
                self.publish_session_event("llm.toolcall", session_id, &payload)?;
            }
        }
        
        Ok(())
    }
    
    /// Publish finish event to Event Fabric
    fn publish_finish_event(&self, session_id: u64, chunk: &CompletionChunkV1) -> Result<(), &'static str> {
        if let Some(finish_reason) = chunk.finish {
            let payload = format!("{{\"sid\":{},\"seq\":{},\"finish\":{}}}", 
                session_id, chunk.seq, finish_reason);
            self.publish_session_event("llm.finish", session_id, &payload)?;
        }
        
        Ok(())
    }
}

/// Global LLM service instance
static LLM_SERVICE: Mutex<Option<LlmService>> = Mutex::new(None);

/// Initialize the LLM service
pub fn init_llm_service() -> Result<(), &'static str> {
    let mut service_guard = LLM_SERVICE.lock();
    if service_guard.is_some() {
        return Err("LLM service already initialized");
    }
    
    let service = LlmService::new();
    *service_guard = Some(service);
    
    crate::logging::info!("LLM Adapter v0 service initialized");
    Ok(())
}

/// Get the LLM service instance
pub fn get_llm_service() -> Result<&'static LlmService, &'static str> {
    let service_guard = LLM_SERVICE.lock();
    service_guard.as_ref().ok_or("LLM service not initialized")
}

/// Check if LLM service is available
pub fn is_llm_available() -> bool {
    if let Ok(service) = get_llm_service() {
        service.is_available()
    } else {
        false
    }
}

/// Open a new LLM session
pub fn open_llm_session(config: AdapterConfigV1) -> Result<SessionHandle, &'static str> {
    let service = get_llm_service()?;
    service.open_session(config)
}

/// Send a prompt to an LLM session
pub fn send_llm_prompt(session_handle: SessionHandle, prompt: &mut PromptV1) -> Result<(), &'static str> {
    let service = get_llm_service()?;
    service.send_prompt(session_handle, prompt)
}

/// Receive completion chunks from an LLM session
pub fn receive_llm_chunks(session_handle: SessionHandle, max_chunks: usize) -> Result<Vec<CompletionChunkV1>, &'static str> {
    let service = get_llm_service()?;
    service.receive_chunks(session_handle, max_chunks)
}

/// Close an LLM session
pub fn close_llm_session(session_handle: SessionHandle) -> Result<(), &'static str> {
    let service = get_llm_service()?;
    service.close_session(session_handle)
}

/// Get LLM session information
pub fn get_llm_session_info(session_handle: SessionHandle) -> Result<SessionInfo, &'static str> {
    let service = get_llm_service()?;
    service.get_session_info(session_handle)
}

/// Get LLM service statistics
pub fn get_llm_stats() -> Result<LlmStats, &'static str> {
    let service = get_llm_service()?;
    Ok(service.get_stats())
}

/// Get available LLM backend types
pub fn get_available_llm_backends() -> Result<Vec<u8>, &'static str> {
    let service = get_llm_service()?;
    Ok(service.get_available_backends())
}

/// LLM service configuration
pub struct LlmServiceConfig {
    /// Maximum sessions allowed
    pub max_sessions: usize,
    /// Default backend type
    pub default_backend: u8,
    /// Enable development backends
    pub enable_dev_backends: bool,
}

impl Default for LlmServiceConfig {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            default_backend: crate::llm::schema::BACKEND_NULL,
            enable_dev_backends: false,
        }
    }
}

/// Initialize LLM service with configuration
pub fn init_llm_service_with_config(config: LlmServiceConfig) -> Result<(), &'static str> {
    let mut service_guard = LLM_SERVICE.lock();
    if service_guard.is_some() {
        return Err("LLM service already initialized");
    }
    
    let mut service = LlmService::new();
    
    // Apply configuration
    if config.enable_dev_backends {
        // In a real implementation, this would enable dev backends
        crate::logging::info!("LLM dev backends enabled");
    }
    
    *service_guard = Some(service);
    
    crate::logging::info!("LLM Adapter v0 service initialized with config: max_sessions={}, default_backend={}", 
        config.max_sessions, config.default_backend);
    Ok(())
}

/// Cleanup LLM service (for testing)
pub fn cleanup_llm_service() {
    let mut service_guard = LLM_SERVICE.lock();
    *service_guard = None;
    crate::logging::info!("LLM service cleaned up");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::schema::{
        PromptV1, MessageV1, AdapterConfigV1, QuotaLimitsV1, ROLE_USER, BACKEND_NULL
    };

    #[test]
    fn test_llm_service_creation() {
        let service = LlmService::new();
        assert!(service.is_available());
        
        let backends = service.get_available_backends();
        assert!(backends.contains(&BACKEND_NULL));
    }

    #[test]
    fn test_llm_service_session_lifecycle() {
        let service = LlmService::new();
        
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![],
        };
        
        // Open session
        let handle = service.open_session(config.clone()).unwrap();
        assert_eq!(handle.0, 1);
        
        // Get session info
        let info = service.get_session_info(handle).unwrap();
        assert_eq!(info.id, 1);
        assert!(info.active);
        
        // Send prompt
        let mut prompt = PromptV1 {
            messages: vec![
                MessageV1 { role: ROLE_USER, content: "Hello".to_string() },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };
        
        service.send_prompt(handle, &mut prompt).unwrap();
        
        // Receive chunks
        let chunks = service.receive_chunks(handle, 10).unwrap();
        assert!(!chunks.is_empty());
        
        // Close session
        service.close_session(handle).unwrap();
        
        // Verify session is closed
        let result = service.get_session_info(handle);
        assert!(result.is_err());
    }

    #[test]
    fn test_llm_service_stats() {
        let service = LlmService::new();
        
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![],
        };
        
        let handle = service.open_session(config).unwrap();
        let stats = service.get_stats();
        assert_eq!(stats.sessions_opened, 1);
        
        service.close_session(handle).unwrap();
        let stats = service.get_stats();
        assert_eq!(stats.sessions_closed, 1);
    }

    #[test]
    fn test_llm_service_global() {
        // Initialize service
        init_llm_service().unwrap();
        assert!(is_llm_available());
        
        // Test global functions
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![],
        };
        
        let handle = open_llm_session(config).unwrap();
        let info = get_llm_session_info(handle).unwrap();
        assert_eq!(info.id, 1);
        
        close_llm_session(handle).unwrap();
        
        // Cleanup
        cleanup_llm_service();
        assert!(!is_llm_available());
    }

    #[test]
    fn test_llm_service_config() {
        let config = LlmServiceConfig {
            max_sessions: 50,
            default_backend: BACKEND_NULL,
            enable_dev_backends: true,
        };
        
        init_llm_service_with_config(config).unwrap();
        assert!(is_llm_available());
        
        cleanup_llm_service();
    }
}
