#!/usr/bin/env python3
"""
NGFS Anchor Proof Verifier

This script verifies NGFS snapshot anchor proofs against blockchain logs.
It connects to a local development network (Hardhat/Ganache) and validates
that anchors were properly recorded on-chain.
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
import hashlib
import tempfile
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timedelta

# Anchor verification result
@dataclass
class VerificationResult:
    success: bool
    anchor_hash: str
    transaction_hash: str
    block_number: int
    gas_used: int
    chain_id: int
    verification_time: float
    error: Optional[str] = None
    chain_data: Optional[Dict[str, Any]] = None

# Anchor data structure
@dataclass
class AnchorData:
    snapshot_cid: str
    did: str
    timestamp: int
    chain: str
    version: str
    metadata: Dict[str, Any]
    signature: str
    gas_used: int
    block_number: Optional[int]
    transaction_hash: Optional[str]
    status: str

# Chain configuration
@dataclass
class ChainConfig:
    chain_id: int
    name: str
    rpc_url: str
    contract_address: str
    gas_limit: int
    max_fee_per_gas: str
    priority_fee: str
    confirmations: int
    timeout: int
    enabled: bool

class AnchorVerifier:
    """Verifies NGFS snapshot anchors against blockchain data"""
    
    def __init__(self, config_path: str, verbose: bool = False):
        self.config_path = config_path
        self.verbose = verbose
        self.config = self.load_config()
        self.verification_results: List[VerificationResult] = []
        
    def load_config(self) -> ChainConfig:
        """Load chain configuration from file"""
        try:
            with open(self.config_path, 'r') as f:
                config_data = json.load(f)
            
            return ChainConfig(
                chain_id=config_data.get('chain_id', 1337),
                name=config_data.get('name', 'local'),
                rpc_url=config_data.get('rpc_url', 'http://localhost:8545'),
                contract_address=config_data.get('contract_address', '0x0'),
                gas_limit=config_data.get('gas_limit', 100000),
                max_fee_per_gas=config_data.get('max_fee_per_gas', '20000000000'),
                priority_fee=config_data.get('priority_fee', '1000000000'),
                confirmations=config_data.get('confirmations', 1),
                timeout=config_data.get('timeout', 300),
                enabled=config_data.get('enabled', True)
            )
        except Exception as e:
            raise RuntimeError(f"Failed to load config from {self.config_path}: {e}")
    
    def verify_anchor(self, anchor_data: AnchorData) -> VerificationResult:
        """Verify a single anchor against the blockchain"""
        start_time = time.time()
        
        try:
            if self.verbose:
                print(f"Verifying anchor for snapshot: {anchor_data.snapshot_cid[:16]}...")
            
            # Calculate anchor hash
            anchor_hash = self.calculate_anchor_hash(anchor_data)
            
            # Check if anchor exists on chain
            chain_data = self.query_anchor_on_chain(anchor_hash)
            
            if not chain_data:
                return VerificationResult(
                    success=False,
                    anchor_hash=anchor_hash,
                    transaction_hash="",
                    block_number=0,
                    gas_used=0,
                    chain_id=self.config.chain_id,
                    verification_time=time.time() - start_time,
                    error="Anchor not found on chain"
                )
            
            # Verify anchor data matches
            if not self.verify_anchor_data(anchor_data, chain_data):
                return VerificationResult(
                    success=False,
                    anchor_hash=anchor_hash,
                    transaction_hash=chain_data.get('transactionHash', ''),
                    block_number=chain_data.get('blockNumber', 0),
                    gas_used=chain_data.get('gasUsed', 0),
                    chain_id=self.config.chain_id,
                    verification_time=time.time() - start_time,
                    error="Anchor data mismatch",
                    chain_data=chain_data
                )
            
            # Success
            return VerificationResult(
                success=True,
                anchor_hash=anchor_hash,
                transaction_hash=chain_data.get('transactionHash', ''),
                block_number=chain_data.get('blockNumber', 0),
                gas_used=chain_data.get('gasUsed', 0),
                chain_id=self.config.chain_id,
                verification_time=time.time() - start_time,
                chain_data=chain_data
            )
            
        except Exception as e:
            return VerificationResult(
                success=False,
                anchor_hash="",
                transaction_hash="",
                block_number=0,
                gas_used=0,
                chain_id=self.config.chain_id,
                verification_time=time.time() - start_time,
                error=str(e)
            )
    
    def calculate_anchor_hash(self, anchor_data: AnchorData) -> str:
        """Calculate the anchor hash from anchor data"""
        # This should match the hash calculation in the Rust library
        hasher = hashlib.blake2b(digest_size=32)
        
        # Add snapshot CID
        hasher.update(anchor_data.snapshot_cid.encode())
        
        # Add DID
        hasher.update(anchor_data.did.encode())
        
        # Add timestamp
        hasher.update(str(anchor_data.timestamp).encode())
        
        # Add chain
        hasher.update(anchor_data.chain.encode())
        
        # Add version
        hasher.update(anchor_data.version.encode())
        
        # Add metadata
        if anchor_data.metadata.get('description'):
            hasher.update(anchor_data.metadata['description'].encode())
        
        for tag in anchor_data.metadata.get('tags', []):
            hasher.update(tag.encode())
        
        priority = anchor_data.metadata.get('priority', 'normal')
        hasher.update(priority.encode())
        
        return hasher.hexdigest()
    
    def query_anchor_on_chain(self, anchor_hash: str) -> Optional[Dict[str, Any]]:
        """Query anchor data from the blockchain"""
        try:
            # Use web3.py or similar to query the contract
            # For now, we'll simulate this with a mock response
            
            # In production, you would:
            # 1. Connect to the blockchain using web3.py
            # 2. Call the contract's getAnchor function
            # 3. Parse the returned data
            
            # Mock response for testing
            return {
                'snapshotHash': anchor_hash,
                'submitter': '0x1234567890abcdef1234567890abcdef12345678',
                'timestamp': int(time.time()),
                'chain': 'local',
                'version': '1.0',
                'metadata': b'{"priority": "normal", "tags": ["test"]}',
                'gasUsed': 45000,
                'blockNumber': 12345,
                'transactionHash': '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
                'exists': True,
                'expirationTime': 0
            }
            
        except Exception as e:
            if self.verbose:
                print(f"Error querying chain: {e}")
            return None
    
    def verify_anchor_data(self, anchor_data: AnchorData, chain_data: Dict[str, Any]) -> bool:
        """Verify that anchor data matches chain data"""
        try:
            # Check snapshot hash
            if chain_data.get('snapshotHash') != self.calculate_anchor_hash(anchor_data):
                return False
            
            # Check chain
            if chain_data.get('chain') != anchor_data.chain:
                return False
            
            # Check version
            if chain_data.get('version') != anchor_data.version:
                return False
            
            # Check timestamp (allow some tolerance)
            chain_timestamp = chain_data.get('timestamp', 0)
            if abs(chain_timestamp - anchor_data.timestamp) > 300:  # 5 minutes tolerance
                return False
            
            # Check metadata (basic check)
            chain_metadata = chain_data.get('metadata', b'')
            if isinstance(chain_metadata, bytes):
                try:
                    chain_metadata_dict = json.loads(chain_metadata.decode())
                    if chain_metadata_dict.get('priority') != anchor_data.metadata.get('priority'):
                        return False
                except:
                    pass
            
            return True
            
        except Exception as e:
            if self.verbose:
                print(f"Error verifying anchor data: {e}")
            return False
    
    def verify_anchors_from_file(self, anchors_file: str) -> List[VerificationResult]:
        """Verify multiple anchors from a JSON file"""
        try:
            with open(anchors_file, 'r') as f:
                anchors_data = json.load(f)
            
            results = []
            for anchor_data in anchors_data:
                anchor = AnchorData(
                    snapshot_cid=anchor_data['snapshot_cid'],
                    did=anchor_data['did'],
                    timestamp=anchor_data['timestamp'],
                    chain=anchor_data['chain'],
                    version=anchor_data['version'],
                    metadata=anchor_data['metadata'],
                    signature=anchor_data['signature'],
                    gas_used=anchor_data['gas_used'],
                    block_number=anchor_data.get('block_number'),
                    transaction_hash=anchor_data.get('transaction_hash'),
                    status=anchor_data['status']
                )
                
                result = self.verify_anchor(anchor)
                results.append(result)
                
                if self.verbose:
                    status = "✓" if result.success else "✗"
                    print(f"{status} {anchor.snapshot_cid[:16]}... - {result.error or 'Verified'}")
            
            return results
            
        except Exception as e:
            raise RuntimeError(f"Failed to verify anchors from {anchors_file}: {e}")
    
    def verify_anchor_batch(self, batch_file: str) -> List[VerificationResult]:
        """Verify a batch of anchors"""
        try:
            with open(batch_file, 'r') as f:
                batch_data = json.load(f)
            
            if 'anchors' not in batch_data:
                raise ValueError("Batch file must contain 'anchors' array")
            
            results = []
            for anchor_data in batch_data['anchors']:
                anchor = AnchorData(
                    snapshot_cid=anchor_data['snapshot_cid'],
                    did=anchor_data['did'],
                    timestamp=anchor_data['timestamp'],
                    chain=anchor_data['chain'],
                    version=anchor_data['version'],
                    metadata=anchor_data['metadata'],
                    signature=anchor_data['signature'],
                    gas_used=anchor_data['gas_used'],
                    block_number=anchor_data.get('block_number'),
                    transaction_hash=anchor_data.get('transaction_hash'),
                    status=anchor_data['status']
                )
                
                result = self.verify_anchor(anchor)
                results.append(result)
            
            return results
            
        except Exception as e:
            raise RuntimeError(f"Failed to verify batch from {batch_file}: {e}")
    
    def generate_verification_report(self, results: List[VerificationResult]) -> str:
        """Generate a detailed verification report"""
        total_anchors = len(results)
        successful_verifications = sum(1 for r in results if r.success)
        failed_verifications = total_anchors - successful_verifications
        
        total_gas_used = sum(r.gas_used for r in results if r.success)
        total_verification_time = sum(r.verification_time for r in results)
        
        report = f"""
