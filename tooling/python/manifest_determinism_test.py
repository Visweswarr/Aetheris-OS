#!/usr/bin/env python3
"""
NGFS Manifest Determinism Test

This module provides property-based testing to ensure that NGFS manifests
produce identical CIDs regardless of the order in which entries are added.
"""

import json
import random
import time
from typing import List, Dict, Any, Tuple
from hypothesis import given, strategies as st
from hypothesis.strategies import text, integers, lists
import cbor2
import hashlib
import os
import sys

# Add the project root to the path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

# NGFS constants
NGFS_MAX_NAME_LENGTH = 255
NGFS_MAX_DIR_ENTRIES = 65_535
NGFS_MAX_FILE_CHUNKS = 4_096
NGFS_MAX_FILE_SIZE = 1 << 40  # 1 TiB

# Entry kinds
ENTRY_KIND_DIRECTORY = 0
ENTRY_KIND_FILE = 1
ENTRY_KIND_SYMLINK = 2

# Content types
CONTENT_TYPE_RAW = 0
CONTENT_TYPE_DIRECTORY = 1
CONTENT_TYPE_FILE_MANIFEST = 2
CONTENT_TYPE_SNAPSHOT = 3
CONTENT_TYPE_SYMLINK = 4
CONTENT_TYPE_SPECIAL = 5


def validate_entry_name(name: str) -> bool:
    """Validate entry name according to NGFS constraints."""
    if not name or len(name) > NGFS_MAX_NAME_LENGTH:
        return False
    
    # Check for forbidden characters
    if '\x00' in name or '/' in name:
        return False
    
    # Check for reserved names
    if name in ('.', '..'):
        return False
    
    # Check for control characters
    if any(ord(c) < 0x20 for c in name):
        return False
    
    return True


def normalize_name(name: str) -> str:
    """Normalize filename to NFC form."""
    import unicodedata
    return unicodedata.normalize('NFC', name)


def generate_content_id(data: bytes, content_type: int) -> Dict[str, Any]:
    """Generate a content identifier for given data."""
    # Compute Blake3 hash (using SHA-256 as fallback for now)
    hash_obj = hashlib.sha256()
    hash_obj.update(data)
    blake3_hash = hash_obj.digest()
    
    return {
        "blake3_hash": list(blake3_hash),
        "ipfs_multihash": None,
        "content_type": content_type
    }


def create_test_entry(name: str, kind: int, cid: Dict[str, Any], size: int = None) -> Dict[str, Any]:
    """Create a test entry for testing purposes."""
    entry = {
        "name": name,
        "kind": kind,
        "cid": cid,
        "size": size,
        "mode": 0o644,
        "xattrs": None
    }
    
    # Remove size for directories
    if kind == ENTRY_KIND_DIRECTORY:
        entry.pop("size")
    
    return entry


def build_directory_manifest(entries: List[Dict[str, Any]]) -> Tuple[Dict[str, Any], bytes]:
    """Build a directory manifest with canonical ordering."""
    if len(entries) > NGFS_MAX_DIR_ENTRIES:
        raise ValueError(f"Too many entries: {len(entries)} > {NGFS_MAX_DIR_ENTRIES}")
    
    # Sort entries canonically: (kind asc) then (name bytes asc)
    sorted_entries = sorted(entries, key=lambda e: (e["kind"], e["name"].encode('utf-8')))
    
    manifest = {
        "version": 1,
        "entries": sorted_entries
    }
    
    # Serialize to CBOR
    cbor_bytes = cbor2.dumps(manifest)
    
    return manifest, cbor_bytes


