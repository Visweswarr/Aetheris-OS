use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use crate::llm::schema::{PromptV1, CompletionChunkV1, LlmMetrics, BACKEND_NULL, BACKEND_DEV_LOCAL};
use crate::llm::session::ChunkSink;

/// LLM backend trait for different model implementations
pub trait LlmBackend: Send + Sync {
    /// Send a prompt and stream completion chunks
    fn send(&self, prompt: &PromptV1, sink: &mut ChunkSink) -> Result<LlmMetrics, &'static str>;
    
    /// Get backend type identifier
    fn get_backend_type(&self) -> u8;
    
    /// Check if backend is available
    fn is_available(&self) -> bool;
}

/// Null backend for deterministic testing and CI
pub struct NullBackend {
    /// Whether backend is available
    available: bool,
}

impl NullBackend {
    /// Create a new null backend
    pub fn new() -> Self {
        Self { available: true }
    }
    
    /// Generate deterministic response based on input
    fn generate_response(&self, prompt: &PromptV1) -> Vec<CompletionChunkV1> {
        let mut chunks = Vec::new();
        let mut seq = 0;
        
        // Find the last user message
        let last_user_msg = prompt.messages.iter()
            .rev()
            .find(|m| m.role == crate::llm::schema::ROLE_USER);
        
        if let Some(user_msg) = last_user_msg {
            // Split content into words and emit as tokens
            let words: Vec<&str> = user_msg.content.split_whitespace().collect();
            
            for word in words {
                seq += 1;
                chunks.push(CompletionChunkV1 {
                    seq,
                    token: Some(word.to_string()),
                    tool: None,
                    finish: None,
                });
            }
            
            // Add finish chunk
            seq += 1;
            chunks.push(CompletionChunkV1 {
                seq,
                token: None,
                tool: None,
                finish: Some(crate::llm::schema::FINISH_STOP),
            });
        } else {
            // No user message, emit default response
            seq += 1;
            chunks.push(CompletionChunkV1 {
                seq,
                token: Some("Hello".to_string()),
                tool: None,
                finish: None,
            });
            
            seq += 1;
            chunks.push(CompletionChunkV1 {
                seq,
                token: Some("world".to_string()),
                tool: None,
                finish: None,
            });
            
            seq += 1;
            chunks.push(CompletionChunkV1 {
                seq,
                token: None,
                tool: None,
                finish: Some(crate::llm::schema::FINISH_STOP),
            });
        }
        
        chunks
    }
}

impl LlmBackend for NullBackend {
    fn send(&self, prompt: &PromptV1, sink: &mut ChunkSink) -> Result<LlmMetrics, &'static str> {
        if !self.available {
            return Err("NullBackend not available");
        }
        
        let chunks = self.generate_response(prompt);
        let mut total_bytes = 0;
        let mut total_tokens = 0;
        
        for chunk in chunks {
            total_tokens += 1;
            if let Some(token) = &chunk.token {
                total_bytes += token.len();
            }
            
            // Send chunk to sink
            sink.push(chunk)?;
        }
        
        Ok(LlmMetrics {
            tokens: total_tokens,
            bytes: total_bytes as u32,
            time_ms: 1, // Minimal time for null backend
            finish_reason: crate::llm::schema::FINISH_STOP,
        })
    }
    
    fn get_backend_type(&self) -> u8 {
        BACKEND_NULL
    }
    
    fn is_available(&self) -> bool {
        self.available
    }
}

/// Development local backend for user-space model hosting
#[cfg(feature = "dev-llm")]
pub struct DevLocalBackend {
    /// Whether backend is available
    available: bool,
    /// Connection to user-space model host
    connection: Option<DevLocalConnection>,
}

#[cfg(feature = "dev-llm")]
impl DevLocalBackend {
    /// Create a new dev local backend
    pub fn new() -> Self {
        Self {
            available: true,
            connection: None,
        }
    }
    
    /// Try to connect to user-space model host
    fn try_connect(&mut self) -> Result<(), &'static str> {
        // In a real implementation, this would establish a PolyBus connection
        // For now, we'll simulate a connection
        self.connection = Some(DevLocalConnection::new());
        Ok(())
    }
}

#[cfg(feature = "dev-llm")]
impl LlmBackend for DevLocalBackend {
    fn send(&self, prompt: &PromptV1, sink: &mut ChunkSink) -> Result<LlmMetrics, &'static str> {
        if !self.available {
            return Err("DevLocalBackend not available");
        }
        
        // In a real implementation, this would:
        // 1. Serialize prompt to CBOR
        // 2. Send over PolyBus to user-space model host
        // 3. Stream completion chunks back
        // 4. Forward chunks to sink
        
        // For now, we'll simulate this with a simple response
        let mut chunks = Vec::new();
        let mut seq = 0;
        
        seq += 1;
        chunks.push(CompletionChunkV1 {
            seq,
            token: Some("Dev".to_string()),
            tool: None,
            finish: None,
        });
        
        seq += 1;
        chunks.push(CompletionChunkV1 {
            seq,
            token: Some("Local".to_string()),
            tool: None,
            finish: None,
        });
        
        seq += 1;
        chunks.push(CompletionChunkV1 {
            seq,
            token: Some("Backend".to_string()),
            tool: None,
            finish: None,
        });
        
        seq += 1;
        chunks.push(CompletionChunkV1 {
            seq,
            token: None,
            tool: None,
            finish: Some(crate::llm::schema::FINISH_STOP),
        });
        
