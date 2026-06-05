# Text-to-Speech (TTS) Integration

## Overview

The Text-to-Speech (TTS) integration for the AI Core Service provides voice synthesis using open TTS (e.g., Coqui-TTS pattern) with local inference, audio streaming, and Opus output encoding. This module enables the AI Core Service to generate natural-sounding speech from text input.

## Architecture

### Components

1. **TextToSpeechTool**: Main tool implementation
2. **TtsBackend**: Mock Coqui-TTS backend integration
3. **AudioDevice**: Audio device glue for playback
4. **OpusEncoder**: Opus audio encoding for efficient compression
5. **CLI Integration**: Command-line interface for TTS synthesis

### Key Features

- **Local Inference**: TTS processing performed locally for privacy and performance
- **Audio Streaming**: Real-time audio streaming for low-latency playback
- **Opus Output**: Efficient Opus encoding for compressed audio output
- **Multiple Formats**: Support for S16LE, F32LE, Opus, and WAV formats
- **Voice Customization**: Configurable voice parameters (speed, pitch, volume)
- **Multi-language Support**: Support for multiple languages and voices
- **File Output**: Option to save synthesized audio to files

## Configuration

### TTS Configuration

```rust
pub struct TtsConfig {
    pub models_dir: PathBuf,           // Path to TTS models
    pub default_model: String,         // Default model name
    pub voice_config: VoiceConfig,     // Voice configuration
    pub audio_config: AudioOutputConfig, // Audio output config
    pub streaming_config: StreamingConfig, // Streaming config
    pub opus_config: OpusConfig,       // Opus encoding config
}
```

### Voice Configuration

```rust
pub struct VoiceConfig {
    pub default_voice: String,         // Default voice ID
    pub speed: f32,                    // Voice speed (0.5-2.0)
    pub pitch: f32,                    // Voice pitch (0.5-2.0)
    pub volume: f32,                   // Voice volume (0.0-1.0)
    pub emotion: Option<String>,       // Voice emotion/style
    pub language: String,              // Language code
}
```

### Audio Output Configuration

```rust
pub struct AudioOutputConfig {
    pub sample_rate: u32,              // Sample rate (Hz)
    pub channels: u16,                 // Number of channels
    pub format: AudioFormat,           // Audio format
    pub output_device: Option<String>, // Output device
    pub enable_streaming: bool,        // Enable streaming
    pub buffer_size: usize,            // Buffer size
}
```

### Streaming Configuration

```rust
pub struct StreamingConfig {
    pub enabled: bool,                 // Enable streaming
    pub chunk_size: usize,             // Chunk size for streaming
    pub latency_ms: u64,               // Streaming latency (ms)
    pub real_time: bool,               // Real-time streaming
}
```

### Opus Configuration

```rust
pub struct OpusConfig {
    pub enabled: bool,                 // Enable Opus encoding
    pub bitrate: u32,                  // Opus bitrate (kbps)
    pub quality: u8,                   // Opus quality (0-10)
    pub frame_size_ms: u16,            // Frame size (ms)
    pub enable_vbr: bool,              // Variable bitrate
}
```

## Usage

### Tool Integration

The TTS tool is automatically registered with the AI Core Service tool registry:

```rust
// Tool is registered as "tts_speak"
let tool = ToolRegistry::get_tool("tts_speak").await?;

// Execute synthesis
let result = tool_registry.execute_tool("tts_speak", &parameters).await?;
```

### CLI Usage

#### Synthesize Text to Speech

```bash
# Basic text synthesis
devctl ai tts speak "Hello, this is a test."

# With specific voice
devctl ai tts speak "Hello, this is a test." --voice en_female_1

# With language specification
devctl ai tts speak "Hola, esto es una prueba." --language es

# With voice parameters
devctl ai tts speak "Hello" --speed 1.2 --pitch 1.1 --volume 0.9

# With Opus output format
devctl ai tts speak "Hello" --output-format Opus

# With streaming enabled
devctl ai tts speak "Hello" --enable-streaming

# Save to file
devctl ai tts speak "Hello" --save-to-file output.wav

# JSON output
devctl ai tts speak "Hello" --json
```

#### List Available Voices

```bash
# List all available voices
devctl ai tts voices

# JSON output
devctl ai tts voices --json
```

### Programmatic Usage

```rust
use aetheris_ai_core::tools::tts::{TextToSpeechTool, TtsInput, default_tts_config};

// Create and initialize tool
let config = default_tts_config();
let mut tool = TextToSpeechTool::new(config)?;
tool.initialize(None).await?;

// Create input
let input = TtsInput {
    text: "Hello, this is a test.".to_string(),
    voice: Some("en_female_1".to_string()),
    language: Some("en".to_string()),
    speed: Some(1.0),
    pitch: Some(1.0),
    volume: Some(0.8),
    output_format: Some(AudioFormat::Opus),
    enable_streaming: Some(true),
    output_device: None,
    save_to_file: None,
};

// Synthesize text
let output = tool.synthesize_text(&input).await?;

println!("Duration: {} ms", output.duration_ms);
println!("Processing time: {} ms", output.processing_time_ms);
println!("Voice used: {}", output.voice_used);
```

## Audio Formats

### Supported Formats

| Format | Description | Use Case |
|--------|-------------|----------|
| S16LE | 16-bit signed little-endian PCM | High quality, uncompressed |
| F32LE | 32-bit float little-endian PCM | High quality, uncompressed |
| Opus | Opus encoded audio | Efficient compression, streaming |
| WAV | WAV file format | File output, compatibility |

### Opus Encoding

Opus encoding provides efficient compression with high quality:

