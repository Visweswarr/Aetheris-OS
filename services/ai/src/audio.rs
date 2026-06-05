//! Audio AI pipeline for speech recognition and analysis

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use serde::{Deserialize, Serialize};
use crate::error::{AiError, AiResult};
use crate::backends::InferenceBackend;
use crate::backends::whisper::WhisperBackend;

// === Deterministic audio processing ===
#[cfg(any())]
use rubato::{FftFixedIn, Resampler};
// Deterministic PCM pipeline
#[cfg(any())]
fn decode_to_mono16k_s16le(bytes: &[u8]) -> anyhow::Result<Vec<i16>> {
    // Use symphonia for decode if present; otherwise a known, repo-embedded wav seeds path.
    // For golden tests we expect WAV S16LE already; enforce format:
    let mut rdr = hound::WavReader::new(std::io::Cursor::new(bytes))?;
    let spec = rdr.spec();
    // Convert to mono
    let mut mono: Vec<f32> = Vec::new();
    if spec.channels == 1 {
        for s in rdr.samples::<i16>() { mono.push(s? as f32 / 32768.0); }
    } else {
        let mut interleaved: Vec<i16> = Vec::new();
        for s in rdr.samples::<i16>() { interleaved.push(s?); }
        for frame in interleaved.chunks_exact(spec.channels as usize) {
            let sum: i32 = frame.iter().map(|v| *v as i32).sum();
            mono.push((sum as f32 / spec.channels as f32) / 32768.0);
        }
    }
    // Resample to 16k
    let in_rate = spec.sample_rate as usize;
    let out_rate = 16_000usize;
    if in_rate != out_rate {
        let chans = 1;
        let chunk = 1024;
        let mut resamp = FftFixedIn::<f32>::new(in_rate, out_rate, 1, chunk, chans)?;
        let mut out = Vec::<f32>::new();
        for chunk in mono.chunks(chunk) {
            let buf = vec![Arc::from(chunk)];
            let got = resamp.process(&buf, None)?;
            out.extend_from_slice(&got[0]);
        }
        mono = out;
    }
    // Quantize deterministically
    let mut s16 = Vec::with_capacity(mono.len());
    for v in mono {
        let cl = v.clamp(-1.0, 1.0);
        let q = (cl * 32767.0).round() as i16;
        s16.push(q);
    }
    Ok(s16)
}

// Whisper call site: set threads=1, temperature=0, beam-search fixed params
// Ensure backend is configured with deterministic flags (no random sampling).

/// Audio pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Model configuration
    pub model: ModelConfig,
    /// VAD configuration
    pub vad: VadConfig,
    /// Postprocessing configuration
    pub postprocessing: PostprocessingConfig,
    /// Deterministic settings
    pub deterministic: DeterministicSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name
    pub name: String,
    /// Model path
    pub path: String,
    /// Model size
    pub model_size: String,
    /// Language code
    pub language: String,
    /// Number of threads
    pub threads: usize,
    /// Enable GPU
    pub enable_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Enable VAD
    pub enable_vad: bool,
    /// VAD threshold
    pub threshold: f32,
    /// Minimum speech duration (ms)
    pub min_speech_duration_ms: u64,
    /// Minimum silence duration (ms)
    pub min_silence_duration_ms: u64,
    /// Pre-padding (ms)
    pub pre_padding_ms: u64,
    /// Post-padding (ms)
    pub post_padding_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostprocessingConfig {
    /// Enable punctuation
    pub enable_punctuation: bool,
    /// Enable capitalization
    pub enable_capitalization: bool,
    /// Enable diarization
    pub enable_diarization: bool,
    /// Output format
    pub output_format: OutputFormat,
    /// Confidence threshold
    pub confidence_threshold: f32,
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
pub enum OutputFormat {
    Text,
    Json,
    Srt,
    Vtt,
    Full,
}

/// Audio pipeline
pub struct AudioPipeline {
    /// Pipeline configuration
    config: AudioConfig,
    /// Whisper backend
    backend: Arc<WhisperBackend>,
    /// Event sender
    event_sender: mpsc::UnboundedSender<AudioEvent>,
    /// Statistics
    stats: AudioStats,
    /// Running state
    running: Arc<std::sync::atomic::AtomicBool>,
    /// VAD state
    vad_state: VadState,
}

/// Audio event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioEvent {
    /// Transcript event
    Transcript(TranscriptEvent),
    /// VAD event
    Vad(VadEvent),
    /// Audio segment event
    Segment(SegmentEvent),
    /// Error event
    Error(ErrorEvent),
    /// Stats event
    Stats(StatsEvent),
}