        let mut total_bytes = 0;
        let mut total_tokens = 0;
        
        for chunk in chunks {
            total_tokens += 1;
            if let Some(token) = &chunk.token {
                total_bytes += token.len();
            }
            
            sink.push(chunk)?;
        }
        
        Ok(LlmMetrics {
            tokens: total_tokens,
            bytes: total_bytes as u32,
            time_ms: 10, // Simulated processing time
            finish_reason: crate::llm::schema::FINISH_STOP,
        })
    }
    
    fn get_backend_type(&self) -> u8 {
        BACKEND_DEV_LOCAL
    }
    
    fn is_available(&self) -> bool {
        self.available
    }
}

/// Development local connection (simulated)
#[cfg(feature = "dev-llm")]
struct DevLocalConnection {
    /// Connection ID
    id: u64,
}

#[cfg(feature = "dev-llm")]
impl DevLocalConnection {
    fn new() -> Self {
        Self { id: 0 }
    }
}

/// Backend factory for creating appropriate backends
pub struct BackendFactory;

impl BackendFactory {
    /// Create a backend based on configuration
    pub fn create_backend(backend_type: u8) -> Result<Box<dyn LlmBackend>, &'static str> {
        match backend_type {
            BACKEND_NULL => {
                Ok(Box::new(NullBackend::new()))
            }
            #[cfg(feature = "dev-llm")]
            BACKEND_DEV_LOCAL => {
                Ok(Box::new(DevLocalBackend::new()))
            }
            _ => {
                Err("Unknown backend type")
            }
        }
    }
    
    /// Get default backend (NullBackend)
    pub fn get_default_backend() -> Box<dyn LlmBackend> {
        Box::new(NullBackend::new())
    }
    
    /// Check if backend type is supported
    pub fn is_backend_supported(backend_type: u8) -> bool {
        match backend_type {
            BACKEND_NULL => true,
            #[cfg(feature = "dev-llm")]
            BACKEND_DEV_LOCAL => true,
            _ => false,
        }
    }
}

/// Backend registry for managing available backends
pub struct BackendRegistry {
    /// Available backends
    backends: BTreeMap<u8, Box<dyn LlmBackend>>,
}

impl BackendRegistry {
    /// Create a new backend registry
    pub fn new() -> Self {
        let mut registry = Self {
            backends: BTreeMap::new(),
        };
        
        // Register default backends
        registry.register_backend(Box::new(NullBackend::new()));
        
        #[cfg(feature = "dev-llm")]
        registry.register_backend(Box::new(DevLocalBackend::new()));
        
        registry
    }
    
    /// Register a backend
    pub fn register_backend(&mut self, backend: Box<dyn LlmBackend>) {
        let backend_type = backend.get_backend_type();
        self.backends.insert(backend_type, backend);
    }
    
    /// Get a backend by type
    pub fn get_backend(&self, backend_type: u8) -> Option<&dyn LlmBackend> {
        self.backends.get(&backend_type).map(|b| b.as_ref())
    }
    
    /// Check if backend type is available
    pub fn is_backend_available(&self, backend_type: u8) -> bool {
        self.backends.get(&backend_type)
            .map(|b| b.is_available())
            .unwrap_or(false)
    }
    
    /// Get list of available backend types
    pub fn get_available_backends(&self) -> Vec<u8> {
        self.backends.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::schema::{PromptV1, MessageV1, ROLE_USER};

    #[test]
    fn test_null_backend_creation() {
        let backend = NullBackend::new();
        assert!(backend.is_available());
        assert_eq!(backend.get_backend_type(), BACKEND_NULL);
    }

    #[test]
    fn test_null_backend_response() {
        let backend = NullBackend::new();
        let prompt = PromptV1 {
            messages: vec![
                MessageV1 { role: ROLE_USER, content: "Hello world".to_string() },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };
        
        // Test response generation
        let response = backend.generate_response(&prompt);
        assert_eq!(response.len(), 3); // 2 tokens + 1 finish
        
        // Check first token
        assert_eq!(response[0].token, Some("Hello".to_string()));
        assert_eq!(response[0].seq, 1);
        
        // Check second token
        assert_eq!(response[1].token, Some("world".to_string()));
        assert_eq!(response[1].seq, 2);
        
        // Check finish
        assert!(response[2].finish.is_some());
        assert_eq!(response[2].seq, 3);
    }

    #[test]
    fn test_backend_factory() {
        let backend = BackendFactory::create_backend(BACKEND_NULL).unwrap();
        assert_eq!(backend.get_backend_type(), BACKEND_NULL);
        
        let result = BackendFactory::create_backend(99);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_factory_support() {
        assert!(BackendFactory::is_backend_supported(BACKEND_NULL));
        assert!(!BackendFactory::is_backend_supported(99));
    }

    #[test]
    fn test_backend_registry() {
        let mut registry = BackendRegistry::new();
        
        assert!(registry.is_backend_available(BACKEND_NULL));
        assert!(!registry.is_backend_available(99));
        
        let available = registry.get_available_backends();
        assert!(available.contains(&BACKEND_NULL));
        
        let backend = registry.get_backend(BACKEND_NULL);
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().get_backend_type(), BACKEND_NULL);
    }

    #[test]
    fn test_backend_factory_default() {
        let backend = BackendFactory::get_default_backend();
        assert_eq!(backend.get_backend_type(), BACKEND_NULL);
    }
}
