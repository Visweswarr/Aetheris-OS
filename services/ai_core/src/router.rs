//! Prompt Router & System Instructions

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};


use crate::error::{AiCoreError, Result};
use crate::ipc::ChatRequest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    Chat, Summarize, Command, Code, Creative, Analysis, Translate, Qa, ComplexWorkflow,
    SystemControl(String), AppControl(String), Settings(String), Hardware(String), Custom(String),
}

impl From<&str> for Intent {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "chat" => Intent::Chat, "summarize" => Intent::Summarize, "command" => Intent::Command,
            "code" => Intent::Code, "creative" => Intent::Creative, "analysis" => Intent::Analysis,
            "translate" => Intent::Translate, "qa" => Intent::Qa, "complex_workflow" => Intent::ComplexWorkflow,
            _ => Intent::Custom(s.to_string()),
        }
    }
}

impl ToString for Intent {
    fn to_string(&self) -> String {
        match self {
            Intent::Chat => "chat".to_string(), Intent::Summarize => "summarize".to_string(),
            Intent::Command => "command".to_string(), Intent::Code => "code".to_string(),
            Intent::Creative => "creative".to_string(), Intent::Analysis => "analysis".to_string(),
            Intent::Translate => "translate".to_string(), Intent::Qa => "qa".to_string(),
            Intent::ComplexWorkflow => "complex_workflow".to_string(),
            Intent::SystemControl(s) => format!("system_control:{}", s),
            Intent::AppControl(s) => format!("app_control:{}", s),
            Intent::Settings(s) => format!("settings:{}", s),
            Intent::Hardware(s) => format!("hardware:{}", s),
            Intent::Custom(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariables {
    pub user_message: String,
    pub conversation_history: Vec<ConversationTurn>,
    pub timestamp: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub conversation_id: Option<String>,
    pub language: Option<String>,
    pub model_name: Option<String>,
    pub system_context: Option<String>,
    pub custom_vars: HashMap<String, String>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct RenderContext {
    pub intent: Intent,
    pub variables: TemplateVariables,
    pub template_path: PathBuf,
    pub deterministic: bool,
}

pub struct PromptRouter {
    templates_dir: PathBuf,
    templates: Arc<RwLock<HashMap<Intent, String>>>,
    deterministic: bool,
    default_seed: Option<u64>,
}

impl PromptRouter {
    pub fn new(templates_dir: PathBuf, deterministic: bool) -> Self {
        Self {
            templates_dir,
            templates: Arc::new(RwLock::new(HashMap::new())),
            deterministic,
            default_seed: if deterministic { Some(42) } else { None },
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Initializing prompt router
        
        let mut templates = self.templates.write().await;
        templates.insert(Intent::Chat, "You are a helpful assistant. {{user_message}}".to_string());
        templates.insert(Intent::Summarize, "Summarize the following: {{user_message}}".to_string());
        templates.insert(Intent::Code, "Generate code for: {{user_message}}".to_string());
        
        // Prompt router initialized
        Ok(())
    }

    pub fn detect_intent(&self, request: &ChatRequest) -> Intent {
        if let Some(metadata) = &request.metadata {
            if let Some(intent_str) = metadata.get("intent") {
                return Intent::from(intent_str.as_str());
            }
        }
        
        let message = request.message.to_lowercase();
        if message.contains("summarize") || message.contains("summary") { Intent::Summarize }
        else if message.contains("code") || message.contains("function") { Intent::Code }
        else if message.contains("translate") { Intent::Translate }
        else if message.contains("analyze") || message.contains("explain") { Intent::Analysis }
        else if message.contains("?") { Intent::Qa }
        else { Intent::Chat }
    }

    pub async fn route_request(&self, request: &ChatRequest) -> Result<RenderContext> {
        let intent = self.detect_intent(request);
        let template_path = self.templates_dir.join(format!("{}.tmpl", intent.to_string()));
        let variables = self.extract_variables(request).await?;
        
        Ok(RenderContext { intent, variables, template_path, deterministic: self.deterministic })
    }

    async fn extract_variables(&self, request: &ChatRequest) -> Result<TemplateVariables> {
        let timestamp = if self.deterministic {
            "2024-01-01T00:00:00Z".to_string()
        } else {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
        };

        let conversation_history = request.conversation_history.as_ref()
            .map(|h| h.iter().map(|t| ConversationTurn {
                role: t.role.clone(), content: t.content.clone(), timestamp: timestamp.clone(),
            }).collect())
            .unwrap_or_default();

        let custom_vars = request.metadata.clone().unwrap_or_default();

        Ok(TemplateVariables {
            user_message: request.message.clone(),
            conversation_history,
            timestamp,
            user_id: request.user_id.clone(),
            session_id: request.session_id.clone(),
            conversation_id: request.conversation_id.clone(),
            language: request.language.clone(),
            model_name: None,
            system_context: None,
            custom_vars,
            seed: self.default_seed,
        })
    }

    pub async fn render_template(&self, context: &RenderContext) -> Result<String> {
        let templates = self.templates.read().await;
        let template = templates.get(&context.intent)
            .ok_or_else(|| AiCoreError::Config(format!("No template for intent: {:?}", context.intent)))?;
        
        let mut rendered = template.clone();
        rendered = rendered.replace("{{user_message}}", &context.variables.user_message);
        rendered = rendered.replace("{{timestamp}}", &context.variables.timestamp);
        
        Ok(rendered)
    }

    pub async fn get_available_intents(&self) -> Vec<Intent> {
        self.templates.read().await.keys().cloned().collect()
    }

    pub async fn has_template(&self, intent: &Intent) -> bool {
        self.templates.read().await.contains_key(intent)
    }

    pub async fn reload_templates(&self) -> Result<()> {
        self.initialize().await
    }
}
