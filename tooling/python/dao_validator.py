#!/usr/bin/env python3
"""
DAO Validator - Python validation framework for Aetheris OS DAO operations

This module provides comprehensive validation for DAO operations including
proposal creation, voting, execution, and governance management.
"""

import json
import time
import uuid
import hashlib
import asyncio
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict
from enum import Enum
import aiohttp
import pytest

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ProposalType(Enum):
    ADD_SKILL = "add-skill"
    UPDATE_POLICY = "update-policy"
    CHANGE_GOVERNANCE = "change-governance"
    EXECUTE_CODE = "execute-code"
    TRANSFER_FUNDS = "transfer-funds"
    CUSTOM = "custom"

class ProposalStatus(Enum):
    DRAFT = "draft"
    ACTIVE = "active"
    VOTING_ENDED = "voting-ended"
    PASSED = "passed"
    FAILED = "failed"
    EXECUTED = "executed"
    CANCELLED = "cancelled"

class VoteChoice(Enum):
    YES = "yes"
    NO = "no"
    ABSTAIN = "abstain"

class ExecutionMethod(Enum):
    CONTRACT_CALL = "contract-call"
    SYSTEM_CALL = "system-call"
    POLICY_UPDATE = "policy-update"
    CONFIG_CHANGE = "config-change"
    CUSTOM = "custom"

class VotingPowerMethod(Enum):
    ONE_VOTE_PER_ADDRESS = "one-vote-per-address"
    TOKEN_BALANCE = "token-balance"
    REPUTATION_SCORE = "reputation-score"
    STAKE_AMOUNT = "stake-amount"
    CUSTOM = "custom"

class MemberStatus(Enum):
    ACTIVE = "active"
    SUSPENDED = "suspended"
    BANNED = "banned"
    INACTIVE = "inactive"

@dataclass
class ExecutionData:
    target: str
    parameters: Dict[str, Any]
    method: ExecutionMethod
    gas_limit: Optional[int] = None
    value: Optional[int] = None

@dataclass
class Proposal:
    id: str
    title: str
    description: str
    proposal_type: ProposalType
    start_time: datetime
    end_time: datetime
    proposer: str
    proposer_did: str
    status: ProposalStatus
    execution_data: Optional[ExecutionData] = None
    created_at: datetime = None
    updated_at: datetime = None

    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.utcnow()
        if self.updated_at is None:
            self.updated_at = datetime.utcnow()

@dataclass
class Vote:
    id: str
    proposal_id: str
    voter: str
    voter_did: str
    choice: VoteChoice
    weight: int
    reason: Optional[str] = None
    signature: str = ""
    timestamp: datetime = None

    def __post_init__(self):
        if self.timestamp is None:
            self.timestamp = datetime.utcnow()

@dataclass
class VoteResult:
    proposal_id: str
    total_votes: int
    yes_votes: int
    no_votes: int
    abstain_votes: int
    total_power: int
    quorum_achieved: bool
    majority_achieved: bool
    result: str
    calculated_at: datetime

@dataclass
class ExecutionResult:
    proposal_id: str
    success: bool
    output: Optional[str] = None
    error: Optional[str] = None
    gas_used: Optional[int] = None
    tx_hash: Optional[str] = None
    executed_at: datetime = None
    executor: str = ""

    def __post_init__(self):
        if self.executed_at is None:
            self.executed_at = datetime.utcnow()

@dataclass
class DAOMember:
    address: str
    did: str
    reputation: int
    stake: int
    voting_power: int
    status: MemberStatus
    joined_at: datetime = None
    last_activity: datetime = None

    def __post_init__(self):
        if self.joined_at is None:
            self.joined_at = datetime.utcnow()
        if self.last_activity is None:
            self.last_activity = datetime.utcnow()

@dataclass
class GovernanceParams:
    min_voting_period: int
    max_voting_period: int
    min_votes_required: int
    majority_threshold: int
    quorum_threshold: int
    execution_delay: int
    proposal_deposit: int
    voting_power_method: VotingPowerMethod

@dataclass
class DAOStats:
    total_proposals: int
    active_proposals: int
    passed_proposals: int
    failed_proposals: int
    executed_proposals: int
    total_votes: int
    total_members: int
    active_members: int
    total_voting_power: int
    last_updated: datetime

