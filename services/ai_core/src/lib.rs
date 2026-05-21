#![allow(dead_code)]
#![allow(unused)]
#![allow(unused_parens)]
#![allow(clippy::all)]
//! Aetheris AI Core Service Library

pub mod adapter;
pub mod agent;
pub mod assistant_kernel;
pub mod audit;
pub mod browser_assist;
pub mod budget;
pub mod cap;
pub mod contracts;
pub mod cognitive;
pub mod drift;
pub mod error;
pub mod generated;
pub mod intents;
pub mod ipc;
pub mod memory;
pub mod metrics;
pub mod model;
pub mod model_loader;
pub mod notifications;
pub mod orchestrator;
pub mod privacy;
pub mod replay;
pub mod router;
pub mod runtime;
pub mod tools;

pub use adapter::{AdapterBackend, AdapterRegistry};
pub use assistant_kernel::AssistantKernel;
pub use audit::AuditTrail;
pub use budget::{BudgetEngine, BudgetLimits, BudgetSnapshot};
pub use cap::{CapTokenManager, CapabilityCheckResult};
pub use cognitive::{append_ltm_event, summarize_ltm, LearnedState, LearningEvent};
pub use contracts::*;
pub use drift::compare_metrics as compare_drift_metrics;
pub use error::{
    get_error_hint, get_retry_policy_for_error, init_global_error_audit_logger,
    init_global_error_hints_manager, retry_with_backoff, AiCoreError, ErrorAuditLogger, ErrorHint,
    ErrorCategory, ErrorHintsManager, Result, RetryAttempt, RetryContext, RetryPolicy,
    RetryPolicyConfig, XorShiftRng,
};
pub use intents::{
    create_system_intent_manager, PlatformAdapter, SystemActionContext, SystemActionResult,
    SystemIntent, SystemIntentManager,
};
pub use ipc::{
    AiCoreMessage, ChatRequest, ChatResponse, PingRequest, PingResponse, ToolCallRequest,
    ToolCallResponse,
};
pub use memory::{
    default_memory_config, default_redaction_policies, MemoryConfig, MemoryEntry, MemoryStore,
    RedactionPolicy,
};
pub use metrics::{
    default_metrics_config, init_global_metrics_collector, record_ai_plan_generated,
    record_browser_summary, record_capability_denial, record_ltm_event, record_model_inference,
    record_heg_plan, record_model_load, record_request, record_runtime_model_execution,
    record_session, record_tokens, record_tool_call,
    AiCoreMetrics, AiCoreMetricsCollector, MetricsConfig, PrivacyMode,
};
pub use model::{ModelBackendImpl, ModelManager, ModelRequest, ModelResponse};
pub use model_loader::{ModelPolicy, SecureModelLoader};
pub use notifications::{
    create_notification_manager, NotificationAction, NotificationActionRequest,
    NotificationActionResponse, NotificationManager,
};
pub use privacy::PrivacyEngine;
pub use replay::{compare_replay, ReplayRecorder};
pub use router::{ConversationTurn, Intent, PromptRouter, RenderContext, TemplateVariables};
pub use tools::{Tool, ToolImplementationType, ToolRegistry, ToolRegistryStats, ToolResult};

use std::sync::Arc;

/// AI Core Service configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub socket: std::path::PathBuf,
    pub model_config: std::path::PathBuf,
    pub tool_config: std::path::PathBuf,
    pub cap_config: std::path::PathBuf,
    pub templates_dir: std::path::PathBuf,
    pub memory_dir: std::path::PathBuf,
    pub tools_dir: std::path::PathBuf,
    pub log_level: String,
    pub deterministic: bool,
    pub audit_log_dir: std::path::PathBuf,
    pub max_sessions: usize,
    pub request_timeout: u64,
    pub metrics_config: MetricsConfig,
}

/// AI Core Service main struct
pub struct AiCoreService {
    config: Config,
    model_manager: Arc<ModelManager>,
    tool_registry: Arc<ToolRegistry>,
    cap_token_manager: Arc<CapTokenManager>,
    prompt_router: Arc<PromptRouter>,
    memory_store: Arc<MemoryStore>,
    notification_manager: Arc<NotificationManager>,
    metrics_collector: Arc<AiCoreMetricsCollector>,
    ipc_server: Arc<ipc::IpcServer>,
}

impl AiCoreService {
    pub async fn new(config: Config) -> Result<Self> {
        // Initializing AI Core Service

        let model_manager =
            Arc::new(ModelManager::new(&config.model_config, config.deterministic).await?);
        let tool_registry = Arc::new(ToolRegistry::new(&config.tool_config).await?);
        let cap_token_manager = Arc::new(CapTokenManager::new(&config.cap_config).await?);
        let prompt_router = Arc::new(PromptRouter::new(
            config.templates_dir.clone(),
            config.deterministic,
        ));
        prompt_router.initialize().await?;

        let memory_config = default_memory_config(config.memory_dir.clone());
        let memory_store = Arc::new(MemoryStore::new(memory_config)?);
        memory_store.initialize().await?;

        let notification_manager = Arc::new(create_notification_manager(tool_registry.clone()));

        let mut metrics_collector = AiCoreMetricsCollector::new(config.metrics_config.clone());
        metrics_collector.start().await?;
        let metrics_collector = Arc::new(metrics_collector);

        init_global_metrics_collector(config.metrics_config.clone())?;
        init_global_error_audit_logger();

        let ipc_server = Arc::new(
            ipc::IpcServer::new(
                &config.socket,
                model_manager.clone(),
                tool_registry.clone(),
                cap_token_manager.clone(),
                prompt_router.clone(),
                memory_store.clone(),
                notification_manager.clone(),
                config.max_sessions,
                config.request_timeout,
            )
            .await?,
        );

        Ok(Self {
            config,
            model_manager,
            tool_registry,
            cap_token_manager,
            prompt_router,
            memory_store,
            notification_manager,
            metrics_collector,
            ipc_server,
        })
    }

    pub async fn start(&self) -> Result<()> {
        // Starting AI Core Service
        self.ipc_server.start().await?;
        // AI Core Service started successfully
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        // Stopping AI Core Service
        self.ipc_server.stop().await?;
        // AI Core Service stopped
        Ok(())
    }
}
