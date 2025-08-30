#!/usr/bin/env python3
"""
Baseline Promotion Tool for Polymera OS

This tool analyzes performance history and proposes new baselines when performance
has stabilized and improved. It generates approval PRs with diff summaries and
impact heatmaps.

Usage:
    python promote_baseline.py [--history-dir DIR] [--baseline-file FILE] [--output-dir DIR]
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
import statistics
import subprocess
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class PerformanceMetric:
    """Represents a performance metric with statistical properties"""
    name: str
    current_value: float
    baseline_value: float
    history_values: List[float]
    mean: float
    std_dev: float
    trend: str
    change_percent: float
    status: str  # improved, stable, regressed

@dataclass
class ConfigurationAnalysis:
    """Analysis results for a specific configuration"""
    config_id: str
    config_name: str
    total_runs: int
    stable_runs: int
    metrics: Dict[str, PerformanceMetric]
    overall_status: str
    promotion_recommended: bool
    rationale: str

@dataclass
class BaselineProposal:
    """Complete baseline promotion proposal"""
    generated_at: str
    current_baseline_version: str
    proposed_baseline_version: str
    configurations: Dict[str, ConfigurationAnalysis]
    summary: Dict[str, Any]
    promotion_ready: bool
    changelog_entry: str

class BaselinePromoter:
    """Handles baseline promotion analysis and proposal generation"""
    
    def __init__(self, history_dir: str, baseline_file: str, output_dir: str):
        self.history_dir = Path(history_dir)
        self.baseline_file = Path(baseline_file)
        self.output_dir = Path(output_dir)
        self.history_data = {}
        self.current_baseline = {}
        self.stability_threshold = 5  # Minimum stable runs required
        self.regression_threshold = 1.0  # Standard deviations above mean for regression
        
    def run(self) -> BaselineProposal:
        """Main execution method"""
        logger.info('🚀 Starting Baseline Promotion Analysis...')
        
        try:
            # Load and analyze data
            self.loadCurrentBaseline()
            self.loadHistoryData()
            
            # Analyze performance stability and trends
            analysis = self.analyzePerformance()
            
            # Generate baseline proposal
            proposal = self.generateProposal(analysis)
            
            # Save proposal and generate artifacts
            self.saveProposal(proposal)
            self.generateArtifacts(proposal)
            
            logger.info('✅ Baseline promotion analysis completed successfully!')
            return proposal
            
        except Exception as error:
            logger.error('❌ Baseline promotion analysis failed:', error)
            raise
    
    def loadCurrentBaseline(self):
        """Load current baseline performance data"""
        logger.info('📈 Loading current baseline...')
        
        if not self.baseline_file.exists():
            logger.warning(f'⚠️  Baseline file not found: {self.baseline_file}')
            self.current_baseline = {
                'version': '0.0.0',
                'generated_at': datetime.now().isoformat(),
                'baselines': {}
            }
            return
        
        try:
            with open(self.baseline_file, 'r') as f:
                self.current_baseline = json.load(f)
            logger.info(f'✅ Loaded baseline version: {self.current_baseline.get("version", "unknown")}')
        except Exception as error:
            logger.error(f'❌ Failed to load baseline: {error}')
            self.current_baseline = {}
    
    def loadHistoryData(self):
        """Load performance history data from all configurations"""
        logger.info('📚 Loading performance history...')
        
        if not self.history_dir.exists():
            logger.warning(f'⚠️  History directory not found: {self.history_dir}')
            self.history_data = {}
            return
        
        history_files = list(self.history_dir.glob('*.json'))
        logger.info(f'📁 Found {len(history_files)} history files')
        
        for file_path in history_files:
            if file_path.name == '.gitkeep':
                continue
                
            try:
                with open(file_path, 'r') as f:
                    data = json.load(f)
                
                config_id = self.extractConfigId(file_path.name)
                if config_id:
                    self.history_data[config_id] = data
                    
            except Exception as error:
                logger.warning(f'⚠️  Failed to load {file_path.name}: {error}')
        
        logger.info(f'✅ Loaded history data for {len(self.history_data)} configurations')
    
    def extractConfigId(self, filename: str) -> Optional[str]:
        """Extract configuration ID from filename"""
        return filename.replace('.json', '')
    
    def analyzePerformance(self) -> Dict[str, ConfigurationAnalysis]:
        """Analyze performance stability and trends for all configurations"""
        logger.info('🔍 Analyzing performance stability...')
        
        analysis = {}
        
        for config_id, history_data in self.history_data.items():
            if not isinstance(history_data, list) or len(history_data) < self.stability_threshold:
                logger.warning(f'⚠️  Insufficient data for {config_id}: {len(history_data) if isinstance(history_data, list) else 0} runs')
                continue
            
            config_analysis = self.analyzeConfiguration(config_id, history_data)
            analysis[config_id] = config_analysis
            
            if config_analysis.promotion_recommended:
                logger.info(f'✅ {config_id}: Promotion recommended - {config_analysis.rationale}')
            else:
                logger.info(f'⏳ {config_id}: Promotion not ready - {config_analysis.rationale}')
        
        return analysis
    
    def analyzeConfiguration(self, config_id: str, history_data: List[Dict]) -> ConfigurationAnalysis:
        """Analyze a single configuration for baseline promotion readiness"""
        # Sort by timestamp and get recent runs
        sorted_data = sorted(history_data, key=lambda x: x.get('timestamp', ''))
        recent_data = sorted_data[-self.stability_threshold:]
        
        # Extract metrics
        metrics = {}
        baseline_metrics = self.current_baseline.get('baselines', {}).get(config_id, {}).get('metrics', {})
        
        # Analyze each metric type
        metric_types = ['latency_p50', 'latency_p95', 'latency_p99', 'throughput', 'memory_usage', 'cpu_usage']
        
        for metric_type in metric_types:
            metric_analysis = self.analyzeMetric(metric_type, recent_data, baseline_metrics.get(metric_type))
            if metric_analysis:
                metrics[metric_type] = metric_analysis
        
        # Determine overall status and promotion recommendation
        overall_status, promotion_recommended, rationale = self.evaluatePromotion(metrics, recent_data)
        
        return ConfigurationAnalysis(
            config_id=config_id,
            config_name=self.formatConfigName(config_id),
            total_runs=len(history_data),
            stable_runs=len(recent_data),
            metrics=metrics,
            overall_status=overall_status,
            promotion_recommended=promotion_recommended,
            rationale=rationale
        )
    
    def analyzeMetric(self, metric_name: str, recent_data: List[Dict], baseline_value: Optional[float]) -> Optional[PerformanceMetric]:
        """Analyze a single metric for stability and improvement"""
        # Extract metric values
        values = []
        for entry in recent_data:
            if 'metrics' in entry and metric_name in entry['metrics']:
                value = entry['metrics'][metric_name]
                if value is not None:
                    values.append(float(value))
        
        if len(values) < 3:  # Need at least 3 values for meaningful analysis
            return None
        
        # Calculate statistics
        current_value = values[-1]
        mean_value = statistics.mean(values)
        std_dev = statistics.stdev(values) if len(values) > 1 else 0.0
        
        # Determine trend
        trend = self.calculateTrend(values)
        
        # Compare with baseline
        if baseline_value is not None:
            change_percent = ((current_value - baseline_value) / baseline_value) * 100
            baseline_value = float(baseline_value)
        else:
            change_percent = 0.0
            baseline_value = current_value
        
        # Determine status
        status = self.determineMetricStatus(current_value, mean_value, std_dev, baseline_value)
        
        return PerformanceMetric(
            name=metric_name,
            current_value=current_value,
            baseline_value=baseline_value,
            history_values=values,
            mean=mean_value,
            std_dev=std_dev,
            trend=trend,
            change_percent=change_percent,
            status=status
        )
    
    def calculateTrend(self, values: List[float]) -> str:
        """Calculate trend direction for a series of values"""
        if len(values) < 2:
            return 'stable'
        
        # Use linear regression slope
        n = len(values)
        x = list(range(n))
        
        sum_x = sum(x)
        sum_y = sum(values)
        sum_xy = sum(x[i] * values[i] for i in range(n))
        sum_x2 = sum(x[i] ** 2 for i in range(n))
        
        numerator = n * sum_xy - sum_x * sum_y
        denominator = n * sum_x2 - sum_x ** 2
        
        if denominator == 0:
            return 'stable'
        
        slope = numerator / denominator
        
        # Threshold for meaningful trend
        if abs(slope) < 0.1:
            return 'stable'
        elif slope > 0:
            return 'increasing'
        else:
            return 'decreasing'
    
    def determineMetricStatus(self, current: float, mean: float, std_dev: float, baseline: float) -> str:
        """Determine if a metric has improved, regressed, or is stable"""
        # Check for regression (worse than mean + 1σ)
        regression_threshold = mean + (self.regression_threshold * std_dev)
        
        if current > regression_threshold:
            return 'regressed'
        elif current < baseline * 0.95:  # 5% improvement threshold
            return 'improved'
        else:
            return 'stable'
    
    def evaluatePromotion(self, metrics: Dict[str, PerformanceMetric], recent_data: List[Dict]) -> Tuple[str, bool, str]:
        """Evaluate whether a configuration is ready for baseline promotion"""
        if not metrics:
            return 'insufficient_data', False, 'No metrics available for analysis'
        
        # Check for any regressions
        regressions = [m for m in metrics.values() if m.status == 'regressed']
        if regressions:
            return 'regressed', False, f'{len(regressions)} metrics show regression'
        
        # Check stability (most metrics should be stable or improved)
        stable_count = sum(1 for m in metrics.values() if m.status in ['stable', 'improved'])
        total_metrics = len(metrics)
        
        if stable_count < total_metrics * 0.8:  # 80% stability threshold
            return 'unstable', False, f'Only {stable_count}/{total_metrics} metrics are stable'
        
        # Check for meaningful improvements
        improvements = [m for m in metrics.values() if m.status == 'improved']
        if improvements:
            return 'ready', True, f'{len(improvements)} metrics improved, {stable_count - len(improvements)} stable'
        else:
            return 'stable', True, f'All {total_metrics} metrics stable, no regressions'
    
    def generateProposal(self, analysis: Dict[str, ConfigurationAnalysis]) -> BaselineProposal:
        """Generate a complete baseline promotion proposal"""
        logger.info('📋 Generating baseline proposal...')
        
        # Count configurations ready for promotion
        ready_configs = [c for c in analysis.values() if c.promotion_recommended]
        total_configs = len(analysis)
        
        # Determine overall promotion readiness
        promotion_ready = len(ready_configs) > 0 and len(ready_configs) >= total_configs * 0.5
        
        # Generate new baseline version
        current_version = self.current_baseline.get('version', '0.0.0')
        proposed_version = self.incrementVersion(current_version)
        
        # Generate changelog entry
        changelog_entry = self.generateChangelogEntry(analysis, proposed_version)
        
        # Create summary
        summary = {
            'total_configurations': total_configs,
            'ready_for_promotion': len(ready_configs),
            'stable_configurations': len([c for c in analysis.values() if c.overall_status == 'stable']),
            'improved_configurations': len([c for c in analysis.values() if c.overall_status == 'ready']),
            'regressed_configurations': len([c for c in analysis.values() if c.overall_status == 'regressed']),
            'promotion_recommended': promotion_ready
        }
        
        return BaselineProposal(
            generated_at=datetime.now(timezone.utc).isoformat(),
            current_baseline_version=current_version,
            proposed_baseline_version=proposed_version,
            configurations=analysis,
            summary=summary,
            promotion_ready=promotion_ready,
            changelog_entry=changelog_entry
        )
    
    def incrementVersion(self, current_version: str) -> str:
        """Increment version number for new baseline"""
        try:
            parts = current_version.split('.')
            if len(parts) >= 3:
                major, minor, patch = parts[0], parts[1], parts[2]
                new_patch = int(patch) + 1
                return f"{major}.{minor}.{new_patch}"
            else:
                return f"{current_version}.1"
        except:
            return f"{current_version}.1"
    
    def generateChangelogEntry(self, analysis: Dict[str, ConfigurationAnalysis], new_version: str) -> str:
        """Generate a changelog entry for the baseline update"""
        timestamp = datetime.now().strftime("%Y-%m-%d")
        
        changelog = f"## [{new_version}] - {timestamp}\n\n"
        changelog += "### Performance Baseline Update\n\n"
        
        # Summary
        ready_count = len([c for c in analysis.values() if c.promotion_recommended])
        total_count = len(analysis)
        
        changelog += f"- **Total Configurations**: {total_count}\n"
        changelog += f"- **Ready for Promotion**: {ready_count}\n"
        changelog += f"- **Status**: {'Ready' if ready_count > 0 else 'Not Ready'}\n\n"
        
        # Configuration details
        if ready_count > 0:
            changelog += "### Promoted Configurations\n\n"
            for config in analysis.values():
                if config.promotion_recommended:
                    changelog += f"- **{config.config_name}** ({config.config_id})\n"
                    changelog += f"  - {config.rationale}\n"
                    changelog += f"  - Stable runs: {config.stable_runs}/{config.total_runs}\n"
                    
                    # Metric improvements
                    improvements = [m for m in config.metrics.values() if m.status == 'improved']
                    if improvements:
                        changelog += f"  - Improved metrics: {', '.join(m.name for m in improvements)}\n"
                    changelog += "\n"
        
        # Metrics summary
        changelog += "### Metrics Summary\n\n"
        all_metrics = {}
        for config in analysis.values():
            for metric_name, metric in config.metrics.items():
                if metric_name not in all_metrics:
                    all_metrics[metric_name] = []
                all_metrics[metric_name].append(metric)
        
        for metric_name, metrics in all_metrics.items():
            improved = len([m for m in metrics if m.status == 'improved'])
            stable = len([m for m in metrics if m.status == 'stable'])
            regressed = len([m for m in metrics if m.status == 'regressed'])
            
            changelog += f"- **{metric_name}**: {improved} improved, {stable} stable, {regressed} regressed\n"
        
        return changelog
    
    def saveProposal(self, proposal: BaselineProposal):
        """Save the baseline proposal to output directory"""
        logger.info('💾 Saving baseline proposal...')
        
        # Ensure output directory exists
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # Save proposal JSON
        proposal_path = self.output_dir / 'baseline_proposal.json'
        with open(proposal_path, 'w', encoding='utf-8') as f:
            json.dump(asdict(proposal), f, indent=2, default=str)
        
        # Save proposed baseline
        proposed_baseline = self.generateProposedBaseline(proposal)
        baseline_path = self.output_dir / 'proposed_baseline.json'
        with open(baseline_path, 'w', encoding='utf-8') as f:
            json.dump(proposed_baseline, f, indent=2)
        
        logger.info(f'✅ Proposal saved to {self.output_dir}')
    
    def generateProposedBaseline(self, proposal: BaselineProposal) -> Dict[str, Any]:
        """Generate the proposed new baseline file"""
        proposed_baseline = {
            'version': proposal.proposed_baseline_version,
            'generated_at': proposal.generated_at,
            'promoted_from': proposal.current_baseline_version,
            'baselines': {}
        }
        
        # Generate new baseline values for each configuration
        for config_id, config_analysis in proposal.configurations.items():
            if config_analysis.promotion_recommended:
                # Use current values as new baseline
                new_baseline = {
                    'name': config_analysis.config_name,
                    'promoted_at': proposal.generated_at,
                    'metrics': {}
                }
                
                for metric_name, metric in config_analysis.metrics.items():
                    new_baseline['metrics'][metric_name] = {
                        'value': metric.current_value,
                        'mean': metric.mean,
                        'std_dev': metric.std_dev,
                        'trend': metric.trend,
                        'status': metric.status
                    }
                
                proposed_baseline['baselines'][config_id] = new_baseline
        
        return proposed_baseline
    
    def generateArtifacts(self, proposal: BaselineProposal):
        """Generate additional artifacts for the proposal"""
        logger.info('🎨 Generating proposal artifacts...')
        
        # Generate impact heatmap
        self.generateImpactHeatmap(proposal)
        
        # Generate summary report
        self.generateSummaryReport(proposal)
        
        # Generate approval checklist
        self.generateApprovalChecklist(proposal)
    
    def generateImpactHeatmap(self, proposal: BaselineProposal):
        """Generate an ASCII impact heatmap"""
        heatmap_path = self.output_dir / 'impact_heatmap.txt'
        
        with open(heatmap_path, 'w', encoding='utf-8') as f:
            f.write("Performance Impact Heatmap\n")
            f.write("=" * 50 + "\n\n")
            
            # Header
            f.write(f"{'Configuration':<25} {'Status':<12} {'Impact':<15}\n")
            f.write("-" * 50 + "\n")
            
            for config_id, config in proposal.configurations.items():
                status_icon = "✅" if config.promotion_recommended else "⏳"
                impact = "HIGH" if config.promotion_recommended else "LOW"
                
                f.write(f"{config.config_name:<25} {status_icon:<12} {impact:<15}\n")
                
                # Show metric details
                for metric_name, metric in config.metrics.items():
                    metric_icon = {
                        'improved': '🟢',
                        'stable': '🟡',
                        'regressed': '🔴'
                    }.get(metric.status, '⚪')
                    
                    f.write(f"  {metric_icon} {metric_name}: {metric.status} ({metric.change_percent:+.1f}%)\n")
                f.write("\n")
    
    def generateSummaryReport(self, proposal: BaselineProposal):
        """Generate a human-readable summary report"""
        report_path = self.output_dir / 'summary_report.md'
        
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write("# Baseline Promotion Summary Report\n\n")
            f.write(f"**Generated**: {proposal.generated_at}\n")
            f.write(f"**Current Version**: {proposal.current_baseline_version}\n")
            f.write(f"**Proposed Version**: {proposal.proposed_baseline_version}\n\n")
            
            f.write("## Executive Summary\n\n")
            f.write(f"- **Total Configurations**: {proposal.summary['total_configurations']}\n")
            f.write(f"- **Ready for Promotion**: {proposal.summary['ready_for_promotion']}\n")
            f.write(f"- **Overall Recommendation**: {'✅ PROMOTE' if proposal.promotion_ready else '⏳ WAIT'}\n\n")
            
            f.write("## Configuration Details\n\n")
            for config_id, config in proposal.configurations.items():
                f.write(f"### {config.config_name} (`{config_id}`)\n\n")
                f.write(f"- **Status**: {config.overall_status}\n")
                f.write(f"- **Promotion Ready**: {'Yes' if config.promotion_recommended else 'No'}\n")
                f.write(f"- **Rationale**: {config.rationale}\n")
                f.write(f"- **Stable Runs**: {config.stable_runs}/{config.total_runs}\n\n")
                
                f.write("**Metrics**:\n")
                for metric_name, metric in config.metrics.items():
                    f.write(f"- {metric_name}: {metric.status} ({metric.change_percent:+.1f}%)\n")
                f.write("\n")
    
    def generateApprovalChecklist(self, proposal: BaselineProposal):
        """Generate an approval checklist for reviewers"""
        checklist_path = self.output_dir / 'approval_checklist.md'
        
        with open(checklist_path, 'w', encoding='utf-8') as f:
            f.write("# Baseline Promotion Approval Checklist\n\n")
            f.write("## Pre-Approval Checks\n\n")
            f.write("- [ ] All configurations have sufficient stable runs (≥5)\n")
            f.write("- [ ] No metrics show regression beyond threshold\n")
            f.write("- [ ] Performance improvements are statistically significant\n")
            f.write("- [ ] Baseline changes are documented in changelog\n\n")
            
            f.write("## Configuration Review\n\n")
            for config_id, config in proposal.configurations.items():
                if config.promotion_recommended:
                    f.write(f"### {config.config_name}\n")
                    f.write(f"- [ ] Review {config.stable_runs} stable runs\n")
                    f.write(f"- [ ] Verify {config.rationale}\n")
                    f.write(f"- [ ] Check metric improvements\n\n")
            
            f.write("## Post-Approval Actions\n\n")
            f.write("- [ ] Merge baseline promotion PR\n")
            f.write("- [ ] Verify new baseline is active in CI\n")
            f.write("- [ ] Update performance documentation\n")
            f.write("- [ ] Notify team of baseline change\n")
    
    def formatConfigName(self, config_id: str) -> str:
        """Format configuration ID for display"""
        return config_id.replace('_', ' ').replace('-', ' ').title()

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Baseline Promotion Tool for Polymera OS")
    parser.add_argument("--history-dir", default="perf/history", help="Directory containing performance history")
    parser.add_argument("--baseline-file", default="perf/baselines/p2.json", help="Current baseline file")
    parser.add_argument("--output-dir", default="perf/baseline_proposal", help="Output directory for proposal")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Validate inputs
    if not os.path.exists(args.history_dir):
        logger.error(f"❌ History directory not found: {args.history_dir}")
        sys.exit(1)
    
    # Create promoter and run
    promoter = BaselinePromoter(args.history_dir, args.baseline_file, args.output_dir)
    
    try:
        proposal = promoter.run()
        
        # Print summary
        print("\n" + "="*60)
        print("BASELINE PROMOTION ANALYSIS COMPLETE")
        print("="*60)
        print(f"Total Configurations: {proposal.summary['total_configurations']}")
        print(f"Ready for Promotion: {proposal.summary['ready_for_promotion']}")
        print(f"Recommendation: {'✅ PROMOTE' if proposal.promotion_ready else '⏳ WAIT'}")
        print(f"Output Directory: {args.output_dir}")
        print("="*60)
        
        if proposal.promotion_ready:
            print("🚀 Baseline promotion is ready!")
            print("Review the generated files and create a promotion PR.")
        else:
            print("⏳ Baseline promotion is not ready yet.")
            print("Review the analysis to understand what needs to be addressed.")
        
    except Exception as error:
        logger.error(f"❌ Baseline promotion failed: {error}")
        sys.exit(1)

if __name__ == "__main__":
    main()