/// Transcript event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Segment ID
    pub segment_id: u64,
    /// Transcript text
    pub text: String,
    /// Confidence score
    pub confidence: f32,
    /// Language detected
    pub language: String,
    /// Model hash
    pub model_hash: String,
    /// Postprocessing hash
    pub postprocessing_hash: String,
    /// Inference time (ms)
    pub inference_time_ms: f64,
    /// Audio duration (ms)
    pub audio_duration_ms: f64,
}

/// VAD event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// VAD state
    pub state: VadStateType,
    /// Confidence score
    pub confidence: f32,
    /// Audio level (dB)
    pub audio_level_db: f32,
    /// Duration (ms)
    pub duration_ms: f64,
}

/// VAD state type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VadStateType {
    Speech,
    Silence,
    Transition,
}

/// Audio segment event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Segment ID
    pub segment_id: u64,
    /// Start time (ms)
    pub start_time_ms: f64,
    /// End time (ms)
    pub end_time_ms: f64,
    /// Audio data
    pub audio_data: AudioData,
    /// VAD confidence
    pub vad_confidence: f32,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Audio format
    pub format: AudioFormat,
    /// Sample rate
    pub sample_rate: u32,
    /// Channels
    pub channels: u16,
    /// Audio data
    pub data: Vec<u8>,
    /// Duration (ms)
    pub duration_ms: f64,
}

/// Audio format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    S16LE,
    F32LE,
    S24LE,
    S32LE,
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
    /// Segment ID (if applicable)
    pub segment_id: Option<u64>,
}

/// Stats event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEvent {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Statistics
    pub stats: AudioStats,
}

/// Audio pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStats {
    /// Total samples processed
    pub samples_processed: u64,
    /// Total transcripts generated
    pub total_transcripts: u64,
    /// Total VAD activations
    pub total_vad_activations: u64,
    /// Average inference time (ms)
    pub avg_inference_time_ms: f64,
    /// Average audio level (dB)
    pub avg_audio_level_db: f64,
    /// Total errors
    pub total_errors: u64,
    /// Last audio timestamp
    pub last_audio_timestamp: Option<std::time::SystemTime>,
    /// Pipeline uptime
    pub uptime: Duration,
}

/// VAD state
#[derive(Debug, Clone)]
struct VadState {
    /// Current state
    current_state: VadStateType,
    /// State start time
    state_start_time: std::time::SystemTime,
    /// Current confidence
    confidence: f32,
    /// Audio buffer
    audio_buffer: Vec<f32>,
    /// Speech segments
    speech_segments: Vec<SpeechSegment>,
}

/// Speech segment
#[derive(Debug, Clone)]
struct SpeechSegment {
    /// Start time
    start_time: std::time::SystemTime,
    /// End time
    end_time: std::time::SystemTime,
    /// Audio data
    audio_data: Vec<f32>,
    /// Confidence
    confidence: f32,
}

impl AudioPipeline {
    /// Create a new audio pipeline
    pub async fn new(
        _device_path: String,
        model_name: String,
        session_config: crate::SessionConfig,
        ai_config: crate::AiConfig,
    ) -> AiResult<Self> {
        // Create audio configuration
        let config = AudioConfig {
            sample_rate: 16000, // Whisper standard
            channels: 1,        // Mono
            model: ModelConfig {
                name: model_name.clone(),
                path: ai_config.model_paths.get(&model_name)
                    .ok_or_else(|| AiError::model_loading(format!("Model '{}' not found", model_name)))?
                    .clone(),
                model_size: ai_config.backends.whisper.model_size.clone(),
                language: session_config.language.unwrap_or_else(|| ai_config.backends.whisper.language.clone()),
                threads: ai_config.backends.whisper.threads,
                enable_gpu: ai_config.backends.whisper.enable_gpu,
            },
            vad: VadConfig {
                enable_vad: session_config.enable_vad,
                threshold: 0.5,
                min_speech_duration_ms: 100,
                min_silence_duration_ms: 200,
                pre_padding_ms: 100,
                post_padding_ms: 100,
            },
            postprocessing: PostprocessingConfig {
                enable_punctuation: true,
                enable_capitalization: true,
                enable_diarization: false,
                output_format: OutputFormat::Text,
                confidence_threshold: session_config.confidence_threshold,
            },
            deterministic: DeterministicSettings {
                seed: session_config.seed.unwrap_or(ai_config.deterministic.default_seed),
                enable_deterministic: ai_config.deterministic.enable_deterministic,
                replay_mode: false,
            },
        };

        // Create Whisper backend
        let backend = Arc::new(WhisperBackend::new(&config.model).await?);

        // Create event channel
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();

        // Initialize statistics
        let stats = AudioStats {
            samples_processed: 0,
            total_transcripts: 0,
            total_vad_activations: 0,
            avg_inference_time_ms: 0.0,
            avg_audio_level_db: 0.0,
            total_errors: 0,
            last_audio_timestamp: None,
            uptime: Duration::from_secs(0),
        };

        // Initialize VAD state
        let vad_state = VadState {
            current_state: VadStateType::Silence,
            state_start_time: std::time::SystemTime::now(),
            confidence: 0.0,
            audio_buffer: Vec::new(),
            speech_segments: Vec::new(),
        };

        Ok(Self {
            config,
            backend,
            event_sender,
            stats,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            vad_state,
        })
    }

