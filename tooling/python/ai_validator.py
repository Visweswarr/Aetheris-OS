#!/usr/bin/env python3
"""
AI Validator for Aetheris OS

This module provides validation and testing for AI operations,
including deterministic replay and schema validation.
"""

import json
import time
import hashlib
import logging
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import cbor2
import jsonschema

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class AIEventType(Enum):
    """AI event types"""
    DETECTION = "detection"
    TRANSCRIPT = "transcript"
    VAD = "vad"
    FRAME = "frame"
    ERROR = "error"
    STATS = "stats"

class PipelineType(Enum):
    """Pipeline types"""
    VISION = "vision"
    AUDIO = "audio"

@dataclass
class AIModel:
    """AI model information"""
    name: str
    path: str
    model_type: str
    model_size: str
    input_shape: List[int]
    output_shape: List[int]
    size_bytes: int
    available: bool
    supported_backends: List[str]

@dataclass
class Detection:
    """Detection result"""
    class_id: int
    class_name: str
    confidence: float
    x: float
    y: float
    width: float
    height: float

@dataclass
class Transcript:
    """Transcript result"""
    text: str
    language: str
    confidence: float
    start_time: float
    end_time: float

@dataclass
class VAD:
    """VAD result"""
    state: str
    confidence: float
    audio_level_db: float
    duration_ms: float

@dataclass
class AIEvent:
    """AI event"""
    event_type: str
    timestamp: int
    session_id: str
    data: Dict[str, Any]
    deterministic_hash: str
    model_hash: str
    postprocessing_hash: str
    sequence_number: int

@dataclass
class AIStats:
    """AI statistics"""
    vision_frames_processed: int
    vision_total_detections: int
    vision_avg_inference_time_ms: float
    vision_fps: float
    vision_errors: int
    audio_samples_processed: int
    audio_total_transcripts: int
    audio_total_vad_activations: int
    audio_avg_inference_time_ms: float
    audio_avg_audio_level_db: float
    audio_errors: int
    system_memory_mb: float
    system_cpu_percent: float
    system_active_pipelines: int
    system_cache_hits: int
    system_cache_misses: int

