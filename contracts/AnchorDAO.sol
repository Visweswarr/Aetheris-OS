// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";

/**
 * @title AnchorDAO
 * @dev Smart contract for anchoring NGFS snapshots with hybrid PQC signature validation
 * @notice This contract enables on-chain anchoring of NGFS snapshot hashes with DID-based verification
 * and supports both legacy (ECDSA/Ed25519) and post-quantum (Dilithium) signatures
 */
contract AnchorDAO is Ownable, ReentrancyGuard, Pausable {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;

    // Events
    event SnapshotAnchored(
        bytes32 indexed snapshotHash,
        uint256 indexed blockNumber,
        uint256 timestamp,
        uint256 size,
        string creatorDID,
        address indexed submitter
    );

    event BatchAnchored(
        bytes32 indexed batchId,
        uint256 indexed blockNumber,
        uint256 snapshotCount,
        string submitterDID,
        address indexed submitter
    );

    event ProposalCreated(
        uint256 indexed proposalId,
        string title,
        string description,
        uint256 startTime,
        uint256 endTime,
        address indexed proposer
    );

    event VoteCast(
        uint256 indexed proposalId,
        address indexed voter,
        uint8 vote,
        uint256 weight,
        string voterDID
    );

    event ProposalExecuted(
        uint256 indexed proposalId,
        bool success,
        bytes returnData
    );

    event PQCMetadataAnchored(
        bytes32 indexed payloadHash,
        bytes32 indexed pqcMetadataHash,
        string algorithm,
        bytes32 publicKeyHash,
        bytes32 signatureHash,
        address indexed submitter
    );

    // Structs
    struct SnapshotAnchor {
        bytes32 snapshotHash;
        uint256 blockNumber;
        uint256 timestamp;
        uint256 size;
        string creatorDID;
        address submitter;
        bool exists;
    }

    struct BatchAnchor {
        bytes32 batchId;
        uint256 blockNumber;
        uint256 timestamp;
        uint256 snapshotCount;
        string submitterDID;
        address submitter;
        bool exists;
    }

    struct Proposal {
        uint256 id;
        string title;
        string description;
        uint256 startTime;
        uint256 endTime;
        address proposer;
        uint256 yesVotes;
        uint256 noVotes;
        uint256 abstainVotes;
        bool executed;
        bool exists;
    }

    struct Vote {
        address voter;
        uint8 vote; // 0 = abstain, 1 = yes, 2 = no
        uint256 weight;
        string voterDID;
        bool exists;
    }

    struct PQCSignatureMetadata {
        bytes32 payloadHash;
        string algorithm;
        bytes32 publicKeyHash;
        bytes32 signatureHash;
        address submitter;
        uint256 timestamp;
        bool exists;
    }

    // State variables
    mapping(bytes32 => SnapshotAnchor) public snapshotAnchors;
    mapping(bytes32 => BatchAnchor) public batchAnchors;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => Vote)) public votes;
    mapping(address => bool) public authorizedSubmitters;
    mapping(string => bool) public registeredDIDs;
    mapping(address => string) public addressToDID;
    mapping(string => address) public didToAddress;

    uint256 public proposalCount;
    uint256 public constant VOTING_DURATION = 7 days;
    uint256 public constant MINIMUM_VOTES = 3;
    uint256 public constant MAJORITY_THRESHOLD = 50; // 50% + 1

    // PQC signature validation metadata. EVM verifies the classical signature
    // and stores Dilithium/ML-DSA hashes for off-chain verifier attestation.
    mapping(bytes32 => bool) public validPQCSignatures;
    mapping(bytes32 => PQCSignatureMetadata) public pqcSignatureMetadata;

    // Modifiers
    modifier onlyAuthorizedSubmitter() {
        require(authorizedSubmitters[msg.sender], "Not authorized to submit anchors");
        _;
    }

    modifier onlyRegisteredDID(string memory did) {
        require(registeredDIDs[did], "DID not registered");
        _;
    }

    modifier validProposal(uint256 proposalId) {
        require(proposals[proposalId].exists, "Proposal does not exist");
        _;
    }

    modifier votingPeriod(uint256 proposalId) {
        Proposal storage proposal = proposals[proposalId];
        require(
            block.timestamp >= proposal.startTime && block.timestamp <= proposal.endTime,
            "Not in voting period"
        );
        _;
    }

    modifier afterVotingPeriod(uint256 proposalId) {
        Proposal storage proposal = proposals[proposalId];
        require(block.timestamp > proposal.endTime, "Voting period not ended");
        _;
    }

    constructor() {
        // Initialize with owner as authorized submitter
        authorizedSubmitters[msg.sender] = true;
    }

    /**
     * @dev Register a DID for a given address
     * @param did The DID to register
     * @param signature Signature proving ownership of the DID
     */
    function registerDID(string memory did, bytes memory signature) external {
        require(!registeredDIDs[did], "DID already registered");
        require(bytes(addressToDID[msg.sender]).length == 0, "Address already has DID");

        // Verify signature (simplified - in production, use proper DID verification)
        bytes32 messageHash = keccak256(abi.encodePacked(did, msg.sender));
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();
        address signer = ethSignedMessageHash.recover(signature);
        
        require(signer == msg.sender, "Invalid signature");

        registeredDIDs[did] = true;
        addressToDID[msg.sender] = did;
        didToAddress[did] = msg.sender;
    }

    /**
     * @dev Authorize an address to submit anchors
     * @param submitter Address to authorize
     */
    function authorizeSubmitter(address submitter) external onlyOwner {
        authorizedSubmitters[submitter] = true;
    }

    /**
     * @dev Deauthorize an address from submitting anchors
     * @param submitter Address to deauthorize
     */
    function deauthorizeSubmitter(address submitter) external onlyOwner {
        authorizedSubmitters[submitter] = false;
    }

    /**
     * @dev Anchor a single NGFS snapshot hash
     * @param snapshotHash The hash of the NGFS snapshot
     * @param timestamp Timestamp when the snapshot was created
     * @param size Size of the snapshot in bytes
     * @param creatorDID DID of the snapshot creator
     * @param legacySignature Legacy signature (ECDSA/Ed25519)
     * @param pqcSignature Post-quantum signature (Dilithium)
     * @param algorithm Signature algorithm used
     * @param publicKey Public key for verification
     */
    function anchorSnapshot(
        bytes32 snapshotHash,
        uint256 timestamp,
        uint256 size,
        string memory creatorDID,
        bytes memory legacySignature,
        bytes memory pqcSignature,
        string memory algorithm,
        bytes memory publicKey
    ) external onlyAuthorizedSubmitter onlyRegisteredDID(creatorDID) whenNotPaused {
        require(!snapshotAnchors[snapshotHash].exists, "Snapshot already anchored");
        require(size > 0, "Invalid snapshot size");
        require(timestamp <= block.timestamp, "Invalid timestamp");

        (
            bytes32 payloadHash,
            bytes32 pqcMetadataHash,
            bytes32 publicKeyHash,
            bytes32 signatureHash
        ) = verifyHybridSignature(
            snapshotHash,
            legacySignature,
            pqcSignature,
            algorithm,
            publicKey,
            msg.sender
        );

        validPQCSignatures[pqcMetadataHash] = true;
        pqcSignatureMetadata[pqcMetadataHash] = PQCSignatureMetadata({
            payloadHash: payloadHash,
            algorithm: algorithm,
            publicKeyHash: publicKeyHash,
            signatureHash: signatureHash,
            submitter: msg.sender,
            timestamp: block.timestamp,
            exists: true
        });

        snapshotAnchors[snapshotHash] = SnapshotAnchor({
            snapshotHash: snapshotHash,
            blockNumber: block.number,
            timestamp: timestamp,
            size: size,
            creatorDID: creatorDID,
            submitter: msg.sender,
            exists: true
        });

        emit PQCMetadataAnchored(
            payloadHash,
            pqcMetadataHash,
            algorithm,
            publicKeyHash,
            signatureHash,
            msg.sender
        );
        emit SnapshotAnchored(snapshotHash, block.number, timestamp, size, creatorDID, msg.sender);
    }

    /**
     * @dev Anchor a batch of NGFS snapshot hashes
     * @param snapshotHashes Array of snapshot hashes
     * @param timestamps Array of timestamps
     * @param sizes Array of sizes
     * @param creatorDIDs Array of creator DIDs
     * @param batchId Unique batch identifier
     * @param submitterDID DID of the batch submitter
     * @param legacySignature Legacy signature for the batch
     * @param pqcSignature Post-quantum signature for the batch
     * @param algorithm Signature algorithm used
     * @param publicKey Public key for verification
     */
    function anchorBatch(
        bytes32[] memory snapshotHashes,
        uint256[] memory timestamps,
        uint256[] memory sizes,
        string[] memory creatorDIDs,
        string memory batchId,
        string memory submitterDID,
        string memory legacySignature,
        string memory pqcSignature,
        string memory algorithm,
        string memory publicKey
    ) external onlyAuthorizedSubmitter onlyRegisteredDID(submitterDID) whenNotPaused {
        require(snapshotHashes.length > 0, "Empty batch");
        require(snapshotHashes.length == timestamps.length, "Array length mismatch");
        require(snapshotHashes.length == sizes.length, "Array length mismatch");
        require(snapshotHashes.length == creatorDIDs.length, "Array length mismatch");

        bytes32 batchIdHash = keccak256(abi.encodePacked(batchId));
        require(!batchAnchors[batchIdHash].exists, "Batch already anchored");

        // Verify hybrid signature for the batch. msg.sender is the expected ECDSA
        // signer because the onlyAuthorizedSubmitter modifier already confirmed
        // they are permitted to submit; the signature binds them to this exact
        // payload (including the PQC metadata) for off-chain attestation.
        require(verifyBatchSignature(
            snapshotHashes, timestamps, sizes, creatorDIDs, batchId,
            legacySignature, pqcSignature, algorithm, publicKey,
            msg.sender
        ), "Invalid batch signature");

        // Anchor individual snapshots
        for (uint256 i = 0; i < snapshotHashes.length; i++) {
            require(!snapshotAnchors[snapshotHashes[i]].exists, "Snapshot already anchored");
            
            snapshotAnchors[snapshotHashes[i]] = SnapshotAnchor({
                snapshotHash: snapshotHashes[i],
                blockNumber: block.number,
                timestamp: timestamps[i],
                size: sizes[i],
                creatorDID: creatorDIDs[i],
                submitter: msg.sender,
                exists: true
            });

            emit SnapshotAnchored(snapshotHashes[i], block.number, timestamps[i], sizes[i], creatorDIDs[i], msg.sender);
        }

        // Record batch anchor
        batchAnchors[batchIdHash] = BatchAnchor({
            batchId: batchIdHash,
            blockNumber: block.number,
            timestamp: block.timestamp,
            snapshotCount: snapshotHashes.length,
            submitterDID: submitterDID,
            submitter: msg.sender,
            exists: true
        });

        emit BatchAnchored(batchIdHash, block.number, snapshotHashes.length, submitterDID, msg.sender);
    }

    /**
     * @dev Create a new DAO proposal
     * @param title Title of the proposal
     * @param description Description of the proposal
     */
    function createProposal(string memory title, string memory description) external onlyRegisteredDID(addressToDID[msg.sender]) {
        proposalCount++;
        
        proposals[proposalCount] = Proposal({
            id: proposalCount,
            title: title,
            description: description,
            startTime: block.timestamp,
            endTime: block.timestamp + VOTING_DURATION,
            proposer: msg.sender,
            yesVotes: 0,
            noVotes: 0,
            abstainVotes: 0,
            executed: false,
            exists: true
        });

        emit ProposalCreated(proposalCount, title, description, block.timestamp, block.timestamp + VOTING_DURATION, msg.sender);
    }

    /**
     * @dev Cast a vote on a proposal
     * @param proposalId ID of the proposal
     * @param vote Vote choice (0 = abstain, 1 = yes, 2 = no)
     * @param weight Voting weight (based on stake/reputation)
     */
    function vote(uint256 proposalId, uint8 vote, uint256 weight) external validProposal(proposalId) votingPeriod(proposalId) {
        require(vote <= 2, "Invalid vote choice");
        require(weight > 0, "Invalid voting weight");
        require(!votes[proposalId][msg.sender].exists, "Already voted");

        string memory voterDID = addressToDID[msg.sender];
        require(bytes(voterDID).length > 0, "Voter not registered");

        votes[proposalId][msg.sender] = Vote({
            voter: msg.sender,
            vote: vote,
            weight: weight,
            voterDID: voterDID,
            exists: true
        });

        // Update proposal vote counts
        Proposal storage proposal = proposals[proposalId];
        if (vote == 0) {
            proposal.abstainVotes += weight;
        } else if (vote == 1) {
            proposal.yesVotes += weight;
        } else {
            proposal.noVotes += weight;
        }

        emit VoteCast(proposalId, msg.sender, vote, weight, voterDID);
    }

    /**
     * @dev Execute a proposal after voting period ends
     * @param proposalId ID of the proposal to execute
     */
    function executeProposal(uint256 proposalId) external validProposal(proposalId) afterVotingPeriod(proposalId) {
        Proposal storage proposal = proposals[proposalId];
        require(!proposal.executed, "Proposal already executed");

        uint256 totalVotes = proposal.yesVotes + proposal.noVotes + proposal.abstainVotes;
        require(totalVotes >= MINIMUM_VOTES, "Insufficient votes");

        bool success = false;
        bytes memory returnData = "";

        // Check if proposal passed (simple majority)
        if (proposal.yesVotes > proposal.noVotes) {
            // Execute proposal logic (simplified for this implementation)
            success = true;
            returnData = abi.encode("Proposal executed successfully");
        }

        proposal.executed = true;

        emit ProposalExecuted(proposalId, success, returnData);
    }

    /**
     * @dev Verify an anchored snapshot
     * @param snapshotHash Hash of the snapshot to verify
     * @return exists Whether the snapshot is anchored
     * @return blockNumber Block number where anchored
     * @return txHash Transaction hash (simplified)
     */
    function verifyAnchor(bytes32 snapshotHash) external view returns (bool exists, uint256 blockNumber, string memory txHash) {
        SnapshotAnchor storage anchor = snapshotAnchors[snapshotHash];
        exists = anchor.exists;
        blockNumber = anchor.blockNumber;
        txHash = "0x0000000000000000000000000000000000000000000000000000000000000000"; // Simplified
    }

    /**
     * @dev Get proposal details
     * @param proposalId ID of the proposal
     * @return title Proposal title
     * @return description Proposal description
     * @return startTime Voting start time
     * @return endTime Voting end time
     * @return proposer Proposal creator
     * @return yesVotes Number of yes votes
     * @return noVotes Number of no votes
     * @return abstainVotes Number of abstain votes
     * @return executed Whether proposal was executed
     */
    function getProposal(uint256 proposalId) external view returns (
        string memory title,
        string memory description,
        uint256 startTime,
        uint256 endTime,
        address proposer,
        uint256 yesVotes,
        uint256 noVotes,
        uint256 abstainVotes,
        bool executed
    ) {
        Proposal storage proposal = proposals[proposalId];
        require(proposal.exists, "Proposal does not exist");
        
        return (
            proposal.title,
            proposal.description,
            proposal.startTime,
            proposal.endTime,
            proposal.proposer,
            proposal.yesVotes,
            proposal.noVotes,
            proposal.abstainVotes,
            proposal.executed
        );
    }

    /**
     * @dev Verify hybrid signature policy.
     *
     * EVM validates the classical ECDSA signature over the exact PQC metadata
     * digest. Dilithium/ML-DSA verification is intentionally off-chain; this
     * function stores hashes that the verifier can attest to without putting a
     * PQC verifier on-chain.
     * @param snapshotHash Hash of the snapshot
     * @param legacySignature Legacy signature
     * @param pqcSignature Post-quantum signature
     * @param algorithm Signature algorithm
     * @param publicKey Public key
     */
    function verifyHybridSignature(
        bytes32 snapshotHash,
        bytes memory legacySignature,
        bytes memory pqcSignature,
        string memory algorithm,
        bytes memory publicKey,
        address expectedSigner
    ) internal pure returns (
        bytes32 payloadHash,
        bytes32 pqcMetadataHash,
        bytes32 publicKeyHash,
        bytes32 signatureHash
    ) {
        require(legacySignature.length == 65, "Invalid ECDSA signature length");
        require(isSupportedPQCAlgorithm(algorithm), "Unsupported PQC algorithm");
        require(publicKey.length >= 32, "Invalid PQC public key");
        require(pqcSignature.length >= 32, "Invalid PQC signature");

        publicKeyHash = keccak256(publicKey);
        signatureHash = keccak256(pqcSignature);
        payloadHash = keccak256(abi.encodePacked(
            snapshotHash,
            algorithm,
            publicKeyHash,
            signatureHash
        ));

        address signer = payloadHash.toEthSignedMessageHash().recover(legacySignature);
        require(signer == expectedSigner, "Invalid ECDSA signer");

        pqcMetadataHash = keccak256(abi.encodePacked(
            payloadHash,
            algorithm,
            publicKeyHash,
            signatureHash
        ));
    }

    function isSupportedPQCAlgorithm(string memory algorithm) internal pure returns (bool) {
        bytes32 algorithmHash = keccak256(bytes(algorithm));
        return algorithmHash == keccak256(bytes("Dilithium2"))
            || algorithmHash == keccak256(bytes("Dilithium3"))
            || algorithmHash == keccak256(bytes("Dilithium5"))
            || algorithmHash == keccak256(bytes("ML-DSA-44"))
            || algorithmHash == keccak256(bytes("ML-DSA-65"))
            || algorithmHash == keccak256(bytes("ML-DSA-87"));
    }

    /**
     * @dev Verify batch signature (simplified implementation)
     * @param snapshotHashes Array of snapshot hashes
     * @param timestamps Array of timestamps
     * @param sizes Array of sizes
     * @param creatorDIDs Array of creator DIDs
     * @param batchId Batch identifier
     * @param legacySignature Legacy signature
     * @param pqcSignature Post-quantum signature
     * @param algorithm Signature algorithm
     * @param publicKey Public key
     * @return valid Whether the signature is valid
     */
    function verifyBatchSignature(
        bytes32[] memory snapshotHashes,
        uint256[] memory timestamps,
        uint256[] memory sizes,
        string[] memory creatorDIDs,
        string memory batchId,
        string memory legacySignature,
        string memory pqcSignature,
        string memory algorithm,
        string memory publicKey,
        address expectedSigner
    ) internal pure returns (bool valid) {
        // Format gates: reject anything that cannot be a real (ECDSA, PQC) pair.
        require(bytes(legacySignature).length == 65, "Invalid ECDSA signature length");
        require(bytes(pqcSignature).length >= 32, "Invalid PQC signature");
        require(isSupportedPQCAlgorithm(algorithm), "Unsupported PQC algorithm");
        require(bytes(publicKey).length >= 32, "Invalid PQC public key");

        // Build batch digest from all snapshot data.
        bytes32 batchDigest = keccak256(abi.encodePacked(batchId));
        for (uint256 i = 0; i < snapshotHashes.length; i++) {
            batchDigest = keccak256(abi.encodePacked(
                batchDigest,
                snapshotHashes[i],
                timestamps[i],
                sizes[i],
                creatorDIDs[i]
            ));
        }

        // Bind the ECDSA signature to the PQC metadata so the two signatures
        // commit to the same payload. Mirrors verifyHybridSignature.
        bytes32 publicKeyHash = keccak256(bytes(publicKey));
        bytes32 signatureHash = keccak256(bytes(pqcSignature));
        bytes32 payloadHash = keccak256(abi.encodePacked(
            batchDigest,
            algorithm,
            publicKeyHash,
            signatureHash
        ));

        // Recover the ECDSA signer and require it to match the expected submitter.
        // Real Dilithium/ML-DSA verification is intentionally off-chain — the
        // payloadHash carries the PQC commitment so verifiers can attest later.
        address signer = payloadHash.toEthSignedMessageHash().recover(bytes(legacySignature));
        require(signer == expectedSigner, "Invalid ECDSA signer");

        return true;
    }

    /**
     * @dev Pause the contract
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpause the contract
     */
    function unpause() external onlyOwner {
        _unpause();
    }
}