    /// Start the audio pipeline
    pub async fn start(&self) -> AiResult<()> {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(AiError::internal("Pipeline is already running"));
        }

        self.running.store(true, std::sync::atomic::Ordering::Relaxed);

        // Start the processing loop
        let pipeline = self.clone();
        tokio::spawn(async move {
            if let Err(e) = pipeline.process_loop().await {
                eprintln!("Audio pipeline error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop the audio pipeline
    pub async fn stop(&self) -> AiResult<()> {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Process audio data
    pub async fn process_audio(&self, audio_data: AudioData) -> AiResult<Option<TranscriptEvent>> {
        let _start_time = Instant::now();
        let segment_id = self.stats.samples_processed;

        // Update VAD state
        let vad_result = self.update_vad_state(&audio_data).await?;

        // If VAD detected speech, process the segment
        if let Some(speech_segment) = vad_result {
            // Run inference
            let inference_start = Instant::now();
            let raw_output = self.backend.infer(&speech_segment.audio_data).await?;
            let inference_time = inference_start.elapsed();

            // Postprocess results
            let transcript = self.postprocess_transcript(&raw_output).await?;

            // Create transcript event
            let transcript_event = TranscriptEvent {
                timestamp: std::time::SystemTime::now(),
                segment_id,
                text: transcript.text,
                confidence: transcript.confidence,
                language: transcript.language,
                model_hash: self.get_model_hash(),
                postprocessing_hash: self.get_postprocessing_hash(),
                inference_time_ms: inference_time.as_secs_f64() * 1000.0,
                audio_duration_ms: speech_segment.audio_data.len() as f64 / self.config.sample_rate as f64 * 1000.0,
            };

            // Update statistics
            self.update_stats(inference_time, 1).await;

            // Send event
            let _ = self.event_sender.send(AudioEvent::Transcript(transcript_event.clone()));

            Ok(Some(transcript_event))
        } else {
            Ok(None)
        }
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> &AudioStats {
        &self.stats
    }

    /// Get event receiver
    pub fn get_event_receiver(&self) -> mpsc::UnboundedReceiver<AudioEvent> {
        let (_, receiver) = mpsc::unbounded_channel();
        receiver
    }

    /// Main processing loop
    async fn process_loop(&self) -> AiResult<()> {
        let buffer_interval = Duration::from_millis(100); // 100ms buffer
        let mut interval = interval(buffer_interval);

        while self.running.load(std::sync::atomic::Ordering::Relaxed) {
            interval.tick().await;

            // In a real implementation, this would capture audio from the microphone
            // For now, we'll simulate audio processing
            let audio_data = self.simulate_audio_capture().await?;
            
            match self.process_audio(audio_data).await {
                Ok(Some(transcript_event)) => {
                    // Transcript was generated
                    let _ = self.event_sender.send(AudioEvent::Transcript(transcript_event));
                }
                Ok(None) => {
                    // No transcript (silence or VAD didn't trigger)
                }
                Err(e) => {
                    let error_event = ErrorEvent {
                        timestamp: std::time::SystemTime::now(),
                        error_type: "processing_error".to_string(),
                        message: e.to_string(),
                        segment_id: Some(self.stats.samples_processed),
                    };
                    
                    let _ = self.event_sender.send(AudioEvent::Error(error_event));
                }
            }
        }

        Ok(())
    }

    /// Update VAD state
    async fn update_vad_state(&self, audio_data: &AudioData) -> AiResult<Option<SpeechSegment>> {
        // In a real implementation, this would:
        // 1. Calculate audio level/energy
        // 2. Apply VAD algorithm
        // 3. Track speech/silence states
        // 4. Buffer audio for speech segments
        
        // For now, we'll simulate VAD
        let audio_level = self.calculate_audio_level(&audio_data.data);
        let is_speech = audio_level > self.config.vad.threshold;
        
        // Simulate VAD event
        let vad_event = VadEvent {
            timestamp: std::time::SystemTime::now(),
            state: if is_speech { VadStateType::Speech } else { VadStateType::Silence },
            confidence: audio_level,
            audio_level_db: 20.0 * (audio_level + 1e-10).log10(),
            duration_ms: audio_data.duration_ms,
        };
        
        let _ = self.event_sender.send(AudioEvent::Vad(vad_event));
        
        // If speech detected, return a speech segment
        if is_speech {
            let speech_segment = SpeechSegment {
                start_time: std::time::SystemTime::now(),
                end_time: std::time::SystemTime::now(),
                audio_data: self.convert_audio_to_f32(&audio_data.data, &audio_data.format),
                confidence: audio_level,
            };
            
            Ok(Some(speech_segment))
        } else {
            Ok(None)
        }
    }

    /// Postprocess transcript
    async fn postprocess_transcript(&self, _raw_output: &[f32]) -> AiResult<TranscriptResult> {
        // In a real implementation, this would:
        // 1. Decode Whisper output
        // 2. Apply postprocessing (punctuation, capitalization)
        // 3. Calculate confidence scores
        // 4. Detect language if needed
        
        // For now, we'll simulate postprocessing
        Ok(TranscriptResult {
            text: "Hello, this is a simulated transcript.".to_string(),
            confidence: 0.85,
            language: self.config.model.language.clone(),
        })
    }

    /// Update pipeline statistics
    async fn update_stats(&self, _inference_time: Duration, _transcript_count: usize) {
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

    /// Calculate audio level
    fn calculate_audio_level(&self, audio_data: &[u8]) -> f32 {
        // In a real implementation, this would calculate RMS or peak level
        // For now, we'll simulate based on data length
        (audio_data.len() as f32) / 1000.0
    }

    /// Convert audio data to f32
    fn convert_audio_to_f32(&self, audio_data: &[u8], format: &AudioFormat) -> Vec<f32> {
        // In a real implementation, this would convert based on format
        // For now, we'll simulate conversion
        match format {
            AudioFormat::S16LE => {
                // Convert S16LE to f32
                audio_data.chunks(2)
                    .map(|chunk| {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        sample as f32 / 32768.0
                    })
                    .collect()
            }
            AudioFormat::F32LE => {
                // Already f32
                audio_data.chunks(4)
                    .map(|chunk| {
                        f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                    })
                    .collect()
            }
            _ => {
                // Default conversion
                vec![0.0; audio_data.len() / 2]
            }
        }
    }

    /// Simulate audio capture
    async fn simulate_audio_capture(&self) -> AiResult<AudioData> {
        // Simulate audio data
        let sample_count = self.config.sample_rate as usize / 10; // 100ms of audio
        let audio_data = vec![0u8; sample_count * 2]; // S16LE format
        
        Ok(AudioData {
            format: AudioFormat::S16LE,
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            data: audio_data,
            duration_ms: 100.0,
        })
    }
}

/// Transcript result
#[derive(Debug, Clone)]
struct TranscriptResult {
    text: String,
    confidence: f32,
    language: String,
}

impl Clone for AudioPipeline {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            backend: self.backend.clone(),
            event_sender: self.event_sender.clone(),
            stats: self.stats.clone(),
            running: self.running.clone(),
            vad_state: VadState {
                current_state: VadStateType::Silence,
                state_start_time: std::time::SystemTime::now(),
                confidence: 0.0,
                audio_buffer: Vec::new(),
                speech_segments: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_pipeline_creation() {
        let config = AudioConfig {
            sample_rate: 16000,
            channels: 1,
            model: ModelConfig {
                name: "test_model".to_string(),
                path: "test_path".to_string(),
                model_size: "tiny".to_string(),
                language: "en".to_string(),
                threads: 4,
                enable_gpu: false,
            },
            vad: VadConfig {
                enable_vad: true,
                threshold: 0.5,
                min_speech_duration_ms: 100,
                min_silence_duration_ms: 200,
                pre_padding_ms: 100,
                post_padding_ms: 100,
            },
            postprocessing: PostprocessingConfig {
                enable_punctuation: true,
                enable_capitalization: true,
                enable_diarization: false,
                output_format: OutputFormat::Text,
                confidence_threshold: 0.5,
            },
            deterministic: DeterministicSettings {
                seed: 42,
                enable_deterministic: true,
                replay_mode: false,
            },
        };

        // Test would require actual Whisper backend
        // For now, just test configuration creation
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_transcript_creation() {
        let transcript = TranscriptEvent {
            timestamp: std::time::SystemTime::now(),
            segment_id: 1,
            text: "Hello world".to_string(),
            confidence: 0.85,
            language: "en".to_string(),
            model_hash: "test_hash".to_string(),
            postprocessing_hash: "test_hash".to_string(),
            inference_time_ms: 100.0,
            audio_duration_ms: 1000.0,
        };

        assert_eq!(transcript.text, "Hello world");
        assert_eq!(transcript.confidence, 0.85);
    }
}
