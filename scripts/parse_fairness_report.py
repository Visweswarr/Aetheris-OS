#!/usr/bin/env python3
"""
Polymera OS Scheduler Fairness Report Parser

This script parses scheduler fairness test output from QEMU serial logs
and validates that all fairness criteria are met. Used as a CI gate
to ensure scheduler performance and fairness.
"""

import sys
import re
import json
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass
from pathlib import Path

@dataclass
class FairnessMetrics:
    """Scheduler fairness test metrics"""
    test_duration_ms: int
    total_tasks: int
    test_passed: bool
    
    # Starvation metrics
    max_wait_time_ms: int
    max_wait_threshold_ms: int
    starvation_violations: int
    starvation_passed: bool
    
    # RT latency metrics
    rt_p95_latency_ms: int
    rt_threshold_ms: int
    rt_violations: int
    rt_passed: bool
    
    # CPU share metrics
    cpu_share_tolerance: float
    cpu_share_violations: int
    cpu_share_passed: bool
    
    # Task counts
    cpu_bound_tasks: int
    io_blocked_tasks: int
    rt_bursty_tasks: int

@dataclass
class TaskSummary:
    """Individual task performance summary"""
    task_id: int
    task_type: str
    runtime_ms: int
    wait_ms: int
    max_wait_ms: int
    cpu_share: float
    executions: int
    io_ops: int

@dataclass
class CpuShareViolation:
    """CPU share violation details"""
    task_type: str
    expected_share: float
    actual_share: float
    deviation: float

