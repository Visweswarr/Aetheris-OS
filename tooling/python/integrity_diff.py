#!/usr/bin/env python3
"""
NGFS Integrity Diff Reporter

This script provides human-friendly diffs between old and new CBOR/JSON files
to help developers debug divergence and understand what changed.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Any, Union
import cbor2
import hashlib


class IntegrityDiffReporter:
    """Reports differences between files for integrity checking"""
    
    def __init__(self):
        self.differences = []
    
    def compare_files(self, old_file: Path, new_file: Path) -> bool:
        """Compare two files and report differences"""
        if not old_file.exists():
            print(f"❌ Old file not found: {old_file}")
            return False
        
        if not new_file.exists():
            print(f"❌ New file not found: {new_file}")
            return False
        
        # Determine file type and compare accordingly
        if old_file.suffix.lower() in ['.cbor', '.bin']:
            return self.compare_cbor_files(old_file, new_file)
        elif old_file.suffix.lower() in ['.json', '.yml', '.yaml']:
            return self.compare_json_files(old_file, new_file)
        else:
            return self.compare_binary_files(old_file, new_file)
    
    def compare_cbor_files(self, old_file: Path, new_file: Path) -> bool:
        """Compare CBOR files and report structural differences"""
        print(f"🔍 Comparing CBOR files:")
        print(f"  Old: {old_file}")
        print(f"  New: {new_file}")
        
        try:
            # Read and parse CBOR files
            old_data = self.read_cbor_file(old_file)
            new_data = self.read_cbor_file(new_file)
            
            if old_data is None or new_data is None:
                return False
            
            # Compare structures
            return self.compare_cbor_structures(old_data, new_data, "root")
            
        except Exception as e:
            print(f"❌ Error comparing CBOR files: {e}")
            return False
    
    def compare_json_files(self, old_file: Path, new_file: Path) -> bool:
        """Compare JSON files and report differences"""
        print(f"🔍 Comparing JSON files:")
        print(f"  Old: {old_file}")
        print(f"  New: {new_file}")
        
        try:
            # Read and parse JSON files
            old_data = self.read_json_file(old_file)
            new_data = self.read_json_file(new_file)
            
            if old_data is None or new_data is None:
                return False
            
            # Compare structures
            return self.compare_json_structures(old_data, new_data, "root")
            
        except Exception as e:
            print(f"❌ Error comparing JSON files: {e}")
            return False
    
    def compare_binary_files(self, old_file: Path, new_file: Path) -> bool:
        """Compare binary files and report differences"""
        print(f"🔍 Comparing binary files:")
        print(f"  Old: {old_file}")
        print(f"  New: {new_file}")
        
        try:
            old_content = old_file.read_bytes()
            new_content = new_file.read_bytes()
            
            # Compare file sizes
            old_size = len(old_content)
            new_size = len(new_content)
            
            print(f"  Old size: {old_size} bytes")
            print(f"  New size: {new_size} bytes")
            
            if old_size != new_size:
                print(f"  ❌ Size mismatch: {old_size} vs {new_size} bytes")
                self.differences.append(f"Size mismatch: {old_size} vs {new_size} bytes")
                return False
            
            # Compare content
            if old_content == new_content:
                print(f"  ✅ Files are identical")
                return True
            else:
                print(f"  ❌ Content differs")
                
                # Find first difference
                first_diff = self.find_first_difference(old_content, new_content)
                if first_diff is not None:
                    print(f"  First difference at byte {first_diff}")
                    self.differences.append(f"Content differs at byte {first_diff}")
                
                return False
                
        except Exception as e:
            print(f"❌ Error comparing binary files: {e}")
            return False
    
    def read_cbor_file(self, file_path: Path) -> Any:
        """Read and parse CBOR file"""
        try:
            content = file_path.read_bytes()
            return cbor2.loads(content)
        except Exception as e:
            print(f"❌ Failed to parse CBOR file {file_path}: {e}")
            return None
    
    def read_json_file(self, file_path: Path) -> Any:
        """Read and parse JSON file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            return json.loads(content)
        except Exception as e:
            print(f"❌ Failed to parse JSON file {file_path}: {e}")
            return None
    
    def compare_cbor_structures(self, old_data: Any, new_data: Any, path: str) -> bool:
        """Recursively compare CBOR structures"""
        if type(old_data) != type(new_data):
            print(f"  ❌ Type mismatch at {path}: {type(old_data).__name__} vs {type(new_data).__name__}")
            self.differences.append(f"Type mismatch at {path}: {type(old_data).__name__} vs {type(new_data).__name__}")
            return False
        
        if isinstance(old_data, dict):
            return self.compare_cbor_dicts(old_data, new_data, path)
        elif isinstance(old_data, list):
            return self.compare_cbor_lists(old_data, new_data, path)
        else:
            if old_data != new_data:
                print(f"  ❌ Value mismatch at {path}: {old_data} vs {new_data}")
                self.differences.append(f"Value mismatch at {path}: {old_data} vs {new_data}")
                return False
            return True
    
    def compare_cbor_dicts(self, old_dict: Dict, new_dict: Dict, path: str) -> bool:
        """Compare CBOR dictionaries"""
        old_keys = set(old_dict.keys())
        new_keys = set(new_dict.keys())
        
        # Check for missing or extra keys
        missing_keys = old_keys - new_keys
        extra_keys = new_keys - old_keys
        
        if missing_keys:
            print(f"  ❌ Missing keys at {path}: {missing_keys}")
            self.differences.append(f"Missing keys at {path}: {missing_keys}")
        
        if extra_keys:
            print(f"  ❌ Extra keys at {path}: {extra_keys}")
            self.differences.append(f"Extra keys at {path}: {extra_keys}")
        
        if missing_keys or extra_keys:
            return False
        
        # Compare common keys
        all_match = True
        for key in old_keys:
            key_path = f"{path}.{key}"
            if not self.compare_cbor_structures(old_dict[key], new_dict[key], key_path):
                all_match = False
        
        return all_match
    
    def compare_cbor_lists(self, old_list: List, new_list: List, path: str) -> bool:
        """Compare CBOR lists"""
        if len(old_list) != len(new_list):
            print(f"  ❌ Length mismatch at {path}: {len(old_list)} vs {len(new_list)}")
            self.differences.append(f"Length mismatch at {path}: {len(old_list)} vs {len(new_list)}")
            return False
        
        # Compare elements
        all_match = True
        for i, (old_item, new_item) in enumerate(zip(old_list, new_list)):
            item_path = f"{path}[{i}]"
            if not self.compare_cbor_structures(old_item, new_item, item_path):
                all_match = False
        
        return all_match
    
    def compare_json_structures(self, old_data: Any, new_data: Any, path: str) -> bool:
        """Recursively compare JSON structures"""
        if type(old_data) != type(new_data):
            print(f"  ❌ Type mismatch at {path}: {type(old_data).__name__} vs {type(new_data).__name__}")
            self.differences.append(f"Type mismatch at {path}: {type(old_data).__name__} vs {type(new_data).__name__}")
            return False
        
        if isinstance(old_data, dict):
            return self.compare_json_dicts(old_data, new_data, path)
        elif isinstance(old_data, list):
            return self.compare_json_lists(old_data, new_data, path)
        else:
            if old_data != new_data:
                print(f"  ❌ Value mismatch at {path}: {old_data} vs {new_data}")
                self.differences.append(f"Value mismatch at {path}: {old_data} vs {new_data}")
                return False
            return True
    
    def compare_json_dicts(self, old_dict: Dict, new_dict: Dict, path: str) -> bool:
        """Compare JSON dictionaries"""
        old_keys = set(old_dict.keys())
        new_keys = set(new_dict.keys())
        
        # Check for missing or extra keys
        missing_keys = old_keys - new_keys
        extra_keys = new_keys - old_keys
        
        if missing_keys:
            print(f"  ❌ Missing keys at {path}: {missing_keys}")
            self.differences.append(f"Missing keys at {path}: {missing_keys}")
        
        if extra_keys:
            print(f"  ❌ Extra keys at {path}: {extra_keys}")
            self.differences.append(f"Extra keys at {path}: {extra_keys}")
        
        if missing_keys or extra_keys:
            return False
        
        # Compare common keys
        all_match = True
        for key in old_keys:
            key_path = f"{path}.{key}"
            if not self.compare_json_structures(old_dict[key], new_dict[key], key_path):
                all_match = False
        
        return all_match
    
    def compare_json_lists(self, old_list: List, new_list: List, path: str) -> bool:
        """Compare JSON lists"""
        if len(old_list) != len(new_list):
            print(f"  ❌ Length mismatch at {path}: {len(old_list)} vs {len(new_list)}")
            self.differences.append(f"Length mismatch at {path}: {len(old_list)} vs {len(new_list)}")
            return False
        
        # Compare elements
        all_match = True
        for i, (old_item, new_item) in enumerate(zip(old_list, new_list)):
            item_path = f"{path}[{i}]"
            if not self.compare_json_structures(old_item, new_item, item_path):
                all_match = False
        
        return all_match
    
    def find_first_difference(self, old_content: bytes, new_content: bytes) -> Union[int, None]:
        """Find the first byte where two binary contents differ"""
        min_len = min(len(old_content), len(new_content))
        
        for i in range(min_len):
            if old_content[i] != new_content[i]:
                return i
        
        return None
    
    def generate_summary(self) -> str:
        """Generate a summary of all differences"""
        if not self.differences:
            return "✅ No differences found"
        
        summary = f"❌ Found {len(self.differences)} differences:\n"
        for i, diff in enumerate(self.differences, 1):
            summary += f"  {i}. {diff}\n"
        
        return summary
    
    def compute_file_hashes(self, old_file: Path, new_file: Path) -> Dict[str, str]:
        """Compute blake3 hashes of both files"""
        hashes = {}
        
        try:
            # Compute old file hash
            old_content = old_file.read_bytes()
            old_hash = hashlib.blake2b(old_content, digest_size=32).hexdigest()
            hashes['old'] = old_hash
            
            # Compute new file hash
            new_content = new_file.read_bytes()
            new_hash = hashlib.blake2b(new_content, digest_size=32).hexdigest()
            hashes['new'] = new_hash
            
        except Exception as e:
            print(f"❌ Error computing file hashes: {e}")
        
        return hashes