class DAOValidator:
    """Main DAO validation class"""
    
    def __init__(self, base_url: str = "http://localhost:8080", api_key: Optional[str] = None):
        self.base_url = base_url
        self.api_key = api_key
        self.session: Optional[aiohttp.ClientSession] = None
        self.test_results: List[Dict[str, Any]] = []
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}
        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def make_request(self, method: str, path: str, data: Optional[Dict] = None) -> Dict[str, Any]:
        """Make HTTP request to DAO API"""
        url = f"{self.base_url}{path}"
        
        try:
            async with self.session.request(
                method=method,
                url=url,
                json=data,
                timeout=aiohttp.ClientTimeout(total=30)
            ) as response:
                if response.status >= 400:
                    raise Exception(f"HTTP {response.status}: {await response.text()}")
                return await response.json()
        except Exception as e:
            logger.error(f"Request failed: {e}")
            raise

    def generate_mock_data(self) -> Dict[str, Any]:
        """Generate mock data for testing"""
        return {
            "proposal_id": str(uuid.uuid4()),
            "title": "Test Proposal",
            "description": "This is a test proposal for validation",
            "proposal_type": ProposalType.ADD_SKILL.value,
            "proposer": "0x1234567890123456789012345678901234567890",
            "proposer_did": "did:key:test",
            "voter": "0x2345678901234567890123456789012345678901",
            "voter_did": "did:key:voter",
            "executor": "0x3456789012345678901234567890123456789012",
        }

    async def validate_proposal_creation(self) -> Dict[str, Any]:
        """Validate proposal creation functionality"""
        logger.info("Testing proposal creation...")
        
        mock_data = self.generate_mock_data()
        
        # Test valid proposal creation
        proposal_data = {
            "title": mock_data["title"],
            "description": mock_data["description"],
            "proposalType": mock_data["proposal_type"],
            "votingPeriod": 604800,  # 7 days
            "proposer": mock_data["proposer"],
            "proposerDID": mock_data["proposer_did"],
        }
        
        try:
            response = await self.make_request("POST", "/api/dao/proposals", proposal_data)
            
            # Validate response structure
            required_fields = ["id", "title", "description", "status", "createdAt"]
            for field in required_fields:
                assert field in response, f"Missing required field: {field}"
            
            # Validate proposal status
            assert response["status"] == ProposalStatus.DRAFT.value
            
            # Validate timestamps
            created_at = datetime.fromisoformat(response["createdAt"].replace("Z", "+00:00"))
            assert (datetime.utcnow() - created_at).total_seconds() < 60
            
            return {
                "test": "proposal_creation",
                "status": "passed",
                "proposal_id": response["id"],
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "proposal_creation",
                "status": "failed",
                "error": str(e)
            }

    async def validate_proposal_activation(self, proposal_id: str) -> Dict[str, Any]:
        """Validate proposal activation functionality"""
        logger.info("Testing proposal activation...")
        
        try:
            response = await self.make_request("POST", f"/api/dao/proposals/{proposal_id}/activate")
            
            # Verify proposal is now active
            proposal = await self.make_request("GET", f"/api/dao/proposals/{proposal_id}")
            assert proposal["status"] == ProposalStatus.ACTIVE.value
            
            return {
                "test": "proposal_activation",
                "status": "passed",
                "proposal_id": proposal_id,
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "proposal_activation",
                "status": "failed",
                "error": str(e)
            }

    async def validate_voting(self, proposal_id: str) -> Dict[str, Any]:
        """Validate voting functionality"""
        logger.info("Testing voting...")
        
        mock_data = self.generate_mock_data()
        
        # Test casting a vote
        vote_data = {
            "proposalId": proposal_id,
            "choice": VoteChoice.YES.value,
            "voter": mock_data["voter"],
            "voterDID": mock_data["voter_did"],
            "weight": 1,
            "reason": "Test vote"
        }
        
        try:
            response = await self.make_request("POST", "/api/dao/votes", vote_data)
            
            # Validate vote response
            required_fields = ["id", "proposalId", "choice", "voter", "timestamp"]
            for field in required_fields:
                assert field in response, f"Missing required field: {field}"
            
            # Verify vote was recorded
            votes = await self.make_request("GET", f"/api/dao/proposals/{proposal_id}/votes")
            assert len(votes["votes"]) > 0
            
            return {
                "test": "voting",
                "status": "passed",
                "vote_id": response["id"],
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "voting",
                "status": "failed",
                "error": str(e)
            }

    async def validate_vote_calculation(self, proposal_id: str) -> Dict[str, Any]:
        """Validate vote result calculation"""
        logger.info("Testing vote calculation...")
        
        try:
            response = await self.make_request("GET", f"/api/dao/proposals/{proposal_id}/result")
            
            # Validate vote result structure
            required_fields = ["proposalId", "totalVotes", "yesVotes", "noVotes", "abstainVotes", "result"]
            for field in required_fields:
                assert field in response, f"Missing required field: {field}"
            
            # Validate vote counts
            assert response["totalVotes"] >= 0
            assert response["yesVotes"] >= 0
            assert response["noVotes"] >= 0
            assert response["abstainVotes"] >= 0
            assert response["totalVotes"] == response["yesVotes"] + response["noVotes"] + response["abstainVotes"]
            
            return {
                "test": "vote_calculation",
                "status": "passed",
                "proposal_id": proposal_id,
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "vote_calculation",
                "status": "failed",
                "error": str(e)
            }

    async def validate_proposal_execution(self, proposal_id: str) -> Dict[str, Any]:
        """Validate proposal execution functionality"""
        logger.info("Testing proposal execution...")
        
        mock_data = self.generate_mock_data()
        
        execution_data = {
            "proposalId": proposal_id,
            "executor": mock_data["executor"]
        }
        
        try:
            response = await self.make_request("POST", "/api/dao/execute", execution_data)
            
            # Validate execution response
            required_fields = ["proposalId", "success", "executedAt", "executor"]
            for field in required_fields:
                assert field in response, f"Missing required field: {field}"
            
            # Verify proposal status changed
            proposal = await self.make_request("GET", f"/api/dao/proposals/{proposal_id}")
            assert proposal["status"] in [ProposalStatus.EXECUTED.value, ProposalStatus.FAILED.value]
            
            return {
                "test": "proposal_execution",
                "status": "passed",
                "proposal_id": proposal_id,
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "proposal_execution",
                "status": "failed",
                "error": str(e)
            }

    async def validate_governance_params(self) -> Dict[str, Any]:
        """Validate governance parameters functionality"""
        logger.info("Testing governance parameters...")
        
        try:
            # Get current governance parameters
            response = await self.make_request("GET", "/api/dao/governance/params")
            
            # Validate governance parameters structure
            required_fields = [
                "minVotingPeriod", "maxVotingPeriod", "minVotesRequired",
                "majorityThreshold", "quorumThreshold", "executionDelay"
            ]
            for field in required_fields:
                assert field in response, f"Missing required field: {field}"
            
            # Validate parameter ranges
            assert response["minVotingPeriod"] > 0
            assert response["maxVotingPeriod"] > response["minVotingPeriod"]
            assert response["minVotesRequired"] > 0
            assert 50 <= response["majorityThreshold"] <= 100
            assert 1 <= response["quorumThreshold"] <= 100
            
            return {
                "test": "governance_params",
                "status": "passed",
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "governance_params",
                "status": "failed",
                "error": str(e)
            }

    async def validate_dao_stats(self) -> Dict[str, Any]:
        """Validate DAO statistics functionality"""
        logger.info("Testing DAO statistics...")
        
        try:
            response = await self.make_request("GET", "/api/dao/stats")
            
            # Validate stats structure
            required_fields = [
                "totalProposals", "activeProposals", "passedProposals",
                "failedProposals", "executedProposals", "totalVotes",
                "totalMembers", "activeMembers", "totalVotingPower"
            ]
            for field in required_fields:
                assert field in response, f"Missing required field: {field}"
            
            # Validate stats consistency
            assert response["totalProposals"] >= 0
            assert response["activeProposals"] >= 0
            assert response["passedProposals"] >= 0
            assert response["failedProposals"] >= 0
            assert response["executedProposals"] >= 0
            assert response["totalVotes"] >= 0
            assert response["totalMembers"] >= 0
            assert response["activeMembers"] >= 0
            assert response["totalVotingPower"] >= 0
            
            return {
                "test": "dao_stats",
                "status": "passed",
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "dao_stats",
                "status": "failed",
                "error": str(e)
            }

    async def validate_member_management(self) -> Dict[str, Any]:
        """Validate member management functionality"""
        logger.info("Testing member management...")
        
        mock_data = self.generate_mock_data()
        
        # Test member creation
        member_data = {
            "address": mock_data["voter"],
            "did": mock_data["voter_did"],
            "reputation": 100,
            "stake": 1000,
            "votingPower": 1,
            "status": MemberStatus.ACTIVE.value
        }
        
        try:
            # Add member
            response = await self.make_request("POST", "/api/dao/members", member_data)
            
            # Verify member was added
            member = await self.make_request("GET", f"/api/dao/members/{mock_data['voter']}")
            assert member["address"] == mock_data["voter"]
            assert member["did"] == mock_data["voter_did"]
            
            # Test member update
            update_data = {"reputation": 150}
            await self.make_request("PUT", f"/api/dao/members/{mock_data['voter']}", update_data)
            
            # Verify update
            updated_member = await self.make_request("GET", f"/api/dao/members/{mock_data['voter']}")
            assert updated_member["reputation"] == 150
            
            return {
                "test": "member_management",
                "status": "passed",
                "member_address": mock_data["voter"],
                "response": response
            }
            
        except Exception as e:
            return {
                "test": "member_management",
                "status": "failed",
                "error": str(e)
            }

    async def validate_deterministic_operations(self) -> Dict[str, Any]:
        """Validate deterministic operations"""
        logger.info("Testing deterministic operations...")
        
        try:
            # Test that identical operations produce identical results
            mock_data = self.generate_mock_data()
            
            # Create two identical proposals
            proposal_data = {
                "title": "Deterministic Test",
                "description": "Testing deterministic behavior",
                "proposalType": ProposalType.ADD_SKILL.value,
                "votingPeriod": 3600,
                "proposer": mock_data["proposer"],
                "proposerDID": mock_data["proposer_did"],
            }
            
            response1 = await self.make_request("POST", "/api/dao/proposals", proposal_data)
            response2 = await self.make_request("POST", "/api/dao/proposals", proposal_data)
            
            # Verify both proposals have valid structure
            assert "id" in response1
            assert "id" in response2
            assert response1["title"] == response2["title"]
            assert response1["description"] == response2["description"]
            
            return {
                "test": "deterministic_operations",
                "status": "passed",
                "proposal1_id": response1["id"],
                "proposal2_id": response2["id"]
            }
            
        except Exception as e:
            return {
                "test": "deterministic_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_performance(self) -> Dict[str, Any]:
        """Validate performance requirements"""
        logger.info("Testing performance...")
        
        try:
            start_time = time.time()
            
            # Test multiple concurrent operations
            tasks = []
            for i in range(10):
                task = self.make_request("GET", "/api/dao/stats")
                tasks.append(task)
            
            results = await asyncio.gather(*tasks)
            end_time = time.time()
            
            # Verify all requests succeeded
            assert len(results) == 10
            for result in results:
                assert "totalProposals" in result
            
            # Check performance (should complete within reasonable time)
            duration = end_time - start_time
            assert duration < 5.0, f"Performance test took too long: {duration}s"
            
            return {
                "test": "performance",
                "status": "passed",
                "duration": duration,
                "requests": len(results)
            }
            
        except Exception as e:
            return {
                "test": "performance",
                "status": "failed",
                "error": str(e)
            }

    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all validation tests"""
        logger.info("Starting comprehensive DAO validation...")
        
        test_results = []
        
        # Test proposal creation
        result = await self.validate_proposal_creation()
        test_results.append(result)
        
        if result["status"] == "passed":
            proposal_id = result["proposal_id"]
            
            # Test proposal activation
            result = await self.validate_proposal_activation(proposal_id)
            test_results.append(result)
            
            # Test voting
            result = await self.validate_voting(proposal_id)
            test_results.append(result)
            
            # Test vote calculation
            result = await self.validate_vote_calculation(proposal_id)
            test_results.append(result)
            
            # Test proposal execution (may fail if voting period hasn't ended)
            result = await self.validate_proposal_execution(proposal_id)
            test_results.append(result)
        
        # Test governance parameters
        result = await self.validate_governance_params()
        test_results.append(result)
        
        # Test DAO statistics
        result = await self.validate_dao_stats()
        test_results.append(result)
        
        # Test member management
        result = await self.validate_member_management()
        test_results.append(result)
        
        # Test deterministic operations
        result = await self.validate_deterministic_operations()
        test_results.append(result)
        
        # Test performance
        result = await self.validate_performance()
        test_results.append(result)
        
        # Calculate summary
        passed_tests = sum(1 for r in test_results if r["status"] == "passed")
        failed_tests = sum(1 for r in test_results if r["status"] == "failed")
        total_tests = len(test_results)
        
        summary = {
            "total_tests": total_tests,
            "passed_tests": passed_tests,
            "failed_tests": failed_tests,
            "success_rate": (passed_tests / total_tests) * 100 if total_tests > 0 else 0,
            "test_results": test_results,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        logger.info(f"Validation complete: {passed_tests}/{total_tests} tests passed")
        return summary

    def save_results(self, results: Dict[str, Any], filename: str = "dao_validation_results.json"):
        """Save validation results to file"""
        with open(filename, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        logger.info(f"Results saved to {filename}")

async def main():
    """Main validation function"""
    async with DAOValidator() as validator:
        results = await validator.run_all_tests()
        validator.save_results(results)
        
        # Print summary
        print(f"\n=== DAO Validation Results ===")
        print(f"Total Tests: {results['total_tests']}")
        print(f"Passed: {results['passed_tests']}")
        print(f"Failed: {results['failed_tests']}")
        print(f"Success Rate: {results['success_rate']:.1f}%")
        
        # Print failed tests
        failed_tests = [r for r in results['test_results'] if r['status'] == 'failed']
        if failed_tests:
            print(f"\nFailed Tests:")
            for test in failed_tests:
                print(f"  - {test['test']}: {test.get('error', 'Unknown error')}")
        
        return results['success_rate'] == 100.0

if __name__ == "__main__":
    success = asyncio.run(main())
    exit(0 if success else 1)