class FairnessReportParser:
    """Parser for scheduler fairness test reports"""
    
    def __init__(self):
        self.metrics: Optional[FairnessMetrics] = None
        self.task_summaries: List[TaskSummary] = []
        self.cpu_violations: List[CpuShareViolation] = []
    
    def parse_serial_log(self, log_file: str) -> bool:
        """Parse fairness report from serial log file"""
        try:
            with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
            
            return self.parse_content(content)
        except Exception as e:
            print(f"Error reading log file: {e}", file=sys.stderr)
            return False
    
    def parse_content(self, content: str) -> bool:
        """Parse fairness report from content string"""
        lines = content.split('\n')
        
        # Find the fairness report section
        in_report = False
        report_lines = []
        
        for line in lines:
            if "=== SCHEDULER FAIRNESS REPORT ===" in line:
                in_report = True
                continue
            elif "=== END SCHEDULER FAIRNESS REPORT ===" in line:
                in_report = False
                break
            elif in_report:
                report_lines.append(line.strip())
        
        if not report_lines:
            print("No fairness report found in log", file=sys.stderr)
            return False
        
        return self.parse_report_lines(report_lines)
    
    def parse_report_lines(self, lines: List[str]) -> bool:
        """Parse individual report lines"""
        try:
            # Parse main metrics
            metrics_data = {}
            
            for line in lines:
                if ':' not in line:
                    continue
                
                key, value = line.split(':', 1)
                key = key.strip()
                value = value.strip()
                
                # Parse different metric types
                if key.startswith('FAIRNESS_'):
                    self.parse_metric_line(key, value, metrics_data)
                elif key.startswith('FAIRNESS_TASK:'):
                    self.parse_task_line(value)
                elif key.startswith('FAIRNESS_CPU_VIOLATION:'):
                    self.parse_violation_line(value)
            
            # Create metrics object
            self.metrics = self.create_metrics_object(metrics_data)
            return True
            
        except Exception as e:
            print(f"Error parsing report lines: {e}", file=sys.stderr)
            return False
    
    def parse_metric_line(self, key: str, value: str, metrics_data: Dict[str, Any]):
        """Parse a single metric line"""
        # Convert value to appropriate type
        if value.lower() in ['true', 'false']:
            parsed_value = value.lower() == 'true'
        elif value.replace('.', '').replace('-', '').isdigit():
            if '.' in value:
                parsed_value = float(value)
            else:
                parsed_value = int(value)
        elif value.endswith('%'):
            parsed_value = float(value[:-1])
        elif value in ['PASS', 'FAIL']:
            parsed_value = value == 'PASS'
        else:
            parsed_value = value
        
        metrics_data[key] = parsed_value
    
    def parse_task_line(self, value: str) -> bool:
        """Parse task summary line"""
        try:
            # Parse task line format: id=100 type=CPU-bound runtime_ms=1234 ...
            parts = value.split()
            task_data = {}
            
            for part in parts:
                if '=' in part:
                    k, v = part.split('=', 1)
                    
                    # Convert values
                    if v.replace('.', '').replace('-', '').isdigit():
                        if '.' in v:
                            task_data[k] = float(v)
                        else:
                            task_data[k] = int(v)
                    elif v.endswith('%'):
                        task_data[k] = float(v[:-1])
                    else:
                        task_data[k] = v
            
            # Create task summary
            if 'id' in task_data and 'type' in task_data:
                task = TaskSummary(
                    task_id=task_data.get('id', 0),
                    task_type=task_data.get('type', ''),
                    runtime_ms=task_data.get('runtime_ms', 0),
                    wait_ms=task_data.get('wait_ms', 0),
                    max_wait_ms=task_data.get('max_wait_ms', 0),
                    cpu_share=task_data.get('share', 0.0),
                    executions=task_data.get('executions', 0),
                    io_ops=task_data.get('io_ops', 0)
                )
                self.task_summaries.append(task)
                return True
        except Exception as e:
            print(f"Error parsing task line: {e}", file=sys.stderr)
        
        return False
    
    def parse_violation_line(self, value: str) -> bool:
        """Parse CPU share violation line"""
        try:
            # Parse violation line format: type=CPU-bound expected=40.0% actual=35.2% deviation=4.8%
            parts = value.split()
            violation_data = {}
            
            for part in parts:
                if '=' in part:
                    k, v = part.split('=', 1)
                    
                    if v.endswith('%'):
                        violation_data[k] = float(v[:-1])
                    else:
                        violation_data[k] = v
            
            # Create violation object
            if 'type' in violation_data:
                violation = CpuShareViolation(
                    task_type=violation_data.get('type', ''),
                    expected_share=violation_data.get('expected', 0.0),
                    actual_share=violation_data.get('actual', 0.0),
                    deviation=violation_data.get('deviation', 0.0)
                )
                self.cpu_violations.append(violation)
                return True
        except Exception as e:
            print(f"Error parsing violation line: {e}", file=sys.stderr)
        
        return False
    
    def create_metrics_object(self, data: Dict[str, Any]) -> FairnessMetrics:
        """Create FairnessMetrics object from parsed data"""
        return FairnessMetrics(
            test_duration_ms=data.get('FAIRNESS_TEST_DURATION_MS', 0),
            total_tasks=data.get('FAIRNESS_TOTAL_TASKS', 0),
            test_passed=data.get('FAIRNESS_TEST_PASSED', False),
            
            max_wait_time_ms=data.get('FAIRNESS_MAX_WAIT_TIME_MS', 0),
            max_wait_threshold_ms=data.get('FAIRNESS_MAX_WAIT_THRESHOLD_MS', 100),
            starvation_violations=data.get('FAIRNESS_STARVATION_VIOLATIONS', 0),
            starvation_passed=data.get('FAIRNESS_STARVATION_PASSED', False),
            
            rt_p95_latency_ms=data.get('FAIRNESS_RT_P95_LATENCY_MS', 0),
            rt_threshold_ms=data.get('FAIRNESS_RT_THRESHOLD_MS', 5),
            rt_violations=data.get('FAIRNESS_RT_VIOLATIONS', 0),
            rt_passed=data.get('FAIRNESS_RT_PASSED', False),
            
            cpu_share_tolerance=data.get('FAIRNESS_CPU_SHARE_TOLERANCE', 15.0),
            cpu_share_violations=data.get('FAIRNESS_CPU_SHARE_VIOLATIONS', 0),
            cpu_share_passed=data.get('FAIRNESS_CPU_SHARE_PASSED', False),
            
            cpu_bound_tasks=data.get('FAIRNESS_CPU_BOUND_TASKS', 0),
            io_blocked_tasks=data.get('FAIRNESS_IO_BLOCKED_TASKS', 0),
            rt_bursty_tasks=data.get('FAIRNESS_RT_BURSTY_TASKS', 0)
        )
    
    def validate_fairness_criteria(self) -> Tuple[bool, List[str]]:
        """Validate all fairness criteria and return results"""
        if not self.metrics:
            return False, ["No fairness metrics found"]
        
        issues = []
        
        # Check starvation criteria
        if not self.metrics.starvation_passed:
            issues.append(f"Starvation detected: {self.metrics.starvation_violations} violations, "
                         f"max wait time {self.metrics.max_wait_time_ms}ms > {self.metrics.max_wait_threshold_ms}ms")
        
        # Check RT latency criteria
        if not self.metrics.rt_passed:
            issues.append(f"RT latency violations: {self.metrics.rt_violations} violations, "
                         f"P95 latency {self.metrics.rt_p95_latency_ms}ms > {self.metrics.rt_threshold_ms}ms")
        
        # Check CPU share criteria
        if not self.metrics.cpu_share_passed:
            issues.append(f"CPU share violations: {self.metrics.cpu_share_violations} violations")
            for violation in self.cpu_violations:
                issues.append(f"  {violation.task_type}: expected {violation.expected_share}%, "
                             f"actual {violation.actual_share}%, deviation {violation.deviation}%")
        
        # Check task counts
        expected_tasks_per_type = 3
        if self.metrics.cpu_bound_tasks != expected_tasks_per_type:
            issues.append(f"Unexpected CPU-bound task count: {self.metrics.cpu_bound_tasks} != {expected_tasks_per_type}")
        
        if self.metrics.io_blocked_tasks != expected_tasks_per_type:
            issues.append(f"Unexpected IO-blocked task count: {self.metrics.io_blocked_tasks} != {expected_tasks_per_type}")
        
        if self.metrics.rt_bursty_tasks != expected_tasks_per_type:
            issues.append(f"Unexpected RT-bursty task count: {self.metrics.rt_bursty_tasks} != {expected_tasks_per_type}")
        
        # Check test duration
        expected_duration = 10000  # 10 seconds
        duration_tolerance = 1000  # 1 second tolerance
        if abs(self.metrics.test_duration_ms - expected_duration) > duration_tolerance:
            issues.append(f"Test duration out of tolerance: {self.metrics.test_duration_ms}ms "
                         f"(expected ~{expected_duration}ms)")
        
        return len(issues) == 0, issues
    
    def generate_summary_report(self) -> str:
        """Generate human-readable summary report"""
        if not self.metrics:
            return "No fairness metrics available"
        
        report = []
        report.append("=== SCHEDULER FAIRNESS TEST SUMMARY ===")
        report.append(f"Test Duration: {self.metrics.test_duration_ms}ms ({self.metrics.test_duration_ms/1000:.1f}s)")
        report.append(f"Total Tasks: {self.metrics.total_tasks}")
        report.append(f"Overall Result: {'PASS' if self.metrics.test_passed else 'FAIL'}")
        report.append("")
        
        report.append("Starvation Analysis:")
        report.append(f"  Max Wait Time: {self.metrics.max_wait_time_ms}ms (threshold: {self.metrics.max_wait_threshold_ms}ms)")
        report.append(f"  Violations: {self.metrics.starvation_violations}")
        report.append(f"  Result: {'PASS' if self.metrics.starvation_passed else 'FAIL'}")
        report.append("")
        
        report.append("RT Latency Analysis:")
        report.append(f"  P95 Wake-to-Run: {self.metrics.rt_p95_latency_ms}ms (threshold: {self.metrics.rt_threshold_ms}ms)")
        report.append(f"  Violations: {self.metrics.rt_violations}")
        report.append(f"  Result: {'PASS' if self.metrics.rt_passed else 'FAIL'}")
        report.append("")
        
        report.append("CPU Share Analysis:")
        report.append(f"  Tolerance: ±{self.metrics.cpu_share_tolerance}%")
        report.append(f"  Violations: {self.metrics.cpu_share_violations}")
        report.append(f"  Result: {'PASS' if self.metrics.cpu_share_passed else 'FAIL'}")
        
        if self.cpu_violations:
            report.append("  Violation Details:")
            for violation in self.cpu_violations:
                report.append(f"    {violation.task_type}: {violation.expected_share}% expected, "
                             f"{violation.actual_share}% actual (deviation: {violation.deviation}%)")
        report.append("")
        
        report.append("Task Distribution:")
        report.append(f"  CPU-bound: {self.metrics.cpu_bound_tasks}")
        report.append(f"  IO-blocked: {self.metrics.io_blocked_tasks}")
        report.append(f"  RT-bursty: {self.metrics.rt_bursty_tasks}")
        
        return '\n'.join(report)
    
    def export_json_report(self, output_file: str) -> bool:
        """Export detailed report as JSON"""
        try:
            report_data = {
                'metrics': self.metrics.__dict__ if self.metrics else None,
                'task_summaries': [task.__dict__ for task in self.task_summaries],
                'cpu_violations': [violation.__dict__ for violation in self.cpu_violations],
                'validation': {
                    'passed': self.validate_fairness_criteria()[0],
                    'issues': self.validate_fairness_criteria()[1]
                }
            }
            
            with open(output_file, 'w') as f:
                json.dump(report_data, f, indent=2)
            
            return True
        except Exception as e:
            print(f"Error exporting JSON report: {e}", file=sys.stderr)
            return False

