#!/usr/bin/env python3
"""
Gas Report Comparison Script for Polymera OS Contracts

This script compares gas reports from different Foundry test runs to detect
gas regressions and improvements in smart contract performance.
"""

import json
import re
import sys
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from enum import Enum


class GasChange(Enum):
    """Enum for gas change types"""
    IMPROVEMENT = "improvement"
    REGRESSION = "regression"
    NO_CHANGE = "no_change"


@dataclass
class GasMetric:
    """Represents a gas metric for a function"""
    function: str
    contract: str
    gas_used: int
    gas_used_deployment: Optional[int] = None
    gas_used_runtime: Optional[int] = None


@dataclass
class GasComparison:
    """Represents a comparison between two gas metrics"""
    function: str
    contract: str
    base_gas: int
    current_gas: int
    difference: int
    percentage_change: float
    change_type: GasChange
    threshold: int


class GasReportParser:
    """Parser for Foundry gas reports"""
    
    def __init__(self, report_path: str):
        self.report_path = Path(report_path)
        self.metrics: List[GasMetric] = []
        
    def parse(self) -> List[GasMetric]:
        """Parse the gas report file"""
        if not self.report_path.exists():
            raise FileNotFoundError(f"Gas report not found: {self.report_path}")
        
        content = self.report_path.read_text()
        
        # Parse different gas report formats
        if "|" in content and "Function" in content:
            return self._parse_table_format(content)
        elif "gas used" in content.lower():
            return self._parse_verbose_format(content)
        else:
            return self._parse_simple_format(content)
    
    def _parse_table_format(self, content: str) -> List[GasMetric]:
        """Parse table format gas report"""
        metrics = []
        lines = content.strip().split('\n')
        
        # Find header line
        header_idx = None
        for i, line in enumerate(lines):
            if '|' in line and 'Function' in line:
                header_idx = i
                break
        
        if header_idx is None:
            return metrics
        
        # Parse data lines
        for line in lines[header_idx + 1:]:
            if '|' not in line or line.strip() == '':
                continue
                
            parts = [p.strip() for p in line.split('|')]
            if len(parts) < 4:
                continue
            
            try:
                function = parts[1].strip()
                contract = parts[0].strip()
                gas_str = parts[2].strip()
                
                # Extract gas values
                gas_match = re.search(r'(\d+)', gas_str)
                if gas_match:
                    gas_used = int(gas_match.group(1))
                    
                    metric = GasMetric(
                        function=function,
                        contract=contract,
                        gas_used=gas_used
                    )
                    metrics.append(metric)
            except (ValueError, IndexError):
                continue
        
        return metrics
    
    def _parse_verbose_format(self, content: str) -> List[GasMetric]:
        """Parse verbose format gas report"""
        metrics = []
        
        # Look for patterns like "Function: mint, Gas used: 123456"
        pattern = r'Function:\s*([^,\n]+),\s*Gas used:\s*(\d+)'
        matches = re.findall(pattern, content)
        
        for function, gas_str in matches:
            try:
                gas_used = int(gas_str)
                # Extract contract name from function if possible
                contract = "Unknown"
                if '.' in function:
                    contract, func_name = function.split('.', 1)
                else:
                    func_name = function
                
                metric = GasMetric(
                    function=func_name.strip(),
                    contract=contract.strip(),
                    gas_used=gas_used
                )
                metrics.append(metric)
            except ValueError:
                continue
        
        return metrics
    
    def _parse_simple_format(self, content: str) -> List[GasMetric]:
        """Parse simple format gas report"""
        metrics = []
        
        # Look for any gas usage patterns
        pattern = r'(\w+)\s*:\s*(\d+)\s*gas'
        matches = re.findall(pattern, content)
        
        for func_name, gas_str in matches:
            try:
                gas_used = int(gas_str)
                metric = GasMetric(
                    function=func_name.strip(),
                    contract="Unknown",
                    gas_used=gas_used
                )
                metrics.append(metric)
            except ValueError:
                continue
        
        return metrics


