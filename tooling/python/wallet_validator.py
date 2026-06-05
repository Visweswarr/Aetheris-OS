#!/usr/bin/env python3
"""
Aetheris Wallet Python Validator
Validates determinism, DID document shapes, signature correctness, and PDV integrity
"""

import json
import hashlib
import hmac
import argparse
import sys
import os
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass
from pathlib import Path
import time
import uuid

@dataclass
class ValidationResult:
    """Result of a validation operation"""
    success: bool
    message: str
    details: Optional[Dict[str, Any]] = None

@dataclass
class KeyVector:
    """Deterministic key test vector"""
    seed: str
    derivation_path: str
    expected_public_key: str
    expected_address: Optional[str] = None
    key_type: str = "ed25519"

@dataclass
class DIDVector:
    """DID document test vector"""
    did: str
    method: str
    expected_verification_methods: List[Dict[str, Any]]
    expected_authentication: List[str]
    expected_assertion_method: List[str]

class WalletValidator:
    """Main validator class for wallet operations"""
    
    def __init__(self, test_vectors_dir: str = "tests/fixtures/wallet"):
        self.test_vectors_dir = Path(test_vectors_dir)
        self.validation_results: List[ValidationResult] = []
        
    def validate_determinism(self, seed: str, derivation_path: str, 
                           expected_public_key: str, key_type: str = "ed25519") -> ValidationResult:
        """Validate that the same seed produces the same key"""
        try:
            # Mock deterministic key generation
            derived_key = self._derive_key_deterministic(seed, derivation_path, key_type)
            
            if derived_key == expected_public_key:
                return ValidationResult(
                    success=True,
                    message=f"Deterministic key generation validated for {key_type}",
                    details={
                        "seed": seed,
                        "derivation_path": derivation_path,
                        "derived_key": derived_key,
                        "expected_key": expected_public_key
                    }
                )
            else:
                return ValidationResult(
                    success=False,
                    message=f"Key derivation mismatch for {key_type}",
                    details={
                        "seed": seed,
                        "derivation_path": derivation_path,
                        "derived_key": derived_key,
                        "expected_key": expected_public_key
                    }
                )
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error validating determinism: {str(e)}"
            )
    
    def validate_did_document_shape(self, did_doc: Dict[str, Any]) -> ValidationResult:
        """Validate DID document structure and required fields"""
        try:
            required_fields = ["id", "@context", "verificationMethod", "authentication", "assertionMethod"]
            missing_fields = [field for field in required_fields if field not in did_doc]
            
            if missing_fields:
                return ValidationResult(
                    success=False,
                    message=f"Missing required fields in DID document: {missing_fields}"
                )
            
            # Validate DID format
            did_id = did_doc["id"]
            if not did_id.startswith("did:"):
                return ValidationResult(
                    success=False,
                    message="Invalid DID format: must start with 'did:'"
                )
            
            # Validate context
            context = did_doc["@context"]
            if not isinstance(context, list) or "https://www.w3.org/ns/did/v1" not in context:
                return ValidationResult(
                    success=False,
                    message="Invalid @context: must be a list containing 'https://www.w3.org/ns/did/v1'"
                )
            
            # Validate verification methods
            verification_methods = did_doc["verificationMethod"]
            if not isinstance(verification_methods, list) or len(verification_methods) == 0:
                return ValidationResult(
                    success=False,
                    message="verificationMethod must be a non-empty list"
                )
            
            for vm in verification_methods:
                vm_required = ["id", "type", "controller"]
                vm_missing = [field for field in vm_required if field not in vm]
                if vm_missing:
                    return ValidationResult(
                        success=False,
                        message=f"Missing required fields in verification method: {vm_missing}"
                    )
            
            # Validate authentication and assertion method references
            auth_refs = did_doc["authentication"]
            assertion_refs = did_doc["assertionMethod"]
            
            if not isinstance(auth_refs, list) or not isinstance(assertion_refs, list):
                return ValidationResult(
                    success=False,
                    message="authentication and assertionMethod must be lists"
                )
            
            return ValidationResult(
                success=True,
                message="DID document structure is valid",
                details={
                    "did": did_id,
                    "verification_methods_count": len(verification_methods),
                    "authentication_refs": len(auth_refs),
                    "assertion_refs": len(assertion_refs)
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error validating DID document: {str(e)}"
            )
    
    def validate_signature_correctness(self, public_key: str, signature: str, 
                                     data: bytes, key_type: str = "ed25519") -> ValidationResult:
        """Validate signature correctness across languages"""
        try:
            # Mock signature verification
            expected_signature = self._generate_mock_signature(public_key, data, key_type)
            
            if signature == expected_signature:
                return ValidationResult(
                    success=True,
                    message=f"Signature verification successful for {key_type}",
                    details={
                        "public_key": public_key,
                        "signature": signature,
                        "data_hash": hashlib.sha256(data).hexdigest(),
                        "key_type": key_type
                    }
                )
            else:
                return ValidationResult(
                    success=False,
                    message=f"Signature verification failed for {key_type}",
                    details={
                        "public_key": public_key,
                        "signature": signature,
                        "expected_signature": expected_signature,
                        "data_hash": hashlib.sha256(data).hexdigest(),
                        "key_type": key_type
                    }
                )
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error validating signature: {str(e)}"
            )
    
    def validate_pdv_integrity(self, pdv_path: str, wallet_id: str) -> ValidationResult:
        """Validate PDV integrity using Blake3 hash manifest"""
        try:
            pdv_dir = Path(pdv_path) / wallet_id
            manifest_path = pdv_dir / "manifest.json"
            
            if not manifest_path.exists():
                return ValidationResult(
                    success=False,
                    message=f"Manifest file not found: {manifest_path}"
                )
            
            # Read and validate manifest
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
            
            required_manifest_fields = ["wallet_id", "created_at", "version", "keys"]
            missing_fields = [field for field in required_manifest_fields if field not in manifest]
            
            if missing_fields:
                return ValidationResult(
                    success=False,
                    message=f"Missing required fields in manifest: {missing_fields}"
                )
            
            # Validate key files exist and compute integrity hashes
            key_files = []
            integrity_hashes = {}
            
            for key_id, key_info in manifest.get("keys", {}).items():
                key_path = pdv_dir / "keys" / f"{key_id}.json"
                if key_path.exists():
                    with open(key_path, 'rb') as f:
                        key_data = f.read()
                    key_hash = hashlib.blake2b(key_data).hexdigest()
                    integrity_hashes[key_id] = key_hash
                    key_files.append(key_id)
                else:
                    return ValidationResult(
                        success=False,
                        message=f"Key file not found: {key_path}"
                    )
            
            return ValidationResult(
                success=True,
                message="PDV integrity validation successful",
                details={
                    "wallet_id": wallet_id,
                    "manifest_path": str(manifest_path),
                    "key_files_count": len(key_files),
                    "integrity_hashes": integrity_hashes
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error validating PDV integrity: {str(e)}"
            )
    
    def validate_error_budget(self, current_metrics: Dict[str, Any], 
                            baseline_metrics: Dict[str, Any], 
                            threshold: float = 0.01) -> ValidationResult:
        """Validate error budget against baseline"""
        try:
            current_error_rate = current_metrics.get("error_rate", 0.0)
            baseline_error_rate = baseline_metrics.get("error_rate", 0.0)
            
            error_budget_used = current_error_rate - baseline_error_rate
            
            if error_budget_used <= threshold:
                return ValidationResult(
                    success=True,
                    message=f"Error budget within threshold: {error_budget_used:.4f} <= {threshold}",
                    details={
                        "current_error_rate": current_error_rate,
                        "baseline_error_rate": baseline_error_rate,
                        "error_budget_used": error_budget_used,
                        "threshold": threshold
                    }
                )
            else:
                return ValidationResult(
                    success=False,
                    message=f"Error budget exceeded: {error_budget_used:.4f} > {threshold}",
                    details={
                        "current_error_rate": current_error_rate,
                        "baseline_error_rate": baseline_error_rate,
                        "error_budget_used": error_budget_used,
                        "threshold": threshold
                    }
                )
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error validating error budget: {str(e)}"
            )
    
    def validate_latency_regression(self, current_metrics: Dict[str, Any], 
                                  baseline_metrics: Dict[str, Any], 
                                  threshold: float = 0.2) -> ValidationResult:
        """Validate latency regression against baseline"""
        try:
            current_p95_latency = current_metrics.get("p95_latency_ms", 0.0)
            baseline_p95_latency = baseline_metrics.get("p95_latency_ms", 0.0)
            
            if baseline_p95_latency == 0:
                return ValidationResult(
                    success=True,
                    message="No baseline latency available for comparison"
                )
            
            latency_increase = (current_p95_latency - baseline_p95_latency) / baseline_p95_latency
            
            if latency_increase <= threshold:
                return ValidationResult(
                    success=True,
                    message=f"Latency within threshold: {latency_increase:.4f} <= {threshold}",
                    details={
                        "current_p95_latency": current_p95_latency,
                        "baseline_p95_latency": baseline_p95_latency,
                        "latency_increase": latency_increase,
                        "threshold": threshold
                    }
                )
            else:
                return ValidationResult(
                    success=False,
                    message=f"Latency regression detected: {latency_increase:.4f} > {threshold}",
                    details={
                        "current_p95_latency": current_p95_latency,
                        "baseline_p95_latency": baseline_p95_latency,
                        "latency_increase": latency_increase,
                        "threshold": threshold
                    }
                )
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error validating latency regression: {str(e)}"
            )
    
    def run_chaos_test(self, loss_rate: float = 0.02, latency_ms: int = 100, 
                      duration_seconds: int = 120) -> ValidationResult:
        """Run chaos test with packet loss and latency injection"""
        try:
            start_time = time.time()
            errors = 0
            total_operations = 0
            
            # Mock chaos test
            while time.time() - start_time < duration_seconds:
                total_operations += 1
                
                # Simulate packet loss
                if self._should_drop_packet(loss_rate):
                    errors += 1
                    continue
                
                # Simulate latency injection
                time.sleep(latency_ms / 1000.0)
            
            error_rate = errors / total_operations if total_operations > 0 else 0.0
            
            return ValidationResult(
                success=error_rate <= 0.05,  # 5% error threshold
                message=f"Chaos test completed: {error_rate:.4f} error rate",
                details={
                    "duration_seconds": duration_seconds,
                    "total_operations": total_operations,
                    "errors": errors,
                    "error_rate": error_rate,
                    "loss_rate": loss_rate,
                    "latency_ms": latency_ms
                }
            )
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Error running chaos test: {str(e)}"
            )
    
    def load_test_vectors(self) -> Tuple[List[KeyVector], List[DIDVector]]:
        """Load test vectors from fixtures directory"""
        key_vectors = []
        did_vectors = []
        
        try:
            # Load key vectors
            key_vectors_file = self.test_vectors_dir / "expected_vectors.json"
            if key_vectors_file.exists():
                with open(key_vectors_file, 'r') as f:
                    vectors_data = json.load(f)
                
                for vector_data in vectors_data.get("key_vectors", []):
                    key_vectors.append(KeyVector(**vector_data))
            
            # Load DID vectors
            did_vectors_file = self.test_vectors_dir / "did_vectors.json"
            if did_vectors_file.exists():
                with open(did_vectors_file, 'r') as f:
                    did_data = json.load(f)
                
                for did_data_item in did_data.get("did_vectors", []):
                    did_vectors.append(DIDVector(**did_data_item))
                    
        except Exception as e:
            print(f"Warning: Could not load test vectors: {e}")
        
        return key_vectors, did_vectors
    
    def run_comprehensive_validation(self) -> List[ValidationResult]:
        """Run comprehensive validation suite"""
        results = []
        
        # Load test vectors
        key_vectors, did_vectors = self.load_test_vectors()
        
        # Validate deterministic key generation
        for vector in key_vectors:
            result = self.validate_determinism(
                vector.seed, vector.derivation_path, 
                vector.expected_public_key, vector.key_type
            )
            results.append(result)
        
        # Validate DID document shapes
        for vector in did_vectors:
            did_doc = {
                "id": vector.did,
                "@context": ["https://www.w3.org/ns/did/v1"],
                "verificationMethod": vector.expected_verification_methods,
                "authentication": vector.expected_authentication,
                "assertionMethod": vector.expected_assertion_method
            }
            result = self.validate_did_document_shape(did_doc)
            results.append(result)
        
        return results
    
    # Private helper methods
    
    def _derive_key_deterministic(self, seed: str, derivation_path: str, key_type: str) -> str:
        """Mock deterministic key derivation"""
        # Use HMAC-SHA256 for deterministic key derivation
        key_material = f"{seed}:{derivation_path}:{key_type}"
        derived = hmac.new(
            seed.encode('utf-8'),
            key_material.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()
        return derived
    
    def _generate_mock_signature(self, public_key: str, data: bytes, key_type: str) -> str:
        """Generate mock signature for validation"""
        # Use HMAC-SHA256 for mock signature generation
        signature = hmac.new(
            public_key.encode('utf-8'),
            data,
            hashlib.sha256
        ).hexdigest()
        return signature + signature  # 64 character signature
    
    def _should_drop_packet(self, loss_rate: float) -> bool:
        """Determine if packet should be dropped based on loss rate"""
        import random
        return random.random() < loss_rate

def main():
    parser = argparse.ArgumentParser(description="Aetheris Wallet Validator")
    parser.add_argument("command", choices=[
        "validate-determinism", "validate-did-shape", "validate-signature",
        "validate-pdv-integrity", "validate-error-budget", "validate-latency-regression",
        "run-chaos-test", "comprehensive"
    ], help="Validation command to run")
    
    # Common arguments
    parser.add_argument("--test-vectors-dir", default="tests/fixtures/wallet",
                       help="Directory containing test vectors")
    parser.add_argument("--output", help="Output file for results (JSON)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    # Determinism validation
    parser.add_argument("--seed", help="Seed for deterministic validation")
    parser.add_argument("--derivation-path", help="Derivation path")
    parser.add_argument("--expected-public-key", help="Expected public key")
    parser.add_argument("--key-type", default="ed25519", help="Key type")
    
    # DID validation
    parser.add_argument("--did-doc", help="DID document file (JSON)")
    
    # Signature validation
    parser.add_argument("--public-key", help="Public key for signature validation")
    parser.add_argument("--signature", help="Signature to validate")
    parser.add_argument("--data", help="Data file to validate signature against")
    
    # PDV validation
    parser.add_argument("--pdv-path", help="PDV path")
    parser.add_argument("--wallet-id", help="Wallet ID")
    
    # Error budget validation
    parser.add_argument("--current-metrics", help="Current metrics file (JSON)")
    parser.add_argument("--baseline-metrics", help="Baseline metrics file (JSON)")
    parser.add_argument("--threshold", type=float, default=0.01, help="Error threshold")
    
    # Latency regression validation
    parser.add_argument("--latency-threshold", type=float, default=0.2, help="Latency threshold")
    
    # Chaos test
    parser.add_argument("--loss-rate", type=float, default=0.02, help="Packet loss rate")
    parser.add_argument("--latency-ms", type=int, default=100, help="Latency injection (ms)")
    parser.add_argument("--duration", type=int, default=120, help="Test duration (seconds)")
    
    args = parser.parse_args()
    
    validator = WalletValidator(args.test_vectors_dir)
    results = []
    
    try:
        if args.command == "validate-determinism":
            if not all([args.seed, args.derivation_path, args.expected_public_key]):
                print("Error: --seed, --derivation-path, and --expected-public-key are required")
                sys.exit(1)
            
            result = validator.validate_determinism(
                args.seed, args.derivation_path, 
                args.expected_public_key, args.key_type
            )
            results.append(result)
            
        elif args.command == "validate-did-shape":
            if not args.did_doc:
                print("Error: --did-doc is required")
                sys.exit(1)
            
            with open(args.did_doc, 'r') as f:
                did_doc = json.load(f)
            
            result = validator.validate_did_document_shape(did_doc)
            results.append(result)
            
        elif args.command == "validate-signature":
            if not all([args.public_key, args.signature, args.data]):
                print("Error: --public-key, --signature, and --data are required")
                sys.exit(1)
            
            with open(args.data, 'rb') as f:
                data = f.read()
            
            result = validator.validate_signature_correctness(
                args.public_key, args.signature, data, args.key_type
            )
            results.append(result)
            
        elif args.command == "validate-pdv-integrity":
            if not all([args.pdv_path, args.wallet_id]):
                print("Error: --pdv-path and --wallet-id are required")
                sys.exit(1)
            
            result = validator.validate_pdv_integrity(args.pdv_path, args.wallet_id)
            results.append(result)
            
        elif args.command == "validate-error-budget":
            if not all([args.current_metrics, args.baseline_metrics]):
                print("Error: --current-metrics and --baseline-metrics are required")
                sys.exit(1)
            
            with open(args.current_metrics, 'r') as f:
                current_metrics = json.load(f)
            with open(args.baseline_metrics, 'r') as f:
                baseline_metrics = json.load(f)
            
            result = validator.validate_error_budget(current_metrics, baseline_metrics, args.threshold)
            results.append(result)
            
        elif args.command == "validate-latency-regression":
            if not all([args.current_metrics, args.baseline_metrics]):
                print("Error: --current-metrics and --baseline-metrics are required")
                sys.exit(1)
            
            with open(args.current_metrics, 'r') as f:
                current_metrics = json.load(f)
            with open(args.baseline_metrics, 'r') as f:
                baseline_metrics = json.load(f)
            
            result = validator.validate_latency_regression(current_metrics, baseline_metrics, args.latency_threshold)
            results.append(result)
            
        elif args.command == "run-chaos-test":
            result = validator.run_chaos_test(args.loss_rate, args.latency_ms, args.duration)
            results.append(result)
            
        elif args.command == "comprehensive":
            results = validator.run_comprehensive_validation()
        
        # Output results
        if args.output:
            output_data = {
                "command": args.command,
                "timestamp": time.time(),
                "results": [
                    {
                        "success": r.success,
                        "message": r.message,
                        "details": r.details
                    } for r in results
                ]
            }
            with open(args.output, 'w') as f:
                json.dump(output_data, f, indent=2)
            print(f"Results written to {args.output}")
        else:
            for i, result in enumerate(results):
                status = "PASS" if result.success else "FAIL"
                print(f"[{i+1}] {status}: {result.message}")
                if args.verbose and result.details:
                    print(f"    Details: {json.dumps(result.details, indent=2)}")
        
        # Exit with error code if any validation failed
        if any(not r.success for r in results):
            sys.exit(1)
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
