#!/usr/bin/env python3
"""
Matrix Performance Analysis

This script analyzes performance metrics across different CPU/SMP/memory
configurations and detects performance regressions.
"""

import sys
import json
import glob
import os
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from collections import defaultdict

def load_matrix_metrics(metrics_dir: str) -> List[Dict]:
    """Load all matrix metrics from the specified directory"""
    metrics_files = glob.glob(os.path.join(metrics_dir, "matrix_metrics_*.json"))
    
    if not metrics_files:
        print(f"Warning: No metrics files found in {metrics_dir}")
        return []
    
    all_metrics = []
    
    for metrics_file in metrics_files:
        try:
            with open(metrics_file, 'r') as f:
                metrics = json.load(f)
                all_metrics.append(metrics)
        except Exception as e:
            print(f"Warning: Could not load {metrics_file}: {e}")
            continue
    
    print(f"Loaded {len(all_metrics)} metrics files")
    return all_metrics

def calculate_baseline_metrics(metrics: List[Dict]) -> Dict[str, Dict]:
    """Calculate baseline metrics for each configuration"""
    baselines = defaultdict(list)
    
    # Group metrics by configuration
    for metric in metrics:
        config = metric['matrix_configuration']['matrix_id']
        baselines[config].append(metric)
    
    # Calculate baseline (median) for each metric type
    baseline_results = {}
    
    for config, config_metrics in baselines.items():
        if len(config_metrics) == 0:
            continue
            
        baseline = {
            'configuration': config,
            'cpu_model': config_metrics[0]['matrix_configuration']['cpu_model'],
            'smp_cores': config_metrics[0]['matrix_configuration']['smp_cores'],
            'memory_mb': config_metrics[0]['matrix_configuration']['memory_mb'],
            'baseline_metrics': {},
            'metric_count': len(config_metrics)
        }
        
        # Calculate baseline for IPC metrics
        ipc_baselines = {}
        for metric_name in ['ipc_throughput', 'p50_latency_ms', 'p95_latency_ms', 'p99_latency_ms']:
            values = []
            for m in config_metrics:
                if metric_name in m['ipc_metrics']:
                    values.append(m['ipc_metrics'][metric_name])
            
            if values:
                values.sort()
                median_idx = len(values) // 2
                ipc_baselines[metric_name] = {
                    'baseline': values[median_idx],
                    'min': values[0],
                    'max': values[-1],
                    'count': len(values)
                }
        
        baseline['baseline_metrics']['ipc'] = ipc_baselines
        
        # Calculate baseline for system metrics
        system_baselines = {}
        for metric_name in ['boot_time_ms', 'memory_usage_mb', 'cpu_utilization_percent']:
            values = []
            for m in config_metrics:
                if metric_name in m['system_metrics']:
                    values.append(m['system_metrics'][metric_name])
            
            if values:
                values.sort()
                median_idx = len(values) // 2
                system_baselines[metric_name] = {
                    'baseline': values[median_idx],
                    'min': values[0],
                    'max': values[-1],
                    'count': len(values)
                }
        
        baseline['baseline_metrics']['system'] = system_baselines
        
        baseline_results[config] = baseline
    
    return baseline_results

def detect_performance_regressions(
    current_metrics: List[Dict],
    baseline_metrics: Dict[str, Dict],
    regression_threshold: float = 20.0
) -> List[Dict]:
    """Detect performance regressions compared to baselines"""
    regressions = []
    
    for metric in current_metrics:
        config = metric['matrix_configuration']['matrix_id']
        
        if config not in baseline_metrics:
            continue
            
        baseline = baseline_metrics[config]
        
        # Check IPC metrics for regressions
        for metric_name, current_value in metric['ipc_metrics'].items():
            if metric_name in baseline['baseline_metrics']['ipc']:
                baseline_value = baseline['baseline_metrics']['ipc'][metric_name]['baseline']
                
                # Calculate regression percentage
                if baseline_value > 0:
                    if metric_name in ['p50_latency_ms', 'p95_latency_ms', 'p99_latency_ms']:
                        # For latency, higher is worse
                        regression_pct = ((current_value - baseline_value) / baseline_value) * 100
                        if regression_pct > regression_threshold:
                            regressions.append({
                                'configuration': config,
                                'metric': metric_name,
                                'baseline_value': baseline_value,
                                'current_value': current_value,
                                'degradation': regression_pct,
                                'metric_type': 'latency'
                            })
                    else:
                        # For throughput, lower is worse
                        regression_pct = ((baseline_value - current_value) / baseline_value) * 100
                        if regression_pct > regression_threshold:
                            regressions.append({
                                'configuration': config,
                                'metric': metric_name,
                                'baseline_value': baseline_value,
                                'current_value': current_value,
                                'degradation': regression_pct,
                                'metric_type': 'throughput'
                            })
        
        # Check system metrics for regressions
        for metric_name, current_value in metric['system_metrics'].items():
            if metric_name in baseline['baseline_metrics']['system']:
                baseline_value = baseline['baseline_metrics']['system'][metric_name]['baseline']
                
                if baseline_value > 0:
                    if metric_name in ['boot_time_ms']:
                        # For boot time, higher is worse
                        regression_pct = ((current_value - baseline_value) / baseline_value) * 100
                        if regression_pct > regression_threshold:
                            regressions.append({
                                'configuration': config,
                                'metric': metric_name,
                                'baseline_value': baseline_value,
                                'current_value': current_value,
                                'degradation': regression_pct,
                                'metric_type': 'system'
                            })
    
    return regressions

