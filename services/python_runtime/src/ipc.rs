use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PythonRuntimeError {
    #[error("AI Core endpoint is not configured; set POLYMERA_AI_CORE_ENDPOINT")]
    MissingEndpoint,
    #[error("AI Core transport is not active for endpoint '{endpoint}'")]
    TransportInactive { endpoint: String },
    #[error("serialization failed: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, PythonRuntimeError>;

#[derive(Debug, Clone)]
pub struct AiCoreClient {
    endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCoreEnvelope {
    pub service: String,
    pub method: String,
    pub payload: serde_json::Value,
}

impl AiCoreClient {
    pub fn from_env() -> Self {
        Self {
            endpoint: std::env::var("POLYMERA_AI_CORE_ENDPOINT").ok(),
        }
    }

    pub fn encode_plan_request(&self, goal: String, session_id: Option<String>, user_id: Option<String>) -> Result<Vec<u8>> {
        let envelope = AiCoreEnvelope {
            service: "ai_core".to_string(),
            method: "PlanRequest".to_string(),
            payload: serde_json::json!({
                "goal": goal,
                "context": null,
                "constraints": [],
                "session_id": session_id,
                "user_id": user_id,
            }),
        };
        serde_json::to_vec(&envelope).map_err(|e| PythonRuntimeError::Serialization(e.to_string()))
    }

    pub fn encode_tool_call(&self, tool_name: String, parameters: serde_json::Value) -> Result<Vec<u8>> {
        let envelope = AiCoreEnvelope {
            service: "ai_core".to_string(),
            method: "ToolCallRequest".to_string(),
            payload: serde_json::json!({
                "tool_name": tool_name,
                "parameters": parameters,
            }),
        };
        serde_json::to_vec(&envelope).map_err(|e| PythonRuntimeError::Serialization(e.to_string()))
    }

    pub fn send_message(&self, _service: String, _payload: Vec<u8>) -> Result<Vec<u8>> {
        let endpoint = self.endpoint.clone().ok_or(PythonRuntimeError::MissingEndpoint)?;
        Err(PythonRuntimeError::TransportInactive { endpoint })
    }
}
