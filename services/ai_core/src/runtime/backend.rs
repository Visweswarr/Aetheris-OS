//! Model Runtime Execution Backend Boundary.
//!
//! Defines the RuntimeBackend trait and the DeterministicLocalBackend implementation,
//! separating execution from HEG scheduling and placement planning.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::contracts::sha256_hex;
use crate::error::{AiCoreError, Result};
use crate::runtime::{AcceleratorKind, HegPlan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeBackendKind {
    DeterministicLocal,
    Ollama,
    Onnx,
    Gguf,
    LlamaCppServer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub gpu_support: bool,
    pub npu_support: bool,
    pub supported_operators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExecutionRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Vec<String>,
    pub heg_plan: Option<HegPlan>,
    pub metadata: HashMap<String, String>,
    pub model_id: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExecutionResult {
    pub generated_text: String,
    pub tokens_generated: u32,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, String>,
    pub model_id: Option<String>,
    pub model_source: Option<String>,
    pub model_revision: Option<String>,
    pub remote_execution: bool,
    pub input_hash: String,
    pub output_hash: String,
}

#[async_trait]
pub trait RuntimeBackend: Send + Sync {
    /// Returns the kind of the execution backend.
    fn kind(&self) -> RuntimeBackendKind;

    /// Returns the capabilities of the backend.
    fn capabilities(&self) -> BackendCapabilities;

    /// Executes the request deterministically or via local models.
    async fn execute(&self, request: &RuntimeExecutionRequest) -> Result<RuntimeExecutionResult>;
}

/// A backend that performs local text analysis and returns structured,
/// deterministic responses. It does not load a model, call the network, or
/// claim accelerator execution.
pub struct DeterministicLocalBackend;

impl DeterministicLocalBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RuntimeBackend for DeterministicLocalBackend {
    fn kind(&self) -> RuntimeBackendKind {
        RuntimeBackendKind::DeterministicLocal
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            gpu_support: false,
            npu_support: false,
            supported_operators: vec![
                "embedding".to_string(),
                "prefill".to_string(),
                "decode".to_string(),
                "tool_call".to_string(),
                "memory_lookup".to_string(),
                "browser_assist".to_string(),
                "log_summary".to_string(),
            ],
        }
    }

    async fn execute(&self, request: &RuntimeExecutionRequest) -> Result<RuntimeExecutionResult> {
        if request.prompt.trim().is_empty() {
            return Err(AiCoreError::ValidationError(
                "runtime prompt must not be empty".to_string(),
            ));
        }

        let prompt_lower = request.prompt.to_lowercase();
        let generated_text = if prompt_lower.contains("hello")
            || prompt_lower.contains("hi")
            || prompt_lower.contains("how are you")
        {
            "Hello! This is a deterministic local assistant response. How can I help you today?"
                .to_string()
        } else if prompt_lower.contains("summarize") || prompt_lower.contains("summary") {
            let topic = extract_keywords(&request.prompt, 6).join(", ");
            format!(
                "Local Extractive Summary:\n- Main topic terms: {}\n- Mode: deterministic local backend\n- Data locality: on-device only",
                if topic.is_empty() { "none" } else { &topic }
            )
        } else if prompt_lower.contains("code") || prompt_lower.contains("function") {
            "// Deterministic Local Backend Code Sketch\nfn main() {\n    println!(\"Hello from Polymera OS deterministic local backend!\");\n}"
                .to_string()
        } else if prompt_lower.contains("status") || prompt_lower.contains("system") {
            "Polymera OS AI Subsystem Status:\n- Backend: DeterministicLocalBackend\n- Health: OK\n- Execution: local deterministic boundary\n- Accelerator execution: planned only"
                .to_string()
        } else {
            deterministic_analysis(&request.prompt, request.metadata.get("workload"))
        };

        let generated_text = truncate_to_max_tokens(generated_text, request.max_tokens);
        let tokens_generated = generated_text.split_whitespace().count() as u32;
        let processing_time_ms = logical_processing_time_ms(
            request.prompt.len(),
            tokens_generated,
            request.heg_plan.as_ref(),
        );

        let mut metadata = HashMap::new();
        metadata.insert("backend_kind".to_string(), "deterministic_local".to_string());
        metadata.insert("execution_mode".to_string(), "deterministic_local".to_string());
        metadata.insert("model_loaded".to_string(), "false".to_string());
        metadata.insert("remote_execution".to_string(), "false".to_string());
        let input_hash = sha256_hex(request.prompt.as_bytes());
        let output_hash = sha256_hex(generated_text.as_bytes());
        metadata.insert("prompt_hash".to_string(), input_hash.clone());
        metadata.insert("output_hash".to_string(), output_hash.clone());
        metadata.insert("tokens_generated".to_string(), tokens_generated.to_string());
        metadata.insert(
            "processing_time_ms".to_string(),
            processing_time_ms.to_string(),
        );
        if let Some(plan) = &request.heg_plan {
            metadata.insert("heg_plan_id".to_string(), plan.graph_id.clone());
            metadata.insert("placement_summary".to_string(), placement_summary(plan));
            metadata.insert(
                "heg_deterministic_hash".to_string(),
                plan.deterministic_hash.clone(),
            );
        }

        Ok(RuntimeExecutionResult {
            generated_text,
            tokens_generated,
            processing_time_ms,
            metadata,
            model_id: None,
            model_source: None,
            model_revision: None,
            remote_execution: false,
            input_hash,
            output_hash,
        })
    }
}

/// OpenAI-compatible local llama.cpp server backend.
///
/// This backend only connects to localhost/loopback endpoints. It does not
/// download models or call remote APIs; operators must start llama-server with a
/// reviewed local model artifact before selecting this backend.
pub struct LlamaCppServerBackend {
    endpoint: String,
    model_id: String,
}

impl LlamaCppServerBackend {
    pub fn new(endpoint: impl Into<String>, model_id: impl Into<String>) -> Result<Self> {
        let endpoint = normalize_local_endpoint(&endpoint.into())?;
        let model_id = model_id.into();
        if model_id.trim().is_empty() {
            return Err(AiCoreError::ValidationError(
                "llama.cpp backend requires --model <id>".to_string(),
            ));
        }
        Ok(Self { endpoint, model_id })
    }
}

#[async_trait]
impl RuntimeBackend for LlamaCppServerBackend {
    fn kind(&self) -> RuntimeBackendKind {
        RuntimeBackendKind::LlamaCppServer
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            gpu_support: true,
            npu_support: false,
            supported_operators: vec!["prefill".to_string(), "decode".to_string()],
        }
    }

    async fn execute(&self, request: &RuntimeExecutionRequest) -> Result<RuntimeExecutionResult> {
        if request.prompt.trim().is_empty() {
            return Err(AiCoreError::ValidationError(
                "runtime prompt must not be empty".to_string(),
            ));
        }
        let model_id = request
            .model_id
            .clone()
            .unwrap_or_else(|| self.model_id.clone());
        let endpoint = request
            .endpoint
            .as_deref()
            .map(normalize_local_endpoint)
            .transpose()?
            .unwrap_or_else(|| self.endpoint.clone());
        let url = openai_chat_url(&endpoint);
        let max_tokens = request.max_tokens.unwrap_or(128);
        let temperature = request.temperature.unwrap_or(0.0);
        let body = serde_json::json!({
            "model": model_id,
            "messages": [{"role": "user", "content": request.prompt}],
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": false
        });
        let started = std::time::Instant::now();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|err| AiCoreError::NetworkError(format!("failed to build local llama.cpp client: {err}")))?;
        let response = client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|err| AiCoreError::ServiceUnavailableError(format!("local llama.cpp server unavailable: {err}")))?;
        if !response.status().is_success() {
            return Err(AiCoreError::ExternalServiceError(format!(
                "local llama.cpp server returned HTTP {}",
                response.status()
            )));
        }
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|err| AiCoreError::SerializationError(format!("invalid llama.cpp response JSON: {err}")))?;
        let generated_text = payload["choices"][0]["message"]["content"]
            .as_str()
            .or_else(|| payload["choices"][0]["text"].as_str())
            .ok_or_else(|| AiCoreError::ProtocolError("llama.cpp response missing generated text".to_string()))?
            .to_string();
        let tokens_generated = payload["usage"]["completion_tokens"]
            .as_u64()
            .unwrap_or_else(|| generated_text.split_whitespace().count() as u64) as u32;
        let processing_time_ms = started.elapsed().as_millis() as u64;
        let input_hash = sha256_hex(request.prompt.as_bytes());
        let output_hash = sha256_hex(generated_text.as_bytes());
        let mut metadata = model_backend_metadata(
            "llama_cpp_server",
            "local_openai_compatible",
            &model_id,
            &endpoint,
            request,
            tokens_generated,
            processing_time_ms,
            &input_hash,
            &output_hash,
        );
        metadata.insert("model_loaded".to_string(), "true".to_string());

        Ok(RuntimeExecutionResult {
            generated_text,
            tokens_generated,
            processing_time_ms,
            metadata,
            model_id: Some(model_id),
            model_source: Some("local_llama_cpp_server".to_string()),
            model_revision: request.metadata.get("model_revision").cloned(),
            remote_execution: false,
            input_hash,
            output_hash,
        })
    }
}

