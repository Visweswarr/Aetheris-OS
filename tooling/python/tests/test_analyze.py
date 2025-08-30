#!/usr/bin/env python3
"""
Tests for NGFS Performance Analyzer

This module tests the performance analyzer's baseline validation,
variance checking, and history writing functionality.
"""

import json
import tempfile
from pathlib import Path
import pytest
import sys

# Add parent directory to path for imports
sys.path.append(str(Path(__file__).parent.parent))
from ngfs_perf_analyze import NgfsPerfAnalyzer


class TestNgfsPerfAnalyzer:
    """Test NGFS Performance Analyzer functionality"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.analyzer = NgfsPerfAnalyzer()
        
        # Create temporary directory for test files
        self.temp_dir = tempfile.mkdtemp()
        self.temp_path = Path(self.temp_dir)
        
        # Sample benchmark data
        self.sample_bench_data = {
            "test": "ngfs_bench",
            "timestamp": 1640995200,
            "duration_ms": 1500,
            "cases": {
                "resolve_path": {
                    "p50": 150,
                    "p95": 400,
                    "p99": 600,
                    "mean": 180,
                    "stdev": 45,
                    "n": 50,
                    "errors": 0
                },
                "stat_file": {
                    "p50": 80,
                    "p95": 250,
                    "p99": 350,
                    "mean": 95,
                    "stdev": 30,
                    "n": 50,
                    "errors": 0
                },
                "read_small": {
                    "p50": 200,
                    "p95": 600,
                    "p99": 800,
                    "mean": 220,
                    "stdev": 55,
                    "n": 50,
                    "errors": 0
                },
                "read_large": {
                    "p50": 800,
                    "p95": 1500,
                    "p99": 2000,
                    "mean": 850,
                    "stdev": 120,
                    "n": 50,
                    "errors": 0
                },
                "snapshot_build": {
                    "p50": 25,
                    "p95": 50,
                    "p99": 75,
                    "mean": 28,
                    "stdev": 8,
                    "n": 50,
                    "errors": 0
                }
            },
            "errors": 0,
            "mismatch": 0
        }
        
        # Sample baseline data
        self.sample_baseline = {
            "version": 1,
            "budgets": {
                "resolve_path": {
                    "p95_max": 400,
                    "variance_limit": 0.35
                },
                "stat_file": {
                    "p95_max": 250,
                    "variance_limit": 0.35
                },
                "read_small": {
                    "p95_max": 600,
                    "variance_limit": 0.25
                },
                "read_large": {
                    "p95_max": 1500,
                    "variance_limit": 0.25
                },
                "snapshot_build": {
                    "p95_max": 50,
                    "variance_limit": 0.35
                }
            }
        }
    
    def test_load_baseline_success(self):
        """Test successful baseline loading"""
        baseline_file = self.temp_path / "baseline.json"
        with open(baseline_file, 'w') as f:
            json.dump(self.sample_baseline, f)
        
        assert self.analyzer.load_baseline(baseline_file)
        assert self.analyzer.baseline is not None
    
    def test_load_baseline_failure(self):
        """Test baseline loading failure"""
        non_existent_file = self.temp_path / "nonexistent.json"
        assert not self.analyzer.load_baseline(non_existent_file)
        assert len(self.analyzer.errors) > 0
    
    def test_load_bench_results_success(self):
        """Test successful benchmark results loading"""
        bench_file = self.temp_path / "bench.json"
        with open(bench_file, 'w') as f:
            json.dump(self.sample_bench_data, f)
        
        result = self.analyzer.load_bench_results(bench_file)
        assert result is not None
        assert result["test"] == "ngfs_bench"
    
    def test_load_bench_results_failure(self):
        """Test benchmark results loading failure"""
        non_existent_file = self.temp_path / "nonexistent.json"
        result = self.analyzer.load_bench_results(non_existent_file)
        assert result is None
        assert len(self.analyzer.errors) > 0
    
    def test_validate_statistical_sanity_success(self):
        """Test successful statistical sanity validation"""
        assert self.analyzer.validate_statistical_sanity(self.sample_bench_data)
        assert len(self.analyzer.errors) == 0
    
    def test_validate_statistical_sanity_missing_cases(self):
        """Test validation failure with missing cases"""
        invalid_data = self.sample_bench_data.copy()
        del invalid_data["cases"]
        
        assert not self.analyzer.validate_statistical_sanity(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_validate_statistical_sanity_missing_case(self):
        """Test validation failure with missing required case"""
        invalid_data = self.sample_bench_data.copy()
        del invalid_data["cases"]["resolve_path"]
        
        assert not self.analyzer.validate_statistical_sanity(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_validate_statistical_sanity_insufficient_samples(self):
        """Test validation failure with insufficient samples"""
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["resolve_path"]["n"] = 25
        
        assert not self.analyzer.validate_statistical_sanity(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_validate_statistical_sanity_errors_detected(self):
        """Test validation failure with errors detected"""
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["resolve_path"]["errors"] = 5
        
        assert not self.analyzer.validate_statistical_sanity(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_validate_statistical_sanity_high_variance_read(self):
        """Test validation failure with high variance in read operations"""
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["read_small"]["stdev"] = 200  # High variance
        
        assert not self.analyzer.validate_statistical_sanity(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_validate_statistical_sanity_high_variance_metadata(self):
        """Test validation failure with high variance in metadata operations"""
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["resolve_path"]["stdev"] = 100  # High variance
        
        assert not self.analyzer.validate_statistical_sanity(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_check_baseline_compliance_success(self):
        """Test successful baseline compliance check"""
        self.analyzer.baseline = self.sample_baseline
        assert self.analyzer.check_baseline_compliance(self.sample_bench_data)
        assert len(self.analyzer.errors) == 0
    
    def test_check_baseline_compliance_p95_exceeded(self):
        """Test baseline compliance failure with p95 exceeded"""
        self.analyzer.baseline = self.sample_baseline
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["resolve_path"]["p95"] = 500  # Exceeds 400 budget
        
        assert not self.analyzer.check_baseline_compliance(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_check_baseline_compliance_variance_exceeded(self):
        """Test baseline compliance failure with variance exceeded"""
        self.analyzer.baseline = self.sample_baseline
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["read_small"]["stdev"] = 200  # High variance
        
        assert not self.analyzer.check_baseline_compliance(invalid_data)
        assert len(self.analyzer.errors) > 0
    
    def test_check_baseline_compliance_no_baseline(self):
        """Test baseline compliance check without baseline"""
        self.analyzer.baseline = None
        assert self.analyzer.check_baseline_compliance(self.sample_bench_data)
        assert len(self.analyzer.warnings) > 0
    
    def test_write_history_entry_success(self):
        """Test successful history entry writing"""
        history_file = self.temp_path / "history.jsonl"
        assert self.analyzer.write_history_entry(self.sample_bench_data, history_file)
        
        # Verify file was created and contains entry
        assert history_file.exists()
        with open(history_file, 'r') as f:
            lines = f.readlines()
            assert len(lines) == 1
            
            entry = json.loads(lines[0])
            assert entry["test"] == "ngfs_bench"
            assert entry["duration_ms"] == 1500
    
    def test_write_history_entry_directory_creation(self):
        """Test history entry writing with directory creation"""
        history_file = self.temp_path / "nested" / "history.jsonl"
        assert self.analyzer.write_history_entry(self.sample_bench_data, history_file)
        assert history_file.exists()
    
    def test_analyze_performance_success(self):
        """Test successful complete performance analysis"""
        bench_file = self.temp_path / "bench.json"
        baseline_file = self.temp_path / "baseline.json"
        history_file = self.temp_path / "history.jsonl"
        
        with open(bench_file, 'w') as f:
            json.dump(self.sample_bench_data, f)
        
        with open(baseline_file, 'w') as f:
            json.dump(self.sample_baseline, f)
        
        assert self.analyzer.analyze_performance(bench_file, baseline_file, history_file)
        assert len(self.analyzer.errors) == 0
    
    def test_analyze_performance_no_baseline(self):
        """Test performance analysis without baseline"""
        bench_file = self.temp_path / "bench.json"
        history_file = self.temp_path / "history.jsonl"
        
        with open(bench_file, 'w') as f:
            json.dump(self.sample_bench_data, f)
        
        assert self.analyzer.analyze_performance(bench_file, None, history_file)
        assert len(self.analyzer.errors) == 0
    
    def test_analyze_performance_no_history(self):
        """Test performance analysis without history"""
        bench_file = self.temp_path / "bench.json"
        baseline_file = self.temp_path / "baseline.json"
        
        with open(bench_file, 'w') as f:
            json.dump(self.sample_bench_data, f)
        
        with open(baseline_file, 'w') as f:
            json.dump(self.sample_baseline, f)
        
        assert self.analyzer.analyze_performance(bench_file, baseline_file, None)
        assert len(self.analyzer.errors) == 0
    
    def test_analyze_performance_validation_failure(self):
        """Test performance analysis with validation failure"""
        invalid_data = self.sample_bench_data.copy()
        invalid_data["cases"]["resolve_path"]["n"] = 25  # Insufficient samples
        
        bench_file = self.temp_path / "bench.json"
        with open(bench_file, 'w') as f:
            json.dump(invalid_data, f)
        
        assert not self.analyzer.analyze_performance(bench_file, None, None)
        assert len(self.analyzer.errors) > 0
    
    def test_print_report_success(self):
        """Test successful report printing"""
        assert self.analyzer.print_report()
    
    def test_print_report_with_errors(self):
        """Test report printing with errors"""
        self.analyzer.errors.append("Test error")
        assert not self.analyzer.print_report()
    
    def test_print_report_with_warnings(self):
        """Test report printing with warnings only"""
        self.analyzer.warnings.append("Test warning")
        assert self.analyzer.print_report()
    
    def teardown_method(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir)


class TestNgfsPerfAnalyzerEdgeCases:
    """Test edge cases and error conditions"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.analyzer = NgfsPerfAnalyzer()
        self.temp_dir = tempfile.mkdtemp()
        self.temp_path = Path(self.temp_dir)
    
    def test_empty_bench_data(self):
        """Test handling of empty benchmark data"""
        empty_data = {}
        assert not self.analyzer.validate_statistical_sanity(empty_data)
        assert len(self.analyzer.errors) > 0
    
    def test_malformed_bench_data(self):
        """Test handling of malformed benchmark data"""
        malformed_data = {
            "test": "ngfs_bench",
            "cases": "not_an_object"
        }
        assert not self.analyzer.validate_statistical_sanity(malformed_data)
        assert len(self.analyzer.errors) > 0
    
    def test_missing_required_fields(self):
        """Test handling of missing required fields in cases"""
        incomplete_data = {
            "test": "ngfs_bench",
            "cases": {
                "resolve_path": {
                    "p50": 150,
                    # Missing other required fields
                }
            }
        }
        assert not self.analyzer.validate_statistical_sanity(incomplete_data)
        assert len(self.analyzer.errors) > 0
    
    def test_invalid_field_types(self):
        """Test handling of invalid field types"""
        invalid_types_data = {
            "test": "ngfs_bench",
            "cases": {
                "resolve_path": {
                    "p50": "not_a_number",
                    "p95": 400,
                    "p99": 600,
                    "mean": 180,
                    "stdev": 45,
                    "n": 50,
                    "errors": 0
                }
            }
        }
        assert not self.analyzer.validate_statistical_sanity(invalid_types_data)
        assert len(self.analyzer.errors) > 0
    
    def test_negative_values(self):
        """Test handling of negative values"""
        negative_data = {
            "test": "ngfs_bench",
            "cases": {
                "resolve_path": {
                    "p50": -150,
                    "p95": 400,
                    "p99": 600,
                    "mean": 180,
                    "stdev": 45,
                    "n": 50,
                    "errors": 0
                }
            }
        }
        assert not self.analyzer.validate_statistical_sanity(negative_data)
        assert len(self.analyzer.errors) > 0
    
    def test_zero_mean_variance_calculation(self):
        """Test variance calculation with zero mean"""
        zero_mean_data = {
            "test": "ngfs_bench",
            "cases": {
                "resolve_path": {
                    "p50": 0,
                    "p95": 0,
                    "p99": 0,
                    "mean": 0,
                    "stdev": 0,
                    "n": 50,
                    "errors": 0
                }
            }
        }
        # Should handle zero mean gracefully
        assert self.analyzer.validate_statistical_sanity(zero_mean_data)
    
    def teardown_method(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
