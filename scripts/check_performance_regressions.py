#!/usr/bin/env python3
"""
Performance Regression Checker

This script checks for performance regressions in the analysis report
and fails CI if regressions exceed the specified threshold.
"""

import sys
import json
from pathlib import Path
from typing import Dict, List

def load_analysis_report(report_file: str) -> Dict:
    """Load the analysis report JSON file"""
    try:
        with open(report_file, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading analysis report: {e}")
        sys.exit(1)

def check_regression_threshold(regressions: List[Dict], threshold: float) -> bool:
    """Check if any regressions exceed the threshold"""
    if not regressions:
        return False
    
    # Check for regressions above threshold
    severe_regressions = [r for r in regressions if r['degradation'] > threshold]
    
    if severe_regressions:
        print(f"🚨 Found {len(severe_regressions)} regressions above {threshold}% threshold:")
        for reg in severe_regressions:
            print(f"  {reg['configuration']}: {reg['metric']} regression {reg['degradation']:.1f}%")
        return True
    
    return False

def analyze_regression_patterns(regressions: List[Dict]) -> Dict:
    """Analyze patterns in performance regressions"""
    if not regressions:
        return {}
    
    # Group by configuration
    by_config = {}
    by_metric = {}
    by_type = {}
    
    for reg in regressions:
        # By configuration
        config = reg['configuration']
        if config not in by_config:
            by_config[config] = []
        by_config[config].append(reg)
        
        # By metric
        metric = reg['metric']
        if metric not in by_metric:
            by_metric[metric] = []
        by_metric[metric].append(reg)
        
        # By type
        metric_type = reg['metric_type']
        if metric_type not in by_type:
            by_type[metric_type] = []
        by_type[metric_type].append(reg)
    
    # Calculate statistics
    analysis = {
        'total_regressions': len(regressions),
        'by_configuration': {
            config: {
                'count': len(regs),
                'avg_degradation': sum(r['degradation'] for r in regs) / len(regs),
                'max_degradation': max(r['degradation'] for r in regs),
                'metrics': list(set(r['metric'] for r in regs))
            }
            for config, regs in by_config.items()
        },
        'by_metric': {
            metric: {
                'count': len(regs),
                'avg_degradation': sum(r['degradation'] for r in regs) / len(regs),
                'max_degradation': max(r['degradation'] for r in regs),
                'configurations': list(set(r['configuration'] for r in regs))
            }
            for metric, regs in by_metric.items()
        },
        'by_type': {
            metric_type: {
                'count': len(regs),
                'avg_degradation': sum(r['degradation'] for r in regs) / len(regs),
                'max_degradation': max(r['degradation'] for r in regs)
            }
            for metric_type, regs in by_type.items()
        }
    }
    
    return analysis

def generate_regression_report(regressions: List[Dict], threshold: float) -> str:
    """Generate a human-readable regression report"""
    if not regressions:
        return "✅ No performance regressions detected"
    
    report = []
    report.append("🚨 Performance Regressions Detected")
    report.append("=" * 50)
    report.append(f"Total regressions: {len(regressions)}")
    report.append(f"Threshold: {threshold}%")
    report.append("")
    
    # Group by severity
    severe = [r for r in regressions if r['degradation'] > threshold * 2]
    moderate = [r for r in regressions if threshold < r['degradation'] <= threshold * 2]
    minor = [r for r in regressions if r['degradation'] <= threshold]
    
    if severe:
        report.append("🔴 SEVERE REGRESSIONS (>{}%):".format(threshold * 2))
        for reg in severe:
            report.append(f"  {reg['configuration']}: {reg['metric']} regression {reg['degradation']:.1f}%")
        report.append("")
    
    if moderate:
        report.append("🟡 MODERATE REGRESSIONS ({}%-{}%):".format(threshold, threshold * 2))
        for reg in moderate:
            report.append(f"  {reg['configuration']}: {reg['metric']} regression {reg['degradation']:.1f}%")
        report.append("")
    
    if minor:
        report.append("🟢 MINOR REGRESSIONS (≤{}%):".format(threshold))
        for reg in minor:
            report.append(f"  {reg['configuration']}: {reg['metric']} regression {reg['degradation']:.1f}%")
        report.append("")
    
    # Summary by configuration
    configs = {}
    for reg in regressions:
        config = reg['configuration']
        if config not in configs:
            configs[config] = []
        configs[config].append(reg)
    
    report.append("📊 Summary by Configuration:")
    for config, regs in configs.items():
        avg_degradation = sum(r['degradation'] for r in regs) / len(regs)
        max_degradation = max(r['degradation'] for r in regs)
        report.append(f"  {config}: {len(regs)} regressions, avg {avg_degradation:.1f}%, max {max_degradation:.1f}%")
    
    return "\n".join(report)

def main():
    """Main entry point"""
    if len(sys.argv) != 3:
        print("Usage: python3 check_performance_regressions.py <analysis_report> <threshold>")
        print("Example: python3 check_performance_regressions.py analysis_report.json 20.0")
        sys.exit(1)
    
    report_file = sys.argv[1]
    threshold = float(sys.argv[2])
    
    if not Path(report_file).exists():
        print(f"Error: Analysis report '{report_file}' not found")
        sys.exit(1)
    
    try:
        # Load analysis report
        print(f"Loading analysis report: {report_file}")
        report = load_analysis_report(report_file)
        
        # Extract regressions
        regressions = report.get('regressions', [])
        print(f"Found {len(regressions)} performance regressions")
        
        # Check threshold
        has_severe_regressions = check_regression_threshold(regressions, threshold)
        
        # Analyze patterns
        if regressions:
            print("\nAnalyzing regression patterns...")
            patterns = analyze_regression_patterns(regressions)
            
            print(f"Regressions by configuration:")
            for config, stats in patterns['by_configuration'].items():
                print(f"  {config}: {stats['count']} regressions, avg {stats['avg_degradation']:.1f}%")
            
            print(f"\nRegressions by metric type:")
            for metric_type, stats in patterns['by_type'].items():
                print(f"  {metric_type}: {stats['count']} regressions, avg {stats['avg_degradation']:.1f}%")
        
        # Generate report
        regression_report = generate_regression_report(regressions, threshold)
        print(f"\n{regression_report}")
        
        # Determine exit status
        if has_severe_regressions:
            print(f"\n❌ FAILED: Performance regressions above {threshold}% threshold detected")
            print("CI will fail due to performance regressions")
            sys.exit(1)
        else:
            print(f"\n✅ PASSED: No performance regressions above {threshold}% threshold")
            print("CI will pass - performance within acceptable bounds")
            sys.exit(0)
            
    except Exception as e:
        print(f"Error during regression check: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
