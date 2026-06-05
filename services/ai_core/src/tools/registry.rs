//! Tool Registry for AI Core Service

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_driver_host::summarizer::{LogSummaryRequest, SummarizerHost};

use crate::browser_assist::{classify_page, summarize_text};
use crate::error::{AiCoreError, Result};
use crate::ipc::ToolCallRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters_schema: Value,
    pub required_capabilities: Vec<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub implementation_type: ToolImplementationType,
    pub timeout_seconds: u64,
    pub requires_confirmation: bool,
    /// Whether this tool performs destructive operations (delete, overwrite, etc.).
    /// When true, the executor always requires explicit approval before running.
    /// This replaces the text-pattern inference that previously guessed destructiveness
    /// from intent text — the tool itself is the authority on whether it is destructive.
    pub destructive: bool,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolImplementationType {
    Builtin,
    External,
    Webhook,
    Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_id: String,
    pub success: bool,
    pub result: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolRegistryStats {
    pub total_tools: usize,
    pub builtin_tools: usize,
    pub external_tools: usize,
    pub webhook_tools: usize,
    pub script_tools: usize,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Tool>>>,
    stats: Arc<RwLock<ToolRegistryStats>>,
}

impl ToolRegistry {
    pub async fn new(_config_path: &std::path::Path) -> Result<Self> {
        let registry = Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ToolRegistryStats::default())),
        };
        registry.initialize().await?;
        Ok(registry)
    }

    pub fn new_mock() -> Self {
        let mut tools = HashMap::new();
        let echo_tool = Self::builtin_echo_tool();
        tools.insert(echo_tool.id.clone(), echo_tool);
        let log_summarizer = Self::builtin_log_summarizer_tool();
        tools.insert(log_summarizer.id.clone(), log_summarizer);
        let browser_summarize = Self::builtin_browser_summarize_tool();
        tools.insert(browser_summarize.id.clone(), browser_summarize);
        let browser_classify = Self::builtin_browser_classify_tool();
        tools.insert(browser_classify.id.clone(), browser_classify);
        let local_llm = Self::builtin_local_llm_summarizer_tool();
        tools.insert(local_llm.id.clone(), local_llm);
        Self {
            tools: Arc::new(RwLock::new(tools)),
            stats: Arc::new(RwLock::new(ToolRegistryStats {
                total_tools: 5,
                builtin_tools: 5,
                ..ToolRegistryStats::default()
            })),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Initializing tool registry
        self.register_builtin_tools().await?;
        Ok(())
    }

    async fn register_builtin_tools(&self) -> Result<()> {
        self.register_tool(Self::builtin_echo_tool()).await?;
        self.register_tool(Self::builtin_log_summarizer_tool())
            .await?;
        self.register_tool(Self::builtin_browser_summarize_tool())
            .await?;
        self.register_tool(Self::builtin_browser_classify_tool())
            .await?;
        self.register_tool(Self::builtin_local_llm_summarizer_tool())
            .await?;
        Ok(())
    }

    fn builtin_echo_tool() -> Tool {
        Tool {
            id: "echo".to_string(),
            name: "echo".to_string(),
            description: "Echo back the input".to_string(),
            version: "1.0.0".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"},
                    "intent_fragment": {"type": "string"},
                    "step_index": {"type": "integer"}
                },
                "required": ["message"]
            }),
            required_capabilities: vec![],
            category: "utility".to_string(),
            tags: vec!["utility".to_string()],
            implementation_type: ToolImplementationType::Builtin,
            timeout_seconds: 30,
            requires_confirmation: false,
            destructive: false,
            metadata: HashMap::new(),
        }
    }

    fn builtin_log_summarizer_tool() -> Tool {
        Tool {
            id: "log_summarizer".to_string(),
            name: "log_summarizer".to_string(),
            description: "Capability-gated deterministic log summarizer via wasm_driver host \
                          fallback"
                .to_string(),
            version: "1.0.0".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "max_bytes": {"type": "integer"},
                    "component_path": {"type": "string"}
                },
                "required": ["path"]
            }),
            required_capabilities: vec!["fs.read".to_string(), "ai.summarize".to_string()],
            category: "ai".to_string(),
            tags: vec![
                "logs".to_string(),
                "summary".to_string(),
                "wasm_driver".to_string(),
            ],
            implementation_type: ToolImplementationType::Builtin,
            timeout_seconds: 30,
            requires_confirmation: true,
            destructive: false,
            metadata: HashMap::from([
                (
                    "metric".to_string(),
                    serde_json::json!("ai_log_summaries_total"),
                ),
                (
                    "wasm_driver_mode".to_string(),
                    serde_json::json!("deterministic-fallback"),
                ),
            ]),
        }
    }

    fn builtin_browser_summarize_tool() -> Tool {
        Tool {
            id: "browser_summarize".to_string(),
            name: "browser_summarize".to_string(),
            description: "Capability-gated deterministic local page/text summarizer".to_string(),
            version: "1.0.0".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "path": {"type": "string"},
                    "max_chars": {"type": "integer"},
                    "max_bytes": {"type": "integer"}
                }
            }),
            required_capabilities: vec!["ai.browser.summarize".to_string()],
            category: "ai".to_string(),
            tags: vec!["browser".to_string(), "summary".to_string(), "local".to_string()],
            implementation_type: ToolImplementationType::Builtin,
            timeout_seconds: 30,
            requires_confirmation: true,
            destructive: false,
            metadata: HashMap::from([(
                "metric".to_string(),
                serde_json::json!("ai_browser_summaries_total"),
            )]),
        }
    }

    fn builtin_browser_classify_tool() -> Tool {
        Tool {
            id: "browser_classify_page".to_string(),
            name: "browser_classify_page".to_string(),
            description: "Capability-gated deterministic local URL/page risk classifier".to_string(),
            version: "1.0.0".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string"},
                    "text": {"type": "string"},
                    "path": {"type": "string"},
                    "max_bytes": {"type": "integer"}
                },
                "required": ["url"]
            }),
            required_capabilities: vec!["ai.browser.classify".to_string()],
            category: "ai".to_string(),
            tags: vec!["browser".to_string(), "security".to_string(), "local".to_string()],
            implementation_type: ToolImplementationType::Builtin,
            timeout_seconds: 30,
            requires_confirmation: true,
            destructive: false,
            metadata: HashMap::new(),
        }
    }

    pub async fn register_tool(&self, tool: Tool) -> Result<()> {
        let tool_id = tool.id.clone();
        {
            let mut tools = self.tools.write().await;
            tools.insert(tool_id.clone(), tool);
        }
        self.update_stats().await;
        // Registered tool
        Ok(())
    }

    pub async fn execute_tool(&self, request: &ToolCallRequest) -> Result<ToolResult> {
        let start = std::time::Instant::now();
        let tool_id = &request.tool_name;

        let _tool = {
            let tools = self.tools.read().await;
            tools
                .get(tool_id)
                .cloned()
                .ok_or_else(|| AiCoreError::ToolNotFound(tool_id.clone()))?
        };

        let result = match tool_id.as_str() {
            "log_summarizer" => self.execute_log_summarizer(request, start).await?,
            "browser_summarize" => self.execute_browser_summarize(request, start).await?,
            "browser_classify_page" => self.execute_browser_classify(request, start).await?,
            "local_llm_summarizer" => self.execute_local_llm_summarizer(request, start).await?,
            _ => ToolResult {
                    tool_id: tool_id.clone(),
                    success: true,
                    result: Some(serde_cbor::to_vec(&request.parameters).unwrap_or_default()),
                    error_message: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    metadata: HashMap::new(),
                },
        };

        self.update_execution_stats(result.execution_time_ms, result.success)
            .await;
        Ok(result)
    }

    async fn execute_log_summarizer(
        &self,
        request: &ToolCallRequest,
        start: std::time::Instant,
    ) -> Result<ToolResult> {
        let path = request
            .parameters
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AiCoreError::InvalidInput(
                    "log_summarizer requires string parameter 'path'".to_string(),
                )
            })?;
        let max_bytes = request
            .parameters
            .get("max_bytes")
            .and_then(Value::as_u64)
            .unwrap_or(2 * 1024 * 1024);
        let path = PathBuf::from(path);
        let metadata = tokio::fs::metadata(&path).await.map_err(|error| {
            AiCoreError::IoError(format!(
                "cannot stat log file '{}': {}",
                path.display(),
                error
            ))
        })?;
        if metadata.len() > max_bytes {
            return Err(AiCoreError::ResourceError(format!(
                "log file '{}' is {} bytes, above max_bytes {}",
                path.display(),
                metadata.len(),
                max_bytes
            )));
        }
        let text = tokio::fs::read_to_string(&path).await.map_err(|error| {
            AiCoreError::IoError(format!(
                "cannot read log file '{}': {}",
                path.display(),
                error
            ))
        })?;
        let component_path = request
            .parameters
            .get("component_path")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);
        let summary = SummarizerHost::new().summarize(LogSummaryRequest {
            source_path: path.clone(),
            text,
            component_path,
        }).await;
        crate::metrics::record_log_summary();
        let payload = serde_json::json!({
            "source_path": summary.source_path,
            "mode": summary.mode,
            "line_count": summary.line_count,
            "error_count": summary.error_count,
            "warning_count": summary.warning_count,
            "info_count": summary.info_count,
            "top_terms": summary.top_terms,
            "summary": summary.summary,
            "metric": "ai_log_summaries_total"
        });
        Ok(ToolResult {
            tool_id: request.tool_name.clone(),
            success: true,
            result: Some(serde_json::to_vec_pretty(&payload)?),
            error_message: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
            metadata: HashMap::from([
                (
                    "metric".to_string(),
                    serde_json::json!("ai_log_summaries_total"),
                ),
                ("mode".to_string(), payload["mode"].clone()),
            ]),
        })
    }

    async fn execute_browser_summarize(
        &self,
        request: &ToolCallRequest,
        start: std::time::Instant,
    ) -> Result<ToolResult> {
        let text = read_text_or_parameter(&request.parameters, "browser_summarize").await?;
        let max_chars = request
            .parameters
            .get("max_chars")
            .and_then(Value::as_u64)
            .unwrap_or(800)
            .min(16_000) as usize;
        let summary = summarize_text(&text, max_chars);
        crate::metrics::record_browser_summary();
        let payload = serde_json::json!({
            "summary": summary.summary,
            "metric": "ai_browser_summaries_total",
            "input_chars": text.chars().count(),
            "max_chars": max_chars
        });
        Ok(ToolResult {
            tool_id: request.tool_name.clone(),
            success: true,
            result: Some(serde_json::to_vec_pretty(&payload)?),
            error_message: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
            metadata: HashMap::from([(
                "metric".to_string(),
                serde_json::json!("ai_browser_summaries_total"),
            )]),
        })
    }

    async fn execute_browser_classify(
        &self,
        request: &ToolCallRequest,
        start: std::time::Instant,
    ) -> Result<ToolResult> {
        let url = request
            .parameters
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AiCoreError::InvalidInput(
                    "browser_classify_page requires string parameter 'url'".to_string(),
                )
            })?;
        let text = read_optional_text_or_parameter(&request.parameters, "browser_classify_page")
            .await?
            .unwrap_or_default();
        let classification = classify_page(url, &text);
        let payload = serde_json::json!({
            "url": url,
            "suspicious": classification.suspicious,
            "reasons": classification.reasons,
            "input_chars": text.chars().count()
        });
        Ok(ToolResult {
            tool_id: request.tool_name.clone(),
            success: true,
            result: Some(serde_json::to_vec_pretty(&payload)?),
            error_message: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
            metadata: HashMap::new(),
        })
    }

    async fn execute_local_llm_summarizer(
        &self,
        request: &ToolCallRequest,
        start: std::time::Instant,
    ) -> Result<ToolResult> {
        let model = request
            .parameters
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("qwen2.5:7b");
        let text = read_text_or_parameter(&request.parameters, "local_llm_summarizer").await?;

        // Fail-closed: attempt HTTP to Ollama; return explicit error if unreachable.
        let ollama_host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AiCoreError::IoError(format!("HTTP client error: {}", e)))?;
        let body = serde_json::json!({
            "model": model,
            "prompt": format!("Summarize the following text concisely:\n\n{}", text),
            "stream": false
        });
        let response = client
            .post(format!("{}/api/generate", ollama_host))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                crate::metrics::record_capability_denial();
                AiCoreError::IoError(format!(
                    "Ollama daemon unreachable at {}: {}. \
                     The local LLM summarizer requires a running Ollama instance. \
                     Do NOT silently fall back — the user needs to know their model didn't run.",
                    ollama_host, e
                ))
            })?;
        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(AiCoreError::IoError(format!(
                "Ollama returned HTTP {}: {}",
                status, body_text
            )));
        }
        let resp_json: Value = response
            .json()
            .await
            .map_err(|e| AiCoreError::IoError(format!("Ollama response parse error: {}", e)))?;
        let summary = resp_json
            .get("response")
            .and_then(Value::as_str)
            .unwrap_or("(no response from model)")
            .to_string();
        crate::metrics::record_llm_call();
        let payload = serde_json::json!({
            "model": model,
            "summary": summary,
            "metric": "ai_llm_calls_total",
            "ollama_host": ollama_host
        });
        Ok(ToolResult {
            tool_id: request.tool_name.clone(),
            success: true,
            result: Some(serde_json::to_vec_pretty(&payload)?),
            error_message: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
            metadata: HashMap::from([
                ("metric".to_string(), serde_json::json!("ai_llm_calls_total")),
                ("model".to_string(), serde_json::json!(model)),
            ]),
        })
    }

    fn builtin_local_llm_summarizer_tool() -> Tool {
        Tool {
            id: "local_llm_summarizer".to_string(),
            name: "local_llm_summarizer".to_string(),
            description: "Summarize text using a local LLM via Ollama. Fail-closed: returns \
                          an explicit error if the Ollama daemon is unreachable."
                .to_string(),
            version: "1.0.0".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "path": {"type": "string"},
                    "model": {"type": "string", "default": "qwen2.5:7b"},
                    "max_bytes": {"type": "integer"}
                }
            }),
            required_capabilities: vec![
                "ai.summarize".to_string(),
                "local_llm:summarize".to_string(),
            ],
            category: "ai".to_string(),
            tags: vec![
                "llm".to_string(),
                "ollama".to_string(),
                "summary".to_string(),
                "local".to_string(),
            ],
            implementation_type: ToolImplementationType::Builtin,
            timeout_seconds: 60,
            requires_confirmation: true,
            destructive: false,
            metadata: HashMap::from([
                (
                    "metric".to_string(),
                    serde_json::json!("ai_llm_calls_total"),
                ),
                (
                    "fail_mode".to_string(),
                    serde_json::json!("fail-closed"),
                ),
            ]),
        }
    }

    pub async fn execute_tool_with_params(
        &self,
        tool_name: &str,
        parameters: &[u8],
    ) -> Result<Vec<u8>> {
        let call_id = format!(
            "call-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let request = ToolCallRequest {
            tool_name: tool_name.to_string(),
            call_id,
            parameters: serde_json::from_slice(parameters).unwrap_or_default(),
            arguments: String::new(),
            cap_token: None,
            user_id: None,
            session_id: None,
        };
        let result = self.execute_tool(&request).await?;
        Ok(result.result.unwrap_or_default())
    }

    async fn update_stats(&self) {
        let tools = self.tools.read().await;
        let mut stats = self.stats.write().await;
        stats.total_tools = tools.len();
        stats.builtin_tools = tools
            .values()
            .filter(|t| t.implementation_type == ToolImplementationType::Builtin)
            .count();
        stats.external_tools = tools
            .values()
            .filter(|t| t.implementation_type == ToolImplementationType::External)
            .count();
        stats.webhook_tools = tools
            .values()
            .filter(|t| t.implementation_type == ToolImplementationType::Webhook)
            .count();
        stats.script_tools = tools
            .values()
            .filter(|t| t.implementation_type == ToolImplementationType::Script)
            .count();
    }

    async fn update_execution_stats(&self, execution_time_ms: u64, success: bool) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }
        let total_time = stats.average_execution_time_ms * (stats.total_executions - 1) as f64;
        stats.average_execution_time_ms =
            (total_time + execution_time_ms as f64) / stats.total_executions as f64;
    }

    pub async fn list_tools(&self) -> Vec<Tool> {
        self.tools.read().await.values().cloned().collect()
    }

    pub async fn get_tool(&self, tool_id: &str) -> Option<Tool> {
        self.tools.read().await.get(tool_id).cloned()
    }

    pub async fn has_tool(&self, tool_id: &str) -> bool {
        self.tools.read().await.contains_key(tool_id)
    }

    pub async fn validate_parameters(&self, tool_id: &str, parameters: &Value) -> Result<()> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(tool_id)
            .ok_or_else(|| AiCoreError::ToolNotFound(tool_id.to_string()))?;

        if !parameters.is_object() {
            return Err(AiCoreError::InvalidInput(format!(
                "parameters for tool '{}' must be a JSON object",
                tool_id
            )));
        }

        if let Some(required) = tool
            .parameters_schema
            .get("required")
            .and_then(Value::as_array)
        {
            for field in required.iter().filter_map(Value::as_str) {
                if parameters.get(field).is_none() {
                    return Err(AiCoreError::InvalidInput(format!(
                        "parameters for tool '{}' are missing required field '{}'",
                        tool_id, field
                    )));
                }
            }
        }

        Ok(())
    }

    pub async fn get_stats(&self) -> ToolRegistryStats {
        self.stats.read().await.clone()
    }
}

