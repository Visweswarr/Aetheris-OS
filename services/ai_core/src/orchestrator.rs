//! Multi-modal AI Orchestrator

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};


use crate::error::{AiCoreError, Result};
use crate::ipc::{ChatRequest, ChatResponse};
use crate::router::{PromptRouter, Intent};
use crate::runtime::RuntimeManager;
use crate::intents::{SystemIntentManager, SystemActionContext};
use crate::tools::stt::{SpeechToTextTool, SttInput};
use crate::agent::{TaskPlanner, StepExecutor, StepState, TaskPlan, StepExecutionContext};
use crate::budget::{BudgetEngine, BudgetLimits};
use crate::contracts::{
    BudgetRequest, FusionInput, FusionPayload, FusionRequest, FusionResponse, Modality,
    PrivacyRequest, RedactionMode,
};
use crate::privacy::PrivacyEngine;

#[derive(Debug)]
pub enum OrchestratorInput {
    Text {
        request: ChatRequest,
        session_id: String,
        response_tx: oneshot::Sender<Result<ChatResponse>>,
    },
    Audio {
        audio_data: Vec<u8>,
        session_id: String,
        response_tx: oneshot::Sender<Result<ChatResponse>>,
    },
    Fusion {
        request: FusionRequest,
        response_tx: oneshot::Sender<Result<FusionResponse>>,
    },
}

pub struct MultiModalOrchestrator {
    rx: mpsc::Receiver<OrchestratorInput>,
    prompt_router: Arc<PromptRouter>,
    intent_manager: Arc<SystemIntentManager>,
    runtime_manager: Arc<RuntimeManager>,
    stt_tool: Arc<SpeechToTextTool>,
    task_planner: Arc<TaskPlanner>,
    step_executor: Arc<StepExecutor>,
}

impl MultiModalOrchestrator {
    pub fn new(
        rx: mpsc::Receiver<OrchestratorInput>,
        prompt_router: Arc<PromptRouter>,
        intent_manager: Arc<SystemIntentManager>,
        runtime_manager: Arc<RuntimeManager>,
        stt_tool: Arc<SpeechToTextTool>,
        task_planner: Arc<TaskPlanner>,
        step_executor: Arc<StepExecutor>,
    ) -> Self {
        Self { rx, prompt_router, intent_manager, runtime_manager, stt_tool, task_planner, step_executor }
    }

    pub async fn run(mut self) {
        // Starting orchestrator
        while let Some(input) = self.rx.recv().await {
            if let Err(e) = self.process_input(input).await {
                eprintln!("Error processing input: {}", e);
            }
        }
        // Orchestrator stopped
    }

    async fn process_input(&self, input: OrchestratorInput) -> Result<()> {
        match input {
            OrchestratorInput::Text { request, session_id, response_tx } => {
                let result = self.handle_text(&request, &session_id).await;
                let _ = response_tx.send(result);
            }
            OrchestratorInput::Audio { audio_data, session_id, response_tx } => {
                let result = self.handle_audio(&audio_data, &session_id).await;
                let _ = response_tx.send(result);
            }
            OrchestratorInput::Fusion { request, response_tx } => {
                let result = self.submit(request).await;
                let _ = response_tx.send(result);
            }
        }
        Ok(())
    }

    /// Submit a canonical Phase 5-A multimodal request.
    pub async fn submit(&self, mut request: FusionRequest) -> Result<FusionResponse> {
        let started = Instant::now();
        if request.inputs.is_empty() {
            return Err(AiCoreError::InvalidInput("fusion request has no inputs".to_string()));
        }

        let budget = BudgetEngine::new(BudgetLimits::default());
        let budget_decision = budget.reserve(BudgetRequest {
            request_id: request.request_id.clone(),
            priority: request.priority,
            cpu_percent: 10,
            memory_mb: 128,
            power_mw: 500,
            expected_duration_ms: request.max_latency_ms.unwrap_or(50),
        }).await;

        if !budget_decision.granted {
            return Ok(FusionResponse {
                request_id: request.request_id,
                session_id: request.session_id,
                response_text: budget_decision.reason.clone(),
                fused_modalities: Vec::new(),
                confidence: 0.0,
                privacy: None,
                budget: Some(budget_decision),
                trace_id: request.trace_id,
                latency_ms: started.elapsed().as_millis() as u64,
                warnings: vec!["budget_denied".to_string()],
            });
        }

        let request_priority = request.priority;
        crate::budget::BudgetEngine::order_by_priority(&mut request.inputs, |input| match input.modality {
            Modality::Audio => crate::contracts::Priority::High,
            Modality::Text | Modality::Json => request_priority,
            Modality::Vision => crate::contracts::Priority::Normal,
        });

        let privacy_engine = PrivacyEngine::default();
        let mut fused_modalities = Vec::new();
        let mut warnings = Vec::new();
        let mut text_segments = Vec::new();
        let mut privacy_fields = std::collections::HashMap::new();

        for input in &request.inputs {
            fused_modalities.push(input.modality);
            match self.fusion_input_to_text(input).await {
                Ok(Some(text)) => {
                    privacy_fields.insert(format!("{:?}", input.modality), serde_json::Value::String(text.clone()));
                    let redacted = privacy_engine.redact_text(&text, RedactionMode::Placeholder);
                    text_segments.push(redacted);
                }
                Ok(None) => {}
                Err(error) => warnings.push(error.to_string()),
            }
        }

        let privacy = privacy_engine.classify_and_redact(PrivacyRequest {
            request_id: request.request_id.clone(),
            user_id: request.user_id.clone(),
            fields: privacy_fields,
            mode: RedactionMode::Placeholder,
        });

        let prompt = if text_segments.is_empty() {
            format!("Process multimodal request with modalities: {:?}", fused_modalities)
        } else {
            text_segments.join("\n")
        };

        let chat_request = ChatRequest {
            message: prompt.clone(),
            prompt,
            session_id: Some(request.session_id.clone()),
            user_id: request.user_id.clone(),
            metadata: Some(request.context.clone()),
            ..Default::default()
        };

        let chat = self.handle_text(&chat_request, &request.session_id).await?;
        budget.release(&request.request_id).await;

        Ok(FusionResponse {
            request_id: request.request_id,
            session_id: request.session_id,
            response_text: chat.response,
            fused_modalities,
            confidence: chat.metrics.as_ref().map(|m| m.confidence).unwrap_or(0.90),
            privacy: Some(privacy),
            budget: Some(budget_decision),
            trace_id: request.trace_id,
            latency_ms: started.elapsed().as_millis() as u64,
            warnings,
        })
    }