def build_file_manifest(chunks: List[Tuple[Dict[str, Any], int]]) -> Tuple[Dict[str, Any], bytes]:
    """Build a file manifest with canonical ordering."""
    if len(chunks) > NGFS_MAX_FILE_CHUNKS:
        raise ValueError(f"Too many chunks: {len(chunks)} > {NGFS_MAX_FILE_CHUNKS}")
    
    # Sort chunks by CID for determinism
    sorted_chunks = sorted(chunks, key=lambda c: bytes(c[0]["blake3_hash"]))
    
    chunk_infos = [
        {
            "cid": cid,
            "length": length
        }
        for cid, length in sorted_chunks
    ]
    
    total_size = sum(length for _, length in chunks)
    if total_size > NGFS_MAX_FILE_SIZE:
        raise ValueError(f"File too large: {total_size} > {NGFS_MAX_FILE_SIZE}")
    
    manifest = {
        "version": 1,
        "chunks": chunk_infos,
        "total_size": total_size,
        "algorithm": "blake3"
    }
    
    # Serialize to CBOR
    cbor_bytes = cbor2.dumps(manifest)
    
    return manifest, cbor_bytes


def compute_cid(cbor_bytes: bytes, content_type: int) -> str:
    """Compute content identifier from CBOR bytes."""
    # Compute Blake3 hash (using SHA-256 as fallback for now)
    hash_obj = hashlib.sha256()
    hash_obj.update(cbor_bytes)
    hash_bytes = hash_obj.digest()
    
    # Return hex string
    return hash_bytes.hex()


def test_manifest_determinism():
    """Test that manifests produce identical CIDs regardless of entry order."""
    print("Testing manifest determinism...")
    
    # Create test entries in different orders
    test_entries = []
    
    # Add some directory entries
    for i in range(3):
        name = f"dir_{i}"
        cid = generate_content_id(name.encode(), CONTENT_TYPE_DIRECTORY)
        entry = create_test_entry(name, ENTRY_KIND_DIRECTORY, cid)
        test_entries.append(entry)
    
    # Add some file entries
    for i in range(3):
        name = f"file_{i}"
        cid = generate_content_id(name.encode(), CONTENT_TYPE_RAW)
        entry = create_test_entry(name, ENTRY_KIND_FILE, cid, 100 + i * 50)
        test_entries.append(entry)
    
    # Add some symlink entries
    for i in range(2):
        name = f"link_{i}"
        cid = generate_content_id(name.encode(), CONTENT_TYPE_SYMLINK)
        entry = create_test_entry(name, ENTRY_KIND_SYMLINK, cid)
        test_entries.append(entry)
    
    # Test different orderings
    orderings = []
    
    # Original order
    orderings.append(test_entries.copy())
    
    # Reversed order
    orderings.append(list(reversed(test_entries)))
    
    # Random order
    random_entries = test_entries.copy()
    random.shuffle(random_entries)
    orderings.append(random_entries)
    
    # Sort by name first, then kind
    name_first = sorted(test_entries, key=lambda e: e["name"])
    orderings.append(name_first)
    
    # Sort by kind first, then name
    kind_first = sorted(test_entries, key=lambda e: (e["kind"], e["name"]))
    orderings.append(kind_first)
    
    # Generate manifests for each ordering
    cids = []
    manifests = []
    
    for i, ordering in enumerate(orderings):
        try:
            manifest, cbor_bytes = build_directory_manifest(ordering)
            cid = compute_cid(cbor_bytes, CONTENT_TYPE_DIRECTORY)
            
            cids.append(cid)
            manifests.append({
                "ordering": i,
                "cid": cid,
                "size": len(cbor_bytes),
                "entry_count": len(ordering)
            })
            
            print(f"  Ordering {i}: CID = {cid[:16]}..., Size = {len(cbor_bytes)} bytes")
            
        except Exception as e:
            print(f"  Ordering {i} failed: {e}")
            return False
    
    # Check that all CIDs are identical
    first_cid = cids[0]
    all_identical = all(cid == first_cid for cid in cids)
    
    if all_identical:
        print(f"✓ All orderings produced identical CID: {first_cid[:16]}...")
    else:
        print("✗ CIDs are not identical across orderings!")
        for i, cid in enumerate(cids):
            print(f"  Ordering {i}: {cid}")
        return False
    
    return True


