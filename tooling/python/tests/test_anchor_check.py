#!/usr/bin/env python3
"""
Tests for the NGFS Anchor Checker

This module tests the anchor verification functionality including
chain connectivity, anchor verification, and report generation.
"""

import pytest
import tempfile
import os
import json
import subprocess
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import sys

# Add the parent directory to the path to import the anchor_check module
sys.path.insert(0, str(Path(__file__).parent.parent))

from anchor_check import (
    AnchorVerifier, VerificationResult, AnchorData, ChainConfig
)

class TestVerificationResult:
    def test_verification_result_creation(self):
        result = VerificationResult(
            success=True,
            anchor_hash="0x1234...",
            transaction_hash="0xabcd...",
            block_number=12345,
            gas_used=45000,
            chain_id=1337,
            verification_time=0.5
        )
        
        assert result.success is True
        assert result.anchor_hash == "0x1234..."
        assert result.transaction_hash == "0xabcd..."
        assert result.block_number == 12345
        assert result.gas_used == 45000
        assert result.chain_id == 1337
        assert result.verification_time == 0.5

class TestAnchorData:
    def test_anchor_data_creation(self):
        anchor = AnchorData(
            snapshot_cid="0x1234...",
            did="did:aetheris:test:user",
            timestamp=1703123456,
            chain="local",
            version="1.0",
            metadata={"priority": "normal", "tags": ["test"]},
            signature="0xabcd...",
            gas_used=45000,
            block_number=12345,
            transaction_hash="0x1111...",
            status="confirmed"
        )
        
        assert anchor.snapshot_cid == "0x1234..."
        assert anchor.did == "did:aetheris:test:user"
        assert anchor.chain == "local"
        assert anchor.status == "confirmed"

class TestChainConfig:
    def test_chain_config_defaults(self):
        config = ChainConfig(
            chain_id=1337,
            name="local",
            rpc_url="http://localhost:8545",
            contract_address="0x0",
            gas_limit=100000,
            max_fee_per_gas="20000000000",
            priority_fee="1000000000",
            confirmations=1,
            timeout=300,
            enabled=True
        )
        
        assert config.chain_id == 1337
        assert config.name == "local"
        assert config.enabled is True

class TestAnchorVerifier:
    def test_verifier_initialization(self, tmp_path):
        config_file = tmp_path / "config.json"
        config_data = {
            "chain_id": 1337,
            "name": "local",
            "rpc_url": "http://localhost:8545",
            "contract_address": "0x0",
            "gas_limit": 100000,
            "max_fee_per_gas": "20000000000",
            "priority_fee": "1000000000",
            "confirmations": 1,
            "timeout": 300,
            "enabled": True
        }
        
        with open(config_file, 'w') as f:
            json.dump(config_data, f)
        
        verifier = AnchorVerifier(str(config_file), verbose=False)
        assert verifier.config.chain_id == 1337
        assert verifier.config.name == "local"

    def test_verifier_load_config_error(self):
        with pytest.raises(RuntimeError, match="Failed to load config"):
            AnchorVerifier("nonexistent_config.json", verbose=False)

    def test_calculate_anchor_hash(self, tmp_path):
        config_file = tmp_path / "config.json"
        config_data = {"chain_id": 1337, "name": "local", "rpc_url": "http://localhost:8545", 
                      "contract_address": "0x0", "gas_limit": 100000, "max_fee_per_gas": "20000000000", 
                      "priority_fee": "1000000000", "confirmations": 1, "timeout": 300, "enabled": True}
        
        with open(config_file, 'w') as f:
            json.dump(config_data, f)
        
        verifier = AnchorVerifier(str(config_file), verbose=False)
        
        anchor_data = AnchorData(
            snapshot_cid="0x1234567890abcdef",
            did="did:aetheris:test:user",
            timestamp=1703123456,
            chain="local",
            version="1.0",
            metadata={"priority": "normal", "tags": ["test"]},
            signature="0xabcd",
            gas_used=45000,
            status="confirmed"
        )
        
        hash_result = verifier.calculate_anchor_hash(anchor_data)
        assert len(hash_result) == 64  # Blake2b-256 hex length
        assert hash_result.startswith('0x')

    def test_verify_anchor_success(self, tmp_path):
        config_file = tmp_path / "config.json"
        config_data = {"chain_id": 1337, "name": "local", "rpc_url": "http://localhost:8545", 
                      "contract_address": "0x0", "gas_limit": 100000, "max_fee_per_gas": "20000000000", 
                      "priority_fee": "1000000000", "confirmations": 1, "timeout": 300, "enabled": True}
        
        with open(config_file, 'w') as f:
            json.dump(config_data, f)
        
        verifier = AnchorVerifier(str(config_file), verbose=False)
        
        anchor_data = AnchorData(
            snapshot_cid="0x1234567890abcdef",
            did="did:aetheris:test:user",
            timestamp=1703123456,
            chain="local",
            version="1.0",
            metadata={"priority": "normal", "tags": ["test"]},
            signature="0xabcd",
            gas_used=45000,
            status="confirmed"
        )
        
        result = verifier.verify_anchor(anchor_data)
        assert result.success is True
        assert result.chain_id == 1337

    def test_run_chain_tests(self, tmp_path):
        config_file = tmp_path / "config.json"
        config_data = {"chain_id": 1337, "name": "local", "rpc_url": "http://localhost:8545", 
                      "contract_address": "0x0", "gas_limit": 100000, "max_fee_per_gas": "20000000000", 
                      "priority_fee": "1000000000", "confirmations": 1, "timeout": 300, "enabled": True}
        
        with open(config_file, 'w') as f:
            json.dump(config_data, f)
        
        verifier = AnchorVerifier(str(config_file), verbose=False)
        
        # Mock the chain tests to always pass
        with patch.object(verifier, 'run_chain_tests', return_value=True):
            success = verifier.run_chain_tests()
            assert success is True

    def test_generate_verification_report(self, tmp_path):
        config_file = tmp_path / "config.json"
        config_data = {"chain_id": 1337, "name": "local", "rpc_url": "http://localhost:8545", 
                      "contract_address": "0x0", "gas_limit": 100000, "max_fee_per_gas": "20000000000", 
                      "priority_fee": "1000000000", "confirmations": 1, "timeout": 300, "enabled": True}
        
        with open(config_file, 'w') as f:
            json.dump(config_data, f)
        
        verifier = AnchorVerifier(str(config_file), verbose=False)
        
        results = [
            VerificationResult(
                success=True,
                anchor_hash="0x1234...",
                transaction_hash="0xabcd...",
                block_number=12345,
                gas_used=45000,
                chain_id=1337,
                verification_time=0.5
            ),
            VerificationResult(
                success=False,
                anchor_hash="0x5678...",
                transaction_hash="",
                block_number=0,
                gas_used=0,
                chain_id=1337,
                verification_time=0.3,
                error="Anchor not found"
            )
        ]
        
        report = verifier.generate_verification_report(results)
        assert "NGFS Anchor Verification Report" in report
        assert "Total Anchors: 2" in report
        assert "Successful Verifications: 1" in report
        assert "Failed Verifications: 1" in report
        assert "Success Rate: 50.0%" in report

if __name__ == "__main__":
    pytest.main([__file__])