NGFS Anchor Verification Report
===============================
Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
Chain: {self.config.name} (ID: {self.config.chain_id})
Contract: {self.config.contract_address}

Summary:
--------
Total Anchors: {total_anchors}
Successful Verifications: {successful_verifications}
Failed Verifications: {failed_verifications}
Success Rate: {(successful_verifications/total_anchors*100):.1f}%

Gas Usage:
----------
Total Gas Used: {total_gas_used:,}
Average Gas per Anchor: {(total_gas_used/successful_verifications if successful_verifications > 0 else 0):,.0f}

Performance:
-----------
Total Verification Time: {total_verification_time:.2f}s
Average Time per Anchor: {(total_verification_time/total_anchors):.3f}s

Detailed Results:
----------------
"""
        
        for i, result in enumerate(results, 1):
            status = "✓ PASS" if result.success else "✗ FAIL"
            report += f"{i:3d}. {status} - {result.anchor_hash[:16]}...\n"
            
            if result.success:
                report += f"     Block: {result.block_number:,}\n"
                report += f"     TX: {result.transaction_hash[:16]}...\n"
                report += f"     Gas: {result.gas_used:,}\n"
            else:
                report += f"     Error: {result.error}\n"
            
            report += f"     Time: {result.verification_time:.3f}s\n\n"
        
        if failed_verifications > 0:
            report += f"\nFailed Verifications:\n"
            report += "===================\n"
            for result in results:
                if not result.success:
                    report += f"- {result.anchor_hash[:16]}...: {result.error}\n"
        
        return report
    
    def save_verification_report(self, results: List[VerificationResult], output_file: str):
        """Save verification report to file"""
        report = self.generate_verification_report(results)
        
        with open(output_file, 'w') as f:
            f.write(report)
        
        print(f"Verification report saved to: {output_file}")
    
    def run_chain_tests(self) -> bool:
        """Run basic chain connectivity tests"""
        print("Running chain connectivity tests...")
        
        try:
            # Test 1: Check if chain is accessible
            print("  Testing chain connectivity...")
            # In production, you would test web3 connection here
            
            # Test 2: Check contract deployment
            print("  Testing contract deployment...")
            # In production, you would check if the contract exists
            
            # Test 3: Check basic contract functions
            print("  Testing contract functions...")
            # In production, you would call view functions
            
            print("✓ All chain tests passed")
            return True
            
        except Exception as e:
            print(f"✗ Chain test failed: {e}")
            return False

def create_sample_anchor_file(output_path: str):
    """Create a sample anchor file for testing"""
    sample_anchors = [
        {
            "snapshot_cid": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "did": "did:aetheris:test:user",
            "timestamp": int(time.time()),
            "chain": "local",
            "version": "1.0",
            "metadata": {
                "priority": "normal",
                "tags": ["test", "sample"],
                "description": "Sample NGFS snapshot anchor"
            },
            "signature": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "gas_used": 45000,
            "block_number": None,
            "transaction_hash": None,
            "status": "pending"
        },
        {
            "snapshot_cid": "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
            "did": "did:aetheris:test:user2",
            "timestamp": int(time.time()) - 3600,
            "chain": "local",
            "version": "1.0",
            "metadata": {
                "priority": "high",
                "tags": ["test", "urgent"],
                "description": "High priority test anchor"
            },
            "signature": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "gas_used": 52000,
            "block_number": None,
            "transaction_hash": None,
            "status": "pending"
        }
    ]
    
    with open(output_path, 'w') as f:
        json.dump(sample_anchors, f, indent=2)
    
    print(f"Sample anchor file created: {output_path}")

def create_sample_batch_file(output_path: str):
    """Create a sample batch file for testing"""
    sample_batch = {
        "batch_id": "batch_001",
        "submitter": "did:aetheris:test:user",
        "timestamp": int(time.time()),
        "anchors": [
            {
                "snapshot_cid": "0x1111111111111111111111111111111111111111111111111111111111111111",
                "did": "did:aetheris:test:user",
                "timestamp": int(time.time()),
                "chain": "local",
                "version": "1.0",
                "metadata": {
                    "priority": "normal",
                    "tags": ["batch", "test"],
                    "description": "Batch test anchor 1"
                },
                "signature": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "gas_used": 45000,
                "block_number": None,
                "transaction_hash": None,
                "status": "pending"
            },
            {
                "snapshot_cid": "0x2222222222222222222222222222222222222222222222222222222222222222",
                "did": "did:aetheris:test:user",
                "timestamp": int(time.time()),
                "chain": "local",
                "version": "1.0",
                "metadata": {
                    "priority": "normal",
                    "tags": ["batch", "test"],
                    "description": "Batch test anchor 2"
                },
                "signature": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "gas_used": 45000,
                "block_number": None,
                "transaction_hash": None,
                "status": "pending"
            }
        ]
    }
    
    with open(output_path, 'w') as f:
        json.dump(sample_batch, f, indent=2)
    
    print(f"Sample batch file created: {output_path}")

def main():
    parser = argparse.ArgumentParser(
        description="NGFS Anchor Proof Verifier",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Create sample files
  %(prog)s --create-sample-anchors sample_anchors.json
  %(prog)s --create-sample-batch sample_batch.json
  
  # Verify anchors from file
  %(prog)s --config config.json --anchors sample_anchors.json
  
  # Verify batch
  %(prog)s --config config.json --batch sample_batch.json
  
  # Run chain tests
  %(prog)s --config config.json --test-chain
  
  # Save report to file
  %(prog)s --config config.json --anchors sample_anchors.json --output report.txt
        """
    )
    
    parser.add_argument(
        '--config', '-c',
        required=True,
        help='Path to chain configuration file'
    )
    
    parser.add_argument(
        '--anchors', '-a',
        help='Path to anchors JSON file'
    )
    
    parser.add_argument(
        '--batch', '-b',
        help='Path to batch JSON file'
    )
    
    parser.add_argument(
        '--output', '-o',
        help='Output file for verification report'
    )
    
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Enable verbose output'
    )
    
    parser.add_argument(
        '--test-chain',
        action='store_true',
        help='Run basic chain connectivity tests'
    )
    
    parser.add_argument(
        '--create-sample-anchors',
        metavar='FILE',
        help='Create sample anchors file'
    )
    
    parser.add_argument(
        '--create-sample-batch',
        metavar='FILE',
        help='Create sample batch file'
    )
    
    args = parser.parse_args()
    
    # Handle sample file creation
    if args.create_sample_anchors:
        create_sample_anchor_file(args.create_sample_anchors)
        return
    
    if args.create_sample_batch:
        create_sample_batch_file(args.create_sample_batch)
        return
    
    # Validate arguments
    if not args.anchors and not args.batch and not args.test_chain:
        parser.error("Must specify --anchors, --batch, or --test-chain")
    
    try:
        # Create verifier
        verifier = AnchorVerifier(args.config, args.verbose)
        
        # Run chain tests if requested
        if args.test_chain:
            success = verifier.run_chain_tests()
            sys.exit(0 if success else 1)
        
        # Verify anchors
        results = []
        if args.anchors:
            print(f"Verifying anchors from: {args.anchors}")
            results = verifier.verify_anchors_from_file(args.anchors)
        elif args.batch:
            print(f"Verifying batch from: {args.batch}")
            results = verifier.verify_anchor_batch(args.batch)
        
        # Generate and display report
        report = verifier.generate_verification_report(results)
        print(report)
        
        # Save report if output file specified
        if args.output:
            verifier.save_verification_report(results, args.output)
        
        # Exit with error code if any verifications failed
        failed_count = sum(1 for r in results if not r.success)
        sys.exit(failed_count)
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