def main():
    parser = argparse.ArgumentParser(description="Compare files for integrity checking")
    parser.add_argument("old_file", type=Path, help="Path to old file")
    parser.add_argument("new_file", type=Path, help="Path to new file")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    # Validate input files
    if not args.old_file.exists():
        print(f"❌ Old file not found: {args.old_file}")
        sys.exit(1)
    
    if not args.new_file.exists():
        print(f"❌ New file not found: {args.new_file}")
        sys.exit(1)
    
    # Create reporter and compare files
    reporter = IntegrityDiffReporter()
    files_match = reporter.compare_files(args.old_file, args.new_file)
    
    # Compute file hashes
    hashes = reporter.compute_file_hashes(args.old_file, args.new_file)
    
    # Generate summary
    summary = reporter.generate_summary()
    
    # Output results
    if args.json:
        output = {
            "old_file": str(args.old_file),
            "new_file": str(args.new_file),
            "files_match": files_match,
            "differences_count": len(reporter.differences),
            "differences": reporter.differences,
            "summary": summary,
            "hashes": hashes
        }
        print(json.dumps(output, indent=2))
    else:
        print("\n" + "="*60)
        print("INTEGRITY DIFF REPORT")
        print("="*60)
        print(summary)
        
        if hashes:
            print(f"\nFile Hashes:")
            print(f"  Old: {hashes.get('old', 'N/A')}")
            print(f"  New: {hashes.get('new', 'N/A')}")
        
        print(f"\nFiles {'match' if files_match else 'differ'}")
    
    # Exit with appropriate code
    sys.exit(0 if files_match else 1)


if __name__ == "__main__":
    main()
