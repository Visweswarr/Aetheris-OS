#!/usr/bin/env python3
"""
Integrity Sentinel for P4-03-A2
Prevents accidental overwrites of existing P4-03-A1 work
"""

import os
import sys
import hashlib
import json
from pathlib import Path
from typing import Dict, Set, List

class IntegritySentinel:
    def __init__(self, workspace_root: str):
        self.workspace_root = Path(workspace_root)
        self.baseline_file = self.workspace_root / "perf/baselines/p4_03_a1_integrity.json"
        self.protected_paths = {
            "services/net/posixnet/src/broker.rs",
            "services/net/posixnet/src/socket.rs", 
            "services/net/posixnet/src/poll.rs",
            "services/net/posixnet/src/ns.rs",
            "services/net/posixnet/src/readiness.rs",
            "services/net/posixnet/src/syscall.rs",
            "services/net/posixnet/src/audit.rs",
            "services/net/posixnet/src/broker_simple.rs",
            "c/libc_aetheris/src/net.c",
            "c/libc_aetheris/include/aetheris/net_broker_v1.h",
            "go/tooling/netctl/main.go",
            "go/tooling/netctl/echo.go",
            "rust/crates/aetheris-net/src/lib.rs",
            "tooling/ts/node-posix-bridge/src/net.ts",
            "policy/net/net.rego",
            "policy/net/policy.wasm",
            "docs/phase-4/P4-03-A1-SOCKETS.md",
            ".github/workflows/p4-03-net.yml",
            "perf/baselines/p4_03_net.json"
        }
        
    def calculate_file_hash(self, file_path: Path) -> str:
        """Calculate SHA-256 hash of file content"""
        if not file_path.exists():
            return "FILE_NOT_FOUND"
        
        with open(file_path, 'rb') as f:
            content = f.read()
        return hashlib.sha256(content).hexdigest()
    
    def load_baseline(self) -> Dict[str, str]:
        """Load baseline file hashes"""
        if not self.baseline_file.exists():
            return {}
        
        try:
            with open(self.baseline_file, 'r') as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError):
            return {}
    
    def save_baseline(self, hashes: Dict[str, str]):
        """Save baseline file hashes"""
        self.baseline_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self.baseline_file, 'w') as f:
            json.dump(hashes, f, indent=2)
    
    def check_integrity(self) -> bool:
        """Check if protected files have been modified"""
        baseline = self.load_baseline()
        violations = []
        
        for rel_path in self.protected_paths:
            full_path = self.workspace_root / rel_path
            current_hash = self.calculate_file_hash(full_path)
            
            if rel_path in baseline:
                if baseline[rel_path] != current_hash:
                    violations.append(f"Modified: {rel_path}")
            else:
                # New file, add to baseline
                baseline[rel_path] = current_hash
        
        if violations:
            print("❌ INTEGRITY VIOLATION DETECTED!")
            print("The following P4-03-A1 files have been modified:")
            for violation in violations:
                print(f"  - {violation}")
            print()
            print("This could indicate accidental overwrite of existing work.")
            print("If this is intentional, set REBASELINE_A2=1 to rebaseline.")
            return False
        
        # Update baseline with any new files
        self.save_baseline(baseline)
        print("✅ Integrity check passed - no protected files modified")
        return True
    
    def rebaseline(self):
        """Rebaseline all protected files"""
        print("🔄 Rebaselining protected files...")
        hashes = {}
        
        for rel_path in self.protected_paths:
            full_path = self.workspace_root / rel_path
            hashes[rel_path] = self.calculate_file_hash(full_path)
        
        self.save_baseline(hashes)
        print(f"✅ Rebaselined {len(hashes)} files")
    
    def list_expected_a2_files(self) -> List[str]:
        """List files expected to be created/modified in A2"""
        return [
            "services/net/posixnet/src/tls.rs",
            "services/net/posixnet/src/metrics.rs", 
            "services/net/quic/src/quicd.rs",
            "services/net/quic/src/bindings.rs",
            "services/identity/src/certops.rs",
            "rust/crates/aetheris-net/src/tls.rs",
            "rust/crates/aetheris-net/src/quic.rs",
            "c/libc_aetheris/include/aetheris/tls_broker_v1.h",
            "c/libc_aetheris/src/tls.c",
            "go/tooling/netctl/tls.go",
            "go/tooling/netctl/quic.go",
            "tooling/ts/node-posix-bridge/src/tls.ts",
            "tooling/ts/node-posix-bridge/src/quic.ts",
            "tooling/python/tls_quic_check.py",
            "policy/net/tls.rego",
            "policy/net/quic.rego",
            "policy/net/tls.wasm",
            "policy/net/quic.wasm",
            "docs/phase-4/P4-03-A2-TLS-QUIC.md",
            ".github/workflows/p4-03-a2-tls-quic.yml",
            "perf/baselines/p4_03_a2.json"
        ]

def main():
    if len(sys.argv) < 2:
        print("Usage: integrity-sentinel.py <workspace_root> [rebaseline]")
        sys.exit(1)
    
    workspace_root = sys.argv[1]
    sentinel = IntegritySentinel(workspace_root)
    
    if len(sys.argv) > 2 and sys.argv[2] == "rebaseline":
        sentinel.rebaseline()
        return
    
    # Check if rebaseline is requested via environment
    if os.environ.get("REBASELINE_A2") == "1":
        print("🔄 REBASELINE_A2=1 detected, rebaselining...")
        sentinel.rebaseline()
        return
    
    # Run integrity check
    if not sentinel.check_integrity():
        print()
        print("Expected A2 files that should be created/modified:")
        for file_path in sentinel.list_expected_a2_files():
            print(f"  - {file_path}")
        sys.exit(1)
    
    print("✅ Pre-flight protection passed - safe to proceed with A2 implementation")

if __name__ == "__main__":
    main()
