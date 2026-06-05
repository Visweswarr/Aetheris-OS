//! Intent Client v0 — userland envelope + transport
//! Sends {intent, args, captoken} as CBOR to an intent socket.
//! For initial integration, supports a to-file:// fallback that writes the CBOR bytes to a file.

use serde::{Deserialize, Serialize};
use crate::error::{AiError, AiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentEnvelope {
    /// Canonical intent type/kind (e.g., "system.backup.ngfs")
    pub intent: String,
    /// Arbitrary arguments payload (JSON)
    pub args: serde_json::Value,
    /// Capability token bytes (JWT/CBOR). Empty means none.
    pub captoken: Vec<u8>,
    /// 16-byte nonce for replay protection
    pub nonce: [u8; 16],
    /// 90 kHz timestamp
    pub ts90k: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResponse {
    /// Status string (e.g., "ok", "error")
    pub status: String,
    /// Preview identifier (if any)
    pub preview_id: Option<String>,
    /// Commit identifier (if any)
    pub commit_id: Option<String>,
    /// Optional message
    pub message: Option<String>,
}

enum Endpoint {
    File(std::path::PathBuf),
    #[cfg(target_family = "unix")]
    Unix(std::path::PathBuf),
    #[cfg(target_os = "windows")]
    Pipe(String),
}

impl Endpoint {
    fn parse(endpoint: &str) -> Result<Self, AiError> {
        if let Some(p) = endpoint.strip_prefix("to-file://") {
            return Ok(Self::File(std::path::PathBuf::from(p)));
        }
        #[cfg(target_family = "unix")]
        if let Some(p) = endpoint.strip_prefix("unix://") {
            return Ok(Self::Unix(std::path::PathBuf::from(p)));
        }
        #[cfg(target_os = "windows")]
        if let Some(p) = endpoint.strip_prefix("pipe://") {
            return Ok(Self::Pipe(p.to_string()));
        }
        Err(AiError::internal(format!("unsupported endpoint: {}", endpoint)))
    }
}

/// Client for sending envelopes to INTENT_SOCKET
pub struct IntentClient {
    endpoint_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeV1 {
    pub v: u8,
    pub captoken: Vec<u8>,
    pub payload: Vec<u8>,
}

impl IntentClient {
    pub fn from_env() -> Option<Self> {
        if let Ok(ep) = std::env::var("INTENT_SOCKET") {
            if !ep.trim().is_empty() {
                return Some(Self { endpoint_raw: ep });
            }
        }
        None
    }

    fn endpoint(&self) -> Result<Endpoint, AiError> { Endpoint::parse(&self.endpoint_raw) }

    fn load_captoken_from_env() -> Option<Vec<u8>> {
        if let Ok(path) = std::env::var("AICORE_CAPTOKEN") {
            if !path.trim().is_empty() {
                if let Ok(bytes) = std::fs::read(path) { return Some(bytes); }
            }
        }
        None
    }

    /// Construct a canonical envelope (fills ts90k and nonce)
    pub fn make_envelope(intent: &str, args: serde_json::Value, captoken: Option<Vec<u8>>) -> IntentEnvelope {
        use rand::RngCore;
        let mut nonce = [0u8; 16];
        if cfg!(test) {
            // deterministic nonce in tests
            nonce = [0x42; 16];
        } else {
            rand::rngs::OsRng.fill_bytes(&mut nonce);
        }
        let ts90k = crate::time::get_time_90khz();
        let token = match captoken {
            Some(t) => t,
            None => Self::load_captoken_from_env().unwrap_or_default(),
        };
        IntentEnvelope {
            intent: intent.to_string(),
            args,
            captoken: token,
            nonce,
            ts90k,
        }
    }

    /// Compute a deterministic preview id for an envelope
    pub fn compute_preview_id(env: &IntentEnvelope) -> String {
        // Hash canonical JSON of select fields to keep stable across transports
        let can = serde_json::json!({
            "intent": env.intent,
            "args": env.args,
            "captoken_hash": blake3::hash(&env.captoken).to_hex().to_string(),
            "ts90k": env.ts90k,
        });
        let s = serde_json::to_string(&can).unwrap_or_default();
        blake3::hash(s.as_bytes()).to_hex().to_string()
    }

    /// Send a preview request and return IntentResponse
    pub async fn preview(&self, env: &IntentEnvelope) -> AiResult<IntentResponse> {
        self.send("preview", env).await
    }

    /// Send a commit request and return IntentResponse
    pub async fn commit(&self, env: &IntentEnvelope) -> AiResult<IntentResponse> {
        self.send("commit", env).await
    }

    async fn send(&self, kind: &str, envlp: &IntentEnvelope) -> AiResult<IntentResponse> {
        #[derive(Serialize)]
        struct Wire<'a> { kind: &'a str, envelope: &'a IntentEnvelope }
        let wire = Wire { kind, envelope: envlp };
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&wire, &mut buf).map_err(|e| AiError::serialization(e.to_string()))?;

        match self.endpoint()? {
            Endpoint::File(path) => {
                if let Some(parent) = path.parent() { tokio::fs::create_dir_all(parent).await.ok(); }
                use tokio::io::AsyncWriteExt;
                let mut f = tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await?;
                f.write_all(&buf).await?;
                f.write_all(b"\n").await?;
                Ok(IntentResponse { status: "ok".into(), preview_id: Some(Self::compute_preview_id(envlp)), commit_id: None, message: Some("file".into()) })
            }
            #[cfg(target_family = "unix")]
            Endpoint::Unix(path) => {
                use tokio::net::UnixStream;
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut stream = UnixStream::connect(path).await.map_err(|e| AiError::io(e.to_string()))?;
                // write len (BE u32) + cbor
                let len = (buf.len() as u32).to_be_bytes();
                stream.write_all(&len).await?;
                stream.write_all(&buf).await?;
                stream.flush().await?;
                // read response
                let mut lbuf = [0u8; 4];
                stream.read_exact(&mut lbuf).await?;
                let rlen = u32::from_be_bytes(lbuf) as usize;
                let mut rbuf = vec![0u8; rlen];
                stream.read_exact(&mut rbuf).await?;
                let resp: IntentResponse = ciborium::de::from_reader(rbuf.as_slice()).map_err(|e| AiError::deserialization(e.to_string()))?;
                Ok(resp)
            }
            #[cfg(target_os = "windows")]
            Endpoint::Pipe(name) => {
                use named_pipe::PipeClient;
                use std::io::{Read, Write};
                // Retry a few times to tolerate race on pipe server creation
                let mut last_err: Option<String> = None;
                let mut attempt = 0;
                let mut stream_opt: Option<PipeClient> = None;
                while attempt < 5 {
                    match PipeClient::connect(&name) {
                        Ok(client) => { stream_opt = Some(client); break; }
                        Err(e) => {
                            last_err = Some(format!("{}", e));
                            std::thread::sleep(std::time::Duration::from_millis(50 * (attempt + 1) as u64));
                            attempt += 1;
                        }
                    }
                }
                let mut stream = stream_opt.ok_or_else(|| AiError::io(format!("pipe open failed: {}", last_err.unwrap_or_else(|| "unknown".into()))))?;
                // write len + cbor
                let len = (buf.len() as u32).to_be_bytes();
                stream.write_all(&len).map_err(|e| AiError::io(e.to_string()))?;
                stream.write_all(&buf).map_err(|e| AiError::io(e.to_string()))?;
                stream.flush().map_err(|e| AiError::io(e.to_string()))?;
                // read response
                let mut lbuf = [0u8; 4];
                stream.read_exact(&mut lbuf).map_err(|e| AiError::io(e.to_string()))?;
                let rlen = u32::from_be_bytes(lbuf) as usize;
                let mut rbuf = vec![0u8; rlen];
                stream.read_exact(&mut rbuf).map_err(|e| AiError::io(e.to_string()))?;
                let resp: IntentResponse = ciborium::de::from_reader(rbuf.as_slice()).map_err(|e| AiError::deserialization(e.to_string()))?;
                Ok(resp)
            }
        }
    }

    /// Send a raw v1 envelope (no kind wrapper) and return raw CBOR response
    pub async fn send_envelope_v1(&self, env: &EnvelopeV1) -> AiResult<Vec<u8>> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(env, &mut buf)
            .map_err(|e| AiError::serialization(e.to_string()))?;
        match self.endpoint()? {
            Endpoint::File(path) => {
                if let Some(parent) = path.parent() { tokio::fs::create_dir_all(parent).await.ok(); }
                use tokio::io::AsyncWriteExt;
                let mut f = tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await?;
                f.write_all(&buf).await?;
                f.write_all(b"\n").await?;
                Ok(Vec::new())
            }
            #[cfg(target_family = "unix")]
            Endpoint::Unix(path) => {
                use tokio::net::UnixStream;
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut stream = UnixStream::connect(path).await.map_err(|e| AiError::io(e.to_string()))?;
                let len = (buf.len() as u32).to_be_bytes();
                stream.write_all(&len).await?;
                stream.write_all(&buf).await?;
                stream.flush().await?;
                let mut lbuf = [0u8; 4];
                stream.read_exact(&mut lbuf).await?;
                let rlen = u32::from_be_bytes(lbuf) as usize;
                let mut rbuf = vec![0u8; rlen];
                stream.read_exact(&mut rbuf).await?;
                Ok(rbuf)
            }
            #[cfg(target_os = "windows")]
            Endpoint::Pipe(name) => {
                use named_pipe::PipeClient;
                use std::io::{Read, Write};
                let client = PipeClient::connect(&name)
                    .map_err(|e| AiError::io(format!("pipe open failed: {}", e)))?;
                let mut stream = client;
                let len = (buf.len() as u32).to_be_bytes();
                stream.write_all(&len).map_err(|e| AiError::io(e.to_string()))?;
                stream.write_all(&buf).map_err(|e| AiError::io(e.to_string()))?;
                stream.flush().map_err(|e| AiError::io(e.to_string()))?;
                let mut lbuf = [0u8; 4];
                stream.read_exact(&mut lbuf).map_err(|e| AiError::io(e.to_string()))?;
                let rlen = u32::from_be_bytes(lbuf) as usize;
                let mut rbuf = vec![0u8; rlen];
                stream.read_exact(&mut rbuf).map_err(|e| AiError::io(e.to_string()))?;
                Ok(rbuf)
            }
        }
    }
}

/// Convenience function: build v1 envelope from name + CBOR params and send.
pub async fn send_intent(_intent_name: &str, params: &serde_cbor::Value) -> AiResult<serde_cbor::Value> {
    let client = IntentClient::from_env().ok_or_else(|| AiError::configuration("INTENT_SOCKET not set"))?;
    // Build payload CBOR bytes deterministically
    let mut payload = Vec::new();
    ciborium::ser::into_writer(params, &mut payload)
        .map_err(|e| AiError::serialization(e.to_string()))?;
    // Captoken: load from env path if set, else empty
    let captoken = if let Ok(tok_path) = std::env::var("AICORE_CAPTOKEN") {
        match std::fs::read(tok_path) { Ok(b) => b, Err(_) => Vec::new() }
    } else { Vec::new() };
    let env = EnvelopeV1 { v: 1, captoken, payload };
    let resp_bytes = client.send_envelope_v1(&env).await?;
    if resp_bytes.is_empty() {
        // to-file path: no response; return empty map
        Ok(serde_cbor::Value::Map(std::collections::BTreeMap::new()))
    } else {
        ciborium::de::from_reader(resp_bytes.as_slice())
            .map_err(|e| AiError::deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_endpoints() {
        #[cfg(target_family = "unix")]
        {
            assert!(matches!(Endpoint::parse("unix:///tmp/sock").unwrap(), Endpoint::Unix(_)));
        }
        #[cfg(target_os = "windows")]
        {
            assert!(matches!(Endpoint::parse("pipe://\\\\.\\pipe\\aetheris-intent").unwrap(), Endpoint::Pipe(_)));
        }
        assert!(matches!(Endpoint::parse("to-file://C:/tmp/out.cbor").unwrap(), Endpoint::File(_)));
    }
}
