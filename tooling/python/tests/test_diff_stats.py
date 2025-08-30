#!/usr/bin/env python3
"""
Tests for NGFS diff stats analyzer
"""

import pytest
import tempfile
import os
import json
from unittest.mock import patch, MagicMock

from ngfs_diff_stats import NgfsDiffAnalyzer, DiffStats, TimelineEntry


class TestNgfsDiffAnalyzer:
    """Test the NGFS diff analyzer class"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.analyzer = NgfsDiffAnalyzer()
        self.sample_diff = {
            "version": 1,
            "timestamp": "2024-01-01T12:00:00Z",
            "old_snapshot": b"old_cid_123",
            "new_snapshot": b"new_cid_456",
            "added": [
                {
                    "path": "/file1.txt",
                    "cid_ngfs": b"cid_1",
                    "size": 1024,
                    "type": 1,  # File
                    "mode": 0o644,
                    "mtime": 1640995200
                },
                {
                    "path": "/dir1",
                    "cid_ngfs": b"cid_2",
                    "size": 0,
                    "type": 2,  # Directory
                    "mode": 0o755,
                    "mtime": 1640995200
                }
            ],
            "removed": [
                {
                    "path": "/file2.txt",
                    "cid_ngfs": b"cid_3",
                    "size": 512,
                    "type": 1,  # File
                    "mode": 0o644,
                    "mtime": 1640995200
                }
            ],
            "modified": [
                {
                    "path": "/file3.txt",
                    "old_cid": b"cid_4",
                    "new_cid": b"cid_5",
                    "old_size": 256,
                    "new_size": 512,
                    "delta_size": 256,
                    "mode": 0o644,
                    "mtime": 1640995200
                }
            ],
            "summary": {
                "total_added": 2,
                "total_removed": 1,
                "total_modified": 1,
                "bytes_added": 1024,
                "bytes_removed": 512,
                "bytes_delta": 512
            }
        }
    
    def teardown_method(self):
        """Clean up after tests"""
        self.analyzer.clear()
    
    def test_init(self):
        """Test analyzer initialization"""
        assert self.analyzer.diffs == []
        assert self.analyzer.timeline == []
    
    def test_load_diff_valid(self):
        """Test loading a valid diff"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(self.sample_diff, f)
            temp_file = f.name
        
        try:
            result = self.analyzer.load_diff(temp_file)
            assert result is True
            assert len(self.analyzer.diffs) == 1
            assert self.analyzer.diffs[0] == self.sample_diff
        finally:
            os.unlink(temp_file)
    
    def test_load_diff_invalid_file(self):
        """Test loading a non-existent file"""
        result = self.analyzer.load_diff("/nonexistent/file.json")
        assert result is False
    
    def test_load_diff_invalid_json(self):
        """Test loading invalid JSON"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write("invalid json content")
            temp_file = f.name
        
        try:
            result = self.analyzer.load_diff(temp_file)
            assert result is False
        finally:
            os.unlink(temp_file)
    
    def test_validate_diff_structure_valid(self):
        """Test validation of valid diff structure"""
        result = self.analyzer.validate_diff_structure(self.sample_diff)
        assert result is True
    
    def test_validate_diff_structure_missing_fields(self):
        """Test validation of diff with missing required fields"""
        invalid_diff = self.sample_diff.copy()
        del invalid_diff["version"]
        
        result = self.analyzer.validate_diff_structure(invalid_diff)
        assert result is False
    
    def test_validate_diff_structure_invalid_version(self):
        """Test validation of diff with invalid version"""
        invalid_diff = self.sample_diff.copy()
        invalid_diff["version"] = 2
        
        result = self.analyzer.validate_diff_structure(invalid_diff)
        assert result is False
    
    def test_validate_diff_structure_invalid_summary(self):
        """Test validation of diff with inconsistent summary"""
        invalid_diff = self.sample_diff.copy()
        invalid_diff["summary"]["total_added"] = 999  # Inconsistent
        
        result = self.analyzer.validate_diff_structure(invalid_diff)
        assert result is False
    
    def test_analyze_diff(self):
        """Test diff analysis"""
        stats = self.analyzer.analyze_diff(self.sample_diff)
        
        assert isinstance(stats, DiffStats)
        assert stats.total_files == 3  # 2 added + 1 removed
        assert stats.total_directories == 1  # 1 added
        assert stats.total_symlinks == 0
        assert stats.bytes_added == 1024
        assert stats.bytes_removed == 512
        assert stats.bytes_modified == 256
        assert stats.net_bytes_change == 512
    
    def test_build_timeline(self):
        """Test timeline building"""
        # Load multiple diffs
        diff1 = self.sample_diff.copy()
        diff1["timestamp"] = "2024-01-01T10:00:00Z"
        
        diff2 = self.sample_diff.copy()
        diff2["timestamp"] = "2024-01-01T14:00:00Z"
        diff2["summary"]["total_added"] = 3
        
        self.analyzer.diffs = [diff1, diff2]
        self.analyzer.build_timeline()
        
        assert len(self.analyzer.timeline) == 2
        assert self.analyzer.timeline[0].timestamp == "2024-01-01T10:00:00Z"
        assert self.analyzer.timeline[1].timestamp == "2024-01-01T14:00:00Z"
    
    def test_print_summary_table(self, capsys):
        """Test summary table printing"""
        self.analyzer.diffs = [self.sample_diff]
        self.analyzer.print_summary_table(self.sample_diff)
        
        captured = capsys.readouterr()
        assert "NGFS Diff Summary" in captured.out
        assert "Added: 2" in captured.out
        assert "Removed: 1" in captured.out
        assert "Modified: 1" in captured.out
        assert "Bytes Added: 1.0 KB" in captured.out
        assert "Bytes Removed: 512 B" in captured.out
        assert "Net Change: +512 B" in captured.out
    
    def test_print_timeline_summary(self, capsys):
        """Test timeline summary printing"""
        # Build timeline first
        diff1 = self.sample_diff.copy()
        diff1["timestamp"] = "2024-01-01T10:00:00Z"
        
        diff2 = self.sample_diff.copy()
        diff2["timestamp"] = "2024-01-01T14:00:00Z"
        
        self.analyzer.diffs = [diff1, diff2]
        self.analyzer.build_timeline()
        
        self.analyzer.print_timeline_summary()
        
        captured = capsys.readouterr()
        assert "Timeline Summary" in captured.out
        assert "Total Diffs: 2" in captured.out
        assert "Total Files Changed: 6" in captured.out  # 2 diffs × 3 files each
    
    def test_merge_diffs_cbor(self):
        """Test merging diffs to CBOR format"""
        self.analyzer.diffs = [self.sample_diff]
        
        with tempfile.NamedTemporaryFile(suffix='.cbor', delete=False) as f:
            temp_file = f.name
        
        try:
            result = self.analyzer.merge_diffs(temp_file, 'cbor')
            assert result is True
            assert os.path.exists(temp_file)
            assert os.path.getsize(temp_file) > 0
        finally:
            os.unlink(temp_file)
    
    def test_merge_diffs_json(self):
        """Test merging diffs to JSON format"""
        self.analyzer.diffs = [self.sample_diff]
        
        with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
            temp_file = f.name
        
        try:
            result = self.analyzer.merge_diffs(temp_file, 'json')
            assert result is True
            assert os.path.exists(temp_file)
            
            # Verify JSON content
            with open(temp_file, 'r') as f:
                merged_data = json.load(f)
                assert "diffs" in merged_data
                assert len(merged_data["diffs"]) == 1
                assert merged_data["diffs"][0]["version"] == 1
        finally:
            os.unlink(temp_file)
    
    def test_merge_diffs_invalid_format(self):
        """Test merging diffs with invalid format"""
        self.analyzer.diffs = [self.sample_diff]
        
        with tempfile.NamedTemporaryFile(suffix='.txt', delete=False) as f:
            temp_file = f.name
        
        try:
            result = self.analyzer.merge_diffs(temp_file, 'invalid')
            assert result is False
        finally:
            os.unlink(temp_file)
    
    def test_calculate_merged_summary(self):
        """Test calculation of merged summary"""
        # Create multiple diffs with different content
        diff1 = self.sample_diff.copy()
        diff1["timestamp"] = "2024-01-01T10:00:00Z"
        
        diff2 = {
            "version": 1,
            "timestamp": "2024-01-01T14:00:00Z",
            "old_snapshot": b"old_cid_789",
            "new_snapshot": b"new_cid_012",
            "added": [
                {
                    "path": "/file4.txt",
                    "cid_ngfs": b"cid_6",
                    "size": 2048,
                    "type": 1,
                    "mode": 0o644,
                    "mtime": 1640995200
                }
            ],
            "removed": [],
            "modified": [],
            "summary": {
                "total_added": 1,
                "total_removed": 0,
                "total_modified": 0,
                "bytes_added": 2048,
                "bytes_removed": 0,
                "bytes_delta": 2048
            }
        }
        
        self.analyzer.diffs = [diff1, diff2]
        merged_summary = self.analyzer.calculate_merged_summary()
        
        assert merged_summary["total_diffs"] == 2
        assert merged_summary["total_files_added"] == 3  # 2 + 1
        assert merged_summary["total_files_removed"] == 1
        assert merged_summary["total_files_modified"] == 1
        assert merged_summary["total_bytes_added"] == 3072  # 1024 + 2048
        assert merged_summary["total_bytes_removed"] == 512
        assert merged_summary["total_bytes_modified"] == 256
    
    def test_export_csv(self):
        """Test CSV export functionality"""
        # Build timeline first
        diff1 = self.sample_diff.copy()
        diff1["timestamp"] = "2024-01-01T10:00:00Z"
        
        self.analyzer.diffs = [diff1]
        self.analyzer.build_timeline()
        
        with tempfile.NamedTemporaryFile(suffix='.csv', delete=False) as f:
            temp_file = f.name
        
        try:
            result = self.analyzer.export_csv(temp_file)
            assert result is True
            assert os.path.exists(temp_file)
            
            # Verify CSV content
            with open(temp_file, 'r') as f:
                content = f.read()
                assert "timestamp" in content
                assert "files_added" in content
                assert "files_removed" in content
                assert "files_modified" in content
                assert "bytes_added" in content
                assert "bytes_removed" in content
                assert "bytes_modified" in content
        finally:
            os.unlink(temp_file)
    
    def test_format_bytes(self):
        """Test byte formatting utility"""
        assert self.analyzer.format_bytes(0) == "0 B"
        assert self.analyzer.format_bytes(1024) == "1.0 KB"
        assert self.analyzer.format_bytes(1024 * 1024) == "1.0 MB"
        assert self.analyzer.format_bytes(1024 * 1024 * 1024) == "1.0 GB"
        assert self.analyzer.format_bytes(1500) == "1.5 KB"
        assert self.analyzer.format_bytes(512) == "512 B"
    
    def test_clear(self):
        """Test clearing analyzer state"""
        self.analyzer.diffs = [self.sample_diff]
        self.analyzer.build_timeline()
        
        assert len(self.analyzer.diffs) == 1
        assert len(self.analyzer.timeline) == 1
        
        self.analyzer.clear()
        
        assert len(self.analyzer.diffs) == 0
        assert len(self.analyzer.timeline) == 0


class TestDiffStats:
    """Test the DiffStats dataclass"""
    
    def test_diff_stats_creation(self):
        """Test creating DiffStats instance"""
        stats = DiffStats(
            total_files=10,
            total_directories=5,
            total_symlinks=2,
            bytes_added=1024,
            bytes_removed=512,
            bytes_modified=256,
            net_bytes_change=768
        )
        
        assert stats.total_files == 10
        assert stats.total_directories == 5
        assert stats.total_symlinks == 2
        assert stats.bytes_added == 1024
        assert stats.bytes_removed == 512
        assert stats.bytes_modified == 256
        assert stats.net_bytes_change == 768
    
    def test_diff_stats_defaults(self):
        """Test DiffStats with default values"""
        stats = DiffStats()
        
        assert stats.total_files == 0
        assert stats.total_directories == 0
        assert stats.total_symlinks == 0
        assert stats.bytes_added == 0
        assert stats.bytes_removed == 0
        assert stats.bytes_modified == 0
        assert stats.net_bytes_change == 0


class TestTimelineEntry:
    """Test the TimelineEntry dataclass"""
    
    def test_timeline_entry_creation(self):
        """Test creating TimelineEntry instance"""
        entry = TimelineEntry(
            timestamp="2024-01-01T12:00:00Z",
            files_added=5,
            files_removed=2,
            files_modified=1,
            bytes_added=2048,
            bytes_removed=1024,
            bytes_modified=512
        )
        
        assert entry.timestamp == "2024-01-01T12:00:00Z"
        assert entry.files_added == 5
        assert entry.files_removed == 2
        assert entry.files_modified == 1
        assert entry.bytes_added == 2048
        assert entry.bytes_removed == 1024
        assert entry.bytes_modified == 512
    
    def test_timeline_entry_defaults(self):
        """Test TimelineEntry with default values"""
        entry = TimelineEntry()
        
        assert entry.timestamp == ""
        assert entry.files_added == 0
        assert entry.files_removed == 0
        assert entry.files_modified == 0
        assert entry.bytes_added == 0
        assert entry.bytes_removed == 0
        assert entry.bytes_modified == 0


class TestIntegration:
    """Integration tests for the analyzer"""
    
    def test_full_workflow(self):
        """Test complete workflow from loading to analysis"""
        analyzer = NgfsDiffAnalyzer()
        
        # Create test diff
        test_diff = {
            "version": 1,
            "timestamp": "2024-01-01T12:00:00Z",
            "old_snapshot": b"old_cid",
            "new_snapshot": b"new_cid",
            "added": [
                {
                    "path": "/test.txt",
                    "cid_ngfs": b"test_cid",
                    "size": 1024,
                    "type": 1,
                    "mode": 0o644,
                    "mtime": 1640995200
                }
            ],
            "removed": [],
            "modified": [],
            "summary": {
                "total_added": 1,
                "total_removed": 0,
                "total_modified": 0,
                "bytes_added": 1024,
                "bytes_removed": 0,
                "bytes_delta": 1024
            }
        }
        
        # Add diff directly
        analyzer.diffs = [test_diff]
        
        # Build timeline
        analyzer.build_timeline()
        assert len(analyzer.timeline) == 1
        
        # Analyze
        stats = analyzer.analyze_diff(test_diff)
        assert stats.total_files == 1
        assert stats.bytes_added == 1024
        
        # Export
        with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
            temp_file = f.name
        
        try:
            result = analyzer.merge_diffs(temp_file, 'json')
            assert result is True
        finally:
            os.unlink(temp_file)
    
    def test_error_handling(self):
        """Test error handling in various scenarios"""
        analyzer = NgfsDiffAnalyzer()
        
        # Test with malformed diff
        malformed_diff = {
            "version": 1,
            "timestamp": "2024-01-01T12:00:00Z",
            # Missing required fields
        }
        
        # Should handle gracefully
        stats = analyzer.analyze_diff(malformed_diff)
        assert stats.total_files == 0  # Default values
        
        # Test with empty diff list
        analyzer.diffs = []
        analyzer.build_timeline()
        assert len(analyzer.timeline) == 0
        
        # Test with invalid file paths
        with tempfile.NamedTemporaryFile(suffix='.txt', delete=False) as f:
            temp_file = f.name
        
        try:
            result = analyzer.merge_diffs(temp_file, 'invalid_format')
            assert result is False
        finally:
            os.unlink(temp_file)