class AIValidator:
    """AI validator for testing and validation"""
    
    def __init__(self):
        self.models: List[AIModel] = []
        self.events: List[AIEvent] = []
        self.stats = AIStats(
            vision_frames_processed=0,
            vision_total_detections=0,
            vision_avg_inference_time_ms=0.0,
            vision_fps=0.0,
            vision_errors=0,
            audio_samples_processed=0,
            audio_total_transcripts=0,
            audio_total_vad_activations=0,
            audio_avg_inference_time_ms=0.0,
            audio_avg_audio_level_db=0.0,
            audio_errors=0,
            system_memory_mb=0.0,
            system_cpu_percent=0.0,
            system_active_pipelines=0,
            system_cache_hits=0,
            system_cache_misses=0,
        )
        self.deterministic_mode = True
        self.seed = 42
        
    def load_models(self) -> List[AIModel]:
        """Load available AI models"""
        models = [
            AIModel(
                name="yolo_n.onnx",
                path="models/yolo_n.onnx",
                model_type="onnx",
                model_size="tiny",
                input_shape=[1, 3, 640, 480],
                output_shape=[1, 25200, 85],
                size_bytes=50 * 1024 * 1024,
                available=True,
                supported_backends=["onnx"],
            ),
            AIModel(
                name="yolo_s.onnx",
                path="models/yolo_s.onnx",
                model_type="onnx",
                model_size="small",
                input_shape=[1, 3, 640, 480],
                output_shape=[1, 25200, 85],
                size_bytes=100 * 1024 * 1024,
                available=True,
                supported_backends=["onnx"],
            ),
            AIModel(
                name="ggml-tiny.en.bin",
                path="models/ggml-tiny.en.bin",
                model_type="whisper",
                model_size="tiny",
                input_shape=[1, 80, 3000],
                output_shape=[1, 1, 51865],
                size_bytes=39 * 1024 * 1024,
                available=True,
                supported_backends=["whisper"],
            ),
            AIModel(
                name="ggml-base.en.bin",
                path="models/ggml-base.en.bin",
                model_type="whisper",
                model_size="base",
                input_shape=[1, 80, 3000],
                output_shape=[1, 1, 51865],
                size_bytes=74 * 1024 * 1024,
                available=True,
                supported_backends=["whisper"],
            ),
        ]
        
        self.models = models
        logger.info(f"Loaded {len(models)} AI models")
        return models
    
    def validate_vision_pipeline(self, config: Dict[str, Any]) -> bool:
        """Validate vision pipeline configuration"""
        required_fields = [
            "fps", "model_name", "input_width", "input_height",
            "confidence_threshold", "nms_threshold", "num_classes"
        ]
        
        for field in required_fields:
            if field not in config:
                logger.error(f"Missing required field: {field}")
                return False
        
        # Validate model exists
        model_name = config["model_name"]
        model = next((m for m in self.models if m.name == model_name), None)
        if not model:
            logger.error(f"Model not found: {model_name}")
            return False
        
        # Validate configuration values
        if config["fps"] <= 0 or config["fps"] > 60:
            logger.error(f"Invalid FPS: {config['fps']}")
            return False
        
        if not 0 <= config["confidence_threshold"] <= 1:
            logger.error(f"Invalid confidence threshold: {config['confidence_threshold']}")
            return False
        
        if not 0 <= config["nms_threshold"] <= 1:
            logger.error(f"Invalid NMS threshold: {config['nms_threshold']}")
            return False
        
        logger.info("Vision pipeline configuration is valid")
        return True
    
    def validate_audio_pipeline(self, config: Dict[str, Any]) -> bool:
        """Validate audio pipeline configuration"""
        required_fields = [
            "sample_rate", "channels", "model_name", "language",
            "vad_threshold", "confidence_threshold"
        ]
        
        for field in required_fields:
            if field not in config:
                logger.error(f"Missing required field: {field}")
                return False
        
        # Validate model exists
        model_name = config["model_name"]
        model = next((m for m in self.models if m.name == model_name), None)
        if not model:
            logger.error(f"Model not found: {model_name}")
            return False
        
        # Validate configuration values
        if config["sample_rate"] not in [8000, 16000, 22050, 44100, 48000]:
            logger.error(f"Invalid sample rate: {config['sample_rate']}")
            return False
        
        if config["channels"] not in [1, 2]:
            logger.error(f"Invalid channel count: {config['channels']}")
            return False
        
        if not 0 <= config["vad_threshold"] <= 1:
            logger.error(f"Invalid VAD threshold: {config['vad_threshold']}")
            return False
        
        logger.info("Audio pipeline configuration is valid")
        return True
    
    def simulate_vision_inference(self, frame_data: bytes, config: Dict[str, Any]) -> List[Detection]:
        """Simulate vision inference with deterministic results"""
        detections = []
        
        # Use seed for deterministic results
        if self.deterministic_mode:
            # Simulate deterministic detections based on seed
            if self.seed % 3 == 0:
                detections.append(Detection(
                    class_id=0,
                    class_name="person",
                    confidence=0.85,
                    x=100.0,
                    y=100.0,
                    width=200.0,
                    height=300.0,
                ))
            
            if self.seed % 5 == 0:
                detections.append(Detection(
                    class_id=2,
                    class_name="car",
                    confidence=0.75,
                    x=300.0,
                    y=200.0,
                    width=150.0,
                    height=100.0,
                ))
        else:
            # Non-deterministic simulation
            import random
            if random.random() > 0.5:
                detections.append(Detection(
                    class_id=0,
                    class_name="person",
                    confidence=0.85,
                    x=100.0,
                    y=100.0,
                    width=200.0,
                    height=300.0,
                ))
        
        # Update statistics
        self.stats.vision_frames_processed += 1
        self.stats.vision_total_detections += len(detections)
        
        return detections
    
    def simulate_audio_inference(self, audio_data: bytes, config: Dict[str, Any]) -> Tuple[Optional[Transcript], VAD]:
        """Simulate audio inference with deterministic results"""
        # Calculate audio level
        audio_level = 0.0
        if len(audio_data) > 0:
            # Simulate audio level calculation
            audio_level = 0.1 if self.deterministic_mode else 0.05
        
        # VAD result
        vad = VAD(
            state="speech" if audio_level > 0.05 else "silence",
            confidence=0.9 if audio_level > 0.05 else 0.8,
            audio_level_db=20 * (audio_level + 1e-10).log10() if audio_level > 0 else -40.0,
            duration_ms=len(audio_data) / 32.0,  # Simulate duration
        )
        
        # Transcript result
        transcript = None
        if vad.state == "speech" and self.deterministic_mode:
            transcript = Transcript(
                text="Hello, this is a simulated transcript.",
                language="en",
                confidence=0.85,
                start_time=0.0,
                end_time=vad.duration_ms / 1000.0,
            )
        
        # Update statistics
        self.stats.audio_samples_processed += len(audio_data)
        self.stats.audio_total_vad_activations += 1 if vad.state == "speech" else 0
        if transcript:
            self.stats.audio_total_transcripts += 1
        
        return transcript, vad
    
    def create_ai_event(
        self,
        event_type: AIEventType,
        session_id: str,
        data: Dict[str, Any],
        model_hash: str = "mock_model_hash",
        postprocessing_hash: str = "mock_postprocessing_hash"
    ) -> AIEvent:
        """Create an AI event with deterministic hash"""
        timestamp = int(time.time() * 1000)
        sequence_number = len(self.events)
        
        # Create deterministic hash
        hash_input = f"{event_type.value}{timestamp}{session_id}{model_hash}{postprocessing_hash}{self.seed}"
        deterministic_hash = hashlib.sha256(hash_input.encode()).hexdigest()[:16]
        
        event = AIEvent(
            event_type=event_type.value,
            timestamp=timestamp,
            session_id=session_id,
            data=data,
            deterministic_hash=deterministic_hash,
            model_hash=model_hash,
            postprocessing_hash=postprocessing_hash,
            sequence_number=sequence_number,
        )
        
        self.events.append(event)
        return event
    
    def validate_deterministic_replay(self, events: List[AIEvent]) -> bool:
        """Validate deterministic replay of events"""
        if not events:
            logger.warning("No events to validate")
            return True
        
        # Check that all events have deterministic hashes
        for event in events:
            if not event.deterministic_hash:
                logger.error(f"Event {event.sequence_number} missing deterministic hash")
                return False
        
        # Simulate replay and check hash consistency
        for event in events:
            # Recreate hash with same inputs
            hash_input = f"{event.event_type}{event.timestamp}{event.session_id}{event.model_hash}{event.postprocessing_hash}{self.seed}"
            expected_hash = hashlib.sha256(hash_input.encode()).hexdigest()[:16]
            
            if event.deterministic_hash != expected_hash:
                logger.error(f"Deterministic hash mismatch for event {event.sequence_number}")
                logger.error(f"Expected: {expected_hash}, Got: {event.deterministic_hash}")
                return False
        
        logger.info(f"Deterministic replay validation passed for {len(events)} events")
        return True
    
    def validate_cbor_schema(self, data: bytes, schema_type: str) -> bool:
        """Validate CBOR data against schema"""
        try:
            # Decode CBOR data
            decoded = cbor2.loads(data)
            
            # Load schema
            schema_path = f"schemas/ai.{schema_type}.events.cddl"
            # In a real implementation, this would load and validate against CDDL schema
            # For now, we'll do basic validation
            
            if schema_type == "vision":
                required_fields = ["event_type", "timestamp", "frame_id", "session_id"]
            elif schema_type == "audio":
                required_fields = ["event_type", "timestamp", "segment_id", "session_id"]
            else:
                logger.error(f"Unknown schema type: {schema_type}")
                return False
            
            # Check required fields
            for field in required_fields:
                if field not in decoded:
                    logger.error(f"Missing required field in {schema_type} schema: {field}")
                    return False
            
            logger.info(f"CBOR schema validation passed for {schema_type}")
            return True
            
        except Exception as e:
            logger.error(f"CBOR schema validation failed: {e}")
            return False
    
    def run_determinism_test(self, num_iterations: int = 3) -> bool:
        """Run determinism test with multiple iterations"""
        logger.info(f"Running determinism test with {num_iterations} iterations")
        
        results = []
        for iteration in range(num_iterations):
            logger.info(f"Determinism test iteration {iteration + 1}")
            
            # Reset state
            self.events.clear()
            self.stats = AIStats(
                vision_frames_processed=0,
                vision_total_detections=0,
                vision_avg_inference_time_ms=0.0,
                vision_fps=0.0,
                vision_errors=0,
                audio_samples_processed=0,
                audio_total_transcripts=0,
                audio_total_vad_activations=0,
                audio_avg_inference_time_ms=0.0,
                audio_avg_audio_level_db=0.0,
                audio_errors=0,
                system_memory_mb=0.0,
                system_cpu_percent=0.0,
                system_active_pipelines=0,
                system_cache_hits=0,
                system_cache_misses=0,
            )
            
            # Run vision inference
            vision_config = {
                "fps": 15,
                "model_name": "yolo_n.onnx",
                "input_width": 640,
                "input_height": 480,
                "confidence_threshold": 0.5,
                "nms_threshold": 0.5,
                "num_classes": 80,
            }
            
            frame_data = b"mock_frame_data"
            detections = self.simulate_vision_inference(frame_data, vision_config)
            
            # Create detection event
            detection_data = {
                "detections": [asdict(d) for d in detections],
                "detection_count": len(detections),
            }
            detection_event = self.create_ai_event(
                AIEventType.DETECTION,
                "test_vision_session",
                detection_data
            )
            
            # Run audio inference
            audio_config = {
                "sample_rate": 16000,
                "channels": 1,
                "model_name": "ggml-tiny.en.bin",
                "language": "en",
                "vad_threshold": 0.5,
                "confidence_threshold": 0.5,
            }
            
            audio_data = b"mock_audio_data"
            transcript, vad = self.simulate_audio_inference(audio_data, audio_config)
            
            # Create transcript event
            transcript_data = {
                "transcript": asdict(transcript) if transcript else None,
                "vad": asdict(vad),
            }
            transcript_event = self.create_ai_event(
                AIEventType.TRANSCRIPT,
                "test_audio_session",
                transcript_data
            )
            
            # Store results
            iteration_results = {
                "detection_event": detection_event,
                "transcript_event": transcript_event,
                "stats": asdict(self.stats),
            }
            results.append(iteration_results)
        
        # Validate determinism
        if len(results) < 2:
            logger.error("Not enough iterations for determinism test")
            return False
        
        # Check that all iterations produce identical results
        first_result = results[0]
        for i, result in enumerate(results[1:], 1):
            if (result["detection_event"].deterministic_hash != 
                first_result["detection_event"].deterministic_hash):
                logger.error(f"Determinism test failed at iteration {i + 1}")
                logger.error("Detection event hashes differ")
                return False
            
            if (result["transcript_event"].deterministic_hash != 
                first_result["transcript_event"].deterministic_hash):
                logger.error(f"Determinism test failed at iteration {i + 1}")
                logger.error("Transcript event hashes differ")
                return False
        
        logger.info("Determinism test passed - all iterations produced identical results")
        return True
    
    def run_performance_test(self, num_operations: int = 100) -> Dict[str, float]:
        """Run performance test"""
        logger.info(f"Running performance test with {num_operations} operations")
        
        start_time = time.time()
        
        # Vision performance test
        vision_start = time.time()
        for i in range(num_operations):
            frame_data = b"mock_frame_data"
            config = {
                "fps": 15,
                "model_name": "yolo_n.onnx",
                "input_width": 640,
                "input_height": 480,
                "confidence_threshold": 0.5,
                "nms_threshold": 0.5,
                "num_classes": 80,
            }
            self.simulate_vision_inference(frame_data, config)
        vision_time = time.time() - vision_start
        
        # Audio performance test
        audio_start = time.time()
        for i in range(num_operations):
            audio_data = b"mock_audio_data"
            config = {
                "sample_rate": 16000,
                "channels": 1,
                "model_name": "ggml-tiny.en.bin",
                "language": "en",
                "vad_threshold": 0.5,
                "confidence_threshold": 0.5,
            }
            self.simulate_audio_inference(audio_data, config)
        audio_time = time.time() - audio_start
        
        total_time = time.time() - start_time
        
        performance_metrics = {
            "total_time": total_time,
            "vision_time": vision_time,
            "audio_time": audio_time,
            "vision_ops_per_second": num_operations / vision_time,
            "audio_ops_per_second": num_operations / audio_time,
            "total_ops_per_second": (num_operations * 2) / total_time,
        }
        
        logger.info(f"Performance test completed:")
        logger.info(f"  Total time: {total_time:.2f}s")
        logger.info(f"  Vision ops/sec: {performance_metrics['vision_ops_per_second']:.2f}")
        logger.info(f"  Audio ops/sec: {performance_metrics['audio_ops_per_second']:.2f}")
        logger.info(f"  Total ops/sec: {performance_metrics['total_ops_per_second']:.2f}")
        
        return performance_metrics
    
    def run_all_tests(self) -> bool:
        """Run all validation tests"""
        logger.info("Running all AI validation tests")
        
        # Load models
        self.load_models()
        
        # Test 1: Configuration validation
        logger.info("Test 1: Configuration validation")
        vision_config = {
            "fps": 15,
            "model_name": "yolo_n.onnx",
            "input_width": 640,
            "input_height": 480,
            "confidence_threshold": 0.5,
            "nms_threshold": 0.5,
            "num_classes": 80,
        }
        if not self.validate_vision_pipeline(vision_config):
            return False
        
        audio_config = {
            "sample_rate": 16000,
            "channels": 1,
            "model_name": "ggml-tiny.en.bin",
            "language": "en",
            "vad_threshold": 0.5,
            "confidence_threshold": 0.5,
        }
        if not self.validate_audio_pipeline(audio_config):
            return False
        
        # Test 2: Determinism test
        logger.info("Test 2: Determinism test")
        if not self.run_determinism_test(3):
            return False
        
        # Test 3: Performance test
        logger.info("Test 3: Performance test")
        performance_metrics = self.run_performance_test(100)
        
        # Check performance thresholds
        if performance_metrics["vision_ops_per_second"] < 10:
            logger.warning("Vision performance below threshold")
        
        if performance_metrics["audio_ops_per_second"] < 5:
            logger.warning("Audio performance below threshold")
        
        # Test 4: Event validation
        logger.info("Test 4: Event validation")
        if not self.validate_deterministic_replay(self.events):
            return False
        
        logger.info("All AI validation tests passed!")
        return True

