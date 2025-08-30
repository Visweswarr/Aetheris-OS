#!/usr/bin/env python3
"""
Tests for ngfs_ls_verify.py - NGFS tree verification and NFC policy enforcement
"""

import os
import tempfile
import json
import pytest
from unittest.mock import patch, MagicMock
import sys
import unicodedata

# Add the parent directory to the path to import the module
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from ngfs_ls_verify import NgfsLsVerifier, check_nfc_policy


class TestNgfsLsVerifier:
    """Test cases for NgfsLsVerifier class"""
    
    def setup_method(self):
        """Set up test fixtures before each test method"""
        self.temp_dir = tempfile.mkdtemp()
        self.mount_point = os.path.join(self.temp_dir, "mnt")
        self.fixtures_dir = os.path.join(self.temp_dir, "fixtures")
        
        # Create test directories
        os.makedirs(self.mount_point, exist_ok=True)
        os.makedirs(self.fixtures_dir, exist_ok=True)
        
        # Create test files in mount point
        self.test_file = os.path.join(self.mount_point, "test_file.txt")
        with open(self.test_file, 'w') as f:
            f.write("test content")
        
        # Create test directory
        self.test_dir = os.path.join(self.mount_point, "test_dir")
        os.makedirs(self.test_dir, exist_ok=True)
        
        # Create nested file
        self.nested_file = os.path.join(self.test_dir, "nested.txt")
        with open(self.nested_file, 'w') as f:
            f.write("nested content")
        
        # Create fixture data
        self.fixture_data = {
            "test_file.txt": {
                "size": 12,
                "sha256": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
            },
            "test_dir": {
                "type": "directory",
                "children": ["nested.txt"]
            },
            "nested.txt": {
                "size": 15,
                "sha256": "b94a8fe5ccb19ba61c4c0873d391e987982fbbd4"
            }
        }
        
        with open(os.path.join(self.fixtures_dir, "fixtures.json"), 'w') as f:
            json.dump(self.fixture_data, f)
        
        self.verifier = NgfsLsVerifier(self.mount_point, self.fixtures_dir)
    
    def teardown_method(self):
        """Clean up test fixtures after each test method"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_init_valid_mount_point(self):
        """Test verifier initialization with valid mount point"""
        verifier = NgfsLsVerifier(self.mount_point, self.fixtures_dir)
        assert verifier.mount_point == self.mount_point
        assert verifier.fixtures_dir == self.fixtures_dir
        assert len(verifier.errors) == 0
        assert len(verifier.warnings) == 0
    
    def test_init_invalid_mount_point(self):
        """Test verifier initialization with invalid mount point"""
        verifier = NgfsLsVerifier("/nonexistent/mount", self.fixtures_dir)
        assert len(verifier.errors) > 0
        assert "does not exist" in verifier.errors[0]
    
    def test_verify_mount_point_exists(self):
        """Test mount point existence verification"""
        result = self.verifier.verify_mount_point()
        assert result is True
        assert len(self.verifier.errors) == 0
    
    def test_verify_mount_point_nonexistent(self):
        """Test mount point verification with nonexistent path"""
        verifier = NgfsLsVerifier("/nonexistent/mount", self.fixtures_dir)
        result = verifier.verify_mount_point()
        assert result is False
        assert len(verifier.errors) > 0
    
    def test_verify_mount_point_not_directory(self):
        """Test mount point verification with file instead of directory"""
        temp_file = tempfile.NamedTemporaryFile(delete=False)
        temp_file.close()
        
        verifier = NgfsLsVerifier(temp_file.name, self.fixtures_dir)
        result = verifier.verify_mount_point()
        assert result is False
        assert len(verifier.errors) > 0
        
        os.unlink(temp_file.name)
    
    def test_verify_mount_point_not_accessible(self):
        """Test mount point verification with inaccessible directory"""
        if os.name != 'nt':  # Skip on Windows
            # Create directory with no permissions
            no_access_dir = os.path.join(self.temp_dir, "no_access")
            os.makedirs(no_access_dir, mode=0o000)
            
            verifier = NgfsLsVerifier(no_access_dir, self.fixtures_dir)
            result = verifier.verify_mount_point()
            assert result is False
            assert len(verifier.errors) > 0
            
            # Restore permissions for cleanup
            os.chmod(no_access_dir, 0o755)
    
    def test_load_fixtures_valid(self):
        """Test loading valid fixture data"""
        result = self.verifier.load_fixtures()
        assert result is True
        assert len(self.verifier.errors) == 0
        assert "test_file.txt" in self.verifier.fixtures
    
    def test_load_fixtures_invalid_json(self):
        """Test loading invalid JSON fixture data"""
        # Create invalid JSON file
        invalid_json = os.path.join(self.fixtures_dir, "invalid.json")
        with open(invalid_json, 'w') as f:
            f.write("invalid json content")
        
        verifier = NgfsLsVerifier(self.mount_point, self.fixtures_dir)
        verifier.fixtures_file = invalid_json
        
        result = verifier.load_fixtures()
        assert result is False
        assert len(verifier.errors) > 0
    
    def test_load_fixtures_nonexistent(self):
        """Test loading fixtures from nonexistent file"""
        verifier = NgfsLsVerifier(self.mount_point, "/nonexistent/fixtures")
        result = verifier.load_fixtures()
        assert result is False
        assert len(verifier.errors) > 0
    
    def test_verify_file_contents_valid(self):
        """Test file content verification with valid data"""
        self.verifier.load_fixtures()
        result = self.verifier.verify_file_contents()
        assert result is True
        assert len(self.verifier.errors) == 0
    
    def test_verify_file_contents_size_mismatch(self):
        """Test file content verification with size mismatch"""
        # Modify fixture to have wrong size
        self.fixture_data["test_file.txt"]["size"] = 999
        with open(os.path.join(self.fixtures_dir, "fixtures.json"), 'w') as f:
            json.dump(self.fixture_data, f)
        
        self.verifier.load_fixtures()
        result = self.verifier.verify_file_contents()
        assert result is False
        assert len(self.verifier.errors) > 0
        assert "size mismatch" in str(self.verifier.errors)
    
    def test_verify_file_contents_sha256_mismatch(self):
        """Test file content verification with SHA256 mismatch"""
        # Modify fixture to have wrong hash
        self.fixture_data["test_file.txt"]["sha256"] = "wrong_hash"
        with open(os.path.join(self.fixtures_dir, "fixtures.json"), 'w') as f:
            json.dump(self.fixture_data, f)
        
        self.verifier.load_fixtures()
        result = self.verifier.verify_file_contents()
        assert result is False
        assert len(self.verifier.errors) > 0
        assert "hash mismatch" in str(self.verifier.errors)
    
    def test_verify_directory_structure_valid(self):
        """Test directory structure verification with valid data"""
        self.verifier.load_fixtures()
        result = self.verifier.verify_directory_structure()
        assert result is True
        assert len(self.verifier.errors) == 0
    
    def test_verify_directory_structure_missing_file(self):
        """Test directory structure verification with missing file"""
        # Remove a file from fixture
        del self.fixture_data["nested.txt"]
        with open(os.path.join(self.fixtures_dir, "fixtures.json"), 'w') as f:
            json.dump(self.fixture_data, f)
        
        self.verifier.load_fixtures()
        result = self.verifier.verify_directory_structure()
        assert result is False
        assert len(self.verifier.errors) > 0
    
    def test_verify_directory_structure_extra_file(self):
        """Test directory structure verification with extra file"""
        # Add extra file to fixture
        self.fixture_data["extra_file.txt"] = {"size": 10, "sha256": "extra_hash"}
        with open(os.path.join(self.fixtures_dir, "fixtures.json"), 'w') as f:
            json.dump(self.fixture_data, f)
        
        self.verifier.load_fixtures()
        result = self.verifier.verify_directory_structure()
        assert result is False
        assert len(self.verifier.errors) > 0
    
    def test_run_verification_success(self):
        """Test complete verification run with success"""
        result = self.verifier.run_verification()
        assert result is True
        assert len(self.verifier.errors) == 0
    
    def test_run_verification_failure(self):
        """Test complete verification run with failure"""
        # Create invalid mount point
        verifier = NgfsLsVerifier("/nonexistent/mount", self.fixtures_dir)
        result = verifier.run_verification()
        assert result is False
        assert len(verifier.errors) > 0
    
    def test_generate_report_success(self):
        """Test report generation for successful verification"""
        self.verifier.run_verification()
        report = self.verifier.generate_report()
        assert "SUCCESS" in report
        assert "0 errors" in report
        assert "0 warnings" in report
    
    def test_generate_report_failure(self):
        """Test report generation for failed verification"""
        verifier = NgfsLsVerifier("/nonexistent/mount", self.fixtures_dir)
        verifier.run_verification()
        report = verifier.generate_report()
        assert "FAILED" in report
        assert "errors" in report
    
    def test_generate_json_output(self):
        """Test JSON output generation"""
        self.verifier.run_verification()
        json_output = self.verifier.generate_json_output()
        
        # Parse JSON to ensure it's valid
        data = json.loads(json_output)
        assert "success" in data
        assert "errors" in data
        assert "warnings" in data
        assert "files_checked" in data
        assert data["success"] is True
    
    def test_clear_state(self):
        """Test clearing verifier state"""
        self.verifier.errors.append("test error")
        self.verifier.warnings.append("test warning")
        
        self.verifier.clear()
        assert len(self.verifier.errors) == 0
        assert len(self.verifier.warnings) == 0
    
    def test_get_errors(self):
        """Test getting errors list"""
        self.verifier.errors = ["error1", "error2"]
        errors = self.verifier.get_errors()
        assert errors == ["error1", "error2"]
        assert errors is not self.verifier.errors  # Should be a copy
    
    def test_get_warnings(self):
        """Test getting warnings list"""
        self.verifier.warnings = ["warning1", "warning2"]
        warnings = self.verifier.get_warnings()
        assert warnings == ["warning1", "warning2"]
        assert warnings is not self.verifier.warnings  # Should be a copy


class TestNfcPolicy:
    """Test cases for NFC policy enforcement"""
    
    def test_valid_filenames(self):
        """Test valid filenames pass NFC policy"""
        valid_names = [
            "normal_file.txt",
            "file_with_underscores",
            "file-with-dashes",
            "file123",
            "file_123",
            "file-123",
            "file_123.txt",
            "file-123.txt",
            "file_123-456.txt",
            "file_123_456.txt"
        ]
        
        for name in valid_names:
            assert check_nfc_policy(name) is True, f"Valid filename failed: {name}"
    
    def test_control_characters_rejected(self):
        """Test filenames with control characters are rejected"""
        control_chars = [chr(i) for i in range(32)] + [chr(127)]
        
        for char in control_chars:
            filename = f"file{char}name.txt"
            result = check_nfc_policy(filename)
            assert result is False, f"Control character {ord(char)} should be rejected"
    
    def test_dotdot_sequences_rejected(self):
        """Test filenames with dotdot sequences are rejected"""
        dotdot_names = [
            "..",
            "file..txt",
            "..file.txt",
            "file..txt",
            "file...txt",
            "..file..txt",
            "file..file.txt"
        ]
        
        for name in dotdot_names:
            result = check_nfc_policy(name)
            assert result is False, f"Dotdot sequence should be rejected: {name}"
    
    def test_leading_trailing_dots_rejected(self):
        """Test filenames with leading or trailing dots are rejected"""
        dot_names = [
            ".file.txt",
            "file.txt.",
            ".file",
            "file.",
            ".",
            "..",
            "..."
        ]
        
        for name in dot_names:
            result = check_nfc_policy(name)
            assert result is False, f"Leading/trailing dots should be rejected: {name}"
    
    def test_empty_filenames_rejected(self):
        """Test empty filenames are rejected"""
        empty_names = ["", "   ", "\t", "\n", "\r"]
        
        for name in empty_names:
            result = check_nfc_policy(name)
            assert result is False, f"Empty filename should be rejected: '{name}'"
    
    def test_unicode_normalization(self):
        """Test Unicode normalization is enforced"""
        # Test combining characters that should be normalized
        combining_chars = [
            ("e\u0301", "é"),  # e + combining acute accent
            ("a\u0308", "ä"),  # a + combining diaeresis
            ("n\u0303", "ñ"),  # n + combining tilde
        ]
        
        for unnormalized, normalized in combining_chars:
            filename = f"file{unnormalized}.txt"
            result = check_nfc_policy(filename)
            # Should pass but generate warning about normalization
            assert result is True, f"Unnormalized filename should pass: {filename}"
    
    def test_special_characters_allowed(self):
        """Test special characters that should be allowed"""
        special_chars = [
            "file@name.txt",
            "file#name.txt",
            "file$name.txt",
            "file%name.txt",
            "file^name.txt",
            "file&name.txt",
            "file*name.txt",
            "file(name.txt",
            "file)name.txt",
            "file+name.txt",
            "file=name.txt",
            "file{name.txt",
            "file}name.txt",
            "file[name.txt",
            "file]name.txt",
            "file|name.txt",
            "file\\name.txt",
            "file:name.txt",
            "file;name.txt",
            "file'name.txt",
            'file"name.txt',
            "file<name.txt",
            "file>name.txt",
            "file,name.txt",
            "file?name.txt",
            "file!name.txt",
            "file~name.txt",
            "file`name.txt"
        ]
        
        for name in special_chars:
            result = check_nfc_policy(name)
            assert result is True, f"Special character should be allowed: {name}"
    
    def test_very_long_filenames(self):
        """Test very long filenames are handled correctly"""
        # Test filename at the limit (255 characters)
        long_name = "a" * 255
        result = check_nfc_policy(long_name)
        assert result is True, "255-character filename should be allowed"
        
        # Test filename exceeding limit
        too_long_name = "a" * 256
        result = check_nfc_policy(too_long_name)
        assert result is False, "256-character filename should be rejected"


class TestIntegration:
    """Integration tests for the verifier"""
    
    def test_full_verification_workflow(self):
        """Test complete verification workflow with real filesystem"""
        with tempfile.TemporaryDirectory() as temp_dir:
            mount_point = os.path.join(temp_dir, "mnt")
            fixtures_dir = os.path.join(temp_dir, "fixtures")
            
            # Create test structure
            os.makedirs(mount_point, exist_ok=True)
            os.makedirs(fixtures_dir, exist_ok=True)
            
            # Create test files
            test_file = os.path.join(mount_point, "test.txt")
            with open(test_file, 'w') as f:
                f.write("test content")
            
            # Create fixture data
            fixture_data = {
                "test.txt": {
                    "size": 12,
                    "sha256": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
                }
            }
            
            with open(os.path.join(fixtures_dir, "fixtures.json"), 'w') as f:
                json.dump(fixture_data, f)
            
            # Run verification
            verifier = NgfsLsVerifier(mount_point, fixtures_dir)
            result = verifier.run_verification()
            
            assert result is True
            assert len(verifier.errors) == 0
    
    def test_error_handling_edge_cases(self):
        """Test error handling for edge cases"""
        with tempfile.TemporaryDirectory() as temp_dir:
            mount_point = os.path.join(temp_dir, "mnt")
            fixtures_dir = os.path.join(temp_dir, "fixtures")
            
            os.makedirs(mount_point, exist_ok=True)
            os.makedirs(fixtures_dir, exist_ok=True)
            
            # Test with corrupted fixture file
            corrupted_fixture = os.path.join(fixtures_dir, "corrupted.json")
            with open(corrupted_fixture, 'w') as f:
                f.write("corrupted json content")
            
            verifier = NgfsLsVerifier(mount_point, fixtures_dir)
            verifier.fixtures_file = corrupted_fixture
            
            result = verifier.run_verification()
            assert result is False
            assert len(verifier.errors) > 0


if __name__ == "__main__":
    pytest.main([__file__])
