# Speech-to-Text (STT) Integration

## Overview

The Speech-to-Text (STT) integration for the AI Core Service provides VAD (Voice Activity Detection) and Whisper-based speech recognition with language auto-detection and deterministic processing. This module wraps the Phase-4 Whisper pipeline as a tool for the AI Core Service.

## Architecture

### Components

1. **SpeechToTextTool**: Main tool implementation
2. **WhisperBackend**: Mock Whisper.cpp backend integration
3. **VAD Integration**: Voice Activity Detection for speech segmentation
4. **Language Detection**: Automatic language detection with fallback
5. **CLI Integration**: Command-line interface for transcription

### Key Features

- **VAD Support**: Voice Activity Detection for speech segmentation
- **Language Auto-Detection**: Automatic language detection with fallback
- **Multiple Output Formats**: Text, JSON, SRT subtitle formats
- **Timestamp Support**: Word-level timestamps for transcription
- **Confidence Scores**: Confidence scores for transcription accuracy
- **Deterministic Processing**: Reproducible results for testing
- **CLI Integration**: Command-line interface via `devctl ai stt`

## Configuration

### STT Configuration

```rust
pub struct SttConfig {
    pub models_dir: PathBuf,           // Path to Whisper models
    pub default_model: String,         // Default model name
    pub vad_config: VadConfig,         // VAD configuration
    pub audio_config: AudioProcessingConfig, // Audio processing config
    pub language_config: LanguageConfig,     // Language detection config
}
```

### VAD Configuration

```rust
pub struct VadConfig {
    pub enabled: bool,                 // Enable VAD processing
    pub min_speech_duration: u64,      // Minimum speech duration (ms)
    pub max_silence_duration: u64,     // Maximum silence duration (ms)
    pub sensitivity: f32,              // VAD sensitivity (0.0-1.0)
    pub frame_size: usize,             // Frame size for VAD processing
}
```

### Audio Processing Configuration

```rust
pub struct AudioProcessingConfig {
    pub sample_rate: u32,              // Target sample rate (Hz)
    pub channels: u16,                 // Number of audio channels
    pub format: String,                // Supported audio formats
    pub max_duration: u64,             // Maximum audio duration (seconds)
    pub enable_preprocessing: bool,    // Enable audio preprocessing
}
```

### Language Configuration

```rust
pub struct LanguageConfig {
    pub auto_detect: bool,             // Enable language auto-detection
    pub fallback_language: String,     // Fallback language
    pub supported_languages: Vec<String>, // Supported languages
    pub min_confidence: f32,           // Minimum confidence for detection
}
```

## Usage

### Tool Integration

The STT tool is automatically registered with the AI Core Service tool registry:

```rust
// Tool is registered as "speech_to_text"
let tool = ToolRegistry::get_tool("speech_to_text").await?;

// Execute transcription
let result = tool_registry.execute_tool("speech_to_text", &parameters).await?;
```

### CLI Usage

#### Transcribe Audio File

```bash
# Basic transcription
devctl ai stt transcribe samples/ask.wav

# With specific model
devctl ai stt transcribe samples/ask.wav --model base

# With language hint
devctl ai stt transcribe samples/ask.wav --language en

# With VAD enabled
devctl ai stt transcribe samples/ask.wav --enable-vad

# With timestamps and confidence scores
devctl ai stt transcribe samples/ask.wav --include-timestamps --include-confidence

# JSON output format
devctl ai stt transcribe samples/ask.wav --output-format json --json

# SRT subtitle format
devctl ai stt transcribe samples/ask.wav --output-format srt
```

#### List Available Models

```bash
# List all available Whisper models
devctl ai stt models

# JSON output
devctl ai stt models --json
```

### Programmatic Usage

```rust
use aetheris_ai_core::tools::stt::{SpeechToTextTool, SttInput, default_stt_config};

// Create and initialize tool
let config = default_stt_config();
let mut tool = SpeechToTextTool::new(config)?;
tool.initialize(None).await?;

// Create input
let input = SttInput {
    file_path: "samples/ask.wav".to_string(),
    model: Some("base".to_string()),
    language: Some("en".to_string()),
    enable_vad: Some(true),
    output_format: Some("text".to_string()),
    include_timestamps: Some(true),
    include_confidence: Some(true),
};

// Process audio
let output = tool.process_audio_file(&input).await?;

println!("Transcription: {}", output.text);
println!("Language: {}", output.language);
println!("Confidence: {:.2}", output.confidence);
```

## Output Formats

### Text Format

```
Hello, this is a test transcription from the audio file.
```

### JSON Format

