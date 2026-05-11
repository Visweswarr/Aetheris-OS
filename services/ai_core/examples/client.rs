//! Example client for AI Core Service
//!
//! Demonstrates how to connect to and interact with the AI Core Service
//! using Unix domain sockets and protobuf messages.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use prost::Message;
use uuid::Uuid;

use aetheris_ai_core::ipc::ai_core::*;

/// Example client for AI Core Service
struct AiCoreClient {
    socket_path: std::path::PathBuf,
}

impl AiCoreClient {
    /// Create a new client
    fn new(socket_path: std::path::PathBuf) -> Self {
        Self { socket_path }
    }
    
    /// Send a message and receive response
    async fn send_message(&self, message: AiCoreMessage) -> Result<AiCoreMessage, Box<dyn std::error::Error>> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        
        // Serialize message
        let mut data = Vec::new();
        message.encode(&mut data)?;
        
        // Send message length
        let length = data.len() as u32;
        let length_bytes = length.to_le_bytes();
        stream.write_all(&length_bytes).await?;
        
        // Send message data
        stream.write_all(&data).await?;
        
        // Read response length
        let mut length_buffer = [0u8; 4];
        stream.read_exact(&mut length_buffer).await?;
        let response_length = u32::from_le_bytes(length_buffer) as usize;
        
        // Read response data
        let mut response_buffer = vec![0u8; response_length];
        stream.read_exact(&mut response_buffer).await?;
        
