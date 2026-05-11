//! Vision AI pipeline for object detection and analysis

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use serde::{Deserialize, Serialize};
use crate::error::{AiError, AiResult};
use crate::backends::onnx::OnnxBackend;
use crate::backends::InferenceBackend;
use crate::enc::FrameEncoder;

// === Deterministic ONNX Runtime setup ===
#[cfg(any())]
use onnxruntime::{environment::Environment, LoggingLevel, GraphOptimizationLevel, ExecutionMode, CpuExecutionProviderOptions, SessionBuilder};
#[cfg(any())]
use once_cell::sync::Lazy;

#[cfg(any())]
static ORT_ENV: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name("aetheris-ai")
        .with_log_level(LoggingLevel::Warning)
        .build()
        .unwrap()
});

// Deterministic resize (box) + normalize helper
#[cfg(any())]
fn det_preprocess_rgb8_to_tensor(img: &image::RgbImage, width: u32, height: u32) -> anyhow::Result<Vec<f32>> {
    use fast_image_resize as fir;
    let mut src_view = fir::ImageView::from_slice_u8(img.width(), img.height(), img.as_raw(), fir::PixelType::U8x3)?;
    let mut dst = fir::Image::new(width, height, fir::PixelType::U8x3);
    let mut resizer = fir::Resizer::new(fir::ResizeAlg::Box);
    resizer.resize(&src_view, &mut dst.view_mut())?;

    let data = dst.buffer().to_vec();
    // HWC -> CHW, normalize and round
    let mut out = Vec::with_capacity((width * height * 3) as usize);
    for c in 0..3 {
        let mut i = c;
        while i < data.len() {
            let v = (data[i] as f32) / 255.0;
            // round to 6 decimals to remove float drift
            let r = (v * 1_000_000.0).round() / 1_000_000.0;
            out.push(r);
            i += 3;
        }
    }
    Ok(out)
}

// Deterministic ONNX session builder
#[cfg(any())]
fn build_ort_session(model_bytes: &[u8]) -> anyhow::Result<onnxruntime::session::Session> {
    let cpu_opts = CpuExecutionProviderOptions::default();
    let mut sb = SessionBuilder::new(&ORT_ENV)?
        .with_optimization_level(GraphOptimizationLevel::Disable)?
        .with_execution_mode(ExecutionMode::Sequential)?
        .with_intra_op_num_threads(1)?
        .with_inter_op_num_threads(1)?;
    sb = unsafe { sb.with_execution_providers([onnxruntime::ExecutionProvider::cpu(cpu_opts)]) }?;
    let session = sb.with_model_from_memory(model_bytes)?;
    Ok(session)
}