/// Local Ollama runtime backend. This is opt-in and fails closed if Ollama is
/// not reachable on localhost.
pub struct OllamaRuntimeBackend {
    endpoint: String,
    model_id: String,
}

impl OllamaRuntimeBackend {
    pub fn new(endpoint: impl Into<String>, model_id: impl Into<String>) -> Result<Self> {
        let endpoint = normalize_local_endpoint(&endpoint.into())?;
        let model_id = model_id.into();
        if model_id.trim().is_empty() {
            return Err(AiCoreError::ValidationError(
                "ollama backend requires --model <id>".to_string(),
            ));
        }
        Ok(Self { endpoint, model_id })
    }
}

#[async_trait]
impl RuntimeBackend for OllamaRuntimeBackend {
    fn kind(&self) -> RuntimeBackendKind {
        RuntimeBackendKind::Ollama
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            gpu_support: true,
            npu_support: false,
            supported_operators: vec!["prefill".to_string(), "decode".to_string()],
        }
    }

    async fn execute(&self, request: &RuntimeExecutionRequest) -> Result<RuntimeExecutionResult> {
        if request.prompt.trim().is_empty() {
            return Err(AiCoreError::ValidationError(
                "runtime prompt must not be empty".to_string(),
            ));
        }
        let model_id = request
            .model_id
            .clone()
            .unwrap_or_else(|| self.model_id.clone());
        let endpoint = request
            .endpoint
            .as_deref()
            .map(normalize_local_endpoint)
            .transpose()?
            .unwrap_or_else(|| self.endpoint.clone());
        let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model_id,
            "prompt": request.prompt,
            "stream": false,
            "options": {
                "temperature": request.temperature.unwrap_or(0.0),
                "num_predict": request.max_tokens.unwrap_or(128)
            }
        });
        let started = std::time::Instant::now();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|err| AiCoreError::NetworkError(format!("failed to build local Ollama client: {err}")))?;
        let response = client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|err| AiCoreError::ServiceUnavailableError(format!("local Ollama server unavailable: {err}")))?;
        if !response.status().is_success() {
            return Err(AiCoreError::ExternalServiceError(format!(
                "local Ollama server returned HTTP {}",
                response.status()
            )));
        }
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|err| AiCoreError::SerializationError(format!("invalid Ollama response JSON: {err}")))?;
        let generated_text = payload["response"]
            .as_str()
            .ok_or_else(|| AiCoreError::ProtocolError("Ollama response missing response field".to_string()))?
            .to_string();
        let tokens_generated = payload["eval_count"]
            .as_u64()
            .unwrap_or_else(|| generated_text.split_whitespace().count() as u64) as u32;
        let processing_time_ms = started.elapsed().as_millis() as u64;
        let input_hash = sha256_hex(request.prompt.as_bytes());
        let output_hash = sha256_hex(generated_text.as_bytes());
        let mut metadata = model_backend_metadata(
            "ollama",
            "local_ollama_generate",
            &model_id,
            &endpoint,
            request,
            tokens_generated,
            processing_time_ms,
            &input_hash,
            &output_hash,
        );
        metadata.insert("model_loaded".to_string(), "true".to_string());

        Ok(RuntimeExecutionResult {
            generated_text,
            tokens_generated,
            processing_time_ms,
            metadata,
            model_id: Some(model_id),
            model_source: Some("local_ollama".to_string()),
            model_revision: request.metadata.get("model_revision").cloned(),
            remote_execution: false,
            input_hash,
            output_hash,
        })
    }
}

