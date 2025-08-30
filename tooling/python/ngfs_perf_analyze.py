#!/usr/bin/env python3
"""
NGFS Performance Analyzer

This script validates benchmark results against baselines, checks statistical sanity,
and maintains performance history for trend analysis.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime
import statistics


class NgfsPerfAnalyzer:
    """Analyzes NGFS performance benchmark results"""
    
    def __init__(self):
        self.errors = []
        self.warnings = []
        self.baseline = None
        self.history_file = None
    
    def load_baseline(self, baseline_path: Path) -> bool:
        """Load performance baseline from file"""
        try:
            with open(baseline_path, 'r') as f:
                self.baseline = json.load(f)
            return True
        except Exception as e:
            self.errors.append(f"Failed to load baseline: {e}")
            return False
    
    def load_bench_results(self, input_path: Path) -> Optional[Dict[str, Any]]:
        """Load benchmark results from file"""
        try:
            with open(input_path, 'r') as f:
                return json.load(f)
        except Exception as e:
            self.errors.append(f"Failed to load bench results: {e}")
            return None
    
    def validate_statistical_sanity(self, bench_data: Dict[str, Any]) -> bool:
        """Validate statistical sanity of benchmark results"""
        if "cases" not in bench_data:
            self.errors.append("Missing 'cases' field in bench data")
            return False
        
        cases = bench_data["cases"]
        required_cases = ["resolve_path", "stat_file", "read_small", "read_large", "snapshot_build"]
        
        for case_name in required_cases:
            if case_name not in cases:
                self.errors.append(f"Missing required case: {case_name}")
                continue
            
            case_data = cases[case_name]
            if not isinstance(case_data, dict):
                self.errors.append(f"Invalid case data for {case_name}")
                continue
            
            # Validate required fields
            required_fields = ["p50", "p95", "p99", "mean", "stdev", "n", "errors"]
            for field in required_fields:
                if field not in case_data:
                    self.errors.append(f"Missing field '{field}' in {case_name}")
                    continue
            
            # Check sample count
            n = case_data.get("n", 0)
            if n < 50:
                self.errors.append(f"Insufficient samples for {case_name}: {n} < 50")
            
            # Check error count
            errors = case_data.get("errors", 0)
            if errors > 0:
                self.errors.append(f"Errors detected in {case_name}: {errors}")
            
            # Check variance limits
            mean = case_data.get("mean", 0)
            stdev = case_data.get("stdev", 0)
            if mean > 0:
                variance_ratio = stdev / mean
                
                if case_name in ["read_small", "read_large"]:
                    if variance_ratio > 0.25:
                        self.errors.append(f"High variance in {case_name}: stdev/mean={variance_ratio:.2f} > 0.25")
                elif case_name in ["resolve_path", "stat_file"]:
                    if variance_ratio > 0.35:
                        self.errors.append(f"High variance in {case_name}: stdev/mean={variance_ratio:.2f} > 0.35")
        
        return len(self.errors) == 0
    
    def check_baseline_compliance(self, bench_data: Dict[str, Any]) -> bool:
        """Check benchmark results against baseline budgets"""
        if not self.baseline:
            self.warnings.append("No baseline loaded, skipping compliance check")
            return True
        
        if "budgets" not in self.baseline:
            self.warnings.append("No budgets found in baseline")
            return True
        
        budgets = self.baseline["budgets"]
        cases = bench_data.get("cases", {})
        
        for case_name, budget_data in budgets.items():
            if case_name not in cases:
                self.warnings.append(f"Case {case_name} not found in bench results")
                continue
            
            case_data = cases[case_name]
            p95 = case_data.get("p95", 0)
            
            # Check p95 budget
            if "p95_max" in budget_data:
                max_p95 = budget_data["p95_max"]
                if p95 > max_p95:
                    self.errors.append(f"{case_name} p95 exceeds budget: {p95} > {max_p95}")
            
            # Check variance budget
            if "variance_limit" in budget_data:
                variance_limit = budget_data["variance_limit"]
                mean = case_data.get("mean", 0)
                stdev = case_data.get("stdev", 0)
                
                if mean > 0:
                    variance_ratio = stdev / mean
                    if variance_ratio > variance_limit:
                        self.errors.append(f"{case_name} variance exceeds limit: {variance_ratio:.2f} > {variance_limit}")
        
        return len(self.errors) == 0
    
    def write_history_entry(self, bench_data: Dict[str, Any], history_path: Path) -> bool:
        """Write benchmark results to history file"""
        try:
            # Ensure history directory exists
            history_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Create history entry
            history_entry = {
                "timestamp": datetime.now().isoformat(),
                "test": bench_data.get("test", "unknown"),
                "duration_ms": bench_data.get("duration_ms", 0),
                "cases": bench_data.get("cases", {}),
                "errors": bench_data.get("errors", 0),
                "mismatch": bench_data.get("mismatch", 0),
            }
            
            # Append to history file
            with open(history_path, 'a') as f:
                f.write(json.dumps(history_entry) + '\n')
            
            return True
        except Exception as e:
            self.errors.append(f"Failed to write history entry: {e}")
            return False
    
    def analyze_performance(self, input_path: Path, baseline_path: Optional[Path], history_path: Optional[Path]) -> bool:
        """Perform complete performance analysis"""
        # Load benchmark results
        bench_data = self.load_bench_results(input_path)
        if not bench_data:
            return False
        
        # Load baseline if provided
        if baseline_path:
            if not self.load_baseline(baseline_path):
                return False
        
        # Validate statistical sanity
        if not self.validate_statistical_sanity(bench_data):
            return False
        
        # Check baseline compliance
        if not self.check_baseline_compliance(bench_data):
            return False
        
        # Write history entry if path provided
        if history_path:
            if not self.write_history_entry(bench_data, history_path):
                return False
        
        return True
    
    def print_report(self):
        """Print analysis report"""
        if self.errors:
            print("❌ Performance analysis failed:")
            for error in self.errors:
                print(f"  ERROR: {error}")
            return False
        
        if self.warnings:
            print("⚠️  Performance analysis passed with warnings:")
            for warning in self.warnings:
                print(f"  WARNING: {warning}")
        else:
            print("✅ Performance analysis passed")
        
        return True


def main():
    parser = argparse.ArgumentParser(description="Analyze NGFS performance benchmark results")
    parser.add_argument("--input", required=True, type=Path, help="Benchmark results file")
    parser.add_argument("--baseline", type=Path, help="Performance baseline file")
    parser.add_argument("--history", type=Path, help="History file path")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    
    args = parser.parse_args()
    
    if not args.input.exists():
        print(f"Input file not found: {args.input}", file=sys.stderr)
        sys.exit(1)
    
    analyzer = NgfsPerfAnalyzer()
    
    # Perform analysis
    success = analyzer.analyze_performance(args.input, args.baseline, args.history)
    
    # Output results
    if args.json:
        output = {
            "valid": success,
            "errors": analyzer.errors,
            "warnings": analyzer.warnings
        }
        print(json.dumps(output))
    else:
        analyzer.print_report()
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