```json
{
  "text": "Hello, this is a test transcription from the audio file.",
  "language": "en",
  "confidence": 0.95,
  "processing_time_ms": 150,
  "model_used": "base",
  "vad_segments": [
    {
      "start_ms": 0,
      "end_ms": 2000,
      "confidence": 0.9
    }
  ],
  "timestamps": [
    {
      "start_ms": 0,
      "end_ms": 2000,
      "text": "Hello, this is a test",
      "confidence": 0.9
    }
  ]
}
```

### SRT Format

```
1
00:00:00,000 --> 00:00:02,000
Hello, this is a test

2
00:00:02,000 --> 00:00:04,000
transcription from the audio file.
```

## Supported Models

| Model | Size | Description |
|-------|------|-------------|
| tiny | 39 MB | Fastest, least accurate |
| base | 74 MB | Good balance of speed and accuracy |
| small | 244 MB | Better accuracy, slower |
| medium | 769 MB | High accuracy, much slower |
| large | 1550 MB | Best accuracy, slowest |

## Supported Languages

- English (en)
- Spanish (es)
- French (fr)
- German (de)
- Italian (it)
- Portuguese (pt)
- Russian (ru)
- Japanese (ja)
- Korean (ko)
- Chinese (zh)

## Supported Audio Formats

- WAV (.wav)
- MP3 (.mp3)
- FLAC (.flac)
- M4A (.m4a)

## VAD Segments

Voice Activity Detection segments provide information about speech activity:

```rust
pub struct VadSegment {
    pub start_ms: u64,        // Start time in milliseconds
    pub end_ms: u64,          // End time in milliseconds
    pub confidence: f32,      // Confidence score (0.0-1.0)
}
```

## Timestamps

Word-level timestamps provide precise timing information:

```rust
pub struct TimeSegment {
    pub start_ms: u64,        // Start time in milliseconds
    pub end_ms: u64,          // End time in milliseconds
    pub text: String,         // Text for this segment
    pub confidence: f32,      // Confidence score (0.0-1.0)
}
```

## Error Handling

The STT tool provides comprehensive error handling:

- **File Not Found**: Audio file does not exist
- **Unsupported Format**: Audio format not supported
- **File Too Large**: Audio file exceeds maximum duration
- **Model Not Found**: Whisper model not available
- **Processing Error**: Transcription processing failed

## Performance

### Processing Time

- **Tiny Model**: ~50ms for 10 seconds of audio
- **Base Model**: ~100ms for 10 seconds of audio
- **Small Model**: ~200ms for 10 seconds of audio
- **Medium Model**: ~500ms for 10 seconds of audio
- **Large Model**: ~1000ms for 10 seconds of audio

### Memory Usage

- **Tiny Model**: ~100MB RAM
- **Base Model**: ~150MB RAM
- **Small Model**: ~300MB RAM
- **Medium Model**: ~800MB RAM
- **Large Model**: ~1500MB RAM

## Security

### Capability Requirements

The STT tool requires the following capabilities:

- `ai:tools` - Basic tool execution permission

### Input Validation

- File path validation
- Audio format validation
- File size limits
- Model availability checks

### Output Sanitization

- Confidence score validation
- Timestamp range validation
- Text content sanitization

## Testing

### Unit Tests

Comprehensive unit tests cover:

- Tool creation and initialization
- Audio file processing
- VAD segment generation
- Timestamp generation
- Language detection
- Error handling
- Performance validation

### Integration Tests

Integration tests cover:

- CLI command execution
- Tool registry integration
- Fixture file processing
- Output format validation

### Test Fixtures

Test fixtures include:

- `sample_audio.wav` - Basic test audio
- `ask.wav` - Question audio sample
- `spanish_audio.wav` - Spanish language sample

## Troubleshooting

### Common Issues

1. **Model Not Found**
   - Ensure Whisper models are installed in the models directory
   - Check model file permissions

2. **Audio Format Not Supported**
   - Verify audio file format is supported
   - Check file extension and encoding

3. **Processing Timeout**
   - Reduce audio file size
   - Use a smaller model
   - Check system resources

4. **Low Confidence Scores**
   - Ensure clear audio quality
   - Check language detection accuracy
   - Verify model suitability

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug devctl ai stt transcribe samples/ask.wav
```

## Future Enhancements

### Planned Features

1. **Real-time Streaming**: Support for real-time audio streaming
2. **Custom Models**: Support for custom Whisper models
3. **Batch Processing**: Process multiple audio files
4. **GPU Acceleration**: CUDA/OpenCL support for faster processing
5. **Advanced VAD**: More sophisticated voice activity detection
6. **Speaker Diarization**: Identify different speakers
7. **Noise Reduction**: Audio preprocessing for better accuracy

### Integration Opportunities

1. **Audio Pipeline**: Integration with Phase-4 audio pipeline
2. **Memory Store**: Store transcription results in memory
3. **System Intents**: Voice command processing
4. **Tool Chaining**: Combine with other AI tools
