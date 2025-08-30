#!/usr/bin/env python3
"""
Performance Baseline Updater

This script updates performance baselines from matrix testing results
for the main branch.
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

def load_existing_baselines() -> Dict:
    """Load existing performance baselines if they exist"""
    baseline_file = Path("performance_baselines.json")
    
    if baseline_file.exists():
        try:
            with open(baseline_file, 'r') as f:
                return json.load(f)
        except Exception as e:
            print(f"Warning: Could not load existing baselines: {e}")
            return {}
    else:
        print("No existing baselines found, creating new baseline file")
        return {}

def update_baselines(
    existing_baselines: Dict,
    current_metrics: List[Dict],
    baselines: Dict[str, Dict]
) -> Dict:
    """Update baselines with current performance data"""
    
    updated_baselines = existing_baselines.copy()
    
    # Add metadata
    updated_baselines['metadata'] = {
        'last_updated': time.time(),
        'last_updated_iso': time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        'baseline_version': updated_baselines.get('metadata', {}).get('baseline_version', 0) + 1,
        'matrix_testing_version': '1.0',
        'total_configurations': len(baselines)
    }
    
    # Update baseline metrics
    updated_baselines['baselines'] = {}
    
    for config, baseline in baselines.items():
        config_baseline = {
            'configuration': baseline['configuration'],
            'cpu_model': baseline['cpu_model'],
            'smp_cores': baseline['smp_cores'],
            'memory_mb': baseline['memory_mb'],
            'last_updated': time.time(),
            'last_updated_iso': time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            'metric_count': baseline['metric_count'],
            'baseline_metrics': baseline['baseline_metrics']
        }
        
        # Add historical data if available
        if config in existing_baselines.get('baselines', {}):
            existing = existing_baselines['baselines'][config]
            config_baseline['history'] = existing.get('history', [])
            
            # Add current baseline to history
            history_entry = {
                'timestamp': time.time(),
                'timestamp_iso': time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                'baseline_version': updated_baselines['metadata']['baseline_version'],
                'metrics': baseline['baseline_metrics']
            }
            
            config_baseline['history'].append(history_entry)
            
            # Keep only last 10 history entries
            if len(config_baseline['history']) > 10:
                config_baseline['history'] = config_baseline['history'][-10:]
        else:
            config_baseline['history'] = []
        
        updated_baselines['baselines'][config] = config_baseline
    
    # Add summary statistics
    updated_baselines['summary'] = {
        'total_configurations': len(baselines),
        'cpu_models': list(set(b['cpu_model'] for b in baselines.values())),
        'smp_cores': list(set(b['smp_cores'] for b in baselines.values())),
        'memory_sizes': list(set(b['memory_mb'] for b in baselines.values())),
        'baseline_coverage': {
            'ipc_metrics': sum(1 for b in baselines.values() if b['baseline_metrics'].get('ipc')),
            'system_metrics': sum(1 for b in baselines.values() if b['baseline_metrics'].get('system'))
        }
    }
    
    return updated_baselines

def generate_baseline_summary(baselines: Dict) -> str:
    """Generate a summary of baseline updates"""
    summary = []
    summary.append("## 📊 Performance Baseline Update Summary")
    summary.append("")
    summary.append(f"**Total Configurations**: {len(baselines)}")
    summary.append(f"**Update Timestamp**: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    summary.append("")
    
    # Group by CPU model
    by_cpu = {}
    for config, baseline in baselines.items():
        cpu = baseline['cpu_model']
        if cpu not in by_cpu:
            by_cpu[cpu] = []
        by_cpu[cpu].append(baseline)
    
    for cpu, cpu_baselines in by_cpu.items():
        summary.append(f"### {cpu}")
        summary.append("")
        
        # Create summary table
        summary.append("| Configuration | IPC Throughput | P50 Latency | P95 Latency | Boot Time |")
        summary.append("|---------------|----------------|-------------|-------------|-----------|")
        
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
            
            summary.append(f"| {config} | {ipc_str} | {p50_str} | {p95_str} | {boot_str} |")
        
        summary.append("")
    
    return "\n".join(summary)

def save_baselines(baselines: Dict, output_file: str = "performance_baselines.json"):
    """Save updated baselines to file"""
    try:
        with open(output_file, 'w') as f:
            json.dump(baselines, f, indent=2)
        print(f"Baselines saved to {output_file}")
    except Exception as e:
        print(f"Error saving baselines: {e}")
        sys.exit(1)

def create_baseline_changelog(
    existing_baselines: Dict,
    updated_baselines: Dict
) -> str:
    """Create a changelog of baseline updates"""
    
    changelog = []
    changelog.append("# 📝 Performance Baseline Changelog")
    changelog.append("")
    changelog.append(f"**Generated**: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    changelog.append(f"**Baseline Version**: {updated_baselines['metadata']['baseline_version']}")
    changelog.append("")
    
    existing = existing_baselines.get('baselines', {})
    updated = updated_baselines['baselines']
    
    # Find new configurations
    new_configs = set(updated.keys()) - set(existing.keys())
    if new_configs:
        changelog.append("## 🆕 New Configurations")
        changelog.append("")
        for config in sorted(new_configs):
            baseline = updated[config]
            changelog.append(f"- **{config}**: {baseline['cpu_model']}, {baseline['smp_cores']} cores, {baseline['memory_mb']}MB")
        changelog.append("")
    
    # Find updated configurations
    updated_configs = []
    for config in updated:
        if config in existing:
            # Check if metrics changed significantly
            old_metrics = existing[config].get('baseline_metrics', {})
            new_metrics = updated[config]['baseline_metrics']
            
            # Simple change detection (could be enhanced)
            if old_metrics != new_metrics:
                updated_configs.append(config)
    
    if updated_configs:
        changelog.append("## 🔄 Updated Configurations")
        changelog.append("")
        for config in sorted(updated_configs):
            changelog.append(f"- **{config}**: Metrics updated")
        changelog.append("")
    
    # Summary
    changelog.append("## 📊 Summary")
    changelog.append("")
    changelog.append(f"- **New Configurations**: {len(new_configs)}")
    changelog.append(f"- **Updated Configurations**: {len(updated_configs)}")
    changelog.append(f"- **Total Configurations**: {len(updated)}")
    changelog.append(f"- **Baseline Version**: {updated_baselines['metadata']['baseline_version']}")
    
    return "\n".join(changelog)

def main():
    """Main entry point"""
    if len(sys.argv) != 2:
        print("Usage: python3 update_performance_baselines.py <analysis_report>")
        print("Example: python3 update_performance_baselines.py analysis_report.json")
        sys.exit(1)
    
    report_file = sys.argv[1]
    
    if not Path(report_file).exists():
        print(f"Error: Analysis report '{report_file}' not found")
        sys.exit(1)
    
    try:
        # Load analysis report
        print(f"Loading analysis report: {report_file}")
        report = load_analysis_report(report_file)
        
        # Extract components
        current_metrics = report.get('current_metrics', [])
        baselines = report.get('baselines', {})
        
        if not baselines:
            print("No baseline data found in analysis report")
            sys.exit(1)
        
        # Load existing baselines
        existing_baselines = load_existing_baselines()
        
        # Update baselines
        print("Updating performance baselines...")
        updated_baselines = update_baselines(existing_baselines, current_metrics, baselines)
        
        # Save updated baselines
        save_baselines(updated_baselines)
        
        # Generate summary
        print("Generating baseline summary...")
        summary = generate_baseline_summary(baselines)
        
        # Save summary to file
        with open("baseline_update_summary.md", 'w') as f:
            f.write(summary)
        print("Baseline update summary saved to baseline_update_summary.md")
        
        # Generate changelog
        print("Generating baseline changelog...")
        changelog = create_baseline_changelog(existing_baselines, updated_baselines)
        
        # Save changelog to file
        with open("baseline_changelog.md", 'w') as f:
            f.write(changelog)
        print("Baseline changelog saved to baseline_changelog.md")
        
        # Output summary
        print(f"\nBaseline update completed:")
        print(f"  Total configurations: {len(baselines)}")
        print(f"  Baseline version: {updated_baselines['metadata']['baseline_version']}")
        print(f"  New baselines: {len(set(baselines.keys()) - set(existing_baselines.get('baselines', {}).keys()))}")
        
    except Exception as e:
        print(f"Error updating baselines: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
