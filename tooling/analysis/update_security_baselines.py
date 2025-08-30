#!/usr/bin/env python3
"""
Security Baseline Update Script for Polymera OS

This script processes security analysis results and updates security baselines
to track trends and detect regressions over time.

Usage:
    python update_security_baselines.py --input DIR --output FILE --commit SHA
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone
import statistics
from dataclasses import dataclass, asdict
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class SecurityMetrics:
    """Security metrics for a specific scope and commit"""
    scope: str
    commit: str
    timestamp: str
    clippy_warnings: int
    clippy_errors: int
    security_vulnerabilities: int
    sidechannel_vulnerabilities: int
    total_issues: int
    overall_status: str

@dataclass
class SecurityBaseline:
    """Security baseline containing historical metrics"""
    project: str
    version: str
    last_updated: str
    total_commits: int
    metrics_by_scope: Dict[str, List[SecurityMetrics]]
    trends: Dict[str, Dict[str, Any]]
    summary: Dict[str, Any]

class SecurityBaselineUpdater:
    """Updates security baselines from analysis results"""
    
    def __init__(self, input_dir: str, output_file: str, commit_sha: str):
        self.input_dir = Path(input_dir)
        self.output_file = Path(output_file)
        self.commit_sha = commit_sha
        self.project = "polymera-os"
        self.version = "0.2.0-phase2"
        
    def load_existing_baseline(self) -> Optional[SecurityBaseline]:
        """Load existing security baseline if it exists"""
        if not self.output_file.exists():
            logger.info("No existing baseline found, creating new one")
            return None
            
        try:
            with open(self.output_file, 'r') as f:
                data = json.load(f)
                
            # Convert back to dataclass
            metrics_by_scope = {}
            for scope, metrics_list in data.get('metrics_by_scope', {}).items():
                metrics_by_scope[scope] = [
                    SecurityMetrics(**metric) for metric in metrics_list
                ]
                
            baseline = SecurityBaseline(
                project=data.get('project', self.project),
                version=data.get('version', self.version),
                last_updated=data.get('last_updated', ''),
                total_commits=data.get('total_commits', 0),
                metrics_by_scope=metrics_by_scope,
                trends=data.get('trends', {}),
                summary=data.get('summary', {})
            )
            
            logger.info(f"Loaded existing baseline with {baseline.total_commits} commits")
            return baseline
            
        except Exception as e:
            logger.warning(f"Failed to load existing baseline: {e}")
            return None
    
    def process_security_results(self) -> Dict[str, SecurityMetrics]:
        """Process security analysis results from input directory"""
        results = {}
        
        # Look for security result artifacts
        for artifact_dir in self.input_dir.glob("security-results-*"):
            logger.info(f"Processing artifact: {artifact_dir.name}")
            
            # Extract scope from artifact name
            # Format: security-results-{scope}-{commit}
            parts = artifact_dir.name.split('-')
            if len(parts) >= 3:
                scope = parts[2]  # Extract scope
                
                # Look for security_report.json in the scope directory
                scope_dir = artifact_dir / scope
                report_file = scope_dir / "security_report.json"
                
                if report_file.exists():
                    try:
                        with open(report_file, 'r') as f:
                            report_data = json.load(f)
                        
                        # Extract metrics
                        summary = report_data.get('summary', {})
                        metrics = SecurityMetrics(
                            scope=scope,
                            commit=self.commit_sha,
                            timestamp=report_data.get('timestamp', datetime.now().isoformat()),
                            clippy_warnings=summary.get('clippy_warnings', 0),
                            clippy_errors=summary.get('clippy_errors', 0),
                            security_vulnerabilities=summary.get('security_vulnerabilities', 0),
                            sidechannel_vulnerabilities=summary.get('sidechannel_vulnerabilities', 0),
                            total_issues=summary.get('total_issues', 0),
                            overall_status=summary.get('overall_status', 'unknown')
                        )
                        
                        results[scope] = metrics
                        logger.info(f"Processed {scope}: {metrics.total_issues} total issues")
                        
                    except Exception as e:
                        logger.error(f"Failed to process {report_file}: {e}")
                else:
                    logger.warning(f"Security report not found in {scope_dir}")
            else:
                logger.warning(f"Could not extract scope from artifact name: {artifact_dir.name}")
        
        return results
    
    def update_baseline(self, existing_baseline: Optional[SecurityBaseline], 
                       new_metrics: Dict[str, SecurityMetrics]) -> SecurityBaseline:
        """Update baseline with new metrics"""
        
        if existing_baseline is None:
            # Create new baseline
            metrics_by_scope = {scope: [metrics] for scope, metrics in new_metrics.items()}
            total_commits = 1
        else:
            # Update existing baseline
            metrics_by_scope = existing_baseline.metrics_by_scope.copy()
            
            # Add new metrics to each scope
            for scope, metrics in new_metrics.items():
                if scope not in metrics_by_scope:
                    metrics_by_scope[scope] = []
                metrics_by_scope[scope].append(metrics)
                
                # Keep only last 50 commits per scope to avoid bloat
                if len(metrics_by_scope[scope]) > 50:
                    metrics_by_scope[scope] = metrics_by_scope[scope][-50:]
            
            total_commits = existing_baseline.total_commits + 1
        
        # Calculate trends
        trends = self._calculate_trends(metrics_by_scope)
        
        # Calculate summary statistics
        summary = self._calculate_summary(metrics_by_scope)
        
        baseline = SecurityBaseline(
            project=self.project,
            version=self.version,
            last_updated=datetime.now(timezone.utc).isoformat(),
            total_commits=total_commits,
            metrics_by_scope=metrics_by_scope,
            trends=trends,
            summary=summary
        )
        
        return baseline
    
    def _calculate_trends(self, metrics_by_scope: Dict[str, List[SecurityMetrics]]) -> Dict[str, Dict[str, Any]]:
        """Calculate trends for each scope and metric type"""
        trends = {}
        
        for scope, metrics_list in metrics_by_scope.items():
            if len(metrics_list) < 2:
                continue
                
            scope_trends = {}
            
            # Calculate trends for each metric type
            metric_types = ['clippy_warnings', 'clippy_errors', 'security_vulnerabilities', 
                          'sidechannel_vulnerabilities', 'total_issues']
            
            for metric_type in metric_types:
                values = [getattr(m, metric_type) for m in metrics_list]
                
                if len(values) >= 2:
                    # Calculate trend (positive = increasing, negative = decreasing)
                    trend_direction = "stable"
                    trend_magnitude = 0.0
                    
                    if len(values) >= 3:
                        # Use linear regression for trend
                        x = list(range(len(values)))
                        slope = self._linear_regression_slope(x, values)
                        
                        if abs(slope) > 0.1:  # Threshold for meaningful trend
                            trend_direction = "increasing" if slope > 0 else "decreasing"
                            trend_magnitude = abs(slope)
                        else:
                            trend_direction = "stable"
                    
                    # Calculate volatility (standard deviation)
                    volatility = statistics.stdev(values) if len(values) > 1 else 0.0
                    
                    scope_trends[metric_type] = {
                        "direction": trend_direction,
                        "magnitude": trend_magnitude,
                        "volatility": volatility,
                        "recent_values": values[-5:],  # Last 5 values
                        "min_value": min(values),
                        "max_value": max(values),
                        "mean_value": statistics.mean(values)
                    }
            
            trends[scope] = scope_trends
        
        return trends
    
    def _linear_regression_slope(self, x: List[int], y: List[int]) -> float:
        """Calculate slope of linear regression line"""
        n = len(x)
        if n < 2:
            return 0.0
        
        sum_x = sum(x)
        sum_y = sum(y)
        sum_xy = sum(x[i] * y[i] for i in range(n))
        sum_x2 = sum(x[i] ** 2 for i in range(n))
        
        numerator = n * sum_xy - sum_x * sum_y
        denominator = n * sum_x2 - sum_x ** 2
        
        if denominator == 0:
            return 0.0
        
        return numerator / denominator
    
    def _calculate_summary(self, metrics_by_scope: Dict[str, List[SecurityMetrics]]) -> Dict[str, Any]:
        """Calculate overall summary statistics"""
        summary = {
            "total_scopes": len(metrics_by_scope),
            "overall_status": "pass",
            "total_issues_current": 0,
            "total_issues_trend": "stable",
            "scope_status": {},
            "critical_metrics": {}
        }
        
        # Calculate current status for each scope
        for scope, metrics_list in metrics_by_scope.items():
            if metrics_list:
                latest = metrics_list[-1]
                summary["scope_status"][scope] = {
                    "current_status": latest.overall_status,
                    "total_issues": latest.total_issues,
                    "last_commit": latest.commit
                }
                summary["total_issues_current"] += latest.total_issues
        
        # Determine overall status
        status_counts = {"pass": 0, "warn": 0, "fail": 0}
        for scope_status in summary["scope_status"].values():
            status_counts[scope_status["current_status"]] += 1
        
        if status_counts["fail"] > 0:
            summary["overall_status"] = "fail"
        elif status_counts["warn"] > 0:
            summary["overall_status"] = "warn"
        else:
            summary["overall_status"] = "pass"
        
        # Calculate overall trend
        all_total_issues = []
        for metrics_list in metrics_by_scope.values():
            if len(metrics_list) >= 2:
                all_total_issues.extend([m.total_issues for m in metrics_list[-2:]])
        
        if len(all_total_issues) >= 4:
            first_half = all_total_issues[:len(all_total_issues)//2]
            second_half = all_total_issues[len(all_total_issues)//2:]
            
            first_avg = statistics.mean(first_half)
            second_avg = statistics.mean(second_half)
            
            if second_avg > first_avg * 1.1:
                summary["total_issues_trend"] = "increasing"
            elif second_avg < first_avg * 0.9:
                summary["total_issues_trend"] = "decreasing"
            else:
                summary["total_issues_trend"] = "stable"
        
        return summary
    
    def save_baseline(self, baseline: SecurityBaseline):
        """Save baseline to output file"""
        # Ensure output directory exists
        self.output_file.parent.mkdir(parents=True, exist_ok=True)
        
        # Convert dataclasses to dict for JSON serialization
        baseline_dict = asdict(baseline)
        
        # Convert SecurityMetrics objects to dicts
        for scope, metrics_list in baseline_dict['metrics_by_scope'].items():
            baseline_dict['metrics_by_scope'][scope] = [
                asdict(metrics) for metrics in metrics_list
            ]
        
        with open(self.output_file, 'w') as f:
            json.dump(baseline_dict, f, indent=2)
        
        logger.info(f"Saved security baseline to {self.output_file}")
    
    def run(self):
        """Main execution method"""
        logger.info(f"Starting security baseline update for commit {self.commit_sha}")
        
        # Process security results
        new_metrics = self.process_security_results()
        
        if not new_metrics:
            logger.warning("No security results found to process")
            return
        
        # Load existing baseline
        existing_baseline = self.load_existing_baseline()
        
        # Update baseline
        updated_baseline = self.update_baseline(existing_baseline, new_metrics)
        
        # Save updated baseline
        self.save_baseline(updated_baseline)
        
        # Print summary
        logger.info("Security baseline update completed:")
        logger.info(f"  - Processed {len(new_metrics)} scopes")
        logger.info(f"  - Total commits in baseline: {updated_baseline.total_commits}")
        logger.info(f"  - Overall status: {updated_baseline.summary['overall_status']}")
        logger.info(f"  - Total current issues: {updated_baseline.summary['total_issues_current']}")

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Update security baselines from analysis results")
    parser.add_argument("--input", required=True, help="Input directory containing security results")
    parser.add_argument("--output", required=True, help="Output file for security baseline")
    parser.add_argument("--commit", required=True, help="Commit SHA for the analysis")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Validate inputs
    if not os.path.exists(args.input):
        logger.error(f"Input directory does not exist: {args.input}")
        sys.exit(1)
    
    # Create updater and run
    updater = SecurityBaselineUpdater(args.input, args.output, args.commit)
    
    try:
        updater.run()
    except Exception as e:
        logger.error(f"Failed to update security baseline: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