def main():
    """Main entry point for CI usage"""
    if len(sys.argv) < 2:
        print("Usage: python3 parse_fairness_report.py <serial_log_file> [output_json]", file=sys.stderr)
        print("Example: python3 parse_fairness_report.py qemu_output.log fairness_report.json", file=sys.stderr)
        sys.exit(1)
    
    log_file = sys.argv[1]
    output_json = sys.argv[2] if len(sys.argv) > 2 else None
    
    parser = FairnessReportParser()
    
    # Parse the serial log
    if not parser.parse_serial_log(log_file):
        print("Failed to parse fairness report from log file", file=sys.stderr)
        sys.exit(1)
    
    # Validate fairness criteria
    passed, issues = parser.validate_fairness_criteria()
    
    # Generate summary
    summary = parser.generate_summary_report()
    print(summary)
    
    # Export JSON if requested
    if output_json:
        if parser.export_json_report(output_json):
            print(f"\nDetailed report exported to: {output_json}")
        else:
            print(f"\nFailed to export JSON report to: {output_json}", file=sys.stderr)
    
    # Print validation results
    print(f"\n=== VALIDATION RESULTS ===")
    print(f"Overall Result: {'PASS' if passed else 'FAIL'}")
    
    if issues:
        print("Issues Found:")
        for issue in issues:
            print(f"  - {issue}")
    else:
        print("All fairness criteria met!")
    
    # Exit with appropriate code for CI
    sys.exit(0 if passed else 1)

if __name__ == "__main__":
    main()

