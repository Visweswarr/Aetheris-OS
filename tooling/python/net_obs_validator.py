#!/usr/bin/env python3
"""
Network Observability Validator for Aetheris OS

This module provides validation tools for metrics and chaos testing,
ensuring that observability data is correct and chaos injections are working as expected.
"""

import json
import argparse
import sys
import os
import time
import statistics
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from pathlib import Path
import requests
import subprocess

class ObservabilityValidator:
    """Validates observability data and chaos testing results."""
    
    def __init__(self, prometheus_url: str = "http://localhost:9090"):
        self.prometheus_url = prometheus_url
        self.validation_results = []
    
    def validate_metrics_file(self, metrics_file: str) -> Dict[str, Any]:
        """Validate a metrics JSON file."""
        try:
            with open(metrics_file, 'r') as f:
                metrics = json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            return {
                "valid": False,
                "error": f"Failed to load metrics file: {e}",
                "file": metrics_file
            }
        
        validation_result = {
            "valid": True,
            "file": metrics_file,
            "metrics_count": len(metrics),
            "metric_types": {},
            "issues": []
        }
        
        # Validate each metric
        for metric in metrics:
            if not self._validate_metric(metric):
                validation_result["valid"] = False
                validation_result["issues"].append(f"Invalid metric: {metric.get('name', 'unknown')}")
        
        # Count metric types
        for metric in metrics:
            metric_type = metric.get("type", "unknown")
            validation_result["metric_types"][metric_type] = validation_result["metric_types"].get(metric_type, 0) + 1
        
        return validation_result
    
    def _validate_metric(self, metric: Dict[str, Any]) -> bool:
        """Validate a single metric."""
        required_fields = ["name", "help", "type", "value", "timestamp"]
        
        # Check required fields
        for field in required_fields:
            if field not in metric:
                return False
        
        # Validate metric type
        valid_types = ["counter", "gauge", "histogram", "summary"]
        if metric["type"] not in valid_types:
            return False
        
        # Validate value structure
        value = metric["value"]
        if not isinstance(value, dict) or "type" not in value or "value" not in value:
            return False
        
        # Validate value type matches metric type
        if value["type"] != metric["type"]:
            return False
        
        # Type-specific validation
        if metric["type"] == "histogram":
            return self._validate_histogram_value(value["value"])
        elif metric["type"] == "summary":
            return self._validate_summary_value(value["value"])
        
        return True
    
    def _validate_histogram_value(self, value: Any) -> bool:
        """Validate histogram value structure."""
        if not isinstance(value, dict):
            return False
        
        required_fields = ["buckets", "count", "sum"]
        for field in required_fields:
            if field not in value:
                return False
        
        # Validate buckets
        buckets = value["buckets"]
        if not isinstance(buckets, list):
            return False
        
        for bucket in buckets:
            if not isinstance(bucket, dict) or "upper_bound" not in bucket or "count" not in bucket:
                return False
        
        return True
    
    def _validate_summary_value(self, value: Any) -> bool:
        """Validate summary value structure."""
        if not isinstance(value, dict):
            return False
        
        required_fields = ["quantiles", "count", "sum"]
        for field in required_fields:
            if field not in value:
                return False
        
        # Validate quantiles
        quantiles = value["quantiles"]
        if not isinstance(quantiles, list):
            return False
        
        for quantile in quantiles:
            if not isinstance(quantile, dict) or "quantile" not in quantile or "value" not in quantile:
                return False
        
        return True
    
    def validate_prometheus_endpoint(self) -> Dict[str, Any]:
        """Validate Prometheus metrics endpoint."""
        try:
            response = requests.get(f"{self.prometheus_url}/metrics", timeout=10)
            response.raise_for_status()
        except requests.RequestException as e:
            return {
                "valid": False,
                "error": f"Failed to connect to Prometheus endpoint: {e}",
                "url": self.prometheus_url
            }
        
        metrics_text = response.text
        lines = metrics_text.strip().split('\n')
        
        validation_result = {
            "valid": True,
            "url": self.prometheus_url,
            "total_lines": len(lines),
            "metric_lines": 0,
            "help_lines": 0,
            "type_lines": 0,
            "issues": []
        }
        
        # Parse Prometheus format
        for line in lines:
            if line.startswith('# HELP'):
                validation_result["help_lines"] += 1
            elif line.startswith('# TYPE'):
                validation_result["type_lines"] += 1
            elif line and not line.startswith('#'):
                validation_result["metric_lines"] += 1
                
                # Validate metric line format
                if not self._validate_prometheus_metric_line(line):
                    validation_result["valid"] = False
                    validation_result["issues"].append(f"Invalid metric line: {line}")
        
        return validation_result
    
    def _validate_prometheus_metric_line(self, line: str) -> bool:
        """Validate a single Prometheus metric line."""
        # Basic format: metric_name{labels} value timestamp
        parts = line.split()
        if len(parts) < 2:
            return False
        
        metric_part = parts[0]
        value_part = parts[1]
        
        # Validate metric name and labels
        if '{' in metric_part:
            name, labels = metric_part.split('{', 1)
            if not labels.endswith('}'):
                return False
        else:
            name = metric_part
        
        if not name or not name.replace('_', '').replace(':', '').isalnum():
            return False
        
        # Validate value
        try:
            float(value_part)
        except ValueError:
            return False
        
        return True
    
    def validate_chaos_results(self, chaos_file: str) -> Dict[str, Any]:
        """Validate chaos injection results."""
        try:
            with open(chaos_file, 'r') as f:
                chaos_results = json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            return {
                "valid": False,
                "error": f"Failed to load chaos results file: {e}",
                "file": chaos_file
            }
        
        validation_result = {
            "valid": True,
            "file": chaos_file,
            "injections_count": len(chaos_results),
            "injection_types": {},
            "issues": []
        }
        
        # Validate each chaos result
        for result in chaos_results:
            if not self._validate_chaos_result(result):
                validation_result["valid"] = False
                validation_result["issues"].append(f"Invalid chaos result: {result.get('chaos_type', 'unknown')}")
        
        # Count injection types
        for result in chaos_results:
            chaos_type = result.get("chaos_type", "unknown")
            validation_result["injection_types"][chaos_type] = validation_result["injection_types"].get(chaos_type, 0) + 1
        
        return validation_result
    
    def _validate_chaos_result(self, result: Dict[str, Any]) -> bool:
        """Validate a single chaos result."""
        required_fields = ["chaos_type", "applied", "duration", "packets_dropped", 
                          "packets_delayed", "connections_churned", "error_count"]
        
        # Check required fields
        for field in required_fields:
            if field not in result:
                return False
        
        # Validate chaos type
        valid_types = ["packet_loss", "latency", "jitter", "connection_churn", 
                      "bandwidth_limit", "cpu_stress", "memory_stress"]
        if result["chaos_type"] not in valid_types:
            return False
        
        # Validate numeric fields
        numeric_fields = ["duration", "packets_dropped", "packets_delayed", 
                         "connections_churned", "error_count"]
        for field in numeric_fields:
            if not isinstance(result[field], (int, float)):
                return False
        
        return True
    
    def validate_trace_file(self, trace_file: str) -> Dict[str, Any]:
        """Validate a trace JSON file."""
        try:
            with open(trace_file, 'r') as f:
                traces = json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            return {
                "valid": False,
                "error": f"Failed to load trace file: {e}",
                "file": trace_file
            }
        
        validation_result = {
            "valid": True,
            "file": trace_file,
            "spans_count": len(traces),
            "trace_ids": set(),
            "span_kinds": {},
            "issues": []
        }
        
        # Validate each span
        for span in traces:
            if not self._validate_span(span):
                validation_result["valid"] = False
                validation_result["issues"].append(f"Invalid span: {span.get('name', 'unknown')}")
            
            # Collect trace IDs
            if "trace_id" in span:
                validation_result["trace_ids"].add(span["trace_id"])
            
            # Count span kinds
            span_kind = span.get("kind", "unknown")
            validation_result["span_kinds"][span_kind] = validation_result["span_kinds"].get(span_kind, 0) + 1
        
        validation_result["unique_traces"] = len(validation_result["trace_ids"])
        
        return validation_result
    
    def _validate_span(self, span: Dict[str, Any]) -> bool:
        """Validate a single span."""
        required_fields = ["trace_id", "span_id", "name", "kind", "start_time", "status"]
        
        # Check required fields
        for field in required_fields:
            if field not in span:
                return False
        
        # Validate span kind
        valid_kinds = [0, 1, 2, 3, 4]  # Internal, Server, Client, Producer, Consumer
        if span["kind"] not in valid_kinds:
            return False
        
        # Validate status
        status = span["status"]
        if not isinstance(status, dict) or "code" not in status:
            return False
        
        valid_status_codes = [0, 1, 2]  # Unset, Ok, Error
        if status["code"] not in valid_status_codes:
            return False
        
        return True
    
    def validate_error_budget(self, metrics_file: str, baseline_file: str, 
                            error_threshold: float = 0.01) -> Dict[str, Any]:
        """Validate error budget against baseline."""
        try:
            with open(metrics_file, 'r') as f:
                current_metrics = json.load(f)
            
            with open(baseline_file, 'r') as f:
                baseline_metrics = json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            return {
                "valid": False,
                "error": f"Failed to load metrics files: {e}",
                "error_budget_exceeded": False
            }
        
        # Find error metrics
        current_errors = self._extract_error_metrics(current_metrics)
        baseline_errors = self._extract_error_metrics(baseline_metrics)
        
        validation_result = {
            "valid": True,
            "error_budget_exceeded": False,
            "current_error_rate": 0.0,
            "baseline_error_rate": 0.0,
            "error_rate_delta": 0.0,
            "threshold": error_threshold,
            "issues": []
        }
        
        if current_errors and baseline_errors:
            current_rate = current_errors["error_rate"]
            baseline_rate = baseline_errors["error_rate"]
            
            validation_result["current_error_rate"] = current_rate
            validation_result["baseline_error_rate"] = baseline_rate
            validation_result["error_rate_delta"] = current_rate - baseline_rate
            
            if current_rate > error_threshold:
                validation_result["valid"] = False
                validation_result["error_budget_exceeded"] = True
                validation_result["issues"].append(f"Error rate {current_rate:.4f} exceeds threshold {error_threshold}")
        
        return validation_result
    
    def _extract_error_metrics(self, metrics: List[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
        """Extract error metrics from a metrics list."""
        error_metrics = [m for m in metrics if "error" in m.get("name", "").lower()]
        
        if not error_metrics:
            return None
        
        total_requests = 0
        total_errors = 0
        
        for metric in error_metrics:
            if "requests" in metric.get("name", "").lower():
                total_requests = metric.get("value", {}).get("value", 0)
            elif "errors" in metric.get("name", "").lower():
                total_errors = metric.get("value", {}).get("value", 0)
        
        if total_requests > 0:
            error_rate = total_errors / total_requests
            return {
                "total_requests": total_requests,
                "total_errors": total_errors,
                "error_rate": error_rate
            }
        
        return None
    
    def validate_latency_regression(self, metrics_file: str, baseline_file: str,
                                  regression_threshold: float = 0.2) -> Dict[str, Any]:
        """Validate latency regression against baseline."""
        try:
            with open(metrics_file, 'r') as f:
                current_metrics = json.load(f)
            
            with open(baseline_file, 'r') as f:
                baseline_metrics = json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            return {
                "valid": False,
                "error": f"Failed to load metrics files: {e}",
                "latency_regression": False
            }
        
        # Find latency metrics
        current_latency = self._extract_latency_metrics(current_metrics)
        baseline_latency = self._extract_latency_metrics(baseline_metrics)
        
        validation_result = {
            "valid": True,
            "latency_regression": False,
            "current_p95": 0.0,
            "baseline_p95": 0.0,
            "latency_delta": 0.0,
            "regression_threshold": regression_threshold,
            "issues": []
        }
        
        if current_latency and baseline_latency:
            current_p95 = current_latency["p95"]
            baseline_p95 = baseline_latency["p95"]
            
            validation_result["current_p95"] = current_p95
            validation_result["baseline_p95"] = baseline_p95
            validation_result["latency_delta"] = (current_p95 - baseline_p95) / baseline_p95
            
            if validation_result["latency_delta"] > regression_threshold:
                validation_result["valid"] = False
                validation_result["latency_regression"] = True
                validation_result["issues"].append(
                    f"Latency regression: {validation_result['latency_delta']:.2%} exceeds threshold {regression_threshold:.2%}"
                )
        
        return validation_result
    
    def _extract_latency_metrics(self, metrics: List[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
        """Extract latency metrics from a metrics list."""
        latency_metrics = [m for m in metrics if "latency" in m.get("name", "").lower()]
        
        if not latency_metrics:
            return None
        
        for metric in latency_metrics:
            if metric.get("type") == "summary":
                value = metric.get("value", {}).get("value", {})
                quantiles = value.get("quantiles", [])
                
                for quantile in quantiles:
                    if quantile.get("quantile") == 0.95:
                        return {
                            "p95": quantile.get("value", 0.0),
                            "p50": next((q.get("value", 0.0) for q in quantiles if q.get("quantile") == 0.5), 0.0),
                            "p99": next((q.get("value", 0.0) for q in quantiles if q.get("quantile") == 0.99), 0.0)
                        }
        
        return None
    
    def run_chaos_test(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Run a chaos test and validate results."""
        test_result = {
            "valid": True,
            "test_config": config,
            "start_time": datetime.now().isoformat(),
            "issues": []
        }
        
        try:
            # Run chaos injection
            cmd = [
                "go", "run", "go/tooling/netctl/main.go", "chaos", "inject",
                "--loss", str(config.get("packet_loss_rate", 0.0)),
                "--latency", str(config.get("latency_ms", 0.0)),
                "--duration", str(config.get("duration_ms", 60000)),
                "--output", config.get("output_file", "chaos_results.json")
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            if result.returncode != 0:
                test_result["valid"] = False
                test_result["issues"].append(f"Chaos test failed: {result.stderr}")
                return test_result
            
            # Validate chaos results
            chaos_validation = self.validate_chaos_results(config.get("output_file", "chaos_results.json"))
            if not chaos_validation["valid"]:
                test_result["valid"] = False
                test_result["issues"].extend(chaos_validation["issues"])
            
            test_result["chaos_validation"] = chaos_validation
            
        except subprocess.TimeoutExpired:
            test_result["valid"] = False
            test_result["issues"].append("Chaos test timed out")
        except Exception as e:
            test_result["valid"] = False
            test_result["issues"].append(f"Chaos test error: {e}")
        
        test_result["end_time"] = datetime.now().isoformat()
        return test_result
    
    def generate_report(self, validation_results: List[Dict[str, Any]]) -> str:
        """Generate a validation report."""
        report = []
        report.append("Network Observability Validation Report")
        report.append("=" * 50)
        report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")
        
        total_validations = len(validation_results)
        valid_validations = sum(1 for r in validation_results if r.get("valid", False))
        
        report.append(f"Total Validations: {total_validations}")
        report.append(f"Valid: {valid_validations}")
        report.append(f"Invalid: {total_validations - valid_validations}")
        report.append("")
        
        for i, result in enumerate(validation_results, 1):
            report.append(f"Validation {i}:")
            report.append(f"  Type: {result.get('type', 'unknown')}")
            report.append(f"  Valid: {result.get('valid', False)}")
            
            if result.get("file"):
                report.append(f"  File: {result['file']}")
            
            if result.get("issues"):
                report.append("  Issues:")
                for issue in result["issues"]:
                    report.append(f"    - {issue}")
            
            if result.get("error"):
                report.append(f"  Error: {result['error']}")
            
            report.append("")
        
        return "\n".join(report)

def main():
    """Main CLI interface."""
    parser = argparse.ArgumentParser(
        description="Network Observability Validator for Aetheris OS"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Validate metrics command
    metrics_parser = subparsers.add_parser("validate-metrics", help="Validate metrics file")
    metrics_parser.add_argument("metrics_file", help="Metrics JSON file to validate")
    
    # Validate Prometheus command
    prometheus_parser = subparsers.add_parser("validate-prometheus", help="Validate Prometheus endpoint")
    prometheus_parser.add_argument("--url", default="http://localhost:9090", help="Prometheus URL")
    
    # Validate chaos command
    chaos_parser = subparsers.add_parser("validate-chaos", help="Validate chaos results")
    chaos_parser.add_argument("chaos_file", help="Chaos results JSON file to validate")
    
    # Validate traces command
    traces_parser = subparsers.add_parser("validate-traces", help="Validate trace file")
    traces_parser.add_argument("trace_file", help="Trace JSON file to validate")
    
    # Error budget command
    error_budget_parser = subparsers.add_parser("validate-error-budget", help="Validate error budget")
    error_budget_parser.add_argument("metrics_file", help="Current metrics file")
    error_budget_parser.add_argument("baseline_file", help="Baseline metrics file")
    error_budget_parser.add_argument("--threshold", type=float, default=0.01, help="Error threshold")
    
    # Latency regression command
    latency_parser = subparsers.add_parser("validate-latency-regression", help="Validate latency regression")
    latency_parser.add_argument("metrics_file", help="Current metrics file")
    latency_parser.add_argument("baseline_file", help="Baseline metrics file")
    latency_parser.add_argument("--threshold", type=float, default=0.2, help="Regression threshold")
    
    # Run chaos test command
    chaos_test_parser = subparsers.add_parser("run-chaos-test", help="Run chaos test")
    chaos_test_parser.add_argument("--loss", type=float, default=0.02, help="Packet loss rate")
    chaos_test_parser.add_argument("--latency", type=float, default=100.0, help="Latency in ms")
    chaos_test_parser.add_argument("--duration", type=int, default=120000, help="Duration in ms")
    chaos_test_parser.add_argument("--output", default="chaos_results.json", help="Output file")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    validator = ObservabilityValidator()
    validation_results = []
    
    try:
        if args.command == "validate-metrics":
            result = validator.validate_metrics_file(args.metrics_file)
            result["type"] = "metrics"
            validation_results.append(result)
            
        elif args.command == "validate-prometheus":
            result = validator.validate_prometheus_endpoint()
            result["type"] = "prometheus"
            validation_results.append(result)
            
        elif args.command == "validate-chaos":
            result = validator.validate_chaos_results(args.chaos_file)
            result["type"] = "chaos"
            validation_results.append(result)
            
        elif args.command == "validate-traces":
            result = validator.validate_trace_file(args.trace_file)
            result["type"] = "traces"
            validation_results.append(result)
            
        elif args.command == "validate-error-budget":
            result = validator.validate_error_budget(args.metrics_file, args.baseline_file, args.threshold)
            result["type"] = "error_budget"
            validation_results.append(result)
            
        elif args.command == "validate-latency-regression":
            result = validator.validate_latency_regression(args.metrics_file, args.baseline_file, args.threshold)
            result["type"] = "latency_regression"
            validation_results.append(result)
            
        elif args.command == "run-chaos-test":
            config = {
                "packet_loss_rate": args.loss,
                "latency_ms": args.latency,
                "duration_ms": args.duration,
                "output_file": args.output
            }
            result = validator.run_chaos_test(config)
            result["type"] = "chaos_test"
            validation_results.append(result)
        
        # Generate and print report
        report = validator.generate_report(validation_results)
        print(report)
        
        # Exit with error code if any validation failed
        if any(not result.get("valid", False) for result in validation_results):
            sys.exit(1)
    
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
