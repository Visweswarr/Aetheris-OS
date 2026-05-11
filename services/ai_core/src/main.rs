//! Aetheris AI Core Service

use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc;

use aetheris_ai_core::error::Result;
use aetheris_ai_core::cap::CapTokenManager;
use aetheris_ai_core::runtime::RuntimeManager;
use aetheris_ai_core::tools::registry::ToolRegistry;
use aetheris_ai_core::tools::stt::{SpeechToTextTool, SttConfig};
use aetheris_ai_core::router::PromptRouter;
use aetheris_ai_core::intents::create_system_intent_manager;
use aetheris_ai_core::agent::{TaskPlanner, StepExecutor, PolicyEnforcer};
use aetheris_ai_core::orchestrator::MultiModalOrchestrator;

async fn wait_for_shutdown() {
    let ctrl_c = async { signal::ctrl_c().await.expect("Failed to install Ctrl+C handler"); };
    #[cfg(unix)]
    let terminate = async { signal::unix::signal(signal::unix::SignalKind::terminate()).expect("Failed").recv().await; };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {} _ = terminate => {} }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _socket = PathBuf::from("/run/aetheris/ai.sock");
    let cap_config = PathBuf::from("/etc/aetheris/ai_core/cap_tokens.toml");
    let tool_config = PathBuf::from("/etc/aetheris/ai_core/tools.toml");
    let templates_dir = PathBuf::from("/etc/aetheris/ai_core/templates");
    let max_sessions = 100usize;
    let deterministic = false;
    
    eprintln!("Starting Aetheris AI Core Service");
    
    let cap_token_manager = Arc::new(CapTokenManager::new(&cap_config).await?);
    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new(&tool_config).await?);
    let prompt_router = Arc::new(PromptRouter::new(templates_dir.clone(), deterministic));
    prompt_router.initialize().await?;
    let system_intent_manager = Arc::new(create_system_intent_manager(cap_token_manager.clone())?);
    let mut stt = SpeechToTextTool::new(SttConfig::default())?;
    stt.initialize(None).await?;
    let stt_tool = Arc::new(stt);
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_token_manager.clone()));
    let task_planner = Arc::new(TaskPlanner::new(runtime_manager.clone(), tool_registry.clone()));
    let step_executor = Arc::new(StepExecutor::new(tool_registry.clone(), policy_enforcer.clone()));
    
    let (tx, rx) = mpsc::channel(max_sessions);
    let orchestrator = MultiModalOrchestrator::new(rx, prompt_router, system_intent_manager, runtime_manager, stt_tool, task_planner, step_executor);
    
    let orchestrator_handle = tokio::spawn(async move { orchestrator.run().await; });
    
    eprintln!("AI Core Service is RUNNING");
    wait_for_shutdown().await;
    
    eprintln!("Shutting down...");
    drop(tx);
    let _ = orchestrator_handle.await;
    eprintln!("Shutdown complete");
    Ok(())
}
