//! Tool Registry for AI Core Service

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;


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
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolImplementationType { Builtin, External, Webhook, Script }

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
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ToolRegistryStats::default())),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Initializing tool registry
        self.register_builtin_tools().await?;
        Ok(())
    }

    async fn register_builtin_tools(&self) -> Result<()> {
        let echo_tool = Tool {
            id: "echo".to_string(),
            name: "echo".to_string(),
            description: "Echo back the input".to_string(),
            version: "1.0.0".to_string(),
            parameters_schema: serde_json::json!({"type": "object", "properties": {"message": {"type": "string"}}}),
            required_capabilities: vec![],
            category: "utility".to_string(),
            tags: vec!["utility".to_string()],
            implementation_type: ToolImplementationType::Builtin,
            timeout_seconds: 30,
            requires_confirmation: false,
            metadata: HashMap::new(),
        };
        self.register_tool(echo_tool).await?;
        Ok(())
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
        
        let tools = self.tools.read().await;
        let _tool = tools.get(tool_id)
            .ok_or_else(|| AiCoreError::ToolNotFound(tool_id.clone()))?;
        
        // Mock execution
        let result = ToolResult {
            tool_id: tool_id.clone(),
            success: true,
            result: Some(serde_cbor::to_vec(&request.parameters).unwrap_or_default()),
            error_message: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
            metadata: HashMap::new(),
        };
        
        self.update_execution_stats(result.execution_time_ms, result.success).await;
        Ok(result)
    }

    pub async fn execute_tool_with_params(&self, tool_name: &str, parameters: &[u8]) -> Result<Vec<u8>> {
        let call_id = format!("call-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
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
        stats.builtin_tools = tools.values().filter(|t| t.implementation_type == ToolImplementationType::Builtin).count();
        stats.external_tools = tools.values().filter(|t| t.implementation_type == ToolImplementationType::External).count();
        stats.webhook_tools = tools.values().filter(|t| t.implementation_type == ToolImplementationType::Webhook).count();
        stats.script_tools = tools.values().filter(|t| t.implementation_type == ToolImplementationType::Script).count();
    }

    async fn update_execution_stats(&self, execution_time_ms: u64, success: bool) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        if success { stats.successful_executions += 1; } else { stats.failed_executions += 1; }
        let total_time = stats.average_execution_time_ms * (stats.total_executions - 1) as f64;
        stats.average_execution_time_ms = (total_time + execution_time_ms as f64) / stats.total_executions as f64;
    }

    pub async fn list_tools(&self) -> Vec<Tool> {
        self.tools.read().await.values().cloned().collect()
    }

    pub async fn get_tool(&self, tool_id: &str) -> Option<Tool> {
        self.tools.read().await.get(tool_id).cloned()
    }

    pub async fn get_stats(&self) -> ToolRegistryStats {
        self.stats.read().await.clone()
    }
}
