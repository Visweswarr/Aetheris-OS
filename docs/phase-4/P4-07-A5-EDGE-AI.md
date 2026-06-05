# P4-07-A5: Edge AI Perception & Encoding

## Overview

The Edge AI Perception & Encoding system provides on-device AI pipelines for vision and audio processing with deterministic replay capabilities and NGFS integration. This system enables real-time object detection, speech recognition, and other AI tasks while maintaining deterministic behavior for reproducible results.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Service Layer                         │
├─────────────────────────────────────────────────────────────┤
│  Vision Pipeline    │  Audio Pipeline    │  Replay Manager  │
│  - Object Detection │  - Speech Recognition│  - Event Replay │
│  - Frame Processing │  - VAD Processing   │  - Determinism   │
│  - Encoding         │  - Transcript Gen   │  - Validation    │
├─────────────────────────────────────────────────────────────┤
│                    Backend Layer                            │
├─────────────────────────────────────────────────────────────┤
│  ONNX Runtime       │  Whisper.cpp       │  Encoders        │
│  - YOLO Models      │  - Speech Models   │  - H.264/H.265   │
│  - CPU/GPU Inference│  - Language Models │  - Opus/AAC      │
│  - Model Loading    │  - VAD Processing  │  - MJPEG         │
├─────────────────────────────────────────────────────────────┤
│                    Hardware Layer                           │
├─────────────────────────────────────────────────────────────┤
│  Camera (V4L2)      │  Microphone (ALSA) │  GPU (Optional)  │
│  - Frame Capture    │  - Audio Capture   │  - Acceleration  │
│  - Format Support   │  - Format Support  │  - Memory Mgmt   │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Input Capture**: Camera/microphone data is captured via HAL
2. **Preprocessing**: Data is preprocessed for AI models
3. **Inference**: AI models process the data
4. **Postprocessing**: Results are processed and formatted
5. **Event Generation**: CBOR events are created with deterministic hashes
6. **NGFS Storage**: Events are stored in NGFS for replay
7. **Output**: Results are sent to applications via polyglot bindings

## Features

### Vision Pipeline

- **Object Detection**: YOLO-based object detection with configurable models
- **Real-time Processing**: Configurable FPS with performance monitoring
- **Multiple Formats**: Support for YUV, RGB, MJPEG, H.264, H.265
- **Encoding**: Optional frame encoding with quality control
- **Deterministic Mode**: Reproducible results with seeded RNG

### Audio Pipeline

- **Speech Recognition**: Whisper-based transcription with language support
- **VAD Processing**: Voice Activity Detection for efficient processing
- **Multiple Languages**: Support for 99+ languages
- **Real-time Processing**: Low-latency audio processing
- **Deterministic Mode**: Reproducible transcriptions

### Replay System

- **Event Recording**: All AI events are recorded with deterministic hashes
- **Deterministic Replay**: Exact reproduction of AI results
- **Schema Validation**: CBOR schema validation for data integrity
- **Performance Monitoring**: Comprehensive statistics and monitoring

## API Reference

### Rust Service API

```rust
// Initialize AI service
let config = AiConfig::default();
let service = AiService::new(config)?;

// Start vision pipeline
let session_id = service.start_vision_pipeline(
    "/dev/video0".to_string(),
    "yolo_n".to_string(),
    SessionConfig {
        fps: Some(15),
        confidence_threshold: 0.5,
        seed: Some(42),
        ..Default::default()
    }
).await?;

// Start audio pipeline
let session_id = service.start_audio_pipeline(
    "hw:0,0".to_string(),
    "whisper_tiny".to_string(),
    SessionConfig {
        language: Some("en".to_string()),
        enable_vad: true,
        seed: Some(42),
        ..Default::default()
    }
).await?;

// Replay events
let events = service.replay_events(
    "snapshot_123",
    "detections",
    true // check determinism
).await?;
```

### C FFI API

```c
// Initialize service
aetheris_ai_service_t* service;
aetheris_ai_config_t config = {0};
aetheris_ai_service_init(&config, &service);

// Start vision pipeline
aetheris_ai_vision_config_t vision_config = {
    .fps = 15,
    .model_name = "yolo_n",
    .confidence_threshold = 0.5,
    .seed = 42
};
aetheris_ai_session_t* session;
aetheris_ai_start_vision_pipeline(service, "/dev/video0", &vision_config, &session);

// Process frame
aetheris_ai_detection_t detections[10];
size_t num_detections;
aetheris_ai_process_frame(session, frame_data, frame_size, 640, 480, 
                         detections, 10, &num_detections);
```

### Go CLI API

```bash
# Start vision pipeline
devctl ai vision start --device /dev/video0 --model yolo_n --fps 15 --seed 42

# Process frame
devctl ai vision process --session vision_123 --input frame.jpg --output detections.json

# Start audio pipeline
devctl ai audio start --device hw:0,0 --model whisper_tiny --lang en --seed 42

# Process audio
devctl ai audio process --session audio_123 --input audio.wav --output transcript.json

# Replay events
devctl ai replay events --snapshot snapshot_123 --topic detections --check-determinism

# Show statistics
devctl ai stats
```

