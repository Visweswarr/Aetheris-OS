#![allow(dead_code)]
#![allow(unused)]
#![allow(unused_parens)]
#![allow(clippy::all)]
//! Aetheris AI Service - Edge AI Perception & Encoding
//!
//! This service provides on-device AI pipelines for vision and audio processing
//! with deterministic replay capabilities and NGFS integration.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub mod vision;
pub mod audio;
pub mod enc;
pub mod replay;
pub mod policy;
pub mod backends;
pub mod error;

// Phase 5 modules
#[cfg(feature = "phase5")]
pub mod orchestrator;
#[cfg(feature = "phase5")]
pub mod assistant;
#[cfg(feature = "phase5")]
pub mod models;
#[cfg(feature = "phase5")]
pub mod planner;
#[cfg(feature = "phase5")]
pub mod api;
#[cfg(feature = "phase5")]
pub mod plan_ui;
#[cfg(feature = "phase5")]
pub mod plan_store;
#[cfg(feature = "phase5")]
pub mod time;
#[cfg(feature = "phase5")]
pub mod intent_client;
#[cfg(feature = "phase5")]
pub mod long_term_memory;
#[cfg(feature = "phase5")]
pub mod learning;
#[cfg(feature = "phase5")]
pub mod aicore;

use error::AiError;
use policy::AiPolicy;
use vision::VisionPipeline;
use audio::AudioPipeline;
use replay::ReplayManager;

// Phase 5 imports
#[cfg(feature = "phase5")]
use orchestrator::{Orchestrates, StubOrchestrator};
#[cfg(feature = "phase5")]
use assistant::{Assists, StubAssistant};
#[cfg(feature = "phase5")]
use models::{ModelLoad, StubModelLoader};
#[cfg(feature = "phase5")]
use planner::{TaskPlanner, StubTaskPlanner, Plan, PlanRequest, PlanResult};

/// Set up deterministic environment variables
pub fn determinism_env() {
    std::env::set_var("ORT_NUM_THREADS", "1");
    std::env::set_var("OMP_NUM_THREADS", "1");
    std::env::set_var("PYTHONHASHSEED", "0");
    // If using whisper.cpp via FFI:
    std::env::set_var("WHISPER_NO_RANDOM", "1");
}

/// AI service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AiConfig {
    /// Default model paths
    pub model_paths: HashMap<String, String>,
    /// Inference backends to use
    pub backends: BackendConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Deterministic settings
    pub deterministic: DeterministicConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct BackendConfig {
    /// ONNX Runtime settings
    pub onnx: OnnxConfig,
    /// Whisper.cpp settings
    pub whisper: WhisperConfig,
    /// Enable GPU acceleration (feature flag)
    pub enable_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnnxConfig {
    /// ONNX Runtime execution provider
    pub execution_provider: String,
    /// Model optimization level
    pub optimization_level: u8,
    /// Inter-op threads
    pub inter_op_threads: usize,
    /// Intra-op threads
    pub intra_op_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    /// Whisper model size
    pub model_size: String,
    /// Language code
    pub language: String,
    /// Number of threads
    pub threads: usize,
    /// Enable GPU acceleration
    pub enable_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum inference latency (ms)
    pub max_latency_ms: u64,
    /// Target FPS for vision pipeline
    pub target_fps: u32,
    /// Audio buffer size (samples)
    pub audio_buffer_size: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicConfig {
    /// Default seed for deterministic inference
    pub default_seed: u64,
    /// Enable deterministic mode
    pub enable_deterministic: bool,
    /// Replay buffer size
    pub replay_buffer_size: usize,
}



impl Default for OnnxConfig {
    fn default() -> Self {
        Self {
            execution_provider: "CPUExecutionProvider".to_string(),
            optimization_level: 1,
            inter_op_threads: 1,
            intra_op_threads: 4,
        }
    }
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model_size: "tiny".to_string(),
            language: "en".to_string(),
            threads: 4,
            enable_gpu: false,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_latency_ms: 100,
            target_fps: 15,
            audio_buffer_size: 4096,
            enable_monitoring: true,
        }
    }
}

impl Default for DeterministicConfig {
    fn default() -> Self {
        Self {
            default_seed: 42,
            enable_deterministic: true,
            replay_buffer_size: 1000,
        }
    }
}