- **Bitrates**: 32-256 kbps
- **Quality**: 0-10 (10 = highest quality)
- **Frame Sizes**: 2.5-60 ms
- **Variable Bitrate**: Optional VBR for better quality
- **Latency**: Low latency for real-time applications

## Voice Parameters

### Speed Control

- **Range**: 0.5 - 2.0
- **Default**: 1.0 (normal speed)
- **0.5**: Half speed (slower)
- **2.0**: Double speed (faster)

### Pitch Control

- **Range**: 0.5 - 2.0
- **Default**: 1.0 (normal pitch)
- **0.5**: Lower pitch (deeper voice)
- **2.0**: Higher pitch (higher voice)

### Volume Control

- **Range**: 0.0 - 1.0
- **Default**: 0.8 (80% volume)
- **0.0**: Silent
- **1.0**: Maximum volume

## Available Voices

### English Voices

| Voice ID | Name | Gender | Description |
|----------|------|--------|-------------|
| en_female_1 | English Female 1 | Female | Clear English female voice |
| en_male_1 | English Male 1 | Male | Clear English male voice |

### Spanish Voices

| Voice ID | Name | Gender | Description |
|----------|------|--------|-------------|
| es_female_1 | Spanish Female 1 | Female | Clear Spanish female voice |

### French Voices

| Voice ID | Name | Gender | Description |
|----------|------|--------|-------------|
| fr_female_1 | French Female 1 | Female | Clear French female voice |

## Streaming

### Real-time Streaming

Real-time streaming enables low-latency audio playback:

- **Chunk Size**: Configurable chunk size for streaming
- **Latency**: Low latency (50-100ms typical)
- **Buffer Management**: Automatic buffer management
- **Device Integration**: Direct audio device integration

### Streaming Configuration

```rust
let streaming_config = StreamingConfig {
    enabled: true,
    chunk_size: 1024,
    latency_ms: 100,
    real_time: true,
};
```

## Audio Device Integration

### Supported Devices

- **Default**: System default audio device
- **PulseAudio**: Linux PulseAudio devices
- **ALSA**: Linux ALSA devices
- **DirectSound**: Windows DirectSound devices
- **Core Audio**: macOS Core Audio devices

### Device Selection

```bash
# Use specific audio device
devctl ai tts speak "Hello" --output-device pulse

# Use default device
devctl ai tts speak "Hello" --output-device default
```

## Performance

### Processing Time

- **Short Text** (< 50 chars): ~50-100ms
- **Medium Text** (50-200 chars): ~100-300ms
- **Long Text** (200+ chars): ~300-1000ms

### Memory Usage

- **Model Loading**: ~100-500MB (depending on model size)
- **Audio Buffers**: ~1-10MB (depending on buffer size)
- **Streaming Buffers**: ~1-5MB (real-time streaming)

### Quality vs Performance

| Quality Setting | Processing Time | Memory Usage | Audio Quality |
|----------------|-----------------|--------------|---------------|
| Fast | Low | Low | Good |
| Balanced | Medium | Medium | Very Good |
| High | High | High | Excellent |

## Security

### Capability Requirements

The TTS tool requires the following capabilities:

- `ai:tools` - Basic tool execution permission

### Input Validation

- Text length limits (maximum 10,000 characters)
- Voice parameter validation (speed, pitch, volume ranges)
- File path validation for save operations
- Audio device validation

### Output Sanitization

- Audio data validation
- File path sanitization
- Device name validation

## Error Handling

### Common Errors

1. **Text Too Long**: Text exceeds maximum length limit
2. **Invalid Voice**: Specified voice not available
3. **Invalid Parameters**: Voice parameters out of range
4. **Device Error**: Audio device not available
5. **File Error**: Cannot save to specified file path

### Error Recovery

- Automatic fallback to default voice
- Parameter clamping to valid ranges
- Device fallback to default device
- File path validation and sanitization

## Testing

### Unit Tests

Comprehensive unit tests cover:

- Tool creation and initialization
- Text synthesis with various parameters
- Voice parameter validation
- Audio format encoding
- Streaming functionality
- File output operations
- Error handling scenarios

### Integration Tests

Integration tests cover:

- CLI command execution
- Tool registry integration
- Audio device integration
- Opus encoding validation

### Performance Tests

Performance tests validate:

- Processing time limits
- Memory usage bounds
- Audio quality metrics
- Streaming latency

## Troubleshooting

### Common Issues

1. **No Audio Output**
   - Check audio device configuration
   - Verify audio device permissions
   - Test with different output devices

2. **Poor Audio Quality**
   - Increase Opus quality setting
   - Use higher bitrate
   - Check voice parameters

3. **High Latency**
   - Enable streaming mode
   - Reduce chunk size
   - Use real-time streaming

4. **Memory Issues**
   - Reduce buffer sizes
   - Use smaller models
   - Enable streaming to reduce memory usage

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug devctl ai tts speak "Hello"
```

## Future Enhancements

### Planned Features

1. **Custom Voices**: Support for custom voice models
2. **Emotion Control**: Advanced emotion and style control
3. **SSML Support**: Speech Synthesis Markup Language support
4. **Batch Processing**: Process multiple texts
5. **GPU Acceleration**: CUDA/OpenCL support for faster processing
6. **Voice Cloning**: Custom voice creation from samples
7. **Multi-speaker**: Support for multiple speakers in one audio

### Integration Opportunities

1. **Audio Pipeline**: Integration with Phase-4 audio pipeline
2. **Memory Store**: Store voice preferences in memory
3. **System Intents**: Voice feedback for system actions
4. **Tool Chaining**: Combine with other AI tools
5. **Real-time Communication**: Integration with voice chat systems
