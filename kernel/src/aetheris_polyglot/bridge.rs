//! Runtime Bridge
//!
//! Translates between language-specific APIs and kernel syscalls, handles
//! capability verification, and manages IPC message serialization.
//!
//! This module provides the communication layer between polyglot sandboxes
//! and the Aetheris kernel, integrating with:
//! - CapTokenManager for capability-based security
//! - AI Event Bus for cross-runtime communication
//! - Policy Enforcer for access control decisions

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

use crate::security::{CapToken, CapabilityStore, validate_cap, get_current_time_ms};
use crate::klog;
use super::backend::{RuntimeError, Value};
use super::sandbox::SandboxId;

/// IPC message format for cross-runtime communication
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Message ID
    pub id: u64,
    /// Source sandbox ID
    pub source: SandboxId,
    /// Destination
    pub destination: Destination,
    /// Message type identifier
    pub message_type: String,
    /// Payload (CBOR-encoded)
    pub payload: Vec<u8>,
    /// Timestamp (milliseconds)
    pub timestamp: u64,
}

/// Message destination
#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    /// Direct to sandbox
    Sandbox(SandboxId),
    /// Publish to topic
    Topic(String),
    /// Broadcast to all
    Broadcast,
}

/// Syscall representation for translation
#[derive(Debug, Clone)]
pub struct Syscall {
    /// Syscall number
    pub number: u32,
    /// Syscall arguments
    pub args: Vec<u64>,
}

/// Syscall result
#[derive(Debug, Clone)]
pub struct SyscallResult {
    /// Return value
    pub value: i64,
    /// Output data (if any)
    pub data: Option<Vec<u8>>,
}

/// Security violation log entry
#[derive(Debug, Clone)]
pub struct SecurityViolation {
    /// Sandbox that attempted the violation
    pub sandbox_id: SandboxId,
    /// Requested resource
    pub resource: String,
    /// Timestamp
    pub timestamp: u64,
    /// Reason for denial
    pub reason: String,
}

/// Aetheris Kernel Operation types for syscall translation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AetherisOp {
    /// File operations
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    /// Memory operations
    Mmap = 9,
    Munmap = 11,
    /// IPC operations (AI Event Bus)
    SendMessage = 100,
    RecvMessage = 101,
    Subscribe = 102,
    Unsubscribe = 103,
    /// Capability operations
    CapRequest = 200,
    CapRelease = 201,
    CapVerify = 202,
    /// AI-specific operations
    AiInvoke = 300,
    AiQuery = 301,
    AiStream = 302,
}

impl AetherisOp {
    /// Get the syscall number for this operation
    pub fn syscall_number(&self) -> u32 {
        *self as u32
    }

    /// Check if this operation requires capability verification
    pub fn requires_capability(&self) -> bool {
        matches!(
            self,
            AetherisOp::Write
                | AetherisOp::Open
                | AetherisOp::SendMessage
                | AetherisOp::AiInvoke
                | AetherisOp::AiQuery
                | AetherisOp::AiStream
        )
    }

    /// Get the operation name for logging
    pub fn name(&self) -> &'static str {
        match self {
            AetherisOp::Read => "read",
            AetherisOp::Write => "write",
            AetherisOp::Open => "open",
            AetherisOp::Close => "close",
            AetherisOp::Mmap => "mmap",
            AetherisOp::Munmap => "munmap",
            AetherisOp::SendMessage => "send_message",
            AetherisOp::RecvMessage => "recv_message",
            AetherisOp::Subscribe => "subscribe",
            AetherisOp::Unsubscribe => "unsubscribe",
            AetherisOp::CapRequest => "cap_request",
            AetherisOp::CapRelease => "cap_release",
            AetherisOp::CapVerify => "cap_verify",
            AetherisOp::AiInvoke => "ai_invoke",
            AetherisOp::AiQuery => "ai_query",
            AetherisOp::AiStream => "ai_stream",
        }
    }
}