async fn read_text_or_parameter(params: &Value, tool_name: &str) -> Result<String> {
    if let Some(text) = params.get("text").and_then(Value::as_str) {
        return Ok(text.to_string());
    }
    read_optional_text_or_parameter(params, tool_name)
        .await?
        .ok_or_else(|| {
            AiCoreError::InvalidInput(format!(
                "{tool_name} requires either string parameter 'text' or 'path'"
            ))
        })
}

async fn read_optional_text_or_parameter(params: &Value, tool_name: &str) -> Result<Option<String>> {
    let Some(path) = params.get("path").and_then(Value::as_str) else {
        return Ok(params.get("text").and_then(Value::as_str).map(str::to_string));
    };
    let max_bytes = params
        .get("max_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(2 * 1024 * 1024);
    let path = PathBuf::from(path);
    let metadata = tokio::fs::metadata(&path).await.map_err(|error| {
        AiCoreError::IoError(format!(
            "{tool_name} cannot stat file '{}': {}",
            path.display(),
            error
        ))
    })?;
    if metadata.len() > max_bytes {
        return Err(AiCoreError::ResourceError(format!(
            "{tool_name} input file '{}' is {} bytes, above max_bytes {}",
            path.display(),
            metadata.len(),
            max_bytes
        )));
    }
    tokio::fs::read_to_string(&path)
        .await
        .map(Some)
        .map_err(|error| {
            AiCoreError::IoError(format!(
                "{tool_name} cannot read file '{}': {}",
                path.display(),
                error
            ))
        })
}