def generate_performance_summary(
    metrics: List[Dict],
    baselines: Dict[str, Dict],
    regressions: List[Dict]
) -> Dict:
    """Generate comprehensive performance summary"""
    
    # Count configurations
    configs = set()
    cpu_models = set()
    smp_configs = set()
    memory_configs = set()
    
    for metric in metrics:
        config = metric['matrix_configuration']
        configs.add(config['matrix_id'])
        cpu_models.add(config['cpu_model'])
        smp_configs.add(config['smp_cores'])
        memory_configs.add(config['memory_mb'])
    
    # Calculate success rates
    total_tests = len(metrics)
    passed_tests = sum(1 for m in metrics if m['test_results'].get('test_passed', False))
    failed_tests = total_tests - passed_tests
    
    # Calculate metric coverage
    ipc_coverage = sum(1 for m in metrics if len(m['ipc_metrics']) > 0)
    system_coverage = sum(1 for m in metrics if len(m['system_metrics']) > 0)
    
    summary = {
        'total_tests': total_tests,
        'passed_tests': passed_tests,
        'failed_tests': failed_tests,
        'success_rate': (passed_tests / total_tests * 100) if total_tests > 0 else 0,
        'configurations': {
            'total': len(configs),
            'cpu_models': list(cpu_models),
            'smp_cores': list(smp_configs),
            'memory_sizes': list(memory_configs)
        },
        'metrics_coverage': {
            'ipc_metrics': ipc_coverage,
            'system_metrics': system_coverage,
            'total_with_metrics': ipc_coverage + system_coverage
        },
        'regressions': {
            'total': len(regressions),
            'by_type': {
                'latency': len([r for r in regressions if r['metric_type'] == 'latency']),
                'throughput': len([r for r in regressions if r['metric_type'] == 'throughput']),
                'system': len([r for r in regressions if r['metric_type'] == 'system'])
            }
        },
        'baseline_metrics': {
            'total_configurations': len(baselines),
            'configurations_with_baselines': list(baselines.keys())
        }
    }
    
    return summary

def create_analysis_report(
    metrics: List[Dict],
    baselines: Dict[str, Dict],
    regressions: List[Dict],
    summary: Dict
) -> Dict:
    """Create comprehensive analysis report"""
    
    report = {
        'analysis_timestamp': summary.get('analysis_timestamp', ''),
        'summary': summary,
        'baselines': baselines,
        'regressions': regressions,
        'current_metrics': metrics,
        'analysis_metadata': {
            'regression_threshold': 20.0,
            'analysis_version': '1.0',
            'total_metrics_analyzed': len(metrics)
        }
    }
    
    return report

def main():
    """Main entry point"""
    if len(sys.argv) != 2:
        print("Usage: python3 analyze_matrix_performance.py <metrics_directory>")
        print("Example: python3 analyze_matrix_performance.py matrix_metrics/")
        sys.exit(1)
    
    metrics_dir = sys.argv[1]
    
    if not Path(metrics_dir).exists():
        print(f"Error: Metrics directory '{metrics_dir}' not found")
        sys.exit(1)
    
    try:
        print(f"Analyzing performance metrics from: {metrics_dir}")
        
        # Load all metrics
        metrics = load_matrix_metrics(metrics_dir)
        
        if not metrics:
            print("No metrics found to analyze")
            sys.exit(1)
        
        # Calculate baseline metrics
        print("Calculating baseline metrics...")
        baselines = calculate_baseline_metrics(metrics)
        
        # Detect performance regressions
        print("Detecting performance regressions...")
        regressions = detect_performance_regressions(metrics, baselines, 20.0)
        
        # Generate summary
        print("Generating performance summary...")
        summary = generate_performance_summary(metrics, baselines, regressions)
        
        # Create analysis report
        report = create_analysis_report(metrics, baselines, regressions, summary)
        
        # Output results
        print(f"\nAnalysis completed:")
        print(f"  Total tests: {summary['total_tests']}")
        print(f"  Passed: {summary['passed_tests']}")
        print(f"  Failed: {summary['failed_tests']}")
        print(f"  Success rate: {summary['success_rate']:.1f}%")
        print(f"  Configurations: {summary['configurations']['total']}")
        print(f"  Regressions detected: {summary['regressions']['total']}")
        
        if regressions:
            print(f"\nRegressions found:")
            for reg in regressions[:5]:  # Show first 5
                print(f"  {reg['configuration']}: {reg['metric']} regression {reg['degradation']:.1f}%")
        
        # Output JSON report
        json.dump(report, sys.stdout, indent=2)
        
    except Exception as e:
        print(f"Error during analysis: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