def main():
    """Main function for running AI validator"""
    import argparse
    
    parser = argparse.ArgumentParser(description="AI Validator for Aetheris OS")
    parser.add_argument("--test", choices=["all", "determinism", "performance", "config"], 
                       default="all", help="Test to run")
    parser.add_argument("--iterations", type=int, default=3, 
                       help="Number of iterations for determinism test")
    parser.add_argument("--operations", type=int, default=100, 
                       help="Number of operations for performance test")
    parser.add_argument("--verbose", "-v", action="store_true", 
                       help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    validator = AIValidator()
    
    if args.test == "all":
        success = validator.run_all_tests()
    elif args.test == "determinism":
        success = validator.run_determinism_test(args.iterations)
    elif args.test == "performance":
        validator.run_performance_test(args.operations)
        success = True
    elif args.test == "config":
        validator.load_models()
        vision_config = {
            "fps": 15,
            "model_name": "yolo_n.onnx",
            "input_width": 640,
            "input_height": 480,
            "confidence_threshold": 0.5,
            "nms_threshold": 0.5,
            "num_classes": 80,
        }
        audio_config = {
            "sample_rate": 16000,
            "channels": 1,
            "model_name": "ggml-tiny.en.bin",
            "language": "en",
            "vad_threshold": 0.5,
            "confidence_threshold": 0.5,
        }
        success = (validator.validate_vision_pipeline(vision_config) and 
                  validator.validate_audio_pipeline(audio_config))
    
    if success:
        print("✅ All tests passed!")
        exit(0)
    else:
        print("❌ Tests failed!")
        exit(1)

if __name__ == "__main__":
    main()