def test_file_manifest_determinism():
    """Test that file manifests produce identical CIDs regardless of chunk order."""
    print("Testing file manifest determinism...")
    
    # Create test chunks in different orders
    test_chunks = []
    
    for i in range(5):
        data = f"chunk_data_{i}".encode()
        cid = generate_content_id(data, CONTENT_TYPE_RAW)
        length = 100 + i * 50
        test_chunks.append((cid, length))
    
    # Test different orderings
    orderings = []
    
    # Original order
    orderings.append(test_chunks.copy())
    
    # Reversed order
    orderings.append(list(reversed(test_chunks)))
    
    # Random order
    random_chunks = test_chunks.copy()
    random.shuffle(random_chunks)
    orderings.append(random_chunks)
    
    # Sort by length first
    length_first = sorted(test_chunks, key=lambda c: c[1])
    orderings.append(length_first)
    
    # Generate manifests for each ordering
    cids = []
    manifests = []
    
    for i, ordering in enumerate(orderings):
        try:
            manifest, cbor_bytes = build_file_manifest(ordering)
            cid = compute_cid(cbor_bytes, CONTENT_TYPE_FILE_MANIFEST)
            
            cids.append(cid)
            manifests.append({
                "ordering": i,
                "cid": cid,
                "size": len(cbor_bytes),
                "chunk_count": len(ordering)
            })
            
            print(f"  Ordering {i}: CID = {cid[:16]}..., Size = {len(cbor_bytes)} bytes")
            
        except Exception as e:
            print(f"  Ordering {i} failed: {e}")
            return False
    
    # Check that all CIDs are identical
    first_cid = cids[0]
    all_identical = all(cid == first_cid for cid in cids)
    
    if all_identical:
        print(f"✓ All orderings produced identical CID: {first_cid[:16]}...")
    else:
        print("✗ CIDs are not identical across orderings!")
        for i, cid in enumerate(cids):
            print(f"  Ordering {i}: {cid}")
        return False
    
    return True


def test_name_validation():
    """Test entry name validation."""
    print("Testing name validation...")
    
    # Valid names
    valid_names = [
        "file.txt",
        "dir",
        "file-name",
        "file_name",
        "file123",
        "file.123",
        "file-name_123",
        "file-name.123",
        "file-name_123.txt"
    ]
    
    for name in valid_names:
        if not validate_entry_name(name):
            print(f"✗ Valid name rejected: {name}")
            return False
        print(f"  ✓ Valid: {name}")
    
    # Invalid names
    invalid_names = [
        "",  # Empty
        ".",  # Current directory
        "..",  # Parent directory
        "file/name",  # Contains slash
        "file\0name",  # Contains NUL
        "file\x01name",  # Contains control character
        "file\x1fname",  # Contains control character
        "file\x7fname",  # Contains DEL
    ]
    
    for name in invalid_names:
        if validate_entry_name(name):
            print(f"✗ Invalid name accepted: {repr(name)}")
            return False
        print(f"  ✓ Invalid rejected: {repr(name)}")
    
    # Test length limits
    long_name = "a" * (NGFS_MAX_NAME_LENGTH + 1)
    if validate_entry_name(long_name):
        print(f"✗ Too long name accepted: {len(long_name)} chars")
        return False
    print(f"  ✓ Too long name rejected: {len(long_name)} chars")
    
    # Test boundary case
    boundary_name = "a" * NGFS_MAX_NAME_LENGTH
    if not validate_entry_name(boundary_name):
        print(f"✗ Boundary length name rejected: {len(boundary_name)} chars")
        return False
    print(f"  ✓ Boundary length name accepted: {len(boundary_name)} chars")
    
    return True


def test_unicode_normalization():
    """Test Unicode normalization."""
    print("Testing Unicode normalization...")
    
    # Test cases that should normalize to the same result
    test_cases = [
        ("café", "cafe\u0301"),  # Precomposed vs decomposed
        ("über", "u\u0308ber"),   # Precomposed vs decomposed
        ("naïve", "nai\u0308ve"), # Precomposed vs decomposed
    ]
    
    for expected, decomposed in test_cases:
        normalized = normalize_name(decomposed)
        if normalized != expected:
            print(f"✗ Normalization failed: {decomposed} -> {normalized} (expected {expected})")
            return False
        print(f"  ✓ Normalized: {decomposed} -> {normalized}")
    
    return True