### TypeScript API

```typescript
import { AIBridge } from './ai_bridge';

// Initialize AI bridge
const ai = new AIBridge({
  enableGPU: false,
  targetFPS: 15,
  enableDeterministic: true
});

await ai.initialize();

// Start vision pipeline
const visionSession = await ai.startVisionPipeline('/dev/video0', {
  modelName: 'yolo_n',
  fps: 15,
  confidenceThreshold: 0.5
});

// Process frame
const detections = await ai.processFrame(visionSession, frameData, 640, 480);

// Listen for events
ai.on('vision:detection', (detection) => {
  console.log('Detection:', detection);
});

// Start audio pipeline
const audioSession = await ai.startAudioPipeline('hw:0,0', {
  modelName: 'whisper_tiny',
  language: 'en'
});

// Process audio
const { transcript, vad } = await ai.processAudio(audioSession, audioData, 16000, 1);
```

### Python API

```python
from ai_validator import AIValidator

# Initialize validator
validator = AIValidator()

# Run determinism test
success = validator.run_determinism_test(iterations=3)

# Run performance test
metrics = validator.run_performance_test(operations=100)

# Validate configuration
vision_config = {
    "fps": 15,
    "model_name": "yolo_n.onnx",
    "confidence_threshold": 0.5
}
is_valid = validator.validate_vision_pipeline(vision_config)
```

## Configuration

### AI Service Configuration

```toml
[ai]
# Model paths
model_paths = { yolo_n = "models/yolo_n.onnx", whisper_tiny = "models/ggml-tiny.en.bin" }

# Backend configuration
[ai.backends]
enable_gpu = false
onnx_execution_provider = "CPUExecutionProvider"
whisper_threads = 4

# Performance settings
[ai.performance]
max_latency_ms = 100
target_fps = 15
audio_buffer_size = 4096
enable_monitoring = true

# Deterministic settings
[ai.deterministic]
default_seed = 42
enable_deterministic = true
replay_buffer_size = 1000
```

### Vision Pipeline Configuration

```toml
[ai.vision]
fps = 15
model_name = "yolo_n"
input_width = 640
input_height = 480
confidence_threshold = 0.5
nms_threshold = 0.5
num_classes = 80
enable_encoding = false
encoder_type = "h264"
quality = 80
bitrate = 1000
```

### Audio Pipeline Configuration

```toml
[ai.audio]
sample_rate = 16000
channels = 1
model_name = "whisper_tiny"
model_size = "tiny"
language = "en"
threads = 4
enable_gpu = false
enable_vad = true
vad_threshold = 0.5
min_speech_duration_ms = 100
min_silence_duration_ms = 200
enable_punctuation = true
enable_capitalization = true
confidence_threshold = 0.5
```

## Models

### Supported Vision Models

| Model | Size | Input | Output | Use Case |
|-------|------|-------|--------|----------|
| YOLO-N | 6MB | 640x480x3 | 25200x85 | Fast object detection |
| YOLO-S | 22MB | 640x480x3 | 25200x85 | Balanced speed/accuracy |
| YOLO-M | 50MB | 640x480x3 | 25200x85 | High accuracy |
| YOLO-L | 87MB | 640x480x3 | 25200x85 | Maximum accuracy |
| YOLO-X | 136MB | 640x480x3 | 25200x85 | State-of-the-art |

### Supported Audio Models

| Model | Size | Languages | Use Case |
|-------|------|-----------|----------|
| Whisper Tiny | 39MB | 99+ | Fast transcription |
| Whisper Base | 74MB | 99+ | Balanced speed/accuracy |
| Whisper Small | 244MB | 99+ | High accuracy |
| Whisper Medium | 769MB | 99+ | Maximum accuracy |
| Whisper Large | 1550MB | 99+ | State-of-the-art |

## Event Schemas

### Vision Events (CBOR)

```cddl
vision_event = {
  event_type: "detection" / "frame" / "error" / "stats",
  timestamp: uint,
  frame_id: uint,
  session_id: text,
  model_info: {
    model_name: text,
    model_hash: text,
    input_shape: [*uint],
    output_shape: [*uint],
    confidence_threshold: float,
  },
  ? detection_info: {
    detections: [*{
      class_id: uint,
      class_name: text,
      confidence: float,
      bbox: { x: float, y: float, width: float, height: float },
    }],
  },
  deterministic_hash: text,
  postprocessing_hash: text,
  sequence_number: uint,
}
```

### Audio Events (CBOR)

```cddl
audio_event = {
  event_type: "transcript" / "vad" / "segment" / "error" / "stats",
  timestamp: uint,
  segment_id: uint,
  session_id: text,
  model_info: {
    model_name: text,
    model_hash: text,
    model_size: text,
    language: text,
  },
  ? transcript_info: {
    text: text,
    language: text,
    confidence: float,
    start_time: float,
    end_time: float,
  },
  ? vad_info: {
    state: "speech" / "silence" / "transition",
    confidence: float,
    audio_level_db: float,
    duration_ms: float,
  },
  deterministic_hash: text,
  postprocessing_hash: text,
  sequence_number: uint,
}
```