/// Vision pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    /// Target FPS
    pub fps: u32,
    /// Model configuration
    pub model: ModelConfig,
    /// Preprocessing configuration
    pub preprocessing: PreprocessingConfig,
    /// Postprocessing configuration
    pub postprocessing: PostprocessingConfig,
    /// Encoding configuration
    pub encoding: EncodingConfig,
    /// Deterministic settings
    pub deterministic: DeterministicSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name
    pub name: String,
    /// Model path
    pub path: String,
    /// Input size (width, height)
    pub input_size: (u32, u32),
    /// Confidence threshold
    pub confidence_threshold: f32,
    /// NMS threshold
    pub nms_threshold: f32,
    /// Number of classes
    pub num_classes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingConfig {
    /// Normalization mean
    pub mean: [f32; 3],
    /// Normalization std
    pub std: [f32; 3],
    /// Color space conversion
    pub color_space: ColorSpace,
    /// Resize method
    pub resize_method: ResizeMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostprocessingConfig {
    /// Enable NMS
    pub enable_nms: bool,
    /// Maximum detections
    pub max_detections: usize,
    /// Class names
    pub class_names: Vec<String>,
    /// Output format
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    /// Enable encoding
    pub enable_encoding: bool,
    /// Encoder type
    pub encoder_type: EncoderType,
    /// Quality settings
    pub quality: u8,
    /// Bitrate (kbps)
    pub bitrate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicSettings {
    /// Random seed
    pub seed: u64,
    /// Enable deterministic mode
    pub enable_deterministic: bool,
    /// Replay mode
    pub replay_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorSpace {
    RGB,
    BGR,
    YUV,
    Grayscale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResizeMethod {
    Bilinear,
    Nearest,
    Lanczos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    BoundingBoxes,
    Masks,
    Keypoints,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncoderType {
    H264,
    H265,
    VP8,
    VP9,
    MJPEG,
}

/// Vision pipeline
pub struct VisionPipeline {
    /// Pipeline configuration
    config: VisionConfig,
    /// ONNX backend
    backend: Arc<OnnxBackend>,
    /// Frame encoder
    encoder: Option<Arc<FrameEncoder>>,
    /// Event sender
    event_sender: mpsc::UnboundedSender<VisionEvent>,
    /// Statistics
    stats: VisionStats,
    /// Running state
    running: Arc<std::sync::atomic::AtomicBool>,
}

/// Vision event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisionEvent {
    /// Detection event
    Detection(DetectionEvent),
    /// Frame event
    Frame(FrameEvent),
    /// Error event
    Error(ErrorEvent),
    /// Stats event
    Stats(StatsEvent),
}

/// Detection event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Frame ID
    pub frame_id: u64,
    /// Detections
    pub detections: Vec<Detection>,
    /// Model hash
    pub model_hash: String,
    /// Postprocessing hash
    pub postprocessing_hash: String,
    /// Inference time (ms)
    pub inference_time_ms: f64,
}

/// Detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    /// Class ID
    pub class_id: u32,
    /// Class name
    pub class_name: String,
    /// Confidence score
    pub confidence: f32,
    /// Bounding box
    pub bbox: BoundingBox,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_cbor::Value>,
}

/// Bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

/// Frame event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Frame ID
    pub frame_id: u64,
    /// Frame data
    pub frame_data: FrameData,
    /// Encoding info
    pub encoding_info: Option<EncodingInfo>,
}

/// Frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    /// Frame format
    pub format: FrameFormat,
    /// Frame dimensions
    pub dimensions: (u32, u32),
    /// Frame data (encoded or raw)
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

/// Frame format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameFormat {
    YUV420,
    RGB24,
    BGR24,
    MJPEG,
    H264,
    H265,
}

/// Encoding info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingInfo {
    /// Encoder type
    pub encoder_type: EncoderType,
    /// Bitrate
    pub bitrate: u32,
    /// Quality
    pub quality: u8,
    /// Encoding time (ms)
    pub encoding_time_ms: f64,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Error type
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Frame ID (if applicable)
    pub frame_id: Option<u64>,
}

/// Stats event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Statistics
    pub stats: VisionStats,
}

/// Vision pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionStats {
    /// Total frames processed
    pub frames_processed: u64,
    /// Total detections
    pub total_detections: u64,
    /// Average inference time (ms)
    pub avg_inference_time_ms: f64,
    /// Average FPS
    pub avg_fps: f64,
    /// Total errors
    pub total_errors: u64,
    /// Last frame timestamp
    pub last_frame_timestamp: Option<std::time::SystemTime>,
    /// Pipeline uptime
    pub uptime: Duration,
}

