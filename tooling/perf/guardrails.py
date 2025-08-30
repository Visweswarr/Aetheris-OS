#!/usr/bin/env python3

"""
Statistical Guardrails for Polymera OS Performance Metrics

This script implements rolling statistical guardrails using mean + 3σ thresholds
to provide robust performance regression detection while reducing false alarms.

Usage: python tooling/perf/guardrails.py [current_perf_file] [history_dir] [config_id]
"""

import json
import os
import sys
import math
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass
from datetime import datetime, timezone
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class GuardrailResult:
    """Result of a guardrail check for a specific metric."""
    metric_name: str
    current_value: float
    mean: float
    std_dev: float
    threshold: float
    delta: float
    delta_percent: float
    passed: bool
    reason: str
    severity: str  # 'pass', 'warning', 'fail'

@dataclass
class ConfigGuardrailResult:
    """Result of guardrail checks for a configuration."""
    config_id: str
    config_name: str
    timestamp: str
    total_metrics: int
    passed_metrics: int
    warning_metrics: int
    failed_metrics: int
    overall_passed: bool
    results: List[GuardrailResult]
    summary: str

class PerformanceGuardrails:
    """Statistical guardrails for performance metrics."""
    
    def __init__(self, history_dir: str, config_id: str, window_size: int = 20):
        self.history_dir = Path(history_dir)
        self.config_id = config_id
        self.window_size = window_size
        self.history_file = self.history_dir / f"{config_id}.json"
        
        # Guardrail policies
        self.policies = {
            'ipc_latency_p50': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 500},
            'ipc_latency_p95': {'warning_threshold': 2.5, 'fail_threshold': 3.0, 'absolute_budget': 1000},
            'ipc_latency_p99': {'warning_threshold': 2.5, 'fail_threshold': 3.0, 'absolute_budget': 2000},
            'wake_to_run_p50': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 300},
            'wake_to_run_p95': {'warning_threshold': 2.5, 'fail_threshold': 3.0, 'absolute_budget': 600},
            'wake_to_run_p99': {'warning_threshold': 2.5, 'fail_threshold': 3.0, 'absolute_budget': 1200},
            'kyber_encapsulation': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 2000},
            'dilithium_sign': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 5000},
            'dilithium_verify': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 3000},
            'mac_generation': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 1000},
            'mac_verification': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 1000},
            'kernel_heap_mb': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 50},
            'user_heap_mb': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 100},
            'stack_usage_mb': {'warning_threshold': 2.0, 'fail_threshold': 3.0, 'absolute_budget': 20}
        }
    
    def load_history(self) -> Dict[str, Any]:
        """Load performance history for the configuration."""
        if not self.history_file.exists():
            logger.info(f"No history file found for {self.config_id}, creating new one")
            return self._create_empty_history()
        
        try:
            with open(self.history_file, 'r') as f:
                history = json.load(f)
            logger.info(f"Loaded history for {self.config_id}: {len(history.get('entries', []))} entries")
            return history
        except (json.JSONDecodeError, IOError) as e:
            logger.warning(f"Failed to load history for {self.config_id}: {e}")
            return self._create_empty_history()
    
    def _create_empty_history(self) -> Dict[str, Any]:
        """Create an empty history structure."""
        return {
            "config_id": self.config_id,
            "created": datetime.now(timezone.utc).isoformat(),
            "last_updated": datetime.now(timezone.utc).isoformat(),
            "window_size": self.window_size,
            "total_entries": 0,
            "entries": []
        }
    
    def save_history(self, history: Dict[str, Any]) -> None:
        """Save performance history to file."""
        try:
            # Ensure directory exists
            self.history_dir.mkdir(parents=True, exist_ok=True)
            
            # Update metadata
            history["last_updated"] = datetime.now(timezone.utc).isoformat()
            history["total_entries"] = len(history["entries"])
            
            # Write to file
            with open(self.history_file, 'w') as f:
                json.dump(history, f, indent=2)
            
            logger.info(f"Saved history for {self.config_id}: {len(history['entries'])} entries")
        except IOError as e:
            logger.error(f"Failed to save history for {self.config_id}: {e}")
            raise
    
    def add_performance_entry(self, history: Dict[str, Any], perf_data: Dict[str, Any]) -> None:
        """Add a new performance entry to history."""
        entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "run_id": os.environ.get("GITHUB_RUN_ID", "unknown"),
            "commit_sha": os.environ.get("GITHUB_SHA", "unknown"),
            "metrics": self._extract_metrics(perf_data)
        }
        
        # Add to entries
        history["entries"].append(entry)
        
        # Maintain window size
        if len(history["entries"]) > self.window_size:
            history["entries"] = history["entries"][-self.window_size:]
        
        logger.info(f"Added performance entry for {self.config_id}, total entries: {len(history['entries'])}")
    
    def _extract_metrics(self, perf_data: Dict[str, Any]) -> Dict[str, float]:
        """Extract performance metrics from performance data."""
        metrics = {}
        
        # IPC Latency metrics
        if 'ipc_latency' in perf_data:
            ipc = perf_data['ipc_latency']
            metrics.update({
                'ipc_latency_p50': ipc.get('p50_ns', 0),
                'ipc_latency_p95': ipc.get('p95_ns', 0),
                'ipc_latency_p99': ipc.get('p99_ns', 0),
                'ipc_latency_mean': ipc.get('mean_ns', 0)
            })
        
        # Wake-to-Run metrics
        if 'wake_to_run' in perf_data:
            wtr = perf_data['wake_to_run']
            metrics.update({
                'wake_to_run_p50': wtr.get('p50_ns', 0),
                'wake_to_run_p95': wtr.get('p95_ns', 0),
                'wake_to_run_p99': wtr.get('p99_ns', 0),
                'wake_to_run_mean': wtr.get('mean_ns', 0)
            })
        
        # PQC Overhead metrics
        if 'pqc_overhead' in perf_data:
            pqc = perf_data['pqc_overhead']
            metrics.update({
                'kyber_encapsulation': pqc.get('kyber_encapsulation_ns', 0),
                'dilithium_sign': pqc.get('dilithium_sign_ns', 0),
                'dilithium_verify': pqc.get('dilithium_verify_ns', 0),
                'mac_generation': pqc.get('mac_generation_ns', 0),
                'mac_verification': pqc.get('mac_verification_ns', 0)
            })
        
        # Memory Usage metrics
        if 'memory_usage' in perf_data:
            mem = perf_data['memory_usage']
            metrics.update({
                'kernel_heap_mb': mem.get('kernel_heap_mb', 0),
                'user_heap_mb': mem.get('user_heap_mb', 0),
                'stack_usage_mb': mem.get('stack_usage_mb', 0)
            })
        
        return metrics
    
    def calculate_statistics(self, history: Dict[str, Any], metric_name: str) -> Tuple[float, float]:
        """Calculate mean and standard deviation for a metric."""
        values = []
        for entry in history.get("entries", []):
            if metric_name in entry.get("metrics", {}):
                value = entry["metrics"][metric_name]
                if value > 0:  # Only include valid values
                    values.append(value)
        
        if len(values) < 2:
            # Need at least 2 values for meaningful statistics
            return 0.0, 0.0
        
        mean = sum(values) / len(values)
        
        if len(values) == 1:
            std_dev = 0.0
        else:
            variance = sum((x - mean) ** 2 for x in values) / (len(values) - 1)
            std_dev = math.sqrt(variance)
        
        return mean, std_dev
    
    def check_guardrail(self, metric_name: str, current_value: float, mean: float, std_dev: float) -> GuardrailResult:
        """Check if a metric passes the guardrail."""
        if metric_name not in self.policies:
            # Unknown metric, pass by default
            return GuardrailResult(
                metric_name=metric_name,
                current_value=current_value,
                mean=mean,
                std_dev=std_dev,
                threshold=0.0,
                delta=0.0,
                delta_percent=0.0,
                passed=True,
                reason="Unknown metric, passing by default",
                severity="pass"
            )
        
        policy = self.policies[metric_name]
        warning_threshold = policy['warning_threshold']
        fail_threshold = policy['fail_threshold']
        absolute_budget = policy['absolute_budget']
        
        # Calculate delta and percentage
        if mean > 0:
            delta = current_value - mean
            delta_percent = (delta / mean) * 100
        else:
            delta = 0.0
            delta_percent = 0.0
        
        # Check absolute budget first
        if current_value > absolute_budget:
            return GuardrailResult(
                metric_name=metric_name,
                current_value=current_value,
                mean=mean,
                std_dev=std_dev,
                threshold=absolute_budget,
                delta=delta,
                delta_percent=delta_percent,
                passed=False,
                reason=f"Exceeds absolute budget ({current_value} > {absolute_budget})",
                severity="fail"
            )
        
        # Check statistical thresholds
        if std_dev > 0:
            # Use mean + N*σ threshold
            warning_threshold_value = mean + (warning_threshold * std_dev)
            fail_threshold_value = mean + (fail_threshold * std_dev)
            
            if current_value > fail_threshold_value:
                return GuardrailResult(
                    metric_name=metric_name,
                    current_value=current_value,
                    mean=mean,
                    std_dev=std_dev,
                    threshold=fail_threshold_value,
                    delta=delta,
                    delta_percent=delta_percent,
                    passed=False,
                    reason=f"Exceeds fail threshold: {current_value} > {fail_threshold_value} (mean + {fail_threshold}σ)",
                    severity="fail"
                )
            elif current_value > warning_threshold_value:
                return GuardrailResult(
                    metric_name=metric_name,
                    current_value=current_value,
                    mean=mean,
                    std_dev=std_dev,
                    threshold=warning_threshold_value,
                    delta=delta,
                    delta_percent=delta_percent,
                    passed=True,
                    reason=f"Exceeds warning threshold: {current_value} > {warning_threshold_value} (mean + {warning_threshold}σ)",
                    severity="warning"
                )
            else:
                return GuardrailResult(
                    metric_name=metric_name,
                    current_value=current_value,
                    mean=mean,
                    std_dev=std_dev,
                    threshold=warning_threshold_value,
                    delta=delta,
                    delta_percent=delta_percent,
                    passed=True,
                    reason=f"Within acceptable range: {current_value} <= {warning_threshold_value}",
                    severity="pass"
                )
        else:
            # No variance, use percentage-based thresholds
            if delta_percent > (fail_threshold * 100):
                return GuardrailResult(
                    metric_name=metric_name,
                    current_value=current_value,
                    mean=mean,
                    std_dev=std_dev,
                    threshold=mean * (1 + fail_threshold),
                    delta=delta,
                    delta_percent=delta_percent,
                    passed=False,
                    reason=f"Exceeds fail threshold: {delta_percent:.1f}% > {fail_threshold * 100}%",
                    severity="fail"
                )
            elif delta_percent > (warning_threshold * 100):
                return GuardrailResult(
                    metric_name=metric_name,
                    current_value=current_value,
                    mean=mean,
                    std_dev=std_dev,
                    threshold=mean * (1 + warning_threshold),
                    delta=delta,
                    delta_percent=delta_percent,
                    passed=True,
                    reason=f"Exceeds warning threshold: {delta_percent:.1f}% > {warning_threshold * 100}%",
                    severity="warning"
                )
            else:
                return GuardrailResult(
                    metric_name=metric_name,
                    current_value=current_value,
                    mean=mean,
                    std_dev=std_dev,
                    threshold=mean * (1 + warning_threshold),
                    delta=delta,
                    delta_percent=delta_percent,
                    passed=True,
                    reason=f"Within acceptable range: {delta_percent:.1f}% <= {warning_threshold * 100}%",
                    severity="pass"
                )
    
    def evaluate_performance(self, perf_data: Dict[str, Any], config_name: str = "") -> ConfigGuardrailResult:
        """Evaluate performance against guardrails."""
        history = self.load_history()
        metrics = self._extract_metrics(perf_data)
        
        results = []
        passed_count = 0
        warning_count = 0
        failed_count = 0
        
        for metric_name, current_value in metrics.items():
            if current_value <= 0:
                continue  # Skip invalid metrics
            
            mean, std_dev = self.calculate_statistics(history, metric_name)
            result = self.check_guardrail(metric_name, current_value, mean, std_dev)
            results.append(result)
            
            if result.severity == "pass":
                passed_count += 1
            elif result.severity == "warning":
                warning_count += 1
            elif result.severity == "fail":
                failed_count += 1
        
        overall_passed = failed_count == 0
        
        # Generate summary
        if failed_count > 0:
            summary = f"FAILED: {failed_count} metrics failed guardrails"
        elif warning_count > 0:
            summary = f"PASSED with warnings: {warning_count} metrics exceeded warning thresholds"
        else:
            summary = f"PASSED: All {passed_count} metrics within acceptable ranges"
        
        return ConfigGuardrailResult(
            config_id=self.config_id,
            config_name=config_name,
            timestamp=datetime.now(timezone.utc).isoformat(),
            total_metrics=len(results),
            passed_metrics=passed_count,
            warning_metrics=warning_count,
            failed_metrics=failed_count,
            overall_passed=overall_passed,
            results=results,
            summary=summary
        )
    
    def update_history_on_success(self, perf_data: Dict[str, Any]) -> None:
        """Update history with new performance data on successful runs."""
        history = self.load_history()
        self.add_performance_entry(history, perf_data)
        self.save_history(history)
        logger.info(f"Updated history for {self.config_id} on successful run")