## Performance

### Benchmarks

| Operation | Latency (p95) | Throughput | Memory |
|-----------|---------------|------------|--------|
| YOLO-N Detection | 15ms | 66 FPS | 50MB |
| YOLO-S Detection | 25ms | 40 FPS | 100MB |
| Whisper Tiny | 200ms | 5 ops/sec | 39MB |
| Whisper Base | 400ms | 2.5 ops/sec | 74MB |
| H.264 Encoding | 10ms | 100 FPS | 20MB |
| Opus Encoding | 5ms | 200 ops/sec | 5MB |

### Performance Tuning

1. **CPU Optimization**:
   - Use appropriate thread counts for ONNX/Whisper
   - Enable CPU optimizations (AVX, SSE)
   - Use deterministic mode for consistent performance

2. **Memory Optimization**:
   - Use smaller models for lower memory usage
   - Enable model caching
   - Optimize buffer sizes

3. **GPU Acceleration**:
   - Enable GPU execution providers
   - Use CUDA/OpenCL backends
   - Optimize GPU memory usage

## Security

### Capability Enforcement

All AI operations require appropriate capabilities:

- `device:camera.read` - Camera access
- `device:mic.read` - Microphone access
- `ai:infer.vision` - Vision inference
- `ai:infer.audio` - Audio inference
- `ai:model.load` - Model loading
- `ai:replay.read` - Event replay

### Audit Trail

All AI operations are audited with:

- Timestamp and session ID
- Capability token used
- Device path and model name
- Policy hash and result
- Deterministic hash for replay

### Data Privacy

- No raw audio/video data is stored
- Only processed results and metadata
- Deterministic hashes for privacy-preserving replay
- Configurable data retention policies

## Troubleshooting

### Common Issues

1. **Model Loading Failures**:
   ```bash
   # Check model file exists
   ls -la models/yolo_n.onnx
   
   # Verify model format
   devctl ai models info --model yolo_n.onnx
   ```

2. **Performance Issues**:
   ```bash
   # Check system resources
   devctl ai stats
   
   # Monitor performance
   devctl ai vision start --fps 10  # Reduce FPS
   ```

3. **Deterministic Replay Failures**:
   ```bash
   # Check seed consistency
   devctl ai replay events --snapshot snapshot_123 --topic detections --check-determinism
   
   # Verify model hashes
   devctl ai models info --model yolo_n.onnx
   ```

### Debug Mode

Enable debug logging:

```bash
# Set environment variable
export AETHERIS_AI_DEBUG=1

# Run with verbose output
devctl ai vision start --device /dev/video0 --model yolo_n --verbose
```

### Log Analysis

AI service logs include:

- Model loading and initialization
- Inference performance metrics
- Error conditions and recovery
- Deterministic hash generation
- Capability checks and policy enforcement

## Development

### Building from Source

```bash
# Build Rust service
cargo build -p aetheris-ai

# Build C FFI
cd c/libc_aetheris
make

# Build Go CLI
cd go/tooling/devctl
go build

# Build TypeScript bridge
cd tooling/ts
npm install
npm run build

# Install Python dependencies
cd tooling/python
pip install -r requirements.txt
```

### Running Tests

```bash
# Run Rust tests
cargo test -p aetheris-ai

# Run C tests
cd c/libc_aetheris
make test

# Run Go tests
cd go/tooling/devctl
go test

# Run TypeScript tests
cd tooling/ts
npm test

# Run Python tests
cd tooling/python
python -m pytest
```

### Adding New Models

1. **Vision Models**:
   - Convert to ONNX format
   - Add model configuration
   - Update model registry
   - Add tests

2. **Audio Models**:
   - Convert to Whisper format
   - Add model configuration
   - Update model registry
   - Add tests

### Contributing

1. Follow the coding standards
2. Add comprehensive tests
3. Update documentation
4. Ensure deterministic behavior
5. Maintain performance benchmarks

## Future Enhancements

### Planned Features

1. **Additional AI Tasks**:
   - Image classification
   - Semantic segmentation
   - Pose estimation
   - Face recognition

2. **Model Optimization**:
   - Quantization support
   - Pruning techniques
   - Custom operators
   - Hardware-specific optimizations

3. **Advanced Features**:
   - Multi-modal processing
   - Real-time adaptation
   - Federated learning
   - Edge-cloud coordination

### Performance Improvements

1. **Hardware Acceleration**:
   - NPU support
   - FPGA acceleration
   - Custom ASIC integration
   - Memory optimization

2. **Algorithm Improvements**:
   - Streaming inference
   - Incremental processing
   - Adaptive quality
   - Dynamic model selection

## References

- [ONNX Runtime Documentation](https://onnxruntime.ai/)
- [Whisper.cpp Documentation](https://github.com/ggerganov/whisper.cpp)
- [RFC 8949 - CBOR](https://tools.ietf.org/html/rfc8949)
- [YOLO Paper](https://arxiv.org/abs/1506.02640)
- [Whisper Paper](https://arxiv.org/abs/2212.04356)
- [Aetheris OS Architecture](../architecture.md)
