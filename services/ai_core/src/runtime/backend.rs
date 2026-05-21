//! Model Runtime Execution Backend Boundary.
//!
//! Defines the RuntimeBackend trait and the DeterministicLocalBackend implementation,
//! separating execution from HEG scheduling and placement planning.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    LlamaCpp,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExecutionResult {
    pub generated_text: String,
    pub tokens_generated: u32,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, String>,
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
        metadata.insert("prompt_hash".to_string(), sha256_hex(request.prompt.as_bytes()));
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
