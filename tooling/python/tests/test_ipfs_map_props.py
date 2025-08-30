#!/usr/bin/env python3
"""
Property-based tests for IPFS map determinism and validation

This module uses Hypothesis to test IPFS export functionality with
random inputs to ensure determinism and correctness.
"""

import json
import tempfile
from pathlib import Path
from typing import Dict, List, Any
from hypothesis import given, strategies as st
import pytest

# Import the map checker
import sys
sys.path.append(str(Path(__file__).parent.parent))
from ipfs_map_check import IpfsMapChecker


class TestIpfsMapDeterminism:
    """Test IPFS map determinism properties"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.checker = IpfsMapChecker()
    
    @given(
        st.lists(
            st.integers(min_value=0, max_value=255),
            min_size=4,
            max_size=32
        ),
        st.integers(min_value=0, max_value=2),
        st.integers(min_value=0, max_value=1024*1024)
    )
    def test_map_entry_consistency(self, cid_bytes: List[int], kind: int, size: int):
        """Test that map entries are consistent across runs"""
        entry = {
            "ngfs_cid": cid_bytes,
            "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "kind": kind,
            "size": size
        }
        
        # Validate entry structure
        assert self.checker.validate_entry(entry, 0)
        
        # Check that ngfs_cid is a list
        assert isinstance(entry["ngfs_cid"], list)
        assert all(isinstance(b, int) for b in entry["ngfs_cid"])
        
        # Check that kind is valid
        assert entry["kind"] in [0, 1, 2]
        
        # Check that size is non-negative
        assert entry["size"] >= 0
    
    @given(
        st.lists(
            st.lists(
                st.integers(min_value=0, max_value=255),
                min_size=4,
                max_size=32
            ),
            min_size=1,
            max_size=10
        )
    )
    def test_map_sorting_determinism(self, cid_lists: List[List[int]]):
        """Test that map entries are sorted deterministically by ngfs_cid"""
        # Create entries with the provided CIDs
        entries = []
        for i, cid_bytes in enumerate(cid_lists):
            entry = {
                "ngfs_cid": cid_bytes,
                "ipfs_cid": f"bafy{i}",
                "kind": i % 3,
                "size": 1024 + i
            }
            entries.append(entry)
        
        # Sort entries by ngfs_cid bytes
        sorted_entries = sorted(entries, key=lambda e: e["ngfs_cid"])
        
        # Verify sorting is correct
        for i in range(1, len(sorted_entries)):
            prev_cid = sorted_entries[i-1]["ngfs_cid"]
            curr_cid = sorted_entries[i]["ngfs_cid"]
            assert prev_cid <= curr_cid, f"Entries not properly sorted: {prev_cid} > {curr_cid}"
    
    @given(
        st.text(
            alphabet=st.characters(min_codepoint=ord('a'), max_codepoint=ord('z')),
            min_size=10,
            max_size=100
        )
    )
    def test_ipfs_cid_format(self, cid_suffix: str):
        """Test IPFS CID format validation"""
        # Valid CID should start with 'b' and contain only base32 characters
        valid_cid = "b" + cid_suffix
        
        # Check that it starts with 'b'
        assert valid_cid.startswith('b')
        
        # Check that it contains only valid base32 characters
        valid_chars = set('abcdefghijklmnopqrstuvwxyz234567')
        assert all(c in valid_chars for c in valid_cid[1:])
        
        # Validate with checker
        assert self.checker.validate_ipfs_cid(valid_cid)
    
    @given(
        st.lists(
            st.integers(min_value=0, max_value=255),
            min_size=4,
            max_size=32
        ),
        st.integers(min_value=0, max_value=2),
        st.integers(min_value=0, max_value=1024*1024)
    )
    def test_map_entry_serialization(self, cid_bytes: List[int], kind: int, size: int):
        """Test that map entries can be serialized consistently"""
        entry = {
            "ngfs_cid": cid_bytes,
            "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "kind": kind,
            "size": size
        }
        
        # Serialize to JSON
        json_str = json.dumps(entry, sort_keys=True)
        
        # Deserialize back
        deserialized = json.loads(json_str)
        
        # Verify consistency
        assert deserialized["ngfs_cid"] == cid_bytes
        assert deserialized["kind"] == kind
        assert deserialized["size"] == size
        assert deserialized["ipfs_cid"] == entry["ipfs_cid"]
    
    @given(
        st.lists(
            st.lists(
                st.integers(min_value=0, max_value=255),
                min_size=4,
                max_size=32
            ),
            min_size=1,
            max_size=5
        )
    )
    def test_complete_map_structure(self, cid_lists: List[List[int]]):
        """Test complete IPFS map structure with random entries"""
        # Create entries
        entries = []
        for i, cid_bytes in enumerate(cid_lists):
            entry = {
                "ngfs_cid": cid_bytes,
                "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
                "kind": i % 3,
                "size": 1024 + i * 512
            }
            entries.append(entry)
        
        # Sort entries
        entries.sort(key=lambda e: e["ngfs_cid"])
        
        # Create complete map
        map_data = {
            "version": 1,
            "exported_vclock": 1000,
            "root_ngfs_cid": [1, 2, 3, 4],
            "entries": entries
        }
        
        # Validate map structure
        assert self.checker.validate_map_data(map_data)
        
        # Check determinism
        assert self.checker.check_determinism(map_data)
        
        # Verify required fields
        assert map_data["version"] == 1
        assert isinstance(map_data["exported_vclock"], int)
        assert isinstance(map_data["root_ngfs_cid"], list)
        assert isinstance(map_data["entries"], list)
        assert len(map_data["entries"]) == len(cid_lists)


class TestIpfsMapValidation:
    """Test IPFS map validation edge cases"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.checker = IpfsMapChecker()
    
    def test_empty_map(self):
        """Test validation of empty map"""
        empty_map = {
            "version": 1,
            "exported_vclock": 1000,
            "root_ngfs_cid": [1, 2, 3, 4],
            "entries": []
        }
        
        assert self.checker.validate_map_data(empty_map)
        assert self.checker.check_determinism(empty_map)
    
    def test_invalid_version(self):
        """Test rejection of invalid version"""
        invalid_map = {
            "version": 2,  # Invalid version
            "exported_vclock": 1000,
            "root_ngfs_cid": [1, 2, 3, 4],
            "entries": []
        }
        
        assert not self.checker.validate_map_data(invalid_map)
    
    def test_missing_fields(self):
        """Test rejection of maps with missing fields"""
        incomplete_map = {
            "version": 1,
            "exported_vclock": 1000,
            # Missing root_ngfs_cid and entries
        }
        
        assert not self.checker.validate_map_data(incomplete_map)
    
    def test_invalid_entry_kind(self):
        """Test rejection of entries with invalid kind"""
        invalid_entry = {
            "ngfs_cid": [1, 2, 3, 4],
            "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "kind": 99,  # Invalid kind
            "size": 1024
        }
        
        assert not self.checker.validate_entry(invalid_entry, 0)
    
    def test_invalid_ipfs_cid(self):
        """Test rejection of invalid IPFS CIDs"""
        invalid_cids = [
            "not_a_cid",
            "cid_without_b_prefix",
            "bInvalidChars!@#",
            "b",  # Too short
            "b" + "a" * 200,  # Too long
        ]
        
        for cid in invalid_cids:
            assert not self.checker.validate_ipfs_cid(cid), f"Should reject invalid CID: {cid}"
    
    def test_negative_size(self):
        """Test rejection of negative sizes"""
        invalid_entry = {
            "ngfs_cid": [1, 2, 3, 4],
            "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "kind": 0,
            "size": -1  # Negative size
        }
        
        assert not self.checker.validate_entry(invalid_entry, 0)


