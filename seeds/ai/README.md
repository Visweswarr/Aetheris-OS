# AI Golden Test Seed Data

This directory contains seed data for AI golden tests to ensure deterministic behavior across different environments and model versions.

## Files

### Vision Seed Images

#### `test_image_1.jpg`
- **Description**: Simple test image with basic geometric shapes
- **Content**: Red square on white background, 64x64 pixels
- **Expected Hash**: `sha256: a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456`
- **Purpose**: Basic object detection test
- **Model**: YOLO nano (yolo_n.onnx)

#### `test_image_2.jpg`
- **Description**: Multi-object test image
- **Content**: Blue circle and green triangle on gray background, 128x128 pixels
- **Expected Hash**: `sha256: b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567`
- **Purpose**: Multi-object detection test
- **Model**: YOLO nano (yolo_n.onnx)

### Audio Seed Files

#### `test_audio_1.wav`
- **Description**: Simple speech sample
- **Content**: "Hello world, this is a test" (English, 16kHz, mono, 2 seconds)
- **Expected Hash**: `sha256: c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890`
- **Purpose**: Basic speech recognition test
- **Model**: Whisper tiny (ggml-tiny.en.bin)

#### `test_audio_2.wav`
- **Description**: Numbers and commands
- **Content**: "One two three, turn on the light" (English, 16kHz, mono, 3 seconds)
- **Expected Hash**: `sha256: d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890ab`
- **Purpose**: Command recognition test
- **Model**: Whisper tiny (ggml-tiny.en.bin)

## Usage

These seed files are used by `tooling/python/ai_golden_test.py` to verify that AI inference produces deterministic results across different environments.

## Expected Detection Results

### Vision Results (YOLO nano)
- **test_image_1.jpg**: 1 detection (red square, confidence > 0.8)
- **test_image_2.jpg**: 2 detections (blue circle, green triangle, confidence > 0.7)

### Audio Results (Whisper tiny)
- **test_audio_1.wav**: "Hello world, this is a test"
- **test_audio_2.wav**: "One two three, turn on the light"

## Hash Verification

The golden test verifies that:
1. Input file hashes match expected values
2. Model inference produces identical detection/transcript hashes
3. No drift occurs between test runs

## Updating Golden Tests

See `docs/phase-4/AI-GOLDEN.md` for procedures on updating golden test expectations.
