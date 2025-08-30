#!/usr/bin/env python3
"""
IPFS Map Checker for NGFS

This script validates IPFS export maps for determinism, multibase correctness,
and block length consistency.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional
import hashlib


class IpfsMapChecker:
    """Validates IPFS export maps for NGFS"""
    
    def __init__(self):
        self.errors = []
        self.warnings = []
    
    def validate_map_file(self, map_file: Path) -> bool:
        """Validate an IPFS map file"""
        try:
            with open(map_file, 'r') as f:
                if map_file.suffix == '.json':
                    map_data = json.load(f)
                else:
                    # Assume CBOR for now - would need cbor2 library
                    self.errors.append(f"CBOR format not yet supported: {map_file}")
                    return False
        except Exception as e:
            self.errors.append(f"Failed to read map file: {e}")
            return False
        
        return self.validate_map_data(map_data)
    
    def validate_map_data(self, map_data: Dict[str, Any]) -> bool:
        """Validate IPFS map data structure"""
        # Check required fields
        required_fields = ['version', 'exported_vclock', 'root_ngfs_cid', 'entries']
        for field in required_fields:
            if field not in map_data:
                self.errors.append(f"Missing required field: {field}")
                return False
        
        # Validate version
        if map_data['version'] != 1:
            self.errors.append(f"Unsupported version: {map_data['version']}")
            return False
        
        # Validate entries
        if not isinstance(map_data['entries'], list):
            self.errors.append("Entries must be a list")
            return False
        
        # Validate each entry
        for i, entry in enumerate(map_data['entries']):
            if not self.validate_entry(entry, i):
                return False
        
        return len(self.errors) == 0
    
    def validate_entry(self, entry: Dict[str, Any], index: int) -> bool:
        """Validate a single IPFS map entry"""
        # Check required fields
        required_fields = ['ngfs_cid', 'ipfs_cid', 'kind', 'size']
        for field in required_fields:
            if field not in entry:
                self.errors.append(f"Entry {index}: missing field '{field}'")
                return False
        
        # Validate ngfs_cid
        if not isinstance(entry['ngfs_cid'], list):
            self.errors.append(f"Entry {index}: ngfs_cid must be a list")
            return False
        
        # Validate ipfs_cid
        if not self.validate_ipfs_cid(entry['ipfs_cid']):
            self.errors.append(f"Entry {index}: invalid IPFS CID: {entry['ipfs_cid']}")
            return False
        
        # Validate kind
        if not isinstance(entry['kind'], int) or entry['kind'] not in [0, 1, 2]:
            self.errors.append(f"Entry {index}: invalid kind: {entry['kind']}")
            return False
        
        # Validate size
        if not isinstance(entry['size'], int) or entry['size'] < 0:
            self.errors.append(f"Entry {index}: invalid size: {entry['size']}")
            return False
        
        return True
    
    def validate_ipfs_cid(self, cid: str) -> bool:
        """Validate IPFS CIDv1 format"""
        if not isinstance(cid, str):
            return False
        
        # Check if it starts with 'b' (base32-lowercase)
        if not cid.startswith('b'):
            return False
        
        # Check length (should be reasonable for base32)
        if len(cid) < 10 or len(cid) > 100:
            return False
        
        # Check if it contains only valid base32 characters
        valid_chars = set('abcdefghijklmnopqrstuvwxyz234567')
        if not all(c in valid_chars for c in cid[1:]):
            return False
        
        return True
    
    def check_determinism(self, map_data: Dict[str, Any]) -> bool:
        """Check if the map is deterministic (sorted by ngfs_cid)"""
        entries = map_data['entries']
        
        # Check if entries are sorted by ngfs_cid bytes
        for i in range(1, len(entries)):
            prev_cid = entries[i-1]['ngfs_cid']
            curr_cid = entries[i]['ngfs_cid']
            
            if prev_cid > curr_cid:
                self.errors.append(f"Entries not sorted: entry {i-1} > entry {i}")
                return False
        
        return True
    
    def check_fixtures(self, map_file: Path, fixtures_dir: Path) -> bool:
        """Check map against fixture expectations"""
        if not fixtures_dir.exists():
            self.warnings.append(f"Fixtures directory not found: {fixtures_dir}")
            return True
        
        # Look for expected map file
        expected_file = fixtures_dir / "ipfs" / "ipfs-map.expected.json"
        if not expected_file.exists():
            self.warnings.append(f"Expected map file not found: {expected_file}")
            return True
        
        try:
            with open(expected_file, 'r') as f:
                expected_data = json.load(f)
            
            # Load actual map data
            with open(map_file, 'r') as f:
                actual_data = json.load(f)
            
            # Compare key fields
            if actual_data['version'] != expected_data['version']:
                self.errors.append(f"Version mismatch: {actual_data['version']} != {expected_data['version']}")
            
            if len(actual_data['entries']) != len(expected_data['entries']):
                self.errors.append(f"Entry count mismatch: {len(actual_data['entries'])} != {len(expected_data['entries'])}")
            
            # Compare first few entries for basic validation
            for i in range(min(3, len(actual_data['entries']), len(expected_data['entries']))):
                actual_entry = actual_data['entries'][i]
                expected_entry = expected_data['entries'][i]
                
                if actual_entry['kind'] != expected_entry['kind']:
                    self.errors.append(f"Entry {i} kind mismatch: {actual_entry['kind']} != {expected_entry['kind']}")
                
                if actual_entry['size'] != expected_entry['size']:
                    self.errors.append(f"Entry {i} size mismatch: {actual_entry['size']} != {expected_entry['size']}")
        
        except Exception as e:
            self.errors.append(f"Failed to check fixtures: {e}")
            return False
        
        return len(self.errors) == 0
    
    def print_report(self):
        """Print validation report"""
        if self.errors:
            print("❌ Validation failed:")
            for error in self.errors:
                print(f"  ERROR: {error}")
            return False
        
        if self.warnings:
            print("⚠️  Validation passed with warnings:")
            for warning in self.warnings:
                print(f"  WARNING: {warning}")
        else:
            print("✅ Validation passed")
        
        return True


def main():
    parser = argparse.ArgumentParser(description="Validate IPFS export maps for NGFS")
    parser.add_argument("--map", required=True, type=Path, help="IPFS map file to validate")
    parser.add_argument("--fixtures", type=Path, help="Fixtures directory for expected values")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    
    args = parser.parse_args()
    
    if not args.map.exists():
        print(f"Map file not found: {args.map}", file=sys.stderr)
        sys.exit(1)
    
    checker = IpfsMapChecker()
    
    # Validate map file
    if not checker.validate_map_file(args.map):
        if args.json:
            output = {
                "valid": False,
                "errors": checker.errors,
                "warnings": checker.warnings
            }
            print(json.dumps(output))
        else:
            checker.print_report()
        sys.exit(1)
    
    # Load map data for additional checks
    try:
        with open(args.map, 'r') as f:
            map_data = json.load(f)
    except Exception as e:
        print(f"Failed to load map data: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Check determinism
    if not checker.check_determinism(map_data):
        if args.json:
            output = {
                "valid": False,
                "errors": checker.errors,
                "warnings": checker.warnings
            }
            print(json.dumps(output))
        else:
            checker.print_report()
        sys.exit(1)
    
    # Check fixtures if provided
    if args.fixtures:
        if not checker.check_fixtures(args.map, args.fixtures):
            if args.json:
                output = {
                    "valid": False,
                    "errors": checker.errors,
                    "warnings": checker.warnings
                }
                print(json.dumps(output))
            else:
                checker.print_report()
            sys.exit(1)
    
    # Output results
    if args.json:
        output = {
            "valid": True,
            "errors": checker.errors,
            "warnings": checker.warnings,
            "entries": len(map_data['entries']),
            "version": map_data['version']
        }
        print(json.dumps(output))
    else:
        success = checker.print_report()
        if success:
            print(f"Map contains {len(map_data['entries'])} entries")
            print(f"Version: {map_data['version']}")
        sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
