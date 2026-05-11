use std::io::{self, Read};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use aetheris_ai_core::ipc::ChatRequest;
use aetheris_ai_core::orchestrator::{MultiModalOrchestrator, OrchestratorInput};
use aetheris_ai_core::router::PromptRouter;
use aetheris_ai_core::intents::create_system_intent_manager;
use aetheris_ai_core::runtime::RuntimeManager;
use aetheris_ai_core::tools::stt::{SpeechToTextTool, SttConfig};
use aetheris_ai_core::tools::registry::ToolRegistry;
use aetheris_ai_core::agent::{TaskPlanner, StepExecutor, PolicyEnforcer};
use aetheris_ai_core::cap::CapTokenManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    
    let message = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&buffer) {
        json["message"].as_str().unwrap_or(&buffer).to_string()
    } else {
        buffer.trim().to_string()
    };
    
    println!("Processing message: {}", message);
    
    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let prompt_router = Arc::new(PromptRouter::new(std::path::PathBuf::from("templates"), true));
    let system_intent_manager = Arc::new(create_system_intent_manager(cap_manager.clone())?);
    let task_planner = Arc::new(TaskPlanner::new(runtime_manager.clone(), tool_registry.clone()));
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager.clone()));
    let step_executor = Arc::new(StepExecutor::new(tool_registry.clone(), policy_enforcer.clone()));
    let mut stt = SpeechToTextTool::new(SttConfig::default())?;
    stt.initialize(None).await?;
    let stt_tool = Arc::new(stt);
    
    let (tx, rx) = mpsc::channel(32);
    let orchestrator = MultiModalOrchestrator::new(rx, prompt_router, system_intent_manager, runtime_manager, stt_tool, task_planner, step_executor);
    
    tokio::spawn(async move { orchestrator.run().await; });
    
    let (resp_tx, resp_rx) = oneshot::channel();
    let request = ChatRequest { message, ..Default::default() };
    
    tx.send(OrchestratorInput::Text { request, session_id: "cli-session".to_string(), response_tx: resp_tx }).await?;
    
    println!("Waiting for response...");
    let response = resp_rx.await??;
    println!("Response: {:?}", response);
    
    Ok(())
}