        // Parse response
        let response = AiCoreMessage::decode(&response_buffer)?;
        Ok(response)
    }
    
    /// Ping the server
    async fn ping(&self) -> Result<PingResponse, Box<dyn std::error::Error>> {
        let request = AiCoreMessage {
            message_type: Some(ai_core_message::MessageType::PingRequest(
                PingRequest {
                    client_id: "example_client".to_string(),
                    version: "1.0.0".to_string(),
                }
            )),
            message_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            session_id: "example_session".to_string(),
            cap_token: None,
        };
        
        let response = self.send_message(request).await?;
        
        match response.message_type {
            Some(ai_core_message::MessageType::PingResponse(ping_response)) => {
                Ok(ping_response)
            }
            Some(ai_core_message::MessageType::ErrorResponse(error_response)) => {
                Err(format!("Server error: {}", error_response.message).into())
            }
            _ => Err("Unexpected response type".into()),
        }
    }
    
    /// Send a chat request
    async fn chat(&self, prompt: &str) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let request = AiCoreMessage {
            message_type: Some(ai_core_message::MessageType::ChatRequest(
                ChatRequest {
                    prompt: prompt.to_string(),
                    config: Some(ChatConfig {
                        model: "mock".to_string(),
                        temperature: 0.7,
                        max_tokens: 100,
                        top_p: 0.9,
                        top_k: 40,
                        enable_tools: false,
                        allowed_tools: Vec::new(),
                    }),
                    context: Vec::new(),
                    stream: false,
                }
            )),
            message_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            session_id: "example_session".to_string(),
            cap_token: Some(CapToken {
                token_id: "example_token".to_string(),
                capability: "ai:chat".to_string(),
                expires_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
                signature: vec![],
                issuer: "aetheris-system".to_string(),
            }),
        };
        
        let response = self.send_message(request).await?;
        
        match response.message_type {
            Some(ai_core_message::MessageType::ChatResponse(chat_response)) => {
                Ok(chat_response)
            }
            Some(ai_core_message::MessageType::ErrorResponse(error_response)) => {
                Err(format!("Server error: {}", error_response.message).into())
            }
            _ => Err("Unexpected response type".into()),
        }
    }
    
    /// Call a tool
    async fn call_tool(&self, tool_name: &str, parameters: HashMap<String, String>) -> Result<ToolCallResponse, Box<dyn std::error::Error>> {
        let request = AiCoreMessage {
            message_type: Some(ai_core_message::MessageType::ToolCallRequest(
                ToolCallRequest {
                    tool_name: tool_name.to_string(),
                    parameters,
                    context: "example_context".to_string(),
                }
            )),
            message_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            session_id: "example_session".to_string(),
            cap_token: Some(CapToken {
                token_id: "example_token".to_string(),
                capability: "ai:tools".to_string(),
                expires_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
                signature: vec![],
                issuer: "aetheris-system".to_string(),
            }),
        };
        
        let response = self.send_message(request).await?;
        
        match response.message_type {
            Some(ai_core_message::MessageType::ToolCallResponse(tool_response)) => {
                Ok(tool_response)
            }
            Some(ai_core_message::MessageType::ErrorResponse(error_response)) => {
                Err(format!("Server error: {}", error_response.message).into())
            }
            _ => Err("Unexpected response type".into()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create client
    let socket_path = std::path::PathBuf::from("/run/aetheris/ai.sock");
    let client = AiCoreClient::new(socket_path);
    
    println!("AI Core Service Client Example");
    println!("==============================");
    
    // Test ping
    println!("\n1. Testing ping...");
    match client.ping().await {
        Ok(response) => {
            println!("✅ Ping successful!");
            println!("   Server ID: {}", response.server_id);
            println!("   Version: {}", response.version);
            println!("   Server Time: {}", response.server_time);
            
            if let Some(status) = response.status {
                println!("   Status: {:?}", ServiceState::from_i32(status.state));
                println!("   Active Sessions: {}", status.active_sessions);
                println!("   Uptime: {}s", status.uptime_seconds);
            }
        }
        Err(e) => {
            println!("❌ Ping failed: {}", e);
            return Err(e);
        }
    }
    
    // Test chat
    println!("\n2. Testing chat...");
    match client.chat("Hello, AI Core Service!").await {
        Ok(response) => {
            println!("✅ Chat successful!");
            println!("   Response: {}", response.response);
            println!("   Complete: {}", response.is_complete);
            
            if let Some(metrics) = response.metrics {
                println!("   Processing Time: {}ms", metrics.processing_time_ms);
                println!("   Tokens Generated: {}", metrics.tokens_generated);
                println!("   Tokens Input: {}", metrics.tokens_input);
                println!("   Confidence: {:.2}", metrics.confidence);
                println!("   Model Used: {}", metrics.model_used);
            }
        }
        Err(e) => {
            println!("❌ Chat failed: {}", e);
        }
    }
    
    // Test echo tool
    println!("\n3. Testing echo tool...");
    let mut parameters = HashMap::new();
    parameters.insert("message".to_string(), "Hello from the echo tool!".to_string());
    
    match client.call_tool("echo", parameters).await {
        Ok(response) => {
            println!("✅ Echo tool successful!");
            println!("   Result: {}", response.result);
            println!("   Success: {}", response.success);
            
            if let Some(metrics) = response.metrics {
                println!("   Execution Time: {}ms", metrics.execution_time_ms);
                println!("   Memory Used: {}MB", metrics.memory_used_mb);
                println!("   Cache Hit: {}", metrics.cache_hit);
                println!("   Tool Version: {}", metrics.tool_version);
            }
        }
        Err(e) => {
            println!("❌ Echo tool failed: {}", e);
        }
    }
    
    // Test calculator tool
    println!("\n4. Testing calculator tool...");
    let mut parameters = HashMap::new();
    parameters.insert("expression".to_string(), "2+2".to_string());
    
    match client.call_tool("calculator", parameters).await {
        Ok(response) => {
            println!("✅ Calculator tool successful!");
            println!("   Result: {}", response.result);
            println!("   Success: {}", response.success);
            
            if let Some(metrics) = response.metrics {
                println!("   Execution Time: {}ms", metrics.execution_time_ms);
                println!("   Memory Used: {}MB", metrics.memory_used_mb);
                println!("   Cache Hit: {}", metrics.cache_hit);
                println!("   Tool Version: {}", metrics.tool_version);
            }
        }
        Err(e) => {
            println!("❌ Calculator tool failed: {}", e);
        }
    }
    
    // Test system info tool
    println!("\n5. Testing system info tool...");
    let mut parameters = HashMap::new();
    parameters.insert("type".to_string(), "basic".to_string());
    
    match client.call_tool("system_info", parameters).await {
        Ok(response) => {
            println!("✅ System info tool successful!");
            println!("   Result: {}", response.result);
            println!("   Success: {}", response.success);
            
            if let Some(metrics) = response.metrics {
                println!("   Execution Time: {}ms", metrics.execution_time_ms);
                println!("   Memory Used: {}MB", metrics.memory_used_mb);
                println!("   Cache Hit: {}", metrics.cache_hit);
                println!("   Tool Version: {}", metrics.tool_version);
            }
        }
        Err(e) => {
            println!("❌ System info tool failed: {}", e);
        }
    }
    
    println!("\n🎉 All tests completed!");
    Ok(())
}
