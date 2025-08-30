#!/usr/bin/env python3
"""
Matrix Testing Metrics Extractor

This script extracts IPC performance metrics from QEMU output logs
for different CPU/SMP/memory configurations.
"""

import sys
import json
import re
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

def extract_ipc_metrics(log_content: str) -> Dict[str, float]:
    """Extract IPC metrics from log content"""
    metrics = {}
    
    # Look for IPC-related metrics in the log
    # These patterns should match the actual log output format
    
    # IPC throughput patterns
    ipc_patterns = [
        r'IPC_THROUGHPUT:\s*(\d+(?:\.\d+)?)\s*msg/s',
        r'IPC_RATE:\s*(\d+(?:\.\d+)?)\s*msg/s',
        r'Messages/sec:\s*(\d+(?:\.\d+)?)',
    ]
    
    for pattern in ipc_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['ipc_throughput'] = float(matches[-1])  # Use last match
            break
    
    # Latency patterns (p50, p95, p99)
    latency_patterns = [
        (r'P50_LATENCY:\s*(\d+(?:\.\d+)?)\s*ms', 'p50_latency_ms'),
        (r'P95_LATENCY:\s*(\d+(?:\.\d+)?)\s*ms', 'p95_latency_ms'),
        (r'P99_LATENCY:\s*(\d+(?:\.\d+)?)\s*ms', 'p99_latency_ms'),
        (r'Latency P50:\s*(\d+(?:\.\d+)?)\s*ms', 'p50_latency_ms'),
        (r'Latency P95:\s*(\d+(?:\.\d+)?)\s*ms', 'p95_latency_ms'),
        (r'Latency P99:\s*(\d+(?:\.\d+)?)\s*ms', 'p99_latency_ms'),
    ]
    
    for pattern, metric_name in latency_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics[metric_name] = float(matches[-1])
    
    # Context switch patterns
    context_switch_patterns = [
        r'CONTEXT_SWITCHES:\s*(\d+)',
        r'Context switches:\s*(\d+)',
        r'Switches:\s*(\d+)',
    ]
    
    for pattern in context_switch_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['context_switches'] = int(matches[-1])
            break
    
    # Memory usage patterns
    memory_patterns = [
        r'MEMORY_USAGE:\s*(\d+(?:\.\d+)?)\s*MB',
        r'Memory used:\s*(\d+(?:\.\d+)?)\s*MB',
        r'RAM usage:\s*(\d+(?:\.\d+)?)\s*MB',
    ]
    
    for pattern in memory_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['memory_usage_mb'] = float(matches[-1])
            break
    
    # CPU utilization patterns
    cpu_patterns = [
        r'CPU_UTILIZATION:\s*(\d+(?:\.\d+)?)\s*%',
        r'CPU usage:\s*(\d+(?:\.\d+)?)\s*%',
        r'CPU load:\s*(\d+(?:\.\d+)?)\s*%',
    ]
    
    for pattern in cpu_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['cpu_utilization_percent'] = float(matches[-1])
            break
    
    # Test duration patterns
    duration_patterns = [
        r'TEST_DURATION:\s*(\d+(?:\.\d+)?)\s*ms',
        r'Duration:\s*(\d+(?:\.\d+)?)\s*ms',
        r'Runtime:\s*(\d+(?:\.\d+)?)\s*ms',
    ]
    
    for pattern in duration_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['test_duration_ms'] = float(matches[-1])
            break
    
    # Task count patterns
    task_patterns = [
        r'TASK_COUNT:\s*(\d+)',
        r'Total tasks:\s*(\d+)',
        r'Tasks:\s*(\d+)',
    ]
    
    for pattern in task_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['task_count'] = int(matches[-1])
            break
    
    return metrics

def extract_system_metrics(log_content: str) -> Dict[str, any]:
    """Extract system-level metrics from log content"""
    metrics = {}
    
    # Look for system boot and initialization metrics
    boot_patterns = [
        r'BOOT_TIME:\s*(\d+(?:\.\d+)?)\s*ms',
        r'Boot completed in:\s*(\d+(?:\.\d+)?)\s*ms',
        r'Initialization:\s*(\d+(?:\.\d+)?)\s*ms',
    ]
    
    for pattern in boot_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['boot_time_ms'] = float(matches[-1])
            break
    
    # Look for memory allocation metrics
    alloc_patterns = [
        r'ALLOCATIONS:\s*(\d+)',
        r'Memory allocations:\s*(\d+)',
        r'Allocs:\s*(\d+)',
    ]
    
    for pattern in alloc_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['allocations'] = int(matches[-1])
            break
    
    # Look for error/warning counts
    error_patterns = [
        r'ERRORS:\s*(\d+)',
        r'Error count:\s*(\d+)',
        r'Errors:\s*(\d+)',
    ]
    
    for pattern in error_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['errors'] = int(matches[-1])
            break
    
    warning_patterns = [
        r'WARNINGS:\s*(\d+)',
        r'Warning count:\s*(\d+)',
        r'Warnings:\s*(\d+)',
    ]
    
    for pattern in warning_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            metrics['warnings'] = int(matches[-1])
            break
    
    return metrics