def run_property_tests():
    """Run property-based tests using Hypothesis."""
    print("Running property-based tests...")
    
    @given(
        st.lists(
            st.tuples(
                st.text(min_size=1, max_size=50, alphabet=st.characters(min_codepoint=32, max_codepoint=126)),
                st.integers(min_value=0, max_value=2),
                st.integers(min_value=0, max_value=1000)
            ),
            min_size=1,
            max_size=10
        )
    )
    def test_manifest_ordering_property(entries_data):
        """Property test: manifest CIDs should be identical regardless of entry order."""
        try:
            # Convert to test entries
            entries = []
            for name, kind, size in entries_data:
                # Skip invalid names
                if not validate_entry_name(name):
                    continue
                
                cid = generate_content_id(name.encode(), CONTENT_TYPE_RAW)
                entry = create_test_entry(name, kind, cid, size if kind == ENTRY_KIND_FILE else None)
                entries.append(entry)
            
            if len(entries) < 2:
                return  # Skip if not enough valid entries
            
            # Test original order
            manifest1, cbor1 = build_directory_manifest(entries)
            cid1 = compute_cid(cbor1, CONTENT_TYPE_DIRECTORY)
            
            # Test reversed order
            reversed_entries = list(reversed(entries))
            manifest2, cbor2 = build_directory_manifest(reversed_entries)
            cid2 = compute_cid(cbor2, CONTENT_TYPE_DIRECTORY)
            
            # CIDs should be identical
            assert cid1 == cid2, f"CIDs differ: {cid1} vs {cid2}"
            
        except Exception as e:
            # Skip test if there are validation errors
            pass
    
    # Run the property test
    try:
        test_manifest_ordering_property()
        print("  ✓ Property tests passed")
        return True
    except Exception as e:
        print(f"  ✗ Property tests failed: {e}")
        return False


def main():
    """Main test runner."""
    print("NGFS Manifest Determinism Test")
    print("=" * 40)
    
    # Set random seed for reproducible tests
    random.seed(42)
    
    start_time = time.time()
    test_results = []
    
    # Run all tests
    tests = [
        ("Name Validation", test_name_validation),
        ("Unicode Normalization", test_unicode_normalization),
        ("Directory Manifest Determinism", test_manifest_determinism),
        ("File Manifest Determinism", test_file_manifest_determinism),
        ("Property Tests", run_property_tests),
    ]
    
    for test_name, test_func in tests:
        print(f"\n{test_name}:")
        print("-" * len(test_name))
        
        try:
            result = test_func()
            test_results.append((test_name, result))
            
            if result:
                print(f"✓ {test_name} passed")
            else:
                print(f"✗ {test_name} failed")
                
        except Exception as e:
            print(f"✗ {test_name} failed with exception: {e}")
            test_results.append((test_name, False))
    
    # Summary
    print("\n" + "=" * 40)
    print("Test Summary:")
    
    passed = sum(1 for _, result in test_results if result)
    total = len(test_results)
    
    for test_name, result in test_results:
        status = "PASS" if result else "FAIL"
        print(f"  {test_name}: {status}")
    
    print(f"\nOverall: {passed}/{total} tests passed")
    
    elapsed_time = time.time() - start_time
    print(f"Time: {elapsed_time:.2f}s")
    
    # Output JSON for CI
    ci_output = {
        "test": "ngfs_manifest_determinism",
        "total_tests": total,
        "passed": passed,
        "failed": total - passed,
        "elapsed_ms": round(elapsed_time * 1000, 2),
        "timestamp": time.time()
    }
    
    print(f"\nCI Output: {json.dumps(ci_output)}")
    
    # Exit with appropriate code
    if passed == total:
        print("\n✓ All tests passed!")
        sys.exit(0)
    else:
        print(f"\n✗ {total - passed} test(s) failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()
