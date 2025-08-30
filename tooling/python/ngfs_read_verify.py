#!/usr/bin/env python3
"""
NGFS Read Verification Script

This script walks a fixture tree and compares bytes against golden data
to verify NGFS read operations work correctly.
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple, Optional

class NgfsReadVerifier:
    def __init__(self, root_path: str, fixtures_path: str):
        self.root_path = Path(root_path)
        self.fixtures_path = Path(fixtures_path)
        self.stats = {
            "files_checked": 0,
            "files_matched": 0,
            "files_mismatch": 0,
            "total_bytes": 0,
            "start_time": time.time(),
        }

    def verify_tree(self) -> bool:
        """Walk the tree and verify all files"""
        print(f"Verifying tree at {self.root_path}")
        print(f"Using fixtures from {self.fixtures_path}")
        
        if not self.root_path.exists():
            print(f"Error: Root path {self.root_path} does not exist")
            return False
            
        if not self.fixtures_path.exists():
            print(f"Error: Fixtures path {self.fixtures_path} does not exist")
            return False

        try:
            self._walk_and_verify(self.root_path)
        except Exception as e:
            print(f"Error during verification: {e}")
            return False

        return self._print_summary()

    def _walk_and_verify(self, current_path: Path):
        """Recursively walk and verify files"""
        try:
            for item in current_path.iterdir():
                if item.is_file():
                    self._verify_file(item)
                elif item.is_dir():
                    self._walk_and_verify(item)
        except PermissionError:
            print(f"Warning: Permission denied accessing {current_path}")
        except Exception as e:
            print(f"Warning: Error accessing {current_path}: {e}")

    def _verify_file(self, file_path: Path):
        """Verify a single file against golden data"""
        self.stats["files_checked"] += 1
        
        try:
            # Read the actual file
            with open(file_path, 'rb') as f:
                actual_data = f.read()
            
            self.stats["total_bytes"] += len(actual_data)
            
            # Look for golden data
            golden_path = self._find_golden_file(file_path)
            if golden_path and golden_path.exists():
                with open(golden_path, 'rb') as f:
                    golden_data = f.read()
                
                if actual_data == golden_data:
                    self.stats["files_matched"] += 1
                    print(f"✓ {file_path} (matched, {len(actual_data)} bytes)")
                else:
                    self.stats["files_mismatch"] += 1
                    print(f"✗ {file_path} (mismatch, expected {len(golden_data)}, got {len(actual_data)} bytes)")
            else:
                # No golden data, just report the file
                print(f"? {file_path} (no golden data, {len(actual_data)} bytes)")
                
        except Exception as e:
            print(f"✗ {file_path} (error: {e})")
            self.stats["files_mismatch"] += 1

    def _find_golden_file(self, file_path: Path) -> Optional[Path]:
        """Find the corresponding golden file in fixtures"""
        # Get relative path from root
        try:
            rel_path = file_path.relative_to(self.root_path)
            golden_path = self.fixtures_path / rel_path
            return golden_path
        except ValueError:
            return None

    def _print_summary(self) -> bool:
        """Print verification summary and return success status"""
        elapsed = time.time() - self.stats["start_time"]
        
        print("\n" + "="*50)
        print("VERIFICATION SUMMARY")
        print("="*50)
        print(f"Files checked: {self.stats['files_checked']}")
        print(f"Files matched: {self.stats['files_matched']}")
        print(f"Files mismatch: {self.stats['files_mismatch']}")
        print(f"Total bytes: {self.stats['total_bytes']:,}")
        print(f"Time elapsed: {elapsed:.2f}s")
        
        if self.stats["files_mismatch"] > 0:
            print(f"\n❌ VERIFICATION FAILED: {self.stats['files_mismatch']} files have mismatches")
            return False
        else:
            print(f"\n✅ VERIFICATION PASSED: All {self.stats['files_matched']} files matched")
            return True

    def output_json_metrics(self):
        """Output JSON metrics for CI parsing"""
        elapsed = time.time() - self.stats["start_time"]
        metrics = {
            "test": "ngfs_read_verify",
            "files_checked": self.stats["files_checked"],
            "files_matched": self.stats["files_matched"],
            "files_mismatch": self.stats["files_mismatch"],
            "total_bytes": self.stats["total_bytes"],
            "elapsed_ms": int(elapsed * 1000),
            "success": self.stats["files_mismatch"] == 0
        }
        print(json.dumps(metrics))

def main():
    parser = argparse.ArgumentParser(description="Verify NGFS read operations against fixtures")
    parser.add_argument("--root", required=True, help="Root path to verify")
    parser.add_argument("--fixtures", required=True, help="Path to fixture data")
    parser.add_argument("--json", action="store_true", help="Output JSON metrics")
    
    args = parser.parse_args()
    
    verifier = NgfsReadVerifier(args.root, args.fixtures)
    success = verifier.verify_tree()
    
    if args.json:
        verifier.output_json_metrics()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