def extract_test_results(log_content: str) -> Dict[str, any]:
    """Extract test result information from log content"""
    results = {}
    
    # Check for test completion
    completion_patterns = [
        r'TEST_COMPLETED:\s*(true|false)',
        r'Test result:\s*(PASS|FAIL|SUCCESS|ERROR)',
        r'All tests:\s*(PASSED|FAILED)',
    ]
    
    for pattern in completion_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            result = matches[-1].lower()
            if result in ['true', 'pass', 'success', 'passed']:
                results['test_passed'] = True
            else:
                results['test_passed'] = False
            break
    
    # Check for specific test failures
    failure_patterns = [
        r'TEST_FAILED:\s*(\w+)',
        r'Failed test:\s*(\w+)',
        r'Error in:\s*(\w+)',
    ]
    
    for pattern in failure_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            results['failed_tests'] = results.get('failed_tests', []) + [matches[-1]]
    
    # Check for test execution time
    execution_patterns = [
        r'EXECUTION_TIME:\s*(\d+(?:\.\d+)?)\s*ms',
        r'Test execution:\s*(\d+(?:\.\d+)?)\s*ms',
        r'Runtime:\s*(\d+(?:\.\d+)?)\s*ms',
    ]
    
    for pattern in execution_patterns:
        matches = re.findall(pattern, log_content, re.IGNORECASE)
        if matches:
            results['execution_time_ms'] = float(matches[-1])
            break
    
    return results

def create_metrics_summary(
    cpu_model: str,
    smp_cores: int,
    memory_mb: int,
    ipc_metrics: Dict[str, float],
    system_metrics: Dict[str, any],
    test_results: Dict[str, any]
) -> Dict[str, any]:
    """Create comprehensive metrics summary"""
    
    # Calculate derived metrics
    derived_metrics = {}
    
    if 'ipc_throughput' in ipc_metrics and 'test_duration_ms' in ipc_metrics:
        duration_sec = ipc_metrics['test_duration_ms'] / 1000.0
        derived_metrics['total_messages'] = ipc_metrics['ipc_throughput'] * duration_sec
        derived_metrics['messages_per_core'] = derived_metrics['total_messages'] / smp_cores
    
    if 'memory_usage_mb' in ipc_metrics:
        derived_metrics['memory_efficiency'] = ipc_metrics['memory_usage_mb'] / memory_mb * 100.0
    
    if 'p50_latency_ms' in ipc_metrics and 'p95_latency_ms' in ipc_metrics:
        derived_metrics['latency_tail'] = ipc_metrics['p95_latency_ms'] / ipc_metrics['p50_latency_ms']
    
    # Create summary
    summary = {
        "matrix_configuration": {
            "cpu_model": cpu_model,
            "smp_cores": smp_cores,
            "memory_mb": memory_mb,
            "matrix_id": f"{cpu_model}-{smp_cores}-{memory_mb}MB"
        },
        "extraction_timestamp": time.time(),
        "extraction_time_iso": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "ipc_metrics": ipc_metrics,
        "system_metrics": system_metrics,
        "test_results": test_results,
        "derived_metrics": derived_metrics,
        "metrics_summary": {
            "has_ipc_data": len(ipc_metrics) > 0,
            "has_system_data": len(system_metrics) > 0,
            "has_test_results": len(test_results) > 0,
            "total_metrics": len(ipc_metrics) + len(system_metrics) + len(test_results)
        }
    }
    
    return summary

def main():
    """Main entry point"""
    if len(sys.argv) != 5:
        print("Usage: python3 extract_matrix_metrics.py <log_file> <cpu_model> <smp_cores> <memory_mb>")
        print("Example: python3 extract_matrix_metrics.py qemu_output.log qemu64 2 512")
        sys.exit(1)
    
    log_file = sys.argv[1]
    cpu_model = sys.argv[2]
    smp_cores = int(sys.argv[3])
    memory_mb = int(sys.argv[4])
    
    # Check if log file exists
    if not Path(log_file).exists():
        print(f"Error: Log file '{log_file}' not found")
        sys.exit(1)
    
    try:
        # Read log file
        with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
            log_content = f.read()
        
        print(f"Processing log file: {log_file}")
        print(f"Configuration: CPU={cpu_model}, SMP={smp_cores}, Memory={memory_mb}MB")
        
        # Extract metrics
        ipc_metrics = extract_ipc_metrics(log_content)
        system_metrics = extract_system_metrics(log_content)
        test_results = extract_test_results(log_content)
        
        print(f"Extracted {len(ipc_metrics)} IPC metrics")
        print(f"Extracted {len(system_metrics)} system metrics")
        print(f"Extracted {len(test_results)} test results")
        
        # Create summary
        summary = create_metrics_summary(
            cpu_model, smp_cores, memory_mb,
            ipc_metrics, system_metrics, test_results
        )
        
        # Output JSON
        json.dump(summary, sys.stdout, indent=2)
        
    except Exception as e:
        print(f"Error processing log file: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()