class GasComparator:
    """Compares gas reports and detects regressions"""
    
    def __init__(self, base_report: str, current_report: str, threshold: int = 5):
        self.base_report = base_report
        self.current_report = current_report
        self.threshold = threshold
        self.base_metrics: List[GasMetric] = []
        self.current_metrics: List[GasMetric] = []
        
    def load_reports(self):
        """Load and parse both gas reports"""
        base_parser = GasReportParser(self.base_report)
        current_parser = GasReportParser(self.current_report)
        
        self.base_metrics = base_parser.parse()
        self.current_metrics = current_parser.parse()
        
        print(f"Loaded {len(self.base_metrics)} base metrics")
        print(f"Loaded {len(self.current_metrics)} current metrics")
    
    def compare(self) -> List[GasComparison]:
        """Compare gas usage between reports"""
        comparisons = []
        
        # Create lookup dictionaries
        base_lookup = {(m.contract, m.function): m for m in self.base_metrics}
        current_lookup = {(m.contract, m.function): m for m in self.current_metrics}
        
        # Compare all functions
        all_functions = set(base_lookup.keys()) | set(current_lookup.keys())
        
        for contract, function in all_functions:
            base_metric = base_lookup.get((contract, function))
            current_metric = current_lookup.get((contract, function))
            
            if base_metric and current_metric:
                # Both exist, compare
                diff = current_metric.gas_used - base_metric.gas_used
                percentage = (diff / base_metric.gas_used) * 100
                
                if abs(percentage) <= self.threshold:
                    change_type = GasChange.NO_CHANGE
                elif diff > 0:
                    change_type = GasChange.REGRESSION
                else:
                    change_type = GasChange.IMPROVEMENT
                
                comparison = GasComparison(
                    function=function,
                    contract=contract,
                    base_gas=base_metric.gas_used,
                    current_gas=current_metric.gas_used,
                    difference=diff,
                    percentage_change=percentage,
                    change_type=change_type,
                    threshold=self.threshold
                )
                comparisons.append(comparison)
            
            elif base_metric and not current_metric:
                # Function removed
                print(f"⚠️  Function removed: {contract}.{function}")
            
            elif not base_metric and current_metric:
                # Function added
                print(f"✨ Function added: {contract}.{function} (gas: {current_metric.gas_used})")
        
        return comparisons
    
    def generate_report(self, comparisons: List[GasComparison]) -> str:
        """Generate a human-readable comparison report"""
        if not comparisons:
            return "No comparable functions found"
        
        # Sort by percentage change (regressions first)
        comparisons.sort(key=lambda x: x.percentage_change, reverse=True)
        
        report = []
        report.append("# Gas Usage Comparison Report")
        report.append("")
        
        # Summary
        regressions = [c for c in comparisons if c.change_type == GasChange.REGRESSION]
        improvements = [c for c in comparisons if c.change_type == GasChange.IMPROVEMENT]
        no_change = [c for c in comparisons if c.change_type == GasChange.NO_CHANGE]
        
        report.append("## Summary")
        report.append(f"- **Total Functions**: {len(comparisons)}")
        report.append(f"- **Regressions**: {len(regressions)}")
        report.append(f"- **Improvements**: {len(improvements)}")
        report.append(f"- **No Change**: {len(no_change)}")
        report.append(f"- **Threshold**: ±{self.threshold}%")
        report.append("")
        
        # Regressions
        if regressions:
            report.append("## 🚨 Gas Regressions")
            report.append("")
            report.append("| Contract | Function | Base Gas | Current Gas | Change | % Change |")
            report.append("|----------|----------|----------|-------------|---------|----------|")
            
            for comp in regressions:
                report.append(
                    f"| {comp.contract} | {comp.function} | {comp.base_gas:,} | "
                    f"{comp.current_gas:,} | +{comp.difference:,} | +{comp.percentage_change:.1f}% |"
                )
            report.append("")
        
        # Improvements
        if improvements:
            report.append("## ✅ Gas Improvements")
            report.append("")
            report.append("| Contract | Function | Base Gas | Current Gas | Change | % Change |")
            report.append("|----------|----------|----------|-------------|---------|----------|")
            
            for comp in improvements:
                report.append(
                    f"| {comp.contract} | {comp.function} | {comp.base_gas:,} | "
                    f"{comp.current_gas:,} | {comp.difference:,} | {comp.percentage_change:.1f}% |"
                )
            report.append("")
        
        # No change
        if no_change:
            report.append("## ➡️ No Significant Change")
            report.append("")
            report.append("| Contract | Function | Gas | % Change |")
            report.append("|----------|----------|-----|----------|")
            
            for comp in no_change:
                report.append(
                    f"| {comp.contract} | {comp.function} | {comp.current_gas:,} | "
                    f"{comp.percentage_change:.1f}% |"
                )
            report.append("")
        
        return "\n".join(report)
    
    def check_regressions(self, comparisons: List[GasComparison]) -> bool:
        """Check if there are any significant regressions"""
        significant_regressions = [
            c for c in comparisons 
            if c.change_type == GasChange.REGRESSION and abs(c.percentage_change) > self.threshold
        ]
        
        if significant_regressions:
            print(f"🚨 Found {len(significant_regressions)} significant gas regressions!")
            return True
        
        print("✅ No significant gas regressions detected")
        return False


def main():
    """Main function"""
    parser = argparse.ArgumentParser(description="Compare Foundry gas reports")
    parser.add_argument("base_report", help="Path to base gas report")
    parser.add_argument("current_report", help="Path to current gas report")
    parser.add_argument("--threshold", "-t", type=int, default=5,
                       help="Percentage threshold for significant changes (default: 5)")
    parser.add_argument("--output", "-o", help="Output file for report")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    
    args = parser.parse_args()
    
    try:
        # Initialize comparator
        comparator = GasComparator(args.base_report, args.current_report, args.threshold)
        
        # Load reports
        print("Loading gas reports...")
        comparator.load_reports()
        
        # Compare reports
        print("Comparing gas usage...")
        comparisons = comparator.compare()
        
        if not comparisons:
            print("No comparable functions found")
            sys.exit(1)
        
        # Generate report
        report = comparator.generate_report(comparisons)
        
        # Output report
        if args.output:
            with open(args.output, 'w') as f:
                f.write(report)
            print(f"Report saved to: {args.output}")
        else:
            print(report)
        
        # Check for regressions
        has_regressions = comparator.check_regressions(comparisons)
        
        # Exit with error if regressions found
        if has_regressions:
            print("\n❌ Gas regressions detected - CI should fail")
            sys.exit(1)
        else:
            print("\n✅ Gas comparison completed successfully")
            sys.exit(0)
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