    async fn fusion_input_to_text(&self, input: &FusionInput) -> Result<Option<String>> {
        match (&input.modality, &input.payload) {
            (Modality::Text, FusionPayload::Text(text)) => Ok(Some(text.clone())),
            (Modality::Json, FusionPayload::Json(value)) => Ok(Some(value.to_string())),
            (Modality::Audio, FusionPayload::Bytes(audio_data)) => {
                let stt_input = SttInput {
                    audio_data: Some(audio_data.clone()),
                    file_path: None,
                    model: input.model_id.clone(),
                    language: input.metadata.get("language").cloned(),
                };
                let stt_output = self.stt_tool.process_audio_file(&stt_input).await?;
                Ok(Some(stt_output.text))
            }
            (Modality::Vision, FusionPayload::Bytes(bytes)) => Ok(Some(format!(
                "vision input: {} bytes, mime={}",
                bytes.len(),
                input.mime_type.as_deref().unwrap_or("application/octet-stream")
            ))),
            (_, FusionPayload::Text(text)) => Ok(Some(text.clone())),
            (_, FusionPayload::Json(value)) => Ok(Some(value.to_string())),
            (_, FusionPayload::Bytes(bytes)) => Ok(Some(format!("binary input: {} bytes", bytes.len()))),
        }
    }

    async fn handle_text(&self, request: &ChatRequest, session_id: &str) -> Result<ChatResponse> {
        let intent = self.prompt_router.detect_intent(request);
        // Detected intent

        match intent {
            Intent::ComplexWorkflow => self.process_complex_workflow(request, session_id).await,
            Intent::SystemControl(_) | Intent::AppControl(_) | Intent::Settings(_) | Intent::Hardware(_) => {
                self.process_system_intent(request, session_id).await
            }
            _ => self.process_generic_query(request).await,
        }
    }

    async fn handle_audio(&self, audio_data: &[u8], session_id: &str) -> Result<ChatResponse> {
        let stt_input = SttInput {
            audio_data: Some(audio_data.to_vec()),
            file_path: None,
            model: None,
            language: None,
        };

        let stt_output = self.stt_tool.process_audio_file(&stt_input).await?;
        
        if stt_output.text.trim().is_empty() {
            return Err(AiCoreError::InvalidInput("Empty transcription".to_string()));
        }

        let request = ChatRequest {
            message: stt_output.text,
            prompt: String::new(),
            ..Default::default()
        };

        self.handle_text(&request, session_id).await
    }

    async fn process_system_intent(&self, request: &ChatRequest, session_id: &str) -> Result<ChatResponse> {
        let system_intent = self.intent_manager.parse_intent(&request.message)?;
        let context = SystemActionContext {
            user_id: "default_user".to_string(),
            session_id: session_id.to_string(),
            cap_token: None,
            metadata: std::collections::HashMap::new(),
        };

        let result = self.intent_manager.execute_intent(&system_intent, &context).await?;

        Ok(ChatResponse {
            response: result.message,
            is_complete: true,
            ..Default::default()
        })
    }

    async fn process_generic_query(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let runtime_request = crate::runtime::RuntimeRequest {
            prompt: if request.prompt.trim().is_empty() { request.message.clone() } else { request.prompt.clone() },
            max_tokens: None,
            temperature: None,
            stop_sequences: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        let response = self.runtime_manager.generate_response(&runtime_request).await?;

        Ok(ChatResponse {
            response: response.generated_text,
            is_complete: true,
            ..Default::default()
        })
    }

    async fn process_complex_workflow(&self, request: &ChatRequest, session_id: &str) -> Result<ChatResponse> {
        // Processing complex workflow

        let context = SystemActionContext {
            user_id: request.user_id.clone().unwrap_or_else(|| "default_user".to_string()),
            session_id: session_id.to_string(),
            cap_token: None,
            metadata: std::collections::HashMap::new(),
        };

        let plan = self.task_planner.generate_plan(&request.message, &context).await?;
        let results = self.step_executor.execute_plan(plan.clone(), context).await?;
        let summary = self.summarize_results(&plan, &results);

        Ok(ChatResponse {
            response: summary,
            is_complete: true,
            ..Default::default()
        })
    }

    fn summarize_results(&self, plan: &TaskPlan, results: &[StepExecutionContext]) -> String {
        let completed = results.iter().filter(|r| r.state == StepState::Completed).count();
        let failed = results.iter().filter(|r| r.state == StepState::Failed).count();
        
        format!(
            "Plan '{}' completed: {} steps succeeded, {} failed",
            plan.original_intent, completed, failed
        )
    }
}
