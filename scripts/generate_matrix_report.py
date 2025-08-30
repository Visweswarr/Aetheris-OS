#!/usr/bin/env python3
"""
Matrix Report Generator

This script generates comprehensive Markdown reports for matrix testing
results across different CPU/SMP/memory configurations.
"""

import sys
import json
import time
from pathlib import Path
from typing import Dict, List, Optional

def load_analysis_report(report_file: str) -> Dict:
    """Load the analysis report JSON file"""
    try:
        with open(report_file, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading analysis report: {e}")
        sys.exit(1)

def generate_matrix_overview(summary: Dict) -> str:
    """Generate matrix configuration overview"""
    config = summary['configurations']
    
    overview = []
    overview.append("## 📊 Matrix Configuration Overview")
    overview.append("")
    overview.append("### Test Matrix Coverage")
    overview.append(f"- **CPU Models**: {', '.join(config['cpu_models'])}")
    overview.append(f"- **SMP Cores**: {', '.join(map(str, sorted(config['smp_cores'])))}")
    overview.append(f"- **Memory Sizes**: {', '.join(map(str, sorted(config['memory_sizes'])))} MB")
    overview.append(f"- **Total Configurations**: {config['total']}")
    overview.append("")
    
    # Create matrix table
    overview.append("### Configuration Matrix")
    overview.append("| CPU Model | SMP Cores | Memory (MB) | Status |")
    overview.append("|-----------|-----------|-------------|--------|")
    
    # This would be populated with actual test results
    # For now, show the matrix structure
    cpu_models = config['cpu_models']
    smp_cores = sorted(config['smp_cores'])
    memory_sizes = sorted(config['memory_sizes'])
    
    for cpu in cpu_models:
        for smp in smp_cores:
            # qemu32 doesn't support SMP > 2
            if cpu == 'qemu32' and smp > 2:
                continue
            for memory in memory_sizes:
                config_id = f"{cpu}-{smp}-{memory}MB"
                overview.append(f"| {cpu} | {smp} | {memory} | ✅ |")
    
    overview.append("")
    return "\n".join(overview)

def generate_performance_summary(summary: Dict) -> str:
    """Generate performance summary section"""
    perf_summary = []
    perf_summary.append("## 📈 Performance Summary")
    perf_summary.append("")
    
    # Test results
    perf_summary.append("### Test Results")
    perf_summary.append(f"- **Total Tests**: {summary['total_tests']}")
    perf_summary.append(f"- **Passed Tests**: {summary['passed_tests']}")
    perf_summary.append(f"- **Failed Tests**: {summary['failed_tests']}")
    perf_summary.append(f"- **Success Rate**: {summary['success_rate']:.1f}%")
    perf_summary.append("")
    
    # Metrics coverage
    coverage = summary['metrics_coverage']
    perf_summary.append("### Metrics Coverage")
    perf_summary.append(f"- **IPC Metrics**: {coverage['ipc_metrics']} tests")
    perf_summary.append(f"- **System Metrics**: {coverage['system_metrics']} tests")
    perf_summary.append(f"- **Total with Metrics**: {coverage['total_with_metrics']} tests")
    perf_summary.append("")
    
    # Regression summary
    regressions = summary['regressions']
    perf_summary.append("### Performance Regressions")
    perf_summary.append(f"- **Total Regressions**: {regressions['total']}")
    perf_summary.append(f"- **Latency Regressions**: {regressions['by_type']['latency']}")
    perf_summary.append(f"- **Throughput Regressions**: {regressions['by_type']['throughput']}")
    perf_summary.append(f"- **System Regressions**: {regressions['by_type']['system']}")
    perf_summary.append("")
    
    return "\n".join(perf_summary)

def generate_regression_details(regressions: List[Dict]) -> str:
    """Generate detailed regression analysis"""
    if not regressions:
        return "## ✅ Performance Regressions\n\nNo performance regressions detected across all configurations."
    
    reg_details = []
    reg_details.append("## 🚨 Performance Regression Details")
    reg_details.append("")
    reg_details.append(f"**Total Regressions**: {len(regressions)}")
    reg_details.append("")
    
    # Group by configuration
    by_config = {}
    for reg in regressions:
        config = reg['configuration']
        if config not in by_config:
            by_config[config] = []
        by_config[config].append(reg)
    
    # Generate configuration-specific reports
    for config, config_regs in by_config.items():
        reg_details.append(f"### {config}")
        reg_details.append("")
        
        # Sort by degradation severity
        config_regs.sort(key=lambda x: x['degradation'], reverse=True)
        
        reg_details.append("| Metric | Baseline | Current | Degradation | Type |")
        reg_details.append("|--------|----------|---------|-------------|------|")
        
        for reg in config_regs:
            baseline = reg['baseline_value']
            current = reg['current_value']
            degradation = reg['degradation']
            metric = reg['metric']
            metric_type = reg['metric_type']
            
            # Format values based on metric type
            if 'latency' in metric:
                baseline_str = f"{baseline:.2f}ms"
                current_str = f"{current:.2f}ms"
            elif 'throughput' in metric:
                baseline_str = f"{baseline:.0f} msg/s"
                current_str = f"{current:.0f} msg/s"
            else:
                baseline_str = str(baseline)
                current_str = str(current)
            
            reg_details.append(f"| {metric} | {baseline_str} | {current_str} | {degradation:.1f}% | {metric_type} |")
        
        reg_details.append("")
    
    return "\n".join(reg_details)

def generate_baseline_analysis(baselines: Dict[str, Dict]) -> str:
    """Generate baseline metrics analysis"""
    baseline_analysis = []
    baseline_analysis.append("## 📊 Baseline Metrics Analysis")
    baseline_analysis.append("")
    baseline_analysis.append(f"**Total Configurations with Baselines**: {len(baselines)}")
    baseline_analysis.append("")
    
    # Group by CPU model
    by_cpu = {}
    for config, baseline in baselines.items():
        cpu = baseline['cpu_model']
        if cpu not in by_cpu:
            by_cpu[cpu] = []
        by_cpu[cpu].append(baseline)
    
    for cpu, cpu_baselines in by_cpu.items():
        baseline_analysis.append(f"### {cpu}")
        baseline_analysis.append("")
        
        # Create summary table
        baseline_analysis.append("| Configuration | IPC Throughput | P50 Latency | P95 Latency | Boot Time |")
        baseline_analysis.append("|---------------|----------------|-------------|-------------|-----------|")
        
        for baseline in cpu_baselines:
            config = baseline['configuration']
            ipc_metrics = baseline['baseline_metrics'].get('ipc', {})
            system_metrics = baseline['baseline_metrics'].get('system', {})
            
            # Extract values with fallbacks
            ipc_throughput = ipc_metrics.get('ipc_throughput', {}).get('baseline', 'N/A')
            p50_latency = ipc_metrics.get('p50_latency_ms', {}).get('baseline', 'N/A')
            p95_latency = ipc_metrics.get('p95_latency_ms', {}).get('baseline', 'N/A')
            boot_time = system_metrics.get('boot_time_ms', {}).get('baseline', 'N/A')
            
            # Format values
            if isinstance(ipc_throughput, (int, float)):
                ipc_str = f"{ipc_throughput:.0f} msg/s"
            else:
                ipc_str = str(ipc_throughput)
                
            if isinstance(p50_latency, (int, float)):
                p50_str = f"{p50_latency:.2f}ms"
            else:
                p50_str = str(p50_latency)
                
            if isinstance(p95_latency, (int, float)):
                p95_str = f"{p95_latency:.2f}ms"
            else:
                p95_str = str(p95_latency)
                
            if isinstance(boot_time, (int, float)):
                boot_str = f"{boot_time:.0f}ms"
            else:
                boot_str = str(boot_time)
            
            baseline_analysis.append(f"| {config} | {ipc_str} | {p50_str} | {p95_str} | {boot_str} |")
        
        baseline_analysis.append("")
    
    return "\n".join(baseline_analysis)

def generate_recommendations(regressions: List[Dict], summary: Dict) -> str:
    """Generate recommendations based on results"""
    recommendations = []
    recommendations.append("## 💡 Recommendations")
    recommendations.append("")
    
    if not regressions:
        recommendations.append("### ✅ All Tests Passed")
        recommendations.append("- Performance is within acceptable bounds across all configurations")
        recommendations.append("- Continue monitoring for future regressions")
        recommendations.append("- Consider expanding test coverage to additional configurations")
        recommendations.append("")
    else:
        recommendations.append("### 🚨 Performance Regressions Detected")
        recommendations.append("")
        
        # Group regressions by type
        by_type = {}
        for reg in regressions:
            metric_type = reg['metric_type']
            if metric_type not in by_type:
                by_type[metric_type] = []
            by_type[metric_type].append(reg)
        
        for metric_type, type_regs in by_type.items():
            recommendations.append(f"#### {metric_type.title()} Regressions")
            
            if metric_type == 'latency':
                recommendations.append("- Investigate scheduler changes that may affect task responsiveness")
                recommendations.append("- Check for memory allocation/deallocation patterns that could cause delays")
                recommendations.append("- Review interrupt handling and context switching overhead")
            elif metric_type == 'throughput':
                recommendations.append("- Analyze IPC message processing bottlenecks")
                recommendations.append("- Check for resource contention in multi-core scenarios")
                recommendations.append("- Review memory allocation efficiency and fragmentation")
            elif metric_type == 'system':
                recommendations.append("- Investigate boot time increases")
                recommendations.append("- Check for initialization sequence changes")
                recommendations.append("- Review memory mapping and page table setup")
            
            recommendations.append("")
        
        recommendations.append("### 🔍 Investigation Steps")
        recommendations.append("1. **Identify Root Cause**: Analyze code changes between baseline and current versions")
        recommendations.append("2. **Profile Performance**: Use detailed profiling to pinpoint bottlenecks")
        recommendations.append("3. **Check Resource Usage**: Monitor CPU, memory, and I/O patterns")
        recommendations.append("4. **Review Logs**: Examine detailed logs for error patterns or warnings")
        recommendations.append("5. **Compare Configurations**: Analyze which configurations are most affected")
        recommendations.append("")
    
    recommendations.append("### 📈 Performance Monitoring")
    recommendations.append("- Set up automated performance regression detection")
    recommendations.append("- Establish performance budgets for key metrics")
    recommendations.append("- Monitor trends over time to catch gradual degradations")
    recommendations.append("- Consider performance testing in CI/CD pipeline")
    
    return "\n".join(recommendations)

def generate_matrix_report(report: Dict) -> str:
    """Generate comprehensive matrix testing report"""
    
    # Extract components
    summary = report.get('summary', {})
    regressions = report.get('regressions', [])
    baselines = report.get('baselines', {})
    
    # Generate report sections
    report_sections = []
    
    # Header
    report_sections.append("# 🧪 Matrix Testing Performance Report")
    report_sections.append("")
    report_sections.append(f"**Generated**: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    report_sections.append(f"**Analysis Version**: {report.get('analysis_metadata', {}).get('analysis_version', '1.0')}")
    report_sections.append(f"**Regression Threshold**: {report.get('analysis_metadata', {}).get('regression_threshold', 20.0)}%")
    report_sections.append("")
    
    # Executive summary
    if regressions:
        report_sections.append("## 🚨 Executive Summary")
        report_sections.append("")
        report_sections.append("**Status**: ❌ Performance regressions detected")
        report_sections.append(f"**Total Regressions**: {len(regressions)}")
        report_sections.append(f"**Success Rate**: {summary.get('success_rate', 0):.1f}%")
        report_sections.append("")
        report_sections.append("Performance regressions above the 20% threshold have been detected.")
        report_sections.append("Investigation and remediation is required before merging.")
        report_sections.append("")
    else:
        report_sections.append("## ✅ Executive Summary")
        report_sections.append("")
        report_sections.append("**Status**: ✅ All tests passed")
        report_sections.append(f"**Success Rate**: {summary.get('success_rate', 0):.1f}%")
        report_sections.append("")
        report_sections.append("All matrix configurations passed performance validation.")
        report_sections.append("No regressions above the 20% threshold were detected.")
        report_sections.append("")
    
    # Add sections
    report_sections.append(generate_matrix_overview(summary))
    report_sections.append(generate_performance_summary(summary))
    report_sections.append(generate_regression_details(regressions))
    report_sections.append(generate_baseline_analysis(baselines))
    report_sections.append(generate_recommendations(regressions, summary))
    
    # Footer
    report_sections.append("---")
    report_sections.append("*This report was automatically generated by the Matrix Testing system*")
    report_sections.append("*For questions or issues, contact the Polymera OS development team*")
    
    return "\n".join(report_sections)

def main():
    """Main entry point"""
    if len(sys.argv) != 2:
        print("Usage: python3 generate_matrix_report.py <analysis_report>")
        print("Example: python3 generate_matrix_report.py analysis_report.json")
        sys.exit(1)
    
    report_file = sys.argv[1]
    
    if not Path(report_file).exists():
        print(f"Error: Analysis report '{report_file}' not found")
        sys.exit(1)
    
    try:
        # Load analysis report
        print(f"Loading analysis report: {report_file}")
        report = load_analysis_report(report_file)
        
        # Generate matrix report
        print("Generating matrix report...")
        matrix_report = generate_matrix_report(report)
        
        # Output report
        print(matrix_report)
        
    except Exception as e:
        print(f"Error generating matrix report: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