def main():
    """Main entry point for the guardrails script."""
    parser = argparse.ArgumentParser(description="Performance Guardrails Checker")
    parser.add_argument("perf_file", help="Path to current performance JSON file")
    parser.add_argument("history_dir", help="Path to history directory")
    parser.add_argument("config_id", help="Configuration ID")
    parser.add_argument("--config-name", default="", help="Configuration name")
    parser.add_argument("--update-history", action="store_true", help="Update history on success")
    parser.add_argument("--output-file", help="Output file for results")
    parser.add_argument("--window-size", type=int, default=20, help="History window size")
    
    args = parser.parse_args()
    
    # Check if performance file exists
    if not os.path.exists(args.perf_file):
        logger.error(f"Performance file not found: {args.perf_file}")
        sys.exit(1)
    
    # Load performance data
    try:
        with open(args.perf_file, 'r') as f:
            perf_data = json.load(f)
    except (json.JSONDecodeError, IOError) as e:
        logger.error(f"Failed to load performance data: {e}")
        sys.exit(1)
    
    # Initialize guardrails
    guardrails = PerformanceGuardrails(args.history_dir, args.config_id, args.window_size)
    
    # Evaluate performance
    result = guardrails.evaluate_performance(perf_data, args.config_name)
    
    # Print results
    print(f"\n🔍 Performance Guardrails Check for {result.config_id}")
    print(f"📊 {result.summary}")
    print(f"📈 Metrics: {result.passed_metrics} passed, {result.warning_metrics} warnings, {result.failed_metrics} failed")
    print(f"⏰ Timestamp: {result.timestamp}")
    
    if result.results:
        print(f"\n📋 Detailed Results:")
        for res in result.results:
            status_emoji = "✅" if res.severity == "pass" else "⚠️" if res.severity == "warning" else "❌"
            print(f"  {status_emoji} {res.metric_name}: {res.current_value:.2f}")
            print(f"     Mean: {res.mean:.2f}, σ: {res.std_dev:.2f}")
            print(f"     Delta: {res.delta:+.2f} ({res.delta_percent:+.1f}%)")
            print(f"     Threshold: {res.threshold:.2f}")
            print(f"     Status: {res.reason}")
            print()
    
    # Update history if requested and passed
    if args.update_history and result.overall_passed:
        guardrails.update_history_on_success(perf_data)
        print("✅ History updated successfully")
    elif args.update_history and not result.overall_passed:
        print("❌ History not updated due to guardrail failures")
    
    # Save results to file if requested
    if args.output_file:
        output_data = {
            "config_id": result.config_id,
            "config_name": result.config_name,
            "timestamp": result.timestamp,
            "overall_passed": result.overall_passed,
            "summary": result.summary,
            "metrics": {
                "total": result.total_metrics,
                "passed": result.passed_metrics,
                "warnings": result.warning_metrics,
                "failed": result.failed_metrics
            },
            "results": [
                {
                    "metric_name": res.metric_name,
                    "current_value": res.current_value,
                    "mean": res.mean,
                    "std_dev": res.std_dev,
                    "threshold": res.threshold,
                    "delta": res.delta,
                    "delta_percent": res.delta_percent,
                    "passed": res.passed,
                    "severity": res.severity,
                    "reason": res.reason
                }
                for res in result.results
            ]
        }
        
        try:
            with open(args.output_file, 'w') as f:
                json.dump(output_data, f, indent=2)
            print(f"📄 Results saved to {args.output_file}")
        except IOError as e:
            logger.error(f"Failed to save results: {e}")
    
    # Exit with appropriate code
    if result.overall_passed:
        print("🎉 All guardrails passed!")
        sys.exit(0)
    else:
        print("💥 Some guardrails failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
