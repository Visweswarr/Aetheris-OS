#!/usr/bin/env python3
"""
Merge baseline performance results into a consolidated Phase 4 baseline file.
"""

import json
import sys
import os
from pathlib import Path
from typing import Dict, Any

def load_json_file(file_path: Path) -> Dict[str, Any]:
    """Load a JSON file and return its contents."""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"Warning: Could not load {file_path}: {e}")
        return {}

def merge_baselines(bench_dir: Path, output_file: Path) -> None:
    """Merge all benchmark files into a consolidated baseline."""
    
    # Expected benchmark files
    benchmark_files = {
        'xr': 'xr.json',
        'hal': 'hal.json', 
        'ai': 'ai.json'
    }
    
    # Initialize consolidated baseline
    baseline = {
        "phase": "Phase 4",
        "version": "0.4.0-phase4-ready",
        "timestamp": "",
        "benchmarks": {},
        "summary": {
            "total_benchmarks": 0,
            "completed_benchmarks": 0,
            "environment": {}
        }
    }
    
    completed_count = 0
    
    # Load each benchmark file
    for benchmark_name, filename in benchmark_files.items():
        file_path = bench_dir / filename
        if file_path.exists():
            benchmark_data = load_json_file(file_path)
            if benchmark_data:
                baseline["benchmarks"][benchmark_name] = benchmark_data
                completed_count += 1
                
                # Extract environment info from first benchmark
                if not baseline["summary"]["environment"] and "environment" in benchmark_data:
                    baseline["summary"]["environment"] = benchmark_data["environment"]
        else:
            print(f"Warning: Benchmark file not found: {file_path}")
    
    # Update summary
    baseline["summary"]["total_benchmarks"] = len(benchmark_files)
    baseline["summary"]["completed_benchmarks"] = completed_count
    
    # Set timestamp
    from datetime import datetime
    baseline["timestamp"] = datetime.utcnow().isoformat() + "Z"
    
    # Write consolidated baseline
    with open(output_file, 'w') as f:
        json.dump(baseline, f, indent=2)
    
    print(f"✅ Consolidated baseline saved to: {output_file}")
    print(f"📊 Benchmarks: {completed_count}/{len(benchmark_files)} completed")
    
    # Print summary
    for benchmark_name in benchmark_files.keys():
        status = "✅" if benchmark_name in baseline["benchmarks"] else "❌"
        print(f"  {status} {benchmark_name.upper()}")

def main():
    """Main function."""
    if len(sys.argv) < 2:
        print("Usage: python merge_baselines.py <bench_dir> [output_file]")
        print("Example: python merge_baselines.py artifacts/bench --out artifacts/bench/baseline.phase4.json")
        sys.exit(1)
    
    bench_dir = Path(sys.argv[1])
    
    # Parse arguments for output file
    output_file = bench_dir / "baseline.phase4.json"
    if len(sys.argv) > 2:
        if sys.argv[2] == "--out" and len(sys.argv) > 3:
            output_file = Path(sys.argv[3])
        else:
            output_file = Path(sys.argv[2])
    
    if not bench_dir.exists():
        print(f"Error: Benchmark directory does not exist: {bench_dir}")
        sys.exit(1)
    
    merge_baselines(bench_dir, output_file)

if __name__ == "__main__":
    main()