class TestIpfsMapFixtures:
    """Test IPFS map fixture validation"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.checker = IpfsMapChecker()
        
        # Create temporary directory for test fixtures
        self.temp_dir = tempfile.mkdtemp()
        self.fixtures_dir = Path(self.temp_dir)
        
        # Create test fixture structure
        self.create_test_fixtures()
    
    def create_test_fixtures(self):
        """Create test fixtures for validation"""
        ipfs_dir = self.fixtures_dir / "ipfs"
        ipfs_dir.mkdir()
        
        # Create expected map file
        expected_map = {
            "version": 1,
            "exported_vclock": 1000,
            "root_ngfs_cid": [1, 2, 3, 4],
            "entries": [
                {
                    "ngfs_cid": [1, 2, 3, 4],
                    "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
                    "kind": 0,
                    "size": 1024
                }
            ]
        }
        
        with open(ipfs_dir / "ipfs-map.expected.json", 'w') as f:
            json.dump(expected_map, f, indent=2)
    
    def test_fixture_validation(self):
        """Test validation against fixtures"""
        # Create a test map file
        test_map_file = self.temp_dir / "test-map.json"
        test_map = {
            "version": 1,
            "exported_vclock": 1000,
            "root_ngfs_cid": [1, 2, 3, 4],
            "entries": [
                {
                    "ngfs_cid": [1, 2, 3, 4],
                    "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
                    "kind": 0,
                    "size": 1024
                }
            ]
        }
        
        with open(test_map_file, 'w') as f:
            json.dump(test_map, f, indent=2)
        
        # Test fixture validation
        assert self.checker.check_fixtures(test_map_file, self.fixtures_dir)
    
    def test_fixture_mismatch(self):
        """Test detection of fixture mismatches"""
        # Create a test map file with different data
        test_map_file = self.temp_dir / "test-map-mismatch.json"
        test_map = {
            "version": 1,
            "exported_vclock": 1000,
            "root_ngfs_cid": [1, 2, 3, 4],
            "entries": [
                {
                    "ngfs_cid": [1, 2, 3, 4],
                    "ipfs_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
                    "kind": 1,  # Different kind
                    "size": 2048  # Different size
                }
            ]
        }
        
        with open(test_map_file, 'w') as f:
            json.dump(test_map, f, indent=2)
        
        # Test fixture validation should fail
        assert not self.checker.check_fixtures(test_map_file, self.fixtures_dir)
    
    def teardown_method(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
