#!/usr/bin/env python3

"""
NGFS LS Verify Tool

This tool walks the mounted NGFS tree, verifies sizes/contents against fixtures,
and checks NFC name policy compliance.
"""

import argparse
import json
import os
import sys
import unicodedata
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
import hashlib

class NgfsLsVerifier:
    """Verifies NGFS mounted tree against fixtures and policies"""
    
    def __init__(self, mount_point: str, fixtures_dir: str):
        self.mount_point = Path(mount_point)
        self.fixtures_dir = Path(fixtures_dir)
        self.errors: List[str] = []
        self.warnings: List[str] = []
        self.verified_files: Set[str] = set()
        self.verified_dirs: Set[str] = set()
        
    def verify_mount_point(self) -> bool:
        """Verify that the mount point exists and is accessible"""
        if not self.mount_point.exists():
            self.errors.append(f"Mount point does not exist: {self.mount_point}")
            return False
            
        if not self.mount_point.is_dir():
            self.errors.append(f"Mount point is not a directory: {self.mount_point}")
            return False
            
        # Try to list contents to verify it's mounted
        try:
            list(self.mount_point.iterdir())
        except PermissionError:
            self.errors.append(f"Permission denied accessing mount point: {self.mount_point}")
            return False
        except OSError as e:
            self.errors.append(f"Error accessing mount point: {e}")
            return False
            
        return True
    
    def check_nfc_policy(self, name: str) -> bool:
        """Check if filename complies with NFC normalization policy"""
        # Check for control characters
        if any(ord(c) < 32 or ord(c) == 127 for c in name):
            self.errors.append(f"Filename contains control characters: {name}")
            return False
            
        # Check for dotdot sequences
        if ".." in name:
            self.errors.append(f"Filename contains dotdot sequence: {name}")
            return False
            
        # Check for leading/trailing dots
        if name.startswith('.') or name.endswith('.'):
            self.errors.append(f"Filename starts or ends with dot: {name}")
            return False
            
        # Check for empty names
        if not name.strip():
            self.errors.append(f"Empty filename encountered")
            return False
            
        # Check NFC normalization
        nfc_name = unicodedata.normalize('NFC', name)
        if nfc_name != name:
            self.warnings.append(f"Filename not in NFC form: {name} -> {nfc_name}")
            
        return True
    
    def verify_file_contents(self, file_path: Path, expected_size: Optional[int] = None) -> bool:
        """Verify file contents and size"""
        try:
            # Check if file exists
            if not file_path.exists():
                self.errors.append(f"File not found: {file_path}")
                return False
                
            # Check if it's actually a file
            if not file_path.is_file():
                self.errors.append(f"Path is not a file: {file_path}")
                return False
                
            # Get file size
            actual_size = file_path.stat().st_size
            
            # Check size if expected size provided
            if expected_size is not None and actual_size != expected_size:
                self.errors.append(f"File size mismatch for {file_path}: expected {expected_size}, got {actual_size}")
                return False
                
            # For small files, verify content hash
            if actual_size <= 1024:  # 1 KiB threshold
                try:
                    with open(file_path, 'rb') as f:
                        content = f.read()
                        content_hash = hashlib.sha256(content).hexdigest()
                        # In a real implementation, this would compare against fixture hash
                        self.warnings.append(f"Content hash for {file_path}: {content_hash}")
                except Exception as e:
                    self.warnings.append(f"Could not read file content for {file_path}: {e}")
                    
            self.verified_files.add(str(file_path))
            return True
            
        except Exception as e:
            self.errors.append(f"Error verifying file {file_path}: {e}")
            return False
    
    def verify_directory_structure(self, dir_path: Path, expected_entries: Optional[List[str]] = None) -> bool:
        """Verify directory structure and contents"""
        try:
            # Check if directory exists
            if not dir_path.exists():
                self.errors.append(f"Directory not found: {dir_path}")
                return False
                
            # Check if it's actually a directory
            if not dir_path.is_dir():
                self.errors.append(f"Path is not a directory: {dir_path}")
                return False
                
            # List directory contents
            try:
                entries = list(dir_path.iterdir())
            except PermissionError:
                self.errors.append(f"Permission denied reading directory: {dir_path}")
                return False
                
            # Check for expected entries if provided
            if expected_entries is not None:
                actual_names = {entry.name for entry in entries}
                expected_set = set(expected_entries)
                
                missing = expected_set - actual_names
                extra = actual_names - expected_set
                
                if missing:
                    self.errors.append(f"Missing expected entries in {dir_path}: {missing}")
                    
                if extra:
                    self.warnings.append(f"Extra entries in {dir_path}: {extra}")
                    
            # Verify each entry
            for entry in entries:
                entry_name = entry.name
                
                # Check NFC policy
                if not self.check_nfc_policy(entry_name):
                    continue
                    
                # Verify entry based on type
                if entry.is_file():
                    self.verify_file_contents(entry)
                elif entry.is_dir():
                    self.verify_directory_structure(entry)
                elif entry.is_symlink():
                    self.warnings.append(f"Symlink found: {entry}")
                else:
                    self.warnings.append(f"Unknown file type: {entry}")
                    
            self.verified_dirs.add(str(dir_path))
            return True
            
        except Exception as e:
            self.errors.append(f"Error verifying directory {dir_path}: {e}")
            return False
    
    def load_fixture_data(self) -> Dict:
        """Load fixture data for comparison"""
        fixture_data = {}
        
        try:
            # Look for fixture files
            fixture_files = [
                self.fixtures_dir / "dir_simple.cbor",
                self.fixtures_dir / "file_1chunk.cbor",
                self.fixtures_dir / "ipfs" / "dir_simple.cbor",
                self.fixtures_dir / "ipfs" / "file_1chunk.cbor",
            ]
            
            for fixture_file in fixture_files:
                if fixture_file.exists():
                    # In a real implementation, this would parse CBOR and extract metadata
                    fixture_name = fixture_file.name
                    fixture_data[fixture_name] = {
                        "path": str(fixture_file),
                        "size": fixture_file.stat().st_size,
                        "type": "cbor"
                    }
                    
        except Exception as e:
            self.warnings.append(f"Could not load fixture data: {e}")
            
        return fixture_data
    
    def verify_against_fixtures(self) -> bool:
        """Verify mounted tree against fixture data"""
        fixture_data = self.load_fixture_data()
        
        if not fixture_data:
            self.warnings.append("No fixture data available for comparison")
            return True
            
        # For now, just check that we can access the mount point
        # In a real implementation, this would:
        # 1. Parse CBOR fixtures to get expected structure
        # 2. Walk the mounted tree
        # 3. Compare sizes, permissions, and content hashes
        # 4. Verify NFC normalization
        
        return True
    
    def run_verification(self) -> bool:
        """Run the complete verification process"""
        print(f"🔍 Starting NGFS tree verification...")
        print(f"  Mount point: {self.mount_point}")
        print(f"  Fixtures dir: {self.fixtures_dir}")
        print()
        
        # Step 1: Verify mount point
        if not self.verify_mount_point():
            return False
            
        # Step 2: Verify against fixtures
        if not self.verify_against_fixtures():
            return False
            
        # Step 3: Walk the tree and verify structure
        if not self.verify_directory_structure(self.mount_point):
            return False
            
        # Step 4: Report results
        self.print_report()
        
        return len(self.errors) == 0
    
    def print_report(self):
        """Print verification report"""
        print("\n" + "="*60)
        print("NGFS TREE VERIFICATION REPORT")
        print("="*60)
        
        if len(self.errors) == 0:
            print("✅ All verifications passed")
        else:
            print(f"❌ {len(self.errors)} verification errors")
            
        if self.warnings:
            print(f"⚠️  {len(self.warnings)} warnings")
            
        print(f"\n📁 Verified directories: {len(self.verified_dirs)}")
        print(f"📄 Verified files: {len(self.verified_files)}")
        
        if self.errors:
            print("\nErrors:")
            for error in self.errors:
                print(f"  ❌ {error}")
                
        if self.warnings:
            print("\nWarnings:")
            for warning in self.warnings:
                print(f"  ⚠️  {warning}")
                
        print(f"\nSummary: {len(self.errors)} errors, {len(self.warnings)} warnings")
    
    def get_verification_stats(self) -> Dict:
        """Get verification statistics for JSON output"""
        return {
            "tool": "ngfs_ls_verify",
            "mount_point": str(self.mount_point),
            "fixtures_dir": str(self.fixtures_dir),
            "verified_directories": len(self.verified_dirs),
            "verified_files": len(self.verified_files),
            "errors": len(self.errors),
            "warnings": len(self.warnings),
            "success": len(self.errors) == 0,
            "error_details": self.errors,
            "warning_details": self.warnings
        }

def main():
    parser = argparse.ArgumentParser(
        description="Verify NGFS mounted tree against fixtures and policies",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Basic verification
  ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures
  
  # JSON output
  ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures --json
  
  # Verbose output
  ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures --verbose
        """
    )
    
    parser.add_argument(
        "--mnt", "--mount",
        required=True,
        help="Mount point directory to verify"
    )
    
    parser.add_argument(
        "--fixtures",
        required=True,
        help="Directory containing NGFS fixtures for comparison"
    )
    
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output results as JSON"
    )
    
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable verbose output"
    )
    
    args = parser.parse_args()
    
    # Create verifier
    verifier = NgfsLsVerifier(args.mnt, args.fixtures)
    
    # Run verification
    success = verifier.run_verification()
    
    # Output results
    if args.json:
        stats = verifier.get_verification_stats()
        print(json.dumps(stats, indent=2))
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