/// Runtime bridge - translates between language APIs and kernel syscalls
/// 
/// The PolyglotBridge serves as the communication layer between polyglot
/// sandboxes and the Aetheris kernel. It integrates with:
/// - CapTokenManager for capability-based security verification
/// - AI Event Bus for cross-runtime IPC
/// - Policy Enforcer for access control decisions
pub struct PolyglotBridge {
    /// Security violation log
    violations: Vec<SecurityViolation>,
    /// Trace enabled flag
    trace_enabled: bool,
    /// Trace events
    trace_events: Vec<TraceEvent>,
    /// Capability store reference for token management
    cap_store: CapabilityStore,
    /// Revoked token cache for fast lookup
    revoked_tokens: Vec<u128>,
}

/// Trace event for syscall tracing
#[derive(Debug, Clone)]
pub struct TraceEvent {
    /// Sandbox ID
    pub sandbox_id: SandboxId,
    /// Syscall number
    pub syscall_number: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Arguments
    pub args: Vec<u64>,
    /// Result (if completed)
    pub result: Option<i64>,
}

impl PolyglotBridge {
    /// Create a new runtime bridge
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            trace_enabled: false,
            trace_events: Vec::new(),
            cap_store: CapabilityStore::new(),
            revoked_tokens: Vec::new(),
        }
    }

    /// Create a bridge with an existing capability store
    pub fn with_cap_store(cap_store: CapabilityStore) -> Self {
        Self {
            violations: Vec::new(),
            trace_enabled: false,
            trace_events: Vec::new(),
            cap_store,
            revoked_tokens: Vec::new(),
        }
    }

    /// Verify capability for an operation using the integrated CapTokenManager
    /// 
    /// This method checks:
    /// 1. Token is not in the revocation list
    /// 2. Token grants access to the destination
    /// 3. Token has not expired
    /// 
    /// On failure, logs a security violation for audit purposes.
    pub fn verify_capability(
        &mut self,
        sandbox_id: SandboxId,
        token: &CapToken,
        destination: u64,
        resource: &str,
    ) -> Result<(), RuntimeError> {
        let current_time = get_current_time_ms();
        
        // Check revocation list first (fast path for revoked tokens)
        if self.is_token_revoked(token.id) {
            self.log_denial(sandbox_id, resource, "Token has been revoked");
            return Err(RuntimeError::CapabilityDenied {
                operation: resource.into(),
                reason: "Capability token has been revoked".into(),
            });
        }
        
        // Validate against the capability store
        if !validate_cap(token, destination, current_time) {
            let reason = if token.is_expired(current_time) {
                "Capability token has expired"
            } else if token.dst != destination {
                "Token does not grant access to destination"
            } else {
                "Capability verification failed"
            };
            
            self.log_denial(sandbox_id, resource, reason);
            
            return Err(RuntimeError::CapabilityDenied {
                operation: resource.into(),
                reason: reason.into(),
            });
        }
        
        Ok(())
    }

    /// Verify capability for an Aetheris operation
    /// 
    /// Checks if the sandbox has permission to perform the specified operation.
    pub fn verify_operation(
        &mut self,
        sandbox_id: SandboxId,
        token: &CapToken,
        op: AetherisOp,
        destination: u64,
    ) -> Result<(), RuntimeError> {
        if !op.requires_capability() {
            return Ok(());
        }
        
        self.verify_capability(sandbox_id, token, destination, op.name())
    }

    /// Revoke a capability token globally
    /// 
    /// Adds the token to the revocation list and propagates to the capability store.
    pub fn revoke_token(&mut self, token_id: u128) {
        if !self.revoked_tokens.contains(&token_id) {
            self.revoked_tokens.push(token_id);
            self.cap_store.revoke_capability_globally(token_id);
            klog!(WARN, "[POLYGLOT] Revoked capability token 0x{:x}", token_id);
        }
    }

    /// Check if a token has been revoked
    pub fn is_token_revoked(&self, token_id: u128) -> bool {
        self.revoked_tokens.contains(&token_id) || self.cap_store.is_token_revoked(token_id)
    }

    /// Get the capability store for direct access
    pub fn cap_store(&self) -> &CapabilityStore {
        &self.cap_store
    }

    /// Get mutable access to the capability store
    pub fn cap_store_mut(&mut self) -> &mut CapabilityStore {
        &mut self.cap_store
    }

    /// Log a capability denial
    pub fn log_denial(&mut self, sandbox_id: SandboxId, resource: &str, reason: &str) {
        let violation = SecurityViolation {
            sandbox_id,
            resource: resource.into(),
            timestamp: get_current_time_ms(),
            reason: reason.into(),
        };
        self.violations.push(violation);
        klog!(WARN, "[POLYGLOT] Access denied for sandbox {:?}: {} - {}", sandbox_id, resource, reason);
    }

    /// Get security violations for a sandbox
    pub fn get_violations(&self, sandbox_id: SandboxId) -> Vec<&SecurityViolation> {
        self.violations.iter().filter(|v| v.sandbox_id == sandbox_id).collect()
    }

    /// Translate a language-specific request to an Aetheris kernel syscall
    /// 
    /// Maps high-level operation names to Aetheris kernel operation codes.
    /// Returns a valid Syscall structure that can be executed by the kernel.
    pub fn translate_syscall(&self, request: &str, args: &[Value]) -> Result<Syscall, RuntimeError> {
        // Map operation names to AetherisOp enum
        let op = match request.to_lowercase().as_str() {
            "read" => AetherisOp::Read,
            "write" => AetherisOp::Write,
            "open" => AetherisOp::Open,
            "close" => AetherisOp::Close,
            "mmap" => AetherisOp::Mmap,
            "munmap" => AetherisOp::Munmap,
            "send_message" | "send" => AetherisOp::SendMessage,
            "recv_message" | "recv" | "receive" => AetherisOp::RecvMessage,
            "subscribe" => AetherisOp::Subscribe,
            "unsubscribe" => AetherisOp::Unsubscribe,
            "cap_request" => AetherisOp::CapRequest,
            "cap_release" => AetherisOp::CapRelease,
            "cap_verify" => AetherisOp::CapVerify,
            "ai_invoke" | "invoke" => AetherisOp::AiInvoke,
            "ai_query" | "query" => AetherisOp::AiQuery,
            "ai_stream" | "stream" => AetherisOp::AiStream,
            _ => {
                return Err(RuntimeError::SyscallFailed {
                    syscall: request.into(),
                    error_code: -1,
                });
            }
        };

        let syscall_args = self.values_to_args(args)?;

        Ok(Syscall {
            number: op.syscall_number(),
            args: syscall_args,
        })
    }

    /// Get the AetherisOp for a syscall number
    pub fn op_from_syscall_number(number: u32) -> Option<AetherisOp> {
        match number {
            0 => Some(AetherisOp::Read),
            1 => Some(AetherisOp::Write),
            2 => Some(AetherisOp::Open),
            3 => Some(AetherisOp::Close),
            9 => Some(AetherisOp::Mmap),
            11 => Some(AetherisOp::Munmap),
            100 => Some(AetherisOp::SendMessage),
            101 => Some(AetherisOp::RecvMessage),
            102 => Some(AetherisOp::Subscribe),
            103 => Some(AetherisOp::Unsubscribe),
            200 => Some(AetherisOp::CapRequest),
            201 => Some(AetherisOp::CapRelease),
            202 => Some(AetherisOp::CapVerify),
            300 => Some(AetherisOp::AiInvoke),
            301 => Some(AetherisOp::AiQuery),
            302 => Some(AetherisOp::AiStream),
            _ => None,
        }
    }

    /// Validate that a syscall is well-formed according to the Aetheris syscall schema
    pub fn validate_syscall(&self, syscall: &Syscall) -> Result<(), RuntimeError> {
        // Check if syscall number is valid
        if Self::op_from_syscall_number(syscall.number).is_none() {
            return Err(RuntimeError::SyscallFailed {
                syscall: format!("syscall_{}", syscall.number),
                error_code: -22, // EINVAL
            });
        }

        // Validate argument count based on operation
        let min_args = match syscall.number {
            0 | 1 => 3,  // read/write: fd, buf, count
            2 => 2,      // open: path, flags
            3 => 1,      // close: fd
            9 => 6,      // mmap: addr, len, prot, flags, fd, offset
            11 => 2,     // munmap: addr, len
            100 | 101 => 2, // send/recv: dest, msg
            102 | 103 => 1, // subscribe/unsubscribe: topic
            200 | 201 | 202 => 1, // cap operations: token_id
            300 | 301 | 302 => 1, // ai operations: request
            _ => 0,
        };

        if syscall.args.len() < min_args {
            return Err(RuntimeError::SyscallFailed {
                syscall: format!("syscall_{}", syscall.number),
                error_code: -22, // EINVAL - not enough arguments
            });
        }

        Ok(())
    }

    /// Convert Value array to syscall arguments
    fn values_to_args(&self, values: &[Value]) -> Result<Vec<u64>, RuntimeError> {
        values.iter().map(|v| {
            match v {
                Value::Int(i) => Ok(*i as u64),
                Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
                _ => Err(RuntimeError::SerializationError(
                    "Cannot convert value to syscall argument".into()
                )),
            }
        }).collect()
    }

    /// Serialize a value to CBOR format
    pub fn serialize_cbor(&self, value: &Value) -> Result<Vec<u8>, RuntimeError> {
        // Simple CBOR encoding
        let mut output = Vec::new();
        self.encode_value(&mut output, value)?;
        Ok(output)
    }

    /// Encode a value to CBOR
    fn encode_value(&self, output: &mut Vec<u8>, value: &Value) -> Result<(), RuntimeError> {
        match value {
            Value::Null => {
                output.push(0xf6); // CBOR null
            }
            Value::Bool(b) => {
                output.push(if *b { 0xf5 } else { 0xf4 }); // CBOR true/false
            }
            Value::Int(i) => {
                if *i >= 0 && *i <= 23 {
                    output.push(*i as u8);
                } else if *i >= 0 && *i <= 255 {
                    output.push(0x18);
                    output.push(*i as u8);
                } else if *i >= 0 && *i <= 65535 {
                    output.push(0x19);
                    output.extend_from_slice(&(*i as u16).to_be_bytes());
                } else if *i >= 0 && *i <= 4294967295 {
                    output.push(0x1a);
                    output.extend_from_slice(&(*i as u32).to_be_bytes());
                } else if *i >= 0 {
                    output.push(0x1b);
                    output.extend_from_slice(&(*i as u64).to_be_bytes());
                } else {
                    // Negative integers
                    let abs = ((-1 - *i) as u64);
                    if abs <= 23 {
                        output.push(0x20 | abs as u8);
                    } else if abs <= 255 {
                        output.push(0x38);
                        output.push(abs as u8);
                    } else if abs <= 65535 {
                        output.push(0x39);
                        output.extend_from_slice(&(abs as u16).to_be_bytes());
                    } else if abs <= 4294967295 {
                        output.push(0x3a);
                        output.extend_from_slice(&(abs as u32).to_be_bytes());
                    } else {
                        output.push(0x3b);
                        output.extend_from_slice(&(abs as u64).to_be_bytes());
                    }
                }
            }
            Value::Float(f) => {
                output.push(0xfb); // CBOR float64
                output.extend_from_slice(&f.to_be_bytes());
            }
            Value::String(s) => {
                let len = s.len();
                if len <= 23 {
                    output.push(0x60 | len as u8);
                } else if len <= 255 {
                    output.push(0x78);
                    output.push(len as u8);
                } else {
                    output.push(0x79);
                    output.extend_from_slice(&(len as u16).to_be_bytes());
                }
                output.extend_from_slice(s.as_bytes());
            }
            Value::Bytes(b) => {
                let len = b.len();
                if len <= 23 {
                    output.push(0x40 | len as u8);
                } else if len <= 255 {
                    output.push(0x58);
                    output.push(len as u8);
                } else {
                    output.push(0x59);
                    output.extend_from_slice(&(len as u16).to_be_bytes());
                }
                output.extend_from_slice(b);
            }
            Value::Array(arr) => {
                let len = arr.len();
                if len <= 23 {
                    output.push(0x80 | len as u8);
                } else if len <= 255 {
                    output.push(0x98);
                    output.push(len as u8);
                } else {
                    output.push(0x99);
                    output.extend_from_slice(&(len as u16).to_be_bytes());
                }
                for item in arr {
                    self.encode_value(output, item)?;
                }
            }
        }
        Ok(())
    }

    /// Deserialize CBOR data to a value
    pub fn deserialize_cbor(&self, data: &[u8]) -> Result<Value, RuntimeError> {
        if data.is_empty() {
            return Err(RuntimeError::DeserializationError {
                offset: 0,
                reason: "Empty input".into(),
            });
        }
        
        let (value, _) = self.decode_value(data, 0)?;
        Ok(value)
    }

    /// Decode a CBOR value at the given offset
    fn decode_value(&self, data: &[u8], offset: usize) -> Result<(Value, usize), RuntimeError> {
        if offset >= data.len() {
            return Err(RuntimeError::DeserializationError {
                offset,
                reason: "Unexpected end of input".into(),
            });
        }

        let initial_byte = data[offset];
        let major_type = initial_byte >> 5;
        let additional_info = initial_byte & 0x1f;

        match major_type {
            0 => {
                // Unsigned integer
                let (value, new_offset) = self.decode_uint(data, offset)?;
                Ok((Value::Int(value as i64), new_offset))
            }
            1 => {
                // Negative integer
                let (value, new_offset) = self.decode_uint(data, offset)?;
                Ok((Value::Int(-1 - value as i64), new_offset))
            }
            2 => {
                // Byte string
                let (len, mut new_offset) = self.decode_length(data, offset)?;
                if new_offset + len > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset: new_offset,
                        reason: "Byte string length exceeds input".into(),
                    });
                }
                let bytes = data[new_offset..new_offset + len].to_vec();
                new_offset += len;
                Ok((Value::Bytes(bytes), new_offset))
            }
            3 => {
                // Text string
                let (len, mut new_offset) = self.decode_length(data, offset)?;
                if new_offset + len > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset: new_offset,
                        reason: "String length exceeds input".into(),
                    });
                }
                let s = core::str::from_utf8(&data[new_offset..new_offset + len])
                    .map_err(|_| RuntimeError::DeserializationError {
                        offset: new_offset,
                        reason: "Invalid UTF-8 in string".into(),
                    })?;
                new_offset += len;
                Ok((Value::String(s.into()), new_offset))
            }
            4 => {
                // Array
                let (len, mut new_offset) = self.decode_length(data, offset)?;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    let (item, next_offset) = self.decode_value(data, new_offset)?;
                    arr.push(item);
                    new_offset = next_offset;
                }
                Ok((Value::Array(arr), new_offset))
            }
            7 => {
                // Simple values and floats
                match additional_info {
                    20 => Ok((Value::Bool(false), offset + 1)),
                    21 => Ok((Value::Bool(true), offset + 1)),
                    22 => Ok((Value::Null, offset + 1)),
                    27 => {
                        // Float64
                        if offset + 9 > data.len() {
                            return Err(RuntimeError::DeserializationError {
                                offset,
                                reason: "Float64 truncated".into(),
                            });
                        }
                        let bytes: [u8; 8] = data[offset + 1..offset + 9].try_into().unwrap();
                        let f = f64::from_be_bytes(bytes);
                        Ok((Value::Float(f), offset + 9))
                    }
                    _ => Err(RuntimeError::DeserializationError {
                        offset,
                        reason: format!("Unsupported simple value: {}", additional_info),
                    }),
                }
            }
            _ => Err(RuntimeError::DeserializationError {
                offset,
                reason: format!("Unsupported major type: {}", major_type),
            }),
        }
    }

    /// Decode an unsigned integer from CBOR
    fn decode_uint(&self, data: &[u8], offset: usize) -> Result<(u64, usize), RuntimeError> {
        let additional_info = data[offset] & 0x1f;
        match additional_info {
            0..=23 => Ok((additional_info as u64, offset + 1)),
            24 => {
                if offset + 2 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated uint8".into(),
                    });
                }
                Ok((data[offset + 1] as u64, offset + 2))
            }
            25 => {
                if offset + 3 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated uint16".into(),
                    });
                }
                let value = u16::from_be_bytes([data[offset + 1], data[offset + 2]]);
                Ok((value as u64, offset + 3))
            }
            26 => {
                if offset + 5 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated uint32".into(),
                    });
                }
                let value = u32::from_be_bytes([
                    data[offset + 1], data[offset + 2],
                    data[offset + 3], data[offset + 4],
                ]);
                Ok((value as u64, offset + 5))
            }
            27 => {
                if offset + 9 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated uint64".into(),
                    });
                }
                let value = u64::from_be_bytes([
                    data[offset + 1], data[offset + 2],
                    data[offset + 3], data[offset + 4],
                    data[offset + 5], data[offset + 6],
                    data[offset + 7], data[offset + 8],
                ]);
                Ok((value, offset + 9))
            }
            _ => Err(RuntimeError::DeserializationError {
                offset,
                reason: "Unsupported integer size".into(),
            }),
        }
    }

    /// Decode a length value from CBOR
    fn decode_length(&self, data: &[u8], offset: usize) -> Result<(usize, usize), RuntimeError> {
        let additional_info = data[offset] & 0x1f;
        match additional_info {
            0..=23 => Ok((additional_info as usize, offset + 1)),
            24 => {
                if offset + 2 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated length".into(),
                    });
                }
                Ok((data[offset + 1] as usize, offset + 2))
            }
            25 => {
                if offset + 3 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated length".into(),
                    });
                }
                let len = u16::from_be_bytes([data[offset + 1], data[offset + 2]]);
                Ok((len as usize, offset + 3))
            }
            26 => {
                if offset + 5 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated length32".into(),
                    });
                }
                let len = u32::from_be_bytes([
                    data[offset + 1], data[offset + 2],
                    data[offset + 3], data[offset + 4],
                ]);
                Ok((len as usize, offset + 5))
            }
            27 => {
                if offset + 9 > data.len() {
                    return Err(RuntimeError::DeserializationError {
                        offset,
                        reason: "Truncated length64".into(),
                    });
                }
                let len = u64::from_be_bytes([
                    data[offset + 1], data[offset + 2],
                    data[offset + 3], data[offset + 4],
                    data[offset + 5], data[offset + 6],
                    data[offset + 7], data[offset + 8],
                ]);
                Ok((len as usize, offset + 9))
            }
            _ => Err(RuntimeError::DeserializationError {
                offset,
                reason: "Unsupported length encoding".into(),
            }),
        }
    }

    /// Serialize a message to CBOR
    pub fn serialize_message(&self, message: &Message) -> Result<Vec<u8>, RuntimeError> {
        let mut output = Vec::new();
        
        // Encode as a 6-element array
        output.push(0x86); // Array of 6 items
        
        // id
        self.encode_value(&mut output, &Value::Int(message.id as i64))?;
        
        // source sandbox id
        self.encode_value(&mut output, &Value::Int(message.source.0 as i64))?;
        
        // destination (encode as array: [type, value])
        match &message.destination {
            Destination::Sandbox(id) => {
                output.push(0x82); // Array of 2
                self.encode_value(&mut output, &Value::Int(0))?;
                self.encode_value(&mut output, &Value::Int(id.0 as i64))?;
            }
            Destination::Topic(topic) => {
                output.push(0x82);
                self.encode_value(&mut output, &Value::Int(1))?;
                self.encode_value(&mut output, &Value::String(topic.clone()))?;
            }
            Destination::Broadcast => {
                output.push(0x82);
                self.encode_value(&mut output, &Value::Int(2))?;
                self.encode_value(&mut output, &Value::Null)?;
            }
        }
        
        // message_type
        self.encode_value(&mut output, &Value::String(message.message_type.clone()))?;
        
        // payload (as bytes)
        self.encode_value(&mut output, &Value::Bytes(message.payload.clone()))?;
        
        // timestamp
        self.encode_value(&mut output, &Value::Int(message.timestamp as i64))?;
        
        Ok(output)
    }

    /// Deserialize a message from CBOR
    pub fn deserialize_message(&self, data: &[u8]) -> Result<Message, RuntimeError> {
        let (value, _) = self.decode_value(data, 0)?;
        
        match value {
            Value::Array(arr) if arr.len() == 6 => {
                let id = match &arr[0] {
                    Value::Int(i) => *i as u64,
                    _ => return Err(RuntimeError::DeserializationError {
                        offset: 0,
                        reason: "Invalid message id".into(),
                    }),
                };
                
                let source = match &arr[1] {
                    Value::Int(i) => SandboxId(*i as u64),
                    _ => return Err(RuntimeError::DeserializationError {
                        offset: 0,
                        reason: "Invalid source sandbox id".into(),
                    }),
                };
                
                let destination = match &arr[2] {
                    Value::Array(dest_arr) if dest_arr.len() == 2 => {
                        match (&dest_arr[0], &dest_arr[1]) {
                            (Value::Int(0), Value::Int(id)) => Destination::Sandbox(SandboxId(*id as u64)),
                            (Value::Int(1), Value::String(topic)) => Destination::Topic(topic.clone()),
                            (Value::Int(2), Value::Null) => Destination::Broadcast,
                            _ => return Err(RuntimeError::DeserializationError {
                                offset: 0,
                                reason: "Invalid destination".into(),
                            }),
                        }
                    }
                    _ => return Err(RuntimeError::DeserializationError {
                        offset: 0,
                        reason: "Invalid destination format".into(),
                    }),
                };
                
                let message_type = match &arr[3] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::DeserializationError {
                        offset: 0,
                        reason: "Invalid message type".into(),
                    }),
                };
                
                let payload = match &arr[4] {
                    Value::Bytes(b) => b.clone(),
                    _ => return Err(RuntimeError::DeserializationError {
                        offset: 0,
                        reason: "Invalid payload".into(),
                    }),
                };
                
                let timestamp = match &arr[5] {
                    Value::Int(i) => *i as u64,
                    _ => return Err(RuntimeError::DeserializationError {
                        offset: 0,
                        reason: "Invalid timestamp".into(),
                    }),
                };
                
                Ok(Message {
                    id,
                    source,
                    destination,
                    message_type,
                    payload,
                    timestamp,
                })
            }
            _ => Err(RuntimeError::DeserializationError {
                offset: 0,
                reason: "Message must be a 6-element array".into(),
            }),
        }
    }

    /// Enable syscall tracing
    pub fn enable_tracing(&mut self) {
        self.trace_enabled = true;
    }

    /// Disable syscall tracing
    pub fn disable_tracing(&mut self) {
        self.trace_enabled = false;
    }

    /// Record a trace event
    pub fn trace_syscall(&mut self, sandbox_id: SandboxId, syscall: &Syscall, result: Option<i64>) {
        if self.trace_enabled {
            self.trace_events.push(TraceEvent {
                sandbox_id,
                syscall_number: syscall.number,
                timestamp: get_current_time_ms(),
                args: syscall.args.clone(),
                result,
            });
        }
    }

    /// Get trace events for a sandbox
    pub fn get_trace_events(&self, sandbox_id: SandboxId) -> Vec<&TraceEvent> {
        self.trace_events.iter().filter(|e| e.sandbox_id == sandbox_id).collect()
    }

    /// Clear trace events
    pub fn clear_trace_events(&mut self) {
        self.trace_events.clear();
    }
}

impl Default for PolyglotBridge {
    fn default() -> Self {
        Self::new()
    }
}
