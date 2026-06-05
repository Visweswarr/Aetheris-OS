#!/usr/bin/env python3
"""
Network Performance Reporting and Analysis for Aetheris OS

This module provides tools for aggregating benchmark runs, computing deltas
vs baselines, and generating performance reports.
"""

import json
import argparse
import sys
import os
import statistics
from datetime import datetime
from typing import Dict, List, Optional, Any, Tuple
from pathlib import Path

class NetworkReport:
    """Network performance reporting and analysis."""
    
    def __init__(self, baseline_file: str = "perf/baselines/p4_03_net_adv.json"):
        self.baseline_file = baseline_file
        self.baselines = self._load_baselines()
    
    def _load_baselines(self) -> Dict[str, Any]:
        """Load baseline metrics from file."""
        if not os.path.exists(self.baseline_file):
            return {}
        
        try:
            with open(self.baseline_file, 'r') as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            print(f"Warning: Failed to load baselines: {e}")
            return {}
    
    def _save_baselines(self) -> None:
        """Save baseline metrics to file."""
        os.makedirs(os.path.dirname(self.baseline_file), exist_ok=True)
        
        try:
            with open(self.baseline_file, 'w') as f:
                json.dump(self.baselines, f, indent=2)
        except IOError as e:
            print(f"Error: Failed to save baselines: {e}")
    
    def aggregate_runs(self, run_files: List[str]) -> Dict[str, Any]:
        """Aggregate multiple benchmark runs."""
        runs = []
        
        for run_file in run_files:
            try:
                with open(run_file, 'r') as f:
                    run_data = json.load(f)
                    runs.append(run_data)
            except (json.JSONDecodeError, IOError) as e:
                print(f"Warning: Failed to load run file {run_file}: {e}")
                continue
        
        if not runs:
            raise ValueError("No valid run files found")
        
        # Aggregate metrics
        aggregated = {
            "runs": len(runs),
            "timestamp": datetime.now().isoformat(),
            "metrics": self._aggregate_metrics(runs),
            "config": runs[0]["config"] if runs else {},
        }
        
        return aggregated
    
    def _aggregate_metrics(self, runs: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Aggregate metrics from multiple runs."""
        metrics = {}
        
        # Extract all metric values
        metric_values = {}
        for run in runs:
            run_metrics = run.get("metrics", {})
            for key, value in run_metrics.items():
                if isinstance(value, (int, float)):
                    if key not in metric_values:
                        metric_values[key] = []
                    metric_values[key].append(value)
        
        # Calculate statistics for each metric
        for key, values in metric_values.items():
            if values:
                metrics[key] = {
                    "mean": statistics.mean(values),
                    "median": statistics.median(values),
                    "stdev": statistics.stdev(values) if len(values) > 1 else 0,
                    "min": min(values),
                    "max": max(values),
                    "count": len(values),
                }
        
        return metrics
    
    def compare_baseline(self, current_metrics: Dict[str, Any], 
                        baseline_key: str) -> Dict[str, Any]:
        """Compare current metrics against baseline."""
        if baseline_key not in self.baselines:
            return {"error": f"Baseline key '{baseline_key}' not found"}
        
        baseline = self.baselines[baseline_key]
        comparison = {
            "baseline_key": baseline_key,
            "timestamp": datetime.now().isoformat(),
            "deltas": {},
            "regressions": [],
            "improvements": [],
        }
        
        # Compare each metric
        for key, current_value in current_metrics.items():
            if key in baseline and isinstance(current_value, (int, float)):
                baseline_value = baseline[key]
                if baseline_value != 0:
                    delta_pct = ((current_value - baseline_value) / baseline_value) * 100
                    comparison["deltas"][key] = {
                        "current": current_value,
                        "baseline": baseline_value,
                        "delta_pct": delta_pct,
                    }
                    
                    # Check for regressions (>10% worse)
                    if delta_pct > 10:
                        comparison["regressions"].append({
                            "metric": key,
                            "delta_pct": delta_pct,
                            "current": current_value,
                            "baseline": baseline_value,
                        })
                    
                    # Check for improvements (>10% better)
                    elif delta_pct < -10:
                        comparison["improvements"].append({
                            "metric": key,
                            "delta_pct": delta_pct,
                            "current": current_value,
                            "baseline": baseline_value,
                        })
        
        return comparison
    
    def update_baseline(self, metrics: Dict[str, Any], 
                       config: Dict[str, Any]) -> str:
        """Update baseline with new metrics."""
        # Generate baseline key from config
        baseline_key = self._generate_baseline_key(config)
        
        # Update baseline
        self.baselines[baseline_key] = metrics
        
        # Save baselines
        self._save_baselines()
        
        return baseline_key
    
    def _generate_baseline_key(self, config: Dict[str, Any]) -> str:
        """Generate baseline key from config."""
        clients = config.get("clients", 0)
        tls_enabled = config.get("tls_enabled", False)
        duration = config.get("duration", 0)
        
        return f"clients_{clients}_tls_{tls_enabled}_duration_{duration}"
    
    def generate_report(self, aggregated_data: Dict[str, Any], 
                       baseline_key: Optional[str] = None) -> str:
        """Generate a human-readable performance report."""
        report = []
        report.append("Network Performance Report")
        report.append("=" * 50)
        report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"Runs aggregated: {aggregated_data['runs']}")
        report.append("")
        
        # Configuration
        config = aggregated_data.get("config", {})
        report.append("Configuration:")
        report.append(f"  Clients: {config.get('clients', 'N/A')}")
        report.append(f"  Duration: {config.get('duration', 'N/A')}")
        report.append(f"  TLS Enabled: {config.get('tls_enabled', 'N/A')}")
        report.append(f"  TLS Profile: {config.get('tls_profile', 'N/A')}")
        report.append(f"  Payload Size: {config.get('payload_size', 'N/A')}")
        report.append("")
        
        # Metrics
        metrics = aggregated_data.get("metrics", {})
        report.append("Performance Metrics:")
        report.append("-" * 30)
        
        for key, stats in metrics.items():
            if isinstance(stats, dict) and "mean" in stats:
                report.append(f"{key}:")
                report.append(f"  Mean: {stats['mean']:.2f}")
                report.append(f"  Median: {stats['median']:.2f}")
                report.append(f"  Std Dev: {stats['stdev']:.2f}")
                report.append(f"  Min: {stats['min']:.2f}")
                report.append(f"  Max: {stats['max']:.2f}")
                report.append(f"  Count: {stats['count']}")
                report.append("")
        
        # Baseline comparison
        if baseline_key:
            comparison = self.compare_baseline(
                {k: v["mean"] for k, v in metrics.items() if "mean" in v},
                baseline_key
            )
            
            if "error" not in comparison:
                report.append("Baseline Comparison:")
                report.append("-" * 30)
                report.append(f"Baseline: {baseline_key}")
                report.append("")
                
                # Deltas
                deltas = comparison.get("deltas", {})
                for metric, delta in deltas.items():
                    report.append(f"{metric}:")
                    report.append(f"  Current: {delta['current']:.2f}")
                    report.append(f"  Baseline: {delta['baseline']:.2f}")
                    report.append(f"  Delta: {delta['delta_pct']:+.2f}%")
                    report.append("")
                
                # Regressions
                regressions = comparison.get("regressions", [])
                if regressions:
                    report.append("⚠️  Performance Regressions (>10% worse):")
                    for reg in regressions:
                        report.append(f"  {reg['metric']}: {reg['delta_pct']:+.2f}%")
                    report.append("")
                
                # Improvements
                improvements = comparison.get("improvements", [])
                if improvements:
                    report.append("✅ Performance Improvements (>10% better):")
                    for imp in improvements:
                        report.append(f"  {imp['metric']}: {imp['delta_pct']:+.2f}%")
                    report.append("")
        
        return "\n".join(report)
    
    def check_regressions(self, aggregated_data: Dict[str, Any], 
                         baseline_key: str, threshold: float = 10.0) -> bool:
        """Check if there are any performance regressions above threshold."""
        comparison = self.compare_baseline(
            {k: v["mean"] for k, v in aggregated_data.get("metrics", {}).items() 
             if isinstance(v, dict) and "mean" in v},
            baseline_key
        )
        
        if "error" in comparison:
            print(f"Error: {comparison['error']}")
            return False
        
        regressions = comparison.get("regressions", [])
        critical_regressions = [r for r in regressions if r["delta_pct"] > threshold]
        
        if critical_regressions:
            print("❌ Performance regressions detected:")
            for reg in critical_regressions:
                print(f"  {reg['metric']}: {reg['delta_pct']:+.2f}% "
                      f"(current: {reg['current']:.2f}, "
                      f"baseline: {reg['baseline']:.2f})")
            return True
        
        print("✅ No performance regressions detected")
        return False

def main():
    """Main CLI interface."""
    parser = argparse.ArgumentParser(
        description="Network Performance Reporting and Analysis"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Aggregate command
    aggregate_parser = subparsers.add_parser("aggregate", help="Aggregate benchmark runs")
    aggregate_parser.add_argument("run_files", nargs="+", help="Benchmark run files")
    aggregate_parser.add_argument("--output", "-o", help="Output file for aggregated data")
    
    # Compare command
    compare_parser = subparsers.add_parser("compare", help="Compare against baseline")
    compare_parser.add_argument("run_file", help="Benchmark run file")
    compare_parser.add_argument("baseline_key", help="Baseline key to compare against")
    compare_parser.add_argument("--threshold", type=float, default=10.0,
                               help="Regression threshold percentage")
    
    # Report command
    report_parser = subparsers.add_parser("report", help="Generate performance report")
    report_parser.add_argument("run_files", nargs="+", help="Benchmark run files")
    report_parser.add_argument("--baseline", help="Baseline key for comparison")
    report_parser.add_argument("--output", "-o", help="Output file for report")
    
    # Update baseline command
    update_parser = subparsers.add_parser("update-baseline", help="Update baseline")
    update_parser.add_argument("run_file", help="Benchmark run file")
    
    # Check regressions command
    check_parser = subparsers.add_parser("check-regressions", help="Check for regressions")
    check_parser.add_argument("run_files", nargs="+", help="Benchmark run files")
    check_parser.add_argument("baseline_key", help="Baseline key to compare against")
    check_parser.add_argument("--threshold", type=float, default=10.0,
                             help="Regression threshold percentage")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    reporter = NetworkReport()
    
    try:
        if args.command == "aggregate":
            aggregated = reporter.aggregate_runs(args.run_files)
            
            if args.output:
                with open(args.output, 'w') as f:
                    json.dump(aggregated, f, indent=2)
                print(f"Aggregated data written to {args.output}")
            else:
                print(json.dumps(aggregated, indent=2))
        
        elif args.command == "compare":
            with open(args.run_file, 'r') as f:
                run_data = json.load(f)
            
            comparison = reporter.compare_baseline(
                run_data.get("metrics", {}),
                args.baseline_key
            )
            
            print(json.dumps(comparison, indent=2))
        
        elif args.command == "report":
            aggregated = reporter.aggregate_runs(args.run_files)
            report = reporter.generate_report(aggregated, args.baseline)
            
            if args.output:
                with open(args.output, 'w') as f:
                    f.write(report)
                print(f"Report written to {args.output}")
            else:
                print(report)
        
        elif args.command == "update-baseline":
            with open(args.run_file, 'r') as f:
                run_data = json.load(f)
            
            baseline_key = reporter.update_baseline(
                run_data.get("metrics", {}),
                run_data.get("config", {})
            )
            
            print(f"Baseline updated with key: {baseline_key}")
        
        elif args.command == "check-regressions":
            aggregated = reporter.aggregate_runs(args.run_files)
            has_regressions = reporter.check_regressions(
                aggregated, args.baseline_key, args.threshold
            )
            
            if has_regressions:
                sys.exit(1)
    
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
