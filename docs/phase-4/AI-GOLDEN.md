# AI Golden Test Documentation

## Overview

AI Golden Tests ensure deterministic behavior of AI inference pipelines across different environments, model versions, and system configurations. These tests verify that the same inputs produce identical outputs, preventing regression and ensuring reproducible AI behavior.

## Table of Contents

1. [What are Golden Tests?](#what-are-golden-tests)
2. [Test Infrastructure](#test-infrastructure)
3. [Running Golden Tests](#running-golden-tests)
4. [Updating Golden Tests](#updating-golden-tests)
5. [When to Update Goldens](#when-to-update-goldens)
6. [Approval Process](#approval-process)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

## What are Golden Tests?

### Purpose
Golden tests verify that AI inference produces deterministic, reproducible results by:
- Running inference on fixed seed data
- Comparing output hashes against expected values
- Detecting drift in model behavior or inference pipeline
- Ensuring consistent results across environments

### Components
1. **Seed Data**: Fixed input files (images, audio) with known hashes
2. **Expected Outputs**: Known detection/transcript hashes for seed data
3. **Test Runner**: Automated test execution and comparison
4. **Report Generation**: Detailed test results and drift analysis

## Test Infrastructure

### Seed Data Structure
```
seeds/ai/
├── README.md              # Seed data documentation
├── test_image_1.jpg       # Simple geometric shape (64x64)
├── test_image_2.jpg       # Multi-object scene (128x128)
├── test_audio_1.wav       # Basic speech sample (2s)
└── test_audio_2.wav       # Numbers and commands (3s)
```

### Expected Hashes

#### Input File Hashes
- **test_image_1.jpg**: `a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456`
- **test_image_2.jpg**: `b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567`
- **test_audio_1.wav**: `c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890`
- **test_audio_2.wav**: `d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890ab`

#### Output Result Hashes
- **test_image_1.jpg**: `e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcd`
- **test_image_2.jpg**: `f6789012345678901234567890abcdef1234567890abcdef1234567890abcde`
- **test_audio_1.wav**: `6789012345678901234567890abcdef1234567890abcdef1234567890abcdef`
- **test_audio_2.wav**: `789012345678901234567890abcdef1234567890abcdef1234567890abcdef0`

### Test Configuration
```python
config = {
    "vision_model": "yolo_n.onnx",
    "audio_model": "ggml-tiny.en.bin",
    "vision_seed": 42,
    "audio_seed": 42,
    "language": "en",
    "deterministic": True
}
```

## Running Golden Tests

### Basic Execution
```bash
# Run all golden tests
python3 tooling/python/ai_golden_test.py

# Run with verbose output
python3 tooling/python/ai_golden_test.py --verbose

# Specify custom directories
python3 tooling/python/ai_golden_test.py --seed-dir seeds/ai --artifacts-dir artifacts/ai
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Run AI Golden Tests
  run: |
    python3 tooling/python/ai_golden_test.py
    if [ $? -ne 0 ]; then
      echo "❌ Golden tests failed - drift detected"
      exit 1
    fi
```

### Expected Output
```
============================================================
AI GOLDEN TEST SUMMARY
============================================================
Total Tests: 4
Passed: 4
Failed: 0
Success Rate: 100.0%

✅ PASS vision_test_image_1.jpg
✅ PASS vision_test_image_2.jpg
✅ PASS audio_test_audio_1.wav
✅ PASS audio_test_audio_2.wav

============================================================
✅ ALL GOLDEN TESTS PASSED
```

## Updating Golden Tests

### When Updates Are Needed

#### Model Updates
- **New Model Versions**: When upgrading YOLO or Whisper models
- **Model Retraining**: When models are retrained with new data
- **Architecture Changes**: When model architecture is modified

#### Pipeline Changes
- **Preprocessing Updates**: Changes to image/audio preprocessing
- **Postprocessing Changes**: Modifications to detection/transcript processing
- **Inference Engine Updates**: Changes to ONNX Runtime or Whisper.cpp

#### System Changes
- **Library Updates**: Updates to underlying AI libraries
- **Hardware Changes**: Different hardware configurations
- **Environment Changes**: New operating systems or dependencies

### Update Process

#### Step 1: Identify the Change
```bash
# Run golden tests to identify failures
python3 tooling/python/ai_golden_test.py

# Review the failure report
cat artifacts/ai/golden.json
```

#### Step 2: Validate the Change
```bash
# Ensure the change is intentional and correct
# Verify that new outputs are expected and valid
# Test with multiple seed files to ensure consistency
```

#### Step 3: Update Expected Hashes
```python
# Update expected_output_hashes in ai_golden_test.py
expected_output_hashes = {
    "test_image_1.jpg": "new_hash_here",
    "test_image_2.jpg": "new_hash_here",
    "test_audio_1.wav": "new_hash_here",
    "test_audio_2.wav": "new_hash_here"
}
```

#### Step 4: Verify Updates
```bash
# Run tests to verify updates
python3 tooling/python/ai_golden_test.py

# Ensure all tests pass
echo $?  # Should be 0
```

#### Step 5: Document Changes
```bash
# Update documentation with change rationale
# Add entry to CHANGELOG.md
# Update seed data README if needed
```

## When to Update Goldens

### Automatic Updates (Not Recommended)
- **Never**: Golden tests should never be automatically updated
- **Risk**: Automatic updates can mask real regressions
- **Principle**: All updates must be intentional and reviewed

### Manual Updates (Required)
- **Model Version Changes**: When upgrading AI models
- **Pipeline Improvements**: When improving preprocessing/postprocessing
- **Bug Fixes**: When fixing bugs that change output format
- **Feature Additions**: When adding new features that affect outputs

### Update Triggers
1. **CI/CD Failures**: Golden tests failing in CI/CD pipeline
2. **Model Deployment**: New model versions deployed
3. **Code Changes**: Changes to AI inference pipeline
4. **Environment Changes**: New system configurations

## Approval Process

### Required Approvals

#### Technical Review
- **AI Engineer**: Review model changes and output validity
- **Senior Developer**: Review code changes and test updates
- **QA Engineer**: Review test coverage and validation

#### Documentation Review
- **Technical Writer**: Review documentation updates
- **Product Manager**: Review impact on product behavior

### Approval Checklist

#### Before Updating
- [ ] **Change Justification**: Clear rationale for the update
- [ ] **Impact Analysis**: Understanding of downstream effects
- [ ] **Validation**: New outputs are correct and expected
- [ ] **Testing**: Comprehensive testing of changes
- [ ] **Documentation**: Updated documentation and comments

#### After Updating
- [ ] **Test Verification**: All golden tests pass
- [ ] **Regression Testing**: No unintended side effects
- [ ] **Performance Impact**: No significant performance degradation
- [ ] **Documentation**: All relevant docs updated
- [ ] **Communication**: Team notified of changes

### Approval Workflow
1. **Create Issue**: Document the need for golden test updates
2. **Technical Review**: AI engineer reviews model/output changes
3. **Code Review**: Senior developer reviews test updates
4. **QA Review**: QA engineer validates test coverage
5. **Final Approval**: Technical lead approves the changes
6. **Implementation**: Update golden tests and documentation
7. **Verification**: Run tests to ensure updates are correct

## Troubleshooting

### Common Issues

#### Hash Mismatches
**Symptoms:**
- Golden tests failing with hash mismatches
- Inconsistent outputs between runs

**Causes:**
- Non-deterministic inference
- Different model versions
- Environment differences
- Code changes

**Solutions:**
```bash
# Check deterministic settings
export AETHERIS_DETERMINISTIC=true
export AETHERIS_SEED=42

# Verify model versions
devctl ai models list

# Check environment consistency
python3 tooling/python/ai_golden_test.py --verbose
```

#### Missing Seed Files
**Symptoms:**
- "File not found" errors
- Input hash mismatches

**Causes:**
- Missing seed data files
- Incorrect file paths
- Corrupted seed files

**Solutions:**
```bash
# Check seed file existence
ls -la seeds/ai/

# Verify file integrity
sha256sum seeds/ai/*

# Regenerate seed files if needed
```

#### Model Loading Failures
**Symptoms:**
- "Model not found" errors
- Inference failures

**Causes:**
- Missing model files
- Incorrect model paths
- Model compatibility issues

**Solutions:**
```bash
# Check model availability
devctl ai models list

# Verify model paths
export AETHERIS_AI_MODEL_PATH=/opt/aetheris/models

# Test model loading
devctl ai models validate
```

### Debug Commands

#### Verbose Testing
```bash
# Run with detailed output
python3 tooling/python/ai_golden_test.py --verbose

# Check test report
cat artifacts/ai/golden.json | jq '.'
```

#### Individual Test Execution
```bash
# Test specific components
python3 -c "
from tooling.python.ai_golden_test import AIGoldenTest
test = AIGoldenTest()
result = test.test_vision_golden('test_image_1.jpg')
print(result)
"
```

#### Environment Validation
```bash
# Check environment settings
echo $AETHERIS_DETERMINISTIC
echo $AETHERIS_SEED
echo $AETHERIS_AI_MODEL_PATH

# Verify model availability
devctl ai models list
```

## Best Practices

### Test Design
1. **Deterministic Inputs**: Use fixed seed data with known properties
2. **Comprehensive Coverage**: Test multiple scenarios and edge cases
3. **Clear Expectations**: Document expected outputs and behaviors
4. **Regular Updates**: Keep golden tests current with system changes

### Maintenance
1. **Regular Review**: Periodically review and update golden tests
2. **Version Control**: Track all changes to golden test expectations
3. **Documentation**: Maintain clear documentation of test purposes
4. **Monitoring**: Monitor golden test performance and reliability

### Security
1. **Access Control**: Limit who can update golden test expectations
2. **Audit Trail**: Maintain logs of all golden test updates
3. **Validation**: Validate all changes before implementation
4. **Rollback**: Maintain ability to rollback problematic updates

### Performance
1. **Efficient Execution**: Optimize golden test execution time
2. **Parallel Testing**: Run tests in parallel when possible
3. **Resource Management**: Monitor resource usage during testing
4. **Caching**: Cache model loads and intermediate results

### Integration
1. **CI/CD Integration**: Include golden tests in continuous integration
2. **Automated Alerts**: Set up alerts for golden test failures
3. **Reporting**: Generate comprehensive test reports
4. **Metrics**: Track golden test performance over time

This comprehensive guide ensures reliable AI golden testing and proper maintenance of deterministic AI behavior in Aetheris OS Phase 4.

## Determinism Checklist

### Environment Setup
- [ ] **Single-thread CPU**: `ORT_NUM_THREADS=1`, `OMP_NUM_THREADS=1`
- [ ] **Sequential execution**: ONNX Runtime in sequential mode
- [ ] **Fixed resize**: Box filter for deterministic image resizing
- [ ] **Fixed resample**: Rubato with fixed sinc parameters for audio
- [ ] **Rounding**: 6 decimal precision to remove float drift
- [ ] **No graph optimization**: Disable ONNX graph optimizations
- [ ] **Deterministic seeds**: Fixed RNG seeds (42) for all operations

### Model Configuration
- [ ] **CPU-only execution**: No GPU acceleration
- [ ] **Fixed model versions**: Pin model hashes
- [ ] **Deterministic preprocessing**: RGB8 → fixed resize → normalize → round
- [ ] **Deterministic postprocessing**: Fixed confidence thresholds and NMS
- [ ] **Temperature=0**: Whisper with no randomness
- [ ] **Beam search**: Fixed beam search parameters

### Testing Requirements
- [ ] **Input validation**: Verify seed file hashes
- [ ] **Output validation**: Compare detection/transcript hashes
- [ ] **Environment isolation**: Use `.env.determinism` file
- [ ] **Reproducible builds**: Same compiler, same flags
- [ ] **CI/CD integration**: Automated golden test execution

## When to Rebaseline

### Automatic Rebaseline (Never)
- **Never**: Golden tests should never be automatically updated
- **Risk**: Automatic updates can mask real regressions
- **Principle**: All updates must be intentional and reviewed

### Manual Rebaseline (Required)
- **Model Version Changes**: When upgrading AI models
- **Pipeline Improvements**: When improving preprocessing/postprocessing
- **Bug Fixes**: When fixing bugs that change output format
- **Feature Additions**: When adding new features that affect outputs

### Rebaseline Triggers
1. **CI/CD Failures**: Golden tests failing in CI/CD pipeline
2. **Model Deployment**: New model versions deployed
3. **Code Changes**: Changes to AI inference pipeline
4. **Environment Changes**: New system configurations

## Using .env.determinism

### Environment File
The `.env.determinism` file contains environment variables for deterministic AI inference:

```bash
ORT_NUM_THREADS=1
OMP_NUM_THREADS=1
OPENBLAS_NUM_THREADS=1
MKL_NUM_THREADS=1
PYTHONHASHSEED=0
WHISPER_THREADS=1
AETHERIS_AI_DETERMINISTIC=1
```

### Loading Environment
```bash
# Load deterministic environment
export $(grep -v '^#' .env.determinism | xargs -d '\n') 2>/dev/null || true

# Run golden tests with deterministic environment
python tooling/python/ai_golden_test.py

# Rebaseline with deterministic environment
bash scripts/ai-rebaseline.sh
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Load Deterministic Environment
  run: |
    export $(grep -v '^#' .env.determinism | xargs -d '\n') 2>/dev/null || true
    
- name: Run AI Golden Tests
  run: |
    python tooling/python/ai_golden_test.py
    if [ $? -ne 0 ]; then
      echo "❌ Golden tests failed - drift detected"
      exit 1
    fi
```

### Verification
```bash
# Check environment variables
echo $ORT_NUM_THREADS
echo $OMP_NUM_THREADS
echo $AETHERIS_AI_DETERMINISTIC

# Verify deterministic behavior
python tooling/python/ai_golden_test.py --verbose
```

This comprehensive guide ensures reliable AI golden testing and proper maintenance of deterministic AI behavior in Aetheris OS Phase 4.
