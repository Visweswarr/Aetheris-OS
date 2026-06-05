// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title DAOGovernance
 * @dev A contract for DAO governance in Aetheris OS
 * @author Aetheris OS Team
 */
contract DAOGovernance {
    struct Proposal {
        uint256 id;
        string title;
        string description;
        uint256 startTime;
        uint256 endTime;
        uint256 yesVotes;
        uint256 noVotes;
        uint256 abstainVotes;
        bool executed;
        bool cancelled;
        address proposer;
        mapping(address => bool) hasVoted;
        mapping(address => uint8) votes; // 0 = no vote, 1 = yes, 2 = no, 3 = abstain
    }
    
    struct Member {
        address memberAddress;
        uint256 votingPower;
        bool active;
        uint256 joinedAt;
    }
    
    mapping(uint256 => Proposal) public proposals;
    mapping(address => Member) public members;
    mapping(address => bool) public authorizedProposers;
    
    address public owner;
    uint256 public proposalCount;
    uint256 public totalVotingPower;
    uint256 public quorumThreshold; // Percentage (e.g., 25 for 25%)
    uint256 public majorityThreshold; // Percentage (e.g., 51 for 51%)
    uint256 public minVotingPeriod; // Minimum voting period in seconds
    uint256 public maxVotingPeriod; // Maximum voting period in seconds
    
    event ProposalCreated(
        uint256 indexed proposalId,
        string title,
        address indexed proposer,
        uint256 startTime,
        uint256 endTime
    );
    
    event VoteCast(
        uint256 indexed proposalId,
        address indexed voter,
        uint8 vote, // 1 = yes, 2 = no, 3 = abstain
        uint256 votingPower
    );
    
    event ProposalExecuted(uint256 indexed proposalId, address indexed executor);
    event ProposalCancelled(uint256 indexed proposalId, address indexed canceller);
    
    event MemberAdded(address indexed member, uint256 votingPower);
    event MemberRemoved(address indexed member);
    event MemberVotingPowerUpdated(address indexed member, uint256 newVotingPower);
    
    event ProposerAuthorized(address indexed proposer, address indexed authorizedBy);
    event ProposerDeauthorized(address indexed proposer, address indexed deauthorizedBy);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call this function");
        _;
    }
    
    modifier onlyMember() {
        require(members[msg.sender].active, "Only members can call this function");
        _;
    }
    
    modifier onlyAuthorizedProposer() {
        require(
            msg.sender == owner || authorizedProposers[msg.sender],
            "Only authorized proposers can call this function"
        );
        _;
    }
    
    modifier validProposal(uint256 _proposalId) {
        require(_proposalId > 0 && _proposalId <= proposalCount, "Invalid proposal ID");
        _;
    }
    
    /**
     * @dev Constructor initializes the DAO governance
     * @param _quorumThreshold Quorum threshold percentage
     * @param _majorityThreshold Majority threshold percentage
     * @param _minVotingPeriod Minimum voting period in seconds
     * @param _maxVotingPeriod Maximum voting period in seconds
     */
    constructor(
        uint256 _quorumThreshold,
        uint256 _majorityThreshold,
        uint256 _minVotingPeriod,
        uint256 _maxVotingPeriod
    ) {
        owner = msg.sender;
        quorumThreshold = _quorumThreshold;
        majorityThreshold = _majorityThreshold;
        minVotingPeriod = _minVotingPeriod;
        maxVotingPeriod = _maxVotingPeriod;
        
        // Add owner as initial member
        members[msg.sender] = Member({
            memberAddress: msg.sender,
            votingPower: 1000, // Initial voting power
            active: true,
            joinedAt: block.timestamp
        });
        
        totalVotingPower = 1000;
        authorizedProposers[msg.sender] = true;
        
        emit MemberAdded(msg.sender, 1000);
    }
    
    /**
     * @dev Create a new proposal
     * @param _title Proposal title
     * @param _description Proposal description
     * @param _votingPeriod Voting period in seconds
     */
    function createProposal(
        string memory _title,
        string memory _description,
        uint256 _votingPeriod
    ) public onlyAuthorizedProposer returns (uint256) {
        require(bytes(_title).length > 0, "Title cannot be empty");
        require(bytes(_description).length > 0, "Description cannot be empty");
        require(_votingPeriod >= minVotingPeriod, "Voting period too short");
        require(_votingPeriod <= maxVotingPeriod, "Voting period too long");
        
        proposalCount++;
        uint256 proposalId = proposalCount;
        
        Proposal storage proposal = proposals[proposalId];
        proposal.id = proposalId;
        proposal.title = _title;
        proposal.description = _description;
        proposal.startTime = block.timestamp;
        proposal.endTime = block.timestamp + _votingPeriod;
        proposal.yesVotes = 0;
        proposal.noVotes = 0;
        proposal.abstainVotes = 0;
        proposal.executed = false;
        proposal.cancelled = false;
        proposal.proposer = msg.sender;
        
        emit ProposalCreated(proposalId, _title, msg.sender, proposal.startTime, proposal.endTime);
        
        return proposalId;
    }
    
    /**
     * @dev Vote on a proposal
     * @param _proposalId Proposal ID
     * @param _vote Vote choice (1 = yes, 2 = no, 3 = abstain)
     */
    function vote(uint256 _proposalId, uint8 _vote) public onlyMember validProposal(_proposalId) {
        Proposal storage proposal = proposals[_proposalId];
        
        require(block.timestamp >= proposal.startTime, "Voting has not started");
        require(block.timestamp <= proposal.endTime, "Voting has ended");
        require(!proposal.hasVoted[msg.sender], "Already voted on this proposal");
        require(!proposal.executed, "Proposal has been executed");
        require(!proposal.cancelled, "Proposal has been cancelled");
        require(_vote >= 1 && _vote <= 3, "Invalid vote choice");
        
        proposal.hasVoted[msg.sender] = true;
        proposal.votes[msg.sender] = _vote;
        
        uint256 votingPower = members[msg.sender].votingPower;
        
        if (_vote == 1) {
            proposal.yesVotes += votingPower;
        } else if (_vote == 2) {
            proposal.noVotes += votingPower;
        } else if (_vote == 3) {
            proposal.abstainVotes += votingPower;
        }
        
        emit VoteCast(_proposalId, msg.sender, _vote, votingPower);
    }
    
    /**
     * @dev Execute a proposal
     * @param _proposalId Proposal ID
     */
    function executeProposal(uint256 _proposalId) public onlyMember validProposal(_proposalId) {
        Proposal storage proposal = proposals[_proposalId];
        
        require(block.timestamp > proposal.endTime, "Voting period has not ended");
        require(!proposal.executed, "Proposal has already been executed");
        require(!proposal.cancelled, "Proposal has been cancelled");
        
        uint256 totalVotes = proposal.yesVotes + proposal.noVotes + proposal.abstainVotes;
        uint256 quorum = (totalVotes * 100) / totalVotingPower;
        uint256 majority = (proposal.yesVotes * 100) / totalVotes;
        
        require(quorum >= quorumThreshold, "Quorum not reached");
        require(majority >= majorityThreshold, "Majority not reached");
        
        proposal.executed = true;
        
        emit ProposalExecuted(_proposalId, msg.sender);
    }
    
    /**
     * @dev Cancel a proposal (only proposer or owner)
     * @param _proposalId Proposal ID
     */
    function cancelProposal(uint256 _proposalId) public validProposal(_proposalId) {
        Proposal storage proposal = proposals[_proposalId];
        
        require(
            msg.sender == proposal.proposer || msg.sender == owner,
            "Only proposer or owner can cancel proposal"
        );
        require(!proposal.executed, "Cannot cancel executed proposal");
        require(!proposal.cancelled, "Proposal has already been cancelled");
        
        proposal.cancelled = true;
        
        emit ProposalCancelled(_proposalId, msg.sender);
    }
    
    /**
     * @dev Add a new member
     * @param _member Address of the new member
     * @param _votingPower Voting power of the new member
     */
    function addMember(address _member, uint256 _votingPower) public onlyOwner {
        require(_member != address(0), "Member address cannot be zero");
        require(!members[_member].active, "Member already exists");
        require(_votingPower > 0, "Voting power must be greater than zero");
        
        members[_member] = Member({
            memberAddress: _member,
            votingPower: _votingPower,
            active: true,
            joinedAt: block.timestamp
        });
        
        totalVotingPower += _votingPower;
        
        emit MemberAdded(_member, _votingPower);
    }
    
    /**
     * @dev Remove a member
     * @param _member Address of the member to remove
     */
    function removeMember(address _member) public onlyOwner {
        require(members[_member].active, "Member does not exist");
        require(_member != owner, "Cannot remove owner");
        
        totalVotingPower -= members[_member].votingPower;
        members[_member].active = false;
        
        emit MemberRemoved(_member);
    }
    
    /**
     * @dev Update member voting power
     * @param _member Address of the member
     * @param _newVotingPower New voting power
     */
    function updateMemberVotingPower(address _member, uint256 _newVotingPower) public onlyOwner {
        require(members[_member].active, "Member does not exist");
        require(_newVotingPower > 0, "Voting power must be greater than zero");
        
        uint256 oldVotingPower = members[_member].votingPower;
        totalVotingPower = totalVotingPower - oldVotingPower + _newVotingPower;
        
        members[_member].votingPower = _newVotingPower;
        
        emit MemberVotingPowerUpdated(_member, _newVotingPower);
    }
    
    /**
     * @dev Authorize a proposer
     * @param _proposer Address of the proposer to authorize
     */
    function authorizeProposer(address _proposer) public onlyOwner {
        require(_proposer != address(0), "Proposer address cannot be zero");
        require(!authorizedProposers[_proposer], "Proposer is already authorized");
        
        authorizedProposers[_proposer] = true;
        emit ProposerAuthorized(_proposer, msg.sender);
    }
    
    /**
     * @dev Deauthorize a proposer
     * @param _proposer Address of the proposer to deauthorize
     */
    function deauthorizeProposer(address _proposer) public onlyOwner {
        require(_proposer != address(0), "Proposer address cannot be zero");
        require(authorizedProposers[_proposer], "Proposer is not authorized");
        require(_proposer != owner, "Cannot deauthorize owner");
        
        authorizedProposers[_proposer] = false;
        emit ProposerDeauthorized(_proposer, msg.sender);
    }
    
    /**
     * @dev Get proposal details
     * @param _proposalId Proposal ID
     * @return title Proposal title
     * @return description Proposal description
     * @return startTime Proposal start time
     * @return endTime Proposal end time
     * @return yesVotes Yes votes count
     * @return noVotes No votes count
     * @return abstainVotes Abstain votes count
     * @return executed Whether proposal is executed
     * @return cancelled Whether proposal is cancelled
     * @return proposer Proposal proposer
     */
    function getProposal(uint256 _proposalId) public view validProposal(_proposalId) returns (
        string memory title,
        string memory description,
        uint256 startTime,
        uint256 endTime,
        uint256 yesVotes,
        uint256 noVotes,
        uint256 abstainVotes,
        bool executed,
        bool cancelled,
        address proposer
    ) {
        Proposal storage proposal = proposals[_proposalId];
        return (
            proposal.title,
            proposal.description,
            proposal.startTime,
            proposal.endTime,
            proposal.yesVotes,
            proposal.noVotes,
            proposal.abstainVotes,
            proposal.executed,
            proposal.cancelled,
            proposal.proposer
        );
    }
    
    /**
     * @dev Get member details
     * @param _member Member address
     * @return memberAddress Member address
     * @return votingPower Member voting power
     * @return active Whether member is active
     * @return joinedAt When member joined
     */
    function getMember(address _member) public view returns (
        address memberAddress,
        uint256 votingPower,
        bool active,
        uint256 joinedAt
    ) {
        Member storage member = members[_member];
        return (member.memberAddress, member.votingPower, member.active, member.joinedAt);
    }
    
    /**
     * @dev Get DAO statistics
     * @return totalProposals Total number of proposals
     * @return totalMembers Total number of active members
     * @return totalVotingPower Total voting power
     * @return quorumThreshold Quorum threshold percentage
     * @return majorityThreshold Majority threshold percentage
     */
    function getDAOStats() public view returns (
        uint256 totalProposals,
        uint256 totalMembers,
        uint256 totalVotingPower,
        uint256 quorumThreshold,
        uint256 majorityThreshold
    ) {
        // Count active members
        uint256 activeMembers = 0;
        // Note: In a real implementation, you would iterate through all members
        // This is a simplified version for demonstration
        
        return (
            proposalCount,
            activeMembers,
            totalVotingPower,
            quorumThreshold,
            majorityThreshold
        );
    }
    
    /**
     * @dev Check if a member has voted on a proposal
     * @param _proposalId Proposal ID
     * @param _member Member address
     * @return hasVoted Whether the member has voted
     * @return vote The vote choice (0 = no vote, 1 = yes, 2 = no, 3 = abstain)
     */
    function getVote(uint256 _proposalId, address _member) public view validProposal(_proposalId) returns (
        bool hasVoted,
        uint8 vote
    ) {
        Proposal storage proposal = proposals[_proposalId];
        return (proposal.hasVoted[_member], proposal.votes[_member]);
    }
    
    /**
     * @dev Get contract information
     * @return contractName Name of the contract
     * @return version Version of the contract
     * @return contractOwner Contract owner
     * @return minVotingPeriod Minimum voting period
     * @return maxVotingPeriod Maximum voting period
     */
    function getContractInfo() public view returns (
        string memory contractName,
        string memory version,
        address contractOwner,
        uint256 minVotingPeriod,
        uint256 maxVotingPeriod
    ) {
        return ("DAOGovernance", "1.0.0", owner, minVotingPeriod, maxVotingPeriod);
    }
    
    /**
     * @dev Emergency function to transfer ownership
     * @param _newOwner Address of the new owner
     */
    function transferOwnership(address _newOwner) public onlyOwner {
        require(_newOwner != address(0), "New owner cannot be zero address");
        owner = _newOwner;
    }
}
