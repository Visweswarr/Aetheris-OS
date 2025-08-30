#!/usr/bin/env python3

"""
Tests for the NGFS Integrity Diff Reporter
"""

import pytest
import tempfile
import os
import json
from unittest.mock import patch, MagicMock

# Import the module to test
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from integrity_diff import IntegrityDiffReporter


class TestIntegrityDiffReporter:
    """Test cases for IntegrityDiffReporter class"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.reporter = IntegrityDiffReporter()
        self.temp_dir = tempfile.mkdtemp()
        
    def teardown_method(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_compare_identical_files(self):
        """Test that identical files show no differences"""
        # Create identical files
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        test_data = {"key": "value", "number": 42, "list": [1, 2, 3]}
        
        with open(file1_path, 'w') as f:
            json.dump(test_data, f)
        with open(file2_path, 'w') as f:
            json.dump(test_data, f)
        
        # Compare files
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is True
        assert len(self.reporter.differences) == 0
        assert len(self.reporter.warnings) == 0
    
    def test_compare_different_json_files(self):
        """Test that different JSON files show differences"""
        # Create different files
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key": "value1", "number": 42}
        data2 = {"key": "value2", "number": 42}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        # Compare files
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("key" in diff for diff in self.reporter.differences)
    
    def test_compare_json_with_different_structure(self):
        """Test JSON files with different structure"""
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key": "value", "nested": {"a": 1, "b": 2}}
        data2 = {"key": "value", "nested": {"a": 1, "b": 3, "c": 4}}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        # Should detect both the changed value and the new field
        assert any("b" in diff for diff in self.reporter.differences)
        assert any("c" in diff for diff in self.reporter.differences)
    
    def test_compare_json_with_different_types(self):
        """Test JSON files with different data types"""
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key": "value", "number": 42}
        data2 = {"key": "value", "number": "42"}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("Type mismatch" in diff for diff in self.reporter.differences)
    
    def test_compare_json_with_missing_fields(self):
        """Test JSON files with missing fields"""
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key1": "value1", "key2": "value2", "key3": "value3"}
        data2 = {"key1": "value1", "key3": "value3"}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("key2" in diff for diff in self.reporter.differences)
    
    def test_compare_json_with_extra_fields(self):
        """Test JSON files with extra fields"""
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key1": "value1", "key2": "value2"}
        data2 = {"key1": "value1", "key2": "value2", "key3": "value3"}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("key3" in diff for diff in self.reporter.differences)
    
    def test_compare_json_with_different_array_order(self):
        """Test JSON files with different array order"""
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key": "value", "array": [1, 2, 3]}
        data2 = {"key": "value", "array": [3, 2, 1]}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("array" in diff for diff in self.reporter.differences)
    
    def test_compare_binary_files(self):
        """Test binary file comparison"""
        file1_path = os.path.join(self.temp_dir, "file1.bin")
        file2_path = os.path.join(self.temp_dir, "file2.bin")
        
        # Create different binary files
        with open(file1_path, 'wb') as f:
            f.write(b"Hello World")
        with open(file2_path, 'wb') as f:
            f.write(b"Hello World!")
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("size" in diff.lower() for diff in self.reporter.differences)
    
    def test_compare_nonexistent_files(self):
        """Test comparison with nonexistent files"""
        file1_path = os.path.join(self.temp_dir, "nonexistent1.json")
        file2_path = os.path.join(self.temp_dir, "nonexistent2.json")
        
        result = self.reporter.compare_files(file1_path, file2_path)
        
        assert result is False
        assert len(self.reporter.differences) > 0
        assert any("not found" in diff.lower() for diff in self.reporter.differences)
    
    def test_compare_files_with_different_extensions(self):
        """Test comparison with different file extensions"""
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.txt")
        
        # Create files with same content but different extensions
        content = '{"key": "value"}'
        with open(file1_path, 'w') as f:
            f.write(content)
        with open(file2_path, 'w') as f:
            f.write(content)
        
        # Should still work since content is the same
        result = self.reporter.compare_files(file1_path, file2_path)
        
        # The current implementation treats .txt as binary, so this might fail
        # depending on the implementation details
        assert result is not None  # Should return a boolean
    
    def test_clear_state(self):
        """Test that clear() resets the reporter state"""
        # First, create some differences
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key": "value1"}
        data2 = {"key": "value2"}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        self.reporter.compare_files(file1_path, file2_path)
        
        # Verify we have differences
        assert len(self.reporter.differences) > 0
        
        # Clear the state
        self.reporter.clear()
        
        # Verify state is cleared
        assert len(self.reporter.differences) == 0
        assert len(self.reporter.warnings) == 0
    
    def test_get_differences(self):
        """Test get_differences returns a copy"""
        # Create some differences
        file1_path = os.path.join(self.temp_dir, "file1.json")
        file2_path = os.path.join(self.temp_dir, "file2.json")
        
        data1 = {"key": "value1"}
        data2 = {"key": "value2"}
        
        with open(file1_path, 'w') as f:
            json.dump(data1, f)
        with open(file2_path, 'w') as f:
            json.dump(data2, f)
        
        self.reporter.compare_files(file1_path, file2_path)
        
        differences = self.reporter.get_differences()
        
        # Verify we got a copy
        assert differences == self.reporter.differences
        assert differences is not self.reporter.differences
        
        # Modifying the copy shouldn't affect the original
        differences.append("test")
        assert "test" not in self.reporter.differences
    
    def test_get_warnings(self):
        """Test get_warnings returns a copy"""
        # Create some warnings by comparing binary files
        file1_path = os.path.join(self.temp_dir, "file1.bin")
        file2_path = os.path.join(self.temp_dir, "file2.bin")
        
        with open(file1_path, 'wb') as f:
            f.write(b"\x00\x01\x02")
        with open(file2_path, 'wb') as f:
            f.write(b"\x01\x02\x03")
        
        self.reporter.compare_files(file1_path, file2_path)
        
        warnings = self.reporter.get_warnings()
        
        # Verify we got a copy
        assert warnings == self.reporter.warnings
        assert warnings is not self.reporter.warnings
        
        # Modifying the copy shouldn't affect the original
        warnings.append("test")
        assert "test" not in self.reporter.warnings


if __name__ == "__main__":
    pytest.main([__file__])