fn deterministic_analysis(prompt: &str, workload: Option<&String>) -> String {
    let keywords = extract_keywords(prompt, 8);
    let keyword_text = if keywords.is_empty() {
        "none".to_string()
    } else {
        keywords.join(", ")
    };
    format!(
        "Deterministic Local Analysis:\n- Prompt bytes: {}\n- Prompt words: {}\n- Key terms: {}\n- Workload: {}\n- Execution boundary: RuntimeBackend::DeterministicLocal",
        prompt.len(),
        prompt.split_whitespace().count(),
        keyword_text,
        workload.map(String::as_str).unwrap_or("general")
    )
}

fn extract_keywords(prompt: &str, limit: usize) -> Vec<String> {
    let mut terms = Vec::new();
    for raw in prompt.split(|c: char| !c.is_ascii_alphanumeric()) {
        let term = raw.trim().to_ascii_lowercase();
        if term.len() < 4 {
            continue;
        }
        if matches!(
            term.as_str(),
            "with" | "from" | "that" | "this" | "what" | "please" | "about" | "into"
        ) {
            continue;
        }
        if !terms.contains(&term) {
            terms.push(term);
        }
        if terms.len() >= limit {
            break;
        }
    }
    terms
}

fn truncate_to_max_tokens(text: String, max_tokens: Option<u32>) -> String {
    let Some(max_tokens) = max_tokens else {
        return text;
    };
    let max_tokens = max_tokens as usize;
    if max_tokens == 0 {
        return String::new();
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.len() <= max_tokens {
        text
    } else {
        tokens[..max_tokens].join(" ")
    }
}

fn logical_processing_time_ms(
    prompt_bytes: usize,
    tokens_generated: u32,
    plan: Option<&HegPlan>,
) -> u64 {
    let node_cost = plan.map(|p| p.nodes.len() as u64).unwrap_or(3);
    1 + (prompt_bytes as u64 / 128) + (tokens_generated as u64 / 32) + node_cost
}

fn placement_summary(plan: &HegPlan) -> String {
    plan.decisions
        .iter()
        .map(|decision| {
            format!(
                "{}:{}",
                decision.node_id,
                accelerator_name(decision.assigned)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn accelerator_name(kind: AcceleratorKind) -> &'static str {
    match kind {
        AcceleratorKind::Cpu => "cpu",
        AcceleratorKind::Igpu => "igpu",
        AcceleratorKind::Npu => "npu",
    }
}

fn model_backend_metadata(
    backend_kind: &str,
    execution_mode: &str,
    model_id: &str,
    endpoint: &str,
    request: &RuntimeExecutionRequest,
    tokens_generated: u32,
    processing_time_ms: u64,
    input_hash: &str,
    output_hash: &str,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("backend_kind".to_string(), backend_kind.to_string());
    metadata.insert("execution_mode".to_string(), execution_mode.to_string());
    metadata.insert("model_id".to_string(), model_id.to_string());
    metadata.insert("endpoint".to_string(), endpoint.to_string());
    metadata.insert("remote_execution".to_string(), "false".to_string());
    metadata.insert("prompt_hash".to_string(), input_hash.to_string());
    metadata.insert("output_hash".to_string(), output_hash.to_string());
    metadata.insert("tokens_generated".to_string(), tokens_generated.to_string());
    metadata.insert("processing_time_ms".to_string(), processing_time_ms.to_string());
    if let Some(plan) = &request.heg_plan {
        metadata.insert("heg_plan_id".to_string(), plan.graph_id.clone());
        metadata.insert("placement_summary".to_string(), placement_summary(plan));
        metadata.insert(
            "heg_deterministic_hash".to_string(),
            plan.deterministic_hash.clone(),
        );
    }
    metadata
}

pub fn normalize_local_endpoint(endpoint: &str) -> Result<String> {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AiCoreError::ValidationError(
            "runtime backend endpoint must not be empty".to_string(),
        ));
    }
    let parsed = reqwest::Url::parse(trimmed)
        .map_err(|err| AiCoreError::ValidationError(format!("invalid runtime backend endpoint: {err}")))?;
    if parsed.scheme() != "http" {
        return Err(AiCoreError::ValidationError(
            "runtime backend endpoint must use http on localhost".to_string(),
        ));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| AiCoreError::ValidationError("runtime backend endpoint missing host".to_string()))?;
    if !matches!(host, "localhost" | "127.0.0.1" | "::1") {
        return Err(AiCoreError::ValidationError(format!(
            "runtime backend endpoint must be local-only, got '{host}'"
        )));
    }
    Ok(trimmed.to_string())
}

fn openai_chat_url(endpoint: &str) -> String {
    let trimmed = endpoint.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{trimmed}/chat/completions")
    } else {
        format!("{trimmed}/v1/chat/completions")
    }
}