/// AI service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AiStats {
    /// Vision pipeline stats
    pub vision: VisionStats,
    /// Audio pipeline stats
    pub audio: AudioStats,
    /// System stats
    pub system: SystemStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionStats {
    /// Total frames processed
    pub frames_processed: u64,
    /// Average inference time (ms)
    pub avg_inference_ms: f64,
    /// FPS achieved
    pub fps: f64,
    /// Detection count
    pub detections: u64,
    /// Errors
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStats {
    /// Total audio samples processed
    pub samples_processed: u64,
    /// Average inference time (ms)
    pub avg_inference_ms: f64,
    /// Transcripts generated
    pub transcripts: u64,
    /// VAD activations
    pub vad_activations: u64,
    /// Errors
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// Memory usage (MB)
    pub memory_mb: f64,
    /// CPU usage (%)
    pub cpu_percent: f64,
    /// Active pipelines
    pub active_pipelines: u32,
    /// Model cache hits
    pub cache_hits: u64,
    /// Model cache misses
    pub cache_misses: u64,
}


impl Default for VisionStats {
    fn default() -> Self {
        Self {
            frames_processed: 0,
            avg_inference_ms: 0.0,
            fps: 0.0,
            detections: 0,
            errors: 0,
        }
    }
}

impl Default for AudioStats {
    fn default() -> Self {
        Self {
            samples_processed: 0,
            avg_inference_ms: 0.0,
            transcripts: 0,
            vad_activations: 0,
            errors: 0,
        }
    }
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_percent: 0.0,
            active_pipelines: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

/// AI service main struct
pub struct AiService {
    /// Service configuration
    config: AiConfig,
    /// Vision pipeline
    vision_pipeline: Arc<RwLock<Option<VisionPipeline>>>,
    /// Audio pipeline
    audio_pipeline: Arc<RwLock<Option<AudioPipeline>>>,
    /// Replay manager
    replay_manager: Arc<ReplayManager>,
    /// Policy manager
    policy_manager: Arc<AiPolicy>,
    /// Service statistics
    stats: Arc<RwLock<AiStats>>,
    /// Active sessions
    active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    
    // Phase 5 components
    #[cfg(feature = "phase5")]
    /// AI Orchestrator
    orchestrator: Arc<dyn Orchestrates>,
    #[cfg(feature = "phase5")]
    /// AI Assistant
    assistant: Arc<dyn Assists>,
    #[cfg(feature = "phase5")]
    /// Model Loader
    model_loader: Arc<dyn ModelLoad>,
    #[cfg(feature = "phase5")]
    /// Task Planner
    task_planner: Arc<dyn TaskPlanner>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub pipeline_type: PipelineType,
    pub device_path: String,
    pub model_name: String,
    pub started_at: std::time::SystemTime,
    pub config: SessionConfig,
}

#[derive(Debug, Clone)]
pub enum PipelineType {
    Vision,
    Audio,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub seed: Option<u64>,
    pub fps: Option<u32>,
    pub language: Option<String>,
    pub enable_vad: bool,
    pub confidence_threshold: f32,
}

impl AiService {
    /// Create a new AI service
    pub fn new(config: AiConfig) -> Result<Self, AiError> {
        let replay_manager = Arc::new(ReplayManager::new(config.deterministic.clone())?);
        let policy_manager = Arc::new(AiPolicy::new()?);
        
        // Initialize Phase 5 components if enabled
        #[cfg(feature = "phase5")]
        let orchestrator = Arc::new(StubOrchestrator::new(policy_manager.clone())) as Arc<dyn Orchestrates>;
        #[cfg(feature = "phase5")]
        let assistant = Arc::new(StubAssistant::new(policy_manager.clone())) as Arc<dyn Assists>;
        #[cfg(feature = "phase5")]
        let model_loader = Arc::new(StubModelLoader::new(policy_manager.clone())) as Arc<dyn ModelLoad>;
        #[cfg(feature = "phase5")]
        let task_planner = Arc::new(StubTaskPlanner::new(policy_manager.clone())) as Arc<dyn TaskPlanner>;
        
        Ok(Self {
            config,
            vision_pipeline: Arc::new(RwLock::new(None)),
            audio_pipeline: Arc::new(RwLock::new(None)),
            replay_manager,
            policy_manager,
            stats: Arc::new(RwLock::new(AiStats::default())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            
            // Phase 5 components
            #[cfg(feature = "phase5")]
            orchestrator,
            #[cfg(feature = "phase5")]
            assistant,
            #[cfg(feature = "phase5")]
            model_loader,
            #[cfg(feature = "phase5")]
            task_planner,
        })
    }

    /// Get service configuration
    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> AiStats {
        self.stats.read().await.clone()
    }

    /// Start vision pipeline
    pub async fn start_vision_pipeline(
        &self,
        device_path: String,
        model_name: String,
        config: SessionConfig,
    ) -> Result<String, AiError> {
        // Check capabilities
        self.policy_manager.check_capability("ai:infer.vision")?;
        
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Create vision pipeline
        let pipeline = VisionPipeline::new(
            device_path.clone(),
            model_name.clone(),
            config.clone(),
            self.config.clone(),
        ).await?;
        
        // Store pipeline
        {
            let mut vision_pipeline = self.vision_pipeline.write().await;
            *vision_pipeline = Some(pipeline);
        }
        
        // Register session
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            pipeline_type: PipelineType::Vision,
            device_path,
            model_name,
            started_at: std::time::SystemTime::now(),
            config,
        };
        
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session_info);
        }
        
        Ok(session_id)
    }

    /// Start audio pipeline
    pub async fn start_audio_pipeline(
        &self,
        device_path: String,
        model_name: String,
        config: SessionConfig,
    ) -> Result<String, AiError> {
        // Check capabilities
        self.policy_manager.check_capability("ai:infer.audio")?;
        
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Create audio pipeline
        let pipeline = AudioPipeline::new(
            device_path.clone(),
            model_name.clone(),
            config.clone(),
            self.config.clone(),
        ).await?;
        
        // Store pipeline
        {
            let mut audio_pipeline = self.audio_pipeline.write().await;
            *audio_pipeline = Some(pipeline);
        }
        
        // Register session
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            pipeline_type: PipelineType::Audio,
            device_path,
            model_name,
            started_at: std::time::SystemTime::now(),
            config,
        };
        
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session_info);
        }
        
        Ok(session_id)
    }

    /// Stop pipeline by session ID
    pub async fn stop_pipeline(&self, session_id: &str) -> Result<(), AiError> {
        let session_info = {
            let sessions = self.active_sessions.read().await;
            sessions.get(session_id).cloned()
        };
        
        if let Some(session_info) = session_info {
            match session_info.pipeline_type {
                PipelineType::Vision => {
                    let mut vision_pipeline = self.vision_pipeline.write().await;
                    *vision_pipeline = None;
                }
                PipelineType::Audio => {
                    let mut audio_pipeline = self.audio_pipeline.write().await;
                    *audio_pipeline = None;
                }
            }
            
            // Remove session
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(session_id);
        }
        
        Ok(())
    }

    /// Get active sessions
    pub async fn get_active_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.active_sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Replay AI events from snapshot
    pub async fn replay_events(
        &self,
        snapshot_id: &str,
        topic: &str,
        check_determinism: bool,
    ) -> Result<Vec<serde_cbor::Value>, AiError> {
        self.replay_manager.replay_events(snapshot_id, topic, check_determinism).await
    }

    /// Get available models
    pub async fn get_available_models(&self) -> Result<Vec<ModelInfo>, AiError> {
        let mut models = Vec::new();
        
        // Check ONNX models
        if let Ok(onnx_models) = backends::onnx::list_available_models().await {
            for model_path in onnx_models {
                models.push(ModelInfo {
                    name: model_path.clone(),
                    model_type: ModelType::Onnx,
                    path: model_path,
                    available: true,
                });
            }
        }
        
        // Check Whisper models
        if let Ok(whisper_models) = backends::whisper::list_available_models().await {
            for model_path in whisper_models {
                models.push(ModelInfo {
                    name: model_path.clone(),
                    model_type: ModelType::Whisper,
                    path: model_path,
                    available: true,
                });
            }
        }
        
        Ok(models)
    }
    
    // Phase 5 methods
    #[cfg(feature = "phase5")]
    /// Get AI Orchestrator
    pub fn orchestrator(&self) -> Arc<dyn Orchestrates> {
        self.orchestrator.clone()
    }
    
    #[cfg(feature = "phase5")]
    /// Get AI Assistant
    pub fn assistant(&self) -> Arc<dyn Assists> {
        self.assistant.clone()
    }
    
    #[cfg(feature = "phase5")]
    /// Get Model Loader
    pub fn model_loader(&self) -> Arc<dyn ModelLoad> {
        self.model_loader.clone()
    }
    
    #[cfg(feature = "phase5")]
    /// Get Task Planner
    pub fn task_planner(&self) -> Arc<dyn TaskPlanner> {
        self.task_planner.clone()
    }
    
    #[cfg(feature = "phase5")]
    /// Generate a plan from a user goal (convenience method)
    pub async fn generate_plan(&self, goal: &str) -> Result<PlanResult, AiError> {
        self.policy_manager.check_capability("ai:plan.generate")?;
        
        let request = PlanRequest {
            goal: goal.to_string(),
            context: None,
            required_tools: vec![],
            priority: planner::StepPriority::Normal,
            max_execution_time: None,
            tags: vec![],
            metadata: std::collections::HashMap::new(),
        };
        
        self.task_planner.generate_plan(request).await
    }
    
    #[cfg(feature = "phase5")]
    /// Get a plan by ID (convenience method)
    pub async fn get_plan(&self, plan_id: &str) -> Result<Plan, AiError> {
        self.policy_manager.check_capability("ai:plan.read")?;
        self.task_planner.get_plan(plan_id).await
    }
    
    #[cfg(feature = "phase5")]
    /// List all plans (convenience method)
    pub async fn list_plans(&self) -> Result<Vec<Plan>, AiError> {
        self.policy_manager.check_capability("ai:plan.list")?;
        self.task_planner.list_plans().await
    }
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: ModelType,
    pub path: String,
    pub available: bool,
}

#[derive(Debug, Clone)]
pub enum ModelType {
    Onnx,
    Whisper,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_service_creation() {
        let config = AiConfig::default();
        let service = AiService::new(config);
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_stats_initialization() {
        let config = AiConfig::default();
        let service = AiService::new(config).unwrap();
        let stats = service.get_stats().await;
        assert_eq!(stats.vision.frames_processed, 0);
        assert_eq!(stats.audio.samples_processed, 0);
    }
}
