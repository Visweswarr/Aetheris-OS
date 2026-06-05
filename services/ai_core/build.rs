use std::fs;
use std::path::Path;

fn main() {
    let proto_dir = Path::new("proto");
    let proto_file = proto_dir.join("ai_core.proto");

    // Use OUT_DIR for generated files
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "src/generated".to_string());
    let out_path = Path::new(&out_dir);

    // Ensure output directory exists
    fs::create_dir_all(out_path).ok();

    // Check if proto file exists
    if !proto_file.exists() {
        eprintln!(
            "Warning: Proto file not found: {:?}, creating stub",
            proto_file
        );
        create_stub_generated_file(out_path);
        return;
    }

    // Try to compile protobuf, but don't fail if protoc is not available
    match prost_build::Config::new().compile_protos(&[&proto_file], &[proto_dir]) {
        Ok(_) => {
            println!("cargo:rerun-if-changed=proto/ai_core.proto");
        }
        Err(e) => {
            eprintln!("Warning: Failed to compile protobuf: {}. Creating stub.", e);
            create_stub_generated_file(out_path);
        }
    }
}

fn create_stub_generated_file(out_dir: &Path) {
    // Create stub file with the expected name (aetheris.ai_core.rs)
    let stub_content = r#"//! Auto-generated stub for protobuf types
//! This file is generated when protoc is not available

/// Stub message type
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AiCoreMessage {
    pub message_id: String,
    pub timestamp: i64,
    pub session_id: String,
}

/// Ping request
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PingRequest {
    pub client_id: String,
    pub version: String,
}

/// Ping response
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PingResponse {
    pub server_id: String,
    pub version: String,
    pub server_time: i64,
}

/// Chat request
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatRequest {
    pub prompt: String,
    pub stream: bool,
    pub conversation_id: String,
}

/// Chat response
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatResponse {
    pub response: String,
    pub is_complete: bool,
    pub conversation_id: String,
}

/// Tool call request
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub call_id: String,
}

/// Tool call response
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolCallResponse {
    pub result: String,
    pub success: bool,
    pub call_id: String,
}

/// Error response
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub details: String,
}

/// Error code enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ErrorCode {
    #[default]
    UnknownError = 0,
    InvalidRequest = 1,
    Unauthorized = 2,
    InternalError = 8,
    Timeout = 9,
}

/// Service state enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ServiceState {
    #[default]
    Starting = 0,
    Running = 1,
    Stopping = 2,
    Stopped = 3,
    Error = 4,
}
"#;

    let stub_file = out_dir.join("aetheris.ai_core.rs");
    if let Err(e) = fs::write(&stub_file, stub_content) {
        eprintln!("Warning: Failed to write stub file: {}", e);
    }
}