impl VisionPipeline {
    /// Create a new vision pipeline
    pub async fn new(
        _device_path: String,
        model_name: String,
        session_config: crate::SessionConfig,
        ai_config: crate::AiConfig,
    ) -> AiResult<Self> {
        // Create vision configuration
        let config = VisionConfig {
            fps: session_config.fps.unwrap_or(ai_config.performance.target_fps),
            model: ModelConfig {
                name: model_name.clone(),
                path: ai_config.model_paths.get(&model_name)
                    .ok_or_else(|| AiError::model_loading(format!("Model '{}' not found", model_name)))?
                    .clone(),
                input_size: (640, 480), // Default size
                confidence_threshold: session_config.confidence_threshold,
                nms_threshold: 0.5,
                num_classes: 80, // COCO classes
            },
            preprocessing: PreprocessingConfig {
                mean: [0.485, 0.456, 0.406], // ImageNet mean
                std: [0.229, 0.224, 0.225],  // ImageNet std
                color_space: ColorSpace::RGB,
                resize_method: ResizeMethod::Bilinear,
            },
            postprocessing: PostprocessingConfig {
                enable_nms: true,
                max_detections: 100,
                class_names: Self::get_coco_class_names(),
                output_format: OutputFormat::BoundingBoxes,
            },
            encoding: EncodingConfig {
                enable_encoding: false, // Disabled by default
                encoder_type: EncoderType::H264,
                quality: 80,
                bitrate: 1000,
            },
            deterministic: DeterministicSettings {
                seed: session_config.seed.unwrap_or(ai_config.deterministic.default_seed),
                enable_deterministic: ai_config.deterministic.enable_deterministic,
                replay_mode: false,
            },
        };

        // Create ONNX backend
        let backend = Arc::new(OnnxBackend::new(&config.model).await?);

        // Create frame encoder if enabled
        let encoder = if config.encoding.enable_encoding {
            Some(Arc::new(FrameEncoder::new(&config.encoding)?))
        } else {
            None
        };

        // Create event channel
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();

        // Initialize statistics
        let stats = VisionStats {
            frames_processed: 0,
            total_detections: 0,
            avg_inference_time_ms: 0.0,
            avg_fps: 0.0,
            total_errors: 0,
            last_frame_timestamp: None,
            uptime: Duration::from_secs(0),
        };

        Ok(Self {
            config,
            backend,
            encoder,
            event_sender,
            stats,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Start the vision pipeline
    pub async fn start(&self) -> AiResult<()> {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(AiError::internal("Pipeline is already running"));
        }

        self.running.store(true, std::sync::atomic::Ordering::Relaxed);

        // Start the processing loop
        let pipeline = self.clone();
        tokio::spawn(async move {
            if let Err(e) = pipeline.process_loop().await {
                eprintln!("Vision pipeline error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop the vision pipeline
    pub async fn stop(&self) -> AiResult<()> {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Process a single frame
    pub async fn process_frame(&self, frame_data: FrameData) -> AiResult<DetectionEvent> {
        let _start_time = Instant::now();
        let frame_id = self.stats.frames_processed;

        // Preprocess frame
        let preprocessed = self.preprocess_frame(&frame_data).await?;

        // Run inference
        let inference_start = Instant::now();
        let raw_output = self.backend.infer(&preprocessed).await?;
        let inference_time = inference_start.elapsed();

        // Postprocess results
        let detections = self.postprocess_detections(&raw_output, &frame_data).await?;
        let detection_count = detections.len();

        // Create detection event
        let detection_event = DetectionEvent {
            timestamp: std::time::SystemTime::now(),
            frame_id,
            detections,
            model_hash: self.get_model_hash(),
            postprocessing_hash: self.get_postprocessing_hash(),
            inference_time_ms: inference_time.as_secs_f64() * 1000.0,
        };

        // Update statistics
        self.update_stats(inference_time, detection_count).await;

        // Send event
        let _ = self.event_sender.send(VisionEvent::Detection(detection_event.clone()));

        Ok(detection_event)
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> &VisionStats {
        &self.stats
    }

    /// Get event receiver
    pub fn get_event_receiver(&self) -> mpsc::UnboundedReceiver<VisionEvent> {
        let (_, receiver) = mpsc::unbounded_channel();
        receiver
    }

    /// Main processing loop
    async fn process_loop(&self) -> AiResult<()> {
        let frame_interval = Duration::from_millis(1000 / self.config.fps as u64);
        let mut interval = interval(frame_interval);
        let mut frame_id = 0u64;

        while self.running.load(std::sync::atomic::Ordering::Relaxed) {
            interval.tick().await;

            // In a real implementation, this would capture frames from the camera
            // For now, we'll simulate frame processing
            let frame_data = self.simulate_frame_capture().await?;
            
            match self.process_frame(frame_data).await {
                Ok(_detection_event) => {
                    frame_id += 1;
                    
                    // Send frame event if encoding is enabled
                    if self.encoder.is_some() {
                        let frame_event = FrameEvent {
                            timestamp: std::time::SystemTime::now(),
                            frame_id,
                            frame_data: self.simulate_frame_data().await?,
                            encoding_info: None, // Would be populated by encoder
                        };
                        
                        let _ = self.event_sender.send(VisionEvent::Frame(frame_event));
                    }
                }
                Err(e) => {
                    let error_event = ErrorEvent {
                        timestamp: std::time::SystemTime::now(),
                        error_type: "processing_error".to_string(),
                        message: e.to_string(),
                        frame_id: Some(frame_id),
                    };
                    
                    let _ = self.event_sender.send(VisionEvent::Error(error_event));
                }
            }
        }

        Ok(())
    }

    /// Preprocess frame for inference
    async fn preprocess_frame(&self, _frame_data: &FrameData) -> AiResult<Vec<f32>> {
        // In a real implementation, this would:
        // 1. Convert color space if needed
        // 2. Resize to model input size
        // 3. Normalize pixel values
        // 4. Convert to tensor format
        
        // For now, we'll simulate preprocessing
        let input_size = self.config.model.input_size;
        let num_pixels = (input_size.0 * input_size.1 * 3) as usize;
        let mut preprocessed = vec![0.0f32; num_pixels];
        
        // Simulate some processing
        for i in 0..num_pixels {
            preprocessed[i] = (i as f32) / (num_pixels as f32);
        }
        
        Ok(preprocessed)
    }

    /// Postprocess detection results
    async fn postprocess_detections(
        &self,
        _raw_output: &[f32],
        _frame_data: &FrameData,
    ) -> AiResult<Vec<Detection>> {
        // In a real implementation, this would:
        // 1. Parse raw model output
        // 2. Apply confidence thresholding
        // 3. Apply NMS if enabled
        // 4. Convert coordinates to frame space
        // 5. Map class IDs to names
        
        let mut detections = Vec::new();
        
        // Simulate some detections
        if self.config.deterministic.seed.is_multiple_of(3) {
            detections.push(Detection {
                class_id: 0,
                class_name: "person".to_string(),
                confidence: 0.85,
                bbox: BoundingBox {
                    x: 100.0,
                    y: 100.0,
                    width: 200.0,
                    height: 300.0,
                },
                metadata: std::collections::HashMap::new(),
            });
        }
        
        Ok(detections)
    }

    /// Update pipeline statistics
    async fn update_stats(&self, _inference_time: Duration, _detection_count: usize) {
        // In a real implementation, this would update the stats atomically
        // For now, we'll just simulate the update
    }

    /// Get model hash for auditing
    fn get_model_hash(&self) -> String {
        // In a real implementation, this would compute a hash of the model
        format!("model_hash_{}", self.config.model.name)
    }

    /// Get postprocessing hash for auditing
    fn get_postprocessing_hash(&self) -> String {
        // In a real implementation, this would compute a hash of the postprocessing config
        "postprocessing_hash".to_string()
    }

    /// Simulate frame capture
    async fn simulate_frame_capture(&self) -> AiResult<FrameData> {
        // Simulate frame data
        Ok(FrameData {
            format: FrameFormat::RGB24,
            dimensions: (640, 480),
            data: vec![0u8; 640 * 480 * 3],
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Simulate frame data
    async fn simulate_frame_data(&self) -> AiResult<FrameData> {
        self.simulate_frame_capture().await
    }

    /// Get COCO class names
    fn get_coco_class_names() -> Vec<String> {
        vec![
            "person".to_string(),
            "bicycle".to_string(),
            "car".to_string(),
            "motorcycle".to_string(),
            "airplane".to_string(),
            "bus".to_string(),
            "train".to_string(),
            "truck".to_string(),
            "boat".to_string(),
            "traffic light".to_string(),
            // ... more classes would be added here
        ]
    }
}

impl Clone for VisionPipeline {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            backend: self.backend.clone(),
            encoder: self.encoder.clone(),
            event_sender: self.event_sender.clone(),
            stats: self.stats.clone(),
            running: self.running.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vision_pipeline_creation() {
        let config = VisionConfig {
            fps: 15,
            model: ModelConfig {
                name: "test_model".to_string(),
                path: "test_path".to_string(),
                input_size: (640, 480),
                confidence_threshold: 0.5,
                nms_threshold: 0.5,
                num_classes: 80,
            },
            preprocessing: PreprocessingConfig {
                mean: [0.485, 0.456, 0.406],
                std: [0.229, 0.224, 0.225],
                color_space: ColorSpace::RGB,
                resize_method: ResizeMethod::Bilinear,
            },
            postprocessing: PostprocessingConfig {
                enable_nms: true,
                max_detections: 100,
                class_names: vec!["person".to_string()],
                output_format: OutputFormat::BoundingBoxes,
            },
            encoding: EncodingConfig {
                enable_encoding: false,
                encoder_type: EncoderType::H264,
                quality: 80,
                bitrate: 1000,
            },
            deterministic: DeterministicSettings {
                seed: 42,
                enable_deterministic: true,
                replay_mode: false,
            },
        };

        // Test would require actual ONNX backend
        // For now, just test configuration creation
        assert_eq!(config.fps, 15);
        assert_eq!(config.model.input_size, (640, 480));
    }

    #[test]
    fn test_detection_creation() {
        let detection = Detection {
            class_id: 0,
            class_name: "person".to_string(),
            confidence: 0.85,
            bbox: BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 200.0,
                height: 300.0,
            },
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(detection.class_name, "person");
        assert_eq!(detection.confidence, 0.85);
    }
}
