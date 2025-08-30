// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NGFS Anchor Contract
 * @dev Smart contract for anchoring NGFS snapshot hashes on the blockchain
 * @author Polymera OS Team
 * @notice This contract provides immutable, verifiable proof of NGFS snapshot existence
 * @custom:security-contact security@polymera-os.com
 */
contract Anchor {
    // Events
    event SnapshotAnchored(
        bytes32 indexed snapshotHash,
        address indexed submitter,
        uint256 timestamp,
        string chain,
        string version,
        bytes metadata,
        uint256 gasUsed,
        uint256 blockNumber,
        bytes32 transactionHash
    );

    event AnchorBatchSubmitted(
        bytes32 indexed batchId,
        address indexed submitter,
        uint256 timestamp,
        uint256 anchorCount,
        uint256 totalGasUsed
    );

    event AnchorMetadataUpdated(
        bytes32 indexed snapshotHash,
        address indexed submitter,
        uint256 timestamp,
        bytes newMetadata
    );

    event AnchorExpired(
        bytes32 indexed snapshotHash,
        address indexed submitter,
        uint256 timestamp,
        uint256 expirationTime
    );

    // Structs
    struct AnchorData {
        bytes32 snapshotHash;
        address submitter;
        uint256 timestamp;
        string chain;
        string version;
        bytes metadata;
        uint256 gasUsed;
        uint256 blockNumber;
        bytes32 transactionHash;
        bool exists;
        uint256 expirationTime;
    }

    struct BatchData {
        bytes32 batchId;
        address submitter;
        uint256 timestamp;
        bytes32[] snapshotHashes;
        uint256 totalGasUsed;
        bool exists;
    }

    // State variables
    mapping(bytes32 => AnchorData) public anchors;
    mapping(bytes32 => BatchData) public batches;
    mapping(address => bytes32[]) public submitterAnchors;
    mapping(address => uint256) public submitterAnchorCount;
    
    uint256 public totalAnchors;
    uint256 public totalBatches;
    uint256 public totalGasUsed;
    
    address public owner;
    uint256 public maxAnchorSize;
    uint256 public maxBatchSize;
    uint256 public defaultExpirationTime;
    bool public paused;
    
    // Modifiers
    modifier onlyOwner() {
        require(msg.sender == owner, "Anchor: caller is not the owner");
        _;
    }
    
    modifier whenNotPaused() {
        require(!paused, "Anchor: contract is paused");
        _;
    }
    
    modifier validSnapshotHash(bytes32 snapshotHash) {
        require(snapshotHash != bytes32(0), "Anchor: invalid snapshot hash");
        _;
    }
    
    modifier anchorExists(bytes32 snapshotHash) {
        require(anchors[snapshotHash].exists, "Anchor: anchor does not exist");
        _;
    }
    
    modifier anchorNotExpired(bytes32 snapshotHash) {
        require(
            anchors[snapshotHash].expirationTime == 0 || 
            block.timestamp < anchors[snapshotHash].expirationTime,
            "Anchor: anchor has expired"
        );
        _;
    }

    // Constructor
    constructor() {
        owner = msg.sender;
        maxAnchorSize = 128; // Maximum anchor data size in bytes
        maxBatchSize = 100;  // Maximum anchors per batch
        defaultExpirationTime = 0; // No expiration by default
        paused = false;
    }

    /**
     * @dev Anchor a single NGFS snapshot hash
     * @param snapshotHash The blake3-256 hash of the NGFS snapshot
     * @param chain The target blockchain identifier
     * @param version The schema version
     * @param metadata Additional anchor metadata (CBOR encoded)
     * @param expirationTime Optional expiration timestamp (0 = no expiration)
     */
    function anchorSnapshot(
        bytes32 snapshotHash,
        string calldata chain,
        string calldata version,
        bytes calldata metadata,
        uint256 expirationTime
    ) external whenNotPaused validSnapshotHash(snapshotHash) {
        require(!anchors[snapshotHash].exists, "Anchor: snapshot already anchored");
        require(metadata.length <= maxAnchorSize, "Anchor: metadata too large");
        
        // Create anchor data
        AnchorData memory anchor = AnchorData({
            snapshotHash: snapshotHash,
            submitter: msg.sender,
            timestamp: block.timestamp,
            chain: chain,
            version: version,
            metadata: metadata,
            gasUsed: 0, // Will be updated after transaction
            blockNumber: block.number,
            transactionHash: bytes32(0), // Will be updated after transaction
            exists: true,
            expirationTime: expirationTime
        });
        
        // Store anchor
        anchors[snapshotHash] = anchor;
        
        // Update submitter tracking
        submitterAnchors[msg.sender].push(snapshotHash);
        submitterAnchorCount[msg.sender]++;
        
        // Update global statistics
        totalAnchors++;
        
        // Emit event
        emit SnapshotAnchored(
            snapshotHash,
            msg.sender,
            block.timestamp,
            chain,
            version,
            metadata,
            0, // gasUsed will be updated
            block.number,
            bytes32(0) // transactionHash will be updated
        );
    }

    /**
     * @dev Submit multiple NGFS snapshot anchors in a batch
     * @param snapshotHashes Array of snapshot hashes
     * @param chains Array of chain identifiers
     * @param versions Array of schema versions
     * @param metadataArray Array of metadata bytes
     * @param expirationTimes Array of expiration timestamps
     */
    function anchorBatch(
        bytes32[] calldata snapshotHashes,
        string[] calldata chains,
        string[] calldata versions,
        bytes[] calldata metadataArray,
        uint256[] calldata expirationTimes
    ) external whenNotPaused {
        require(
            snapshotHashes.length == chains.length &&
            chains.length == versions.length &&
            versions.length == metadataArray.length &&
            metadataArray.length == expirationTimes.length,
            "Anchor: array length mismatch"
        );
        require(snapshotHashes.length <= maxBatchSize, "Anchor: batch too large");
        require(snapshotHashes.length > 0, "Anchor: empty batch");
        
        bytes32 batchId = keccak256(abi.encodePacked(
            msg.sender,
            block.timestamp,
            snapshotHashes
        ));
        
        uint256 batchGasUsed = 0;
        
        // Process each anchor in the batch
        for (uint256 i = 0; i < snapshotHashes.length; i++) {
            bytes32 snapshotHash = snapshotHashes[i];
            require(!anchors[snapshotHash].exists, "Anchor: snapshot already anchored");
            require(metadataArray[i].length <= maxAnchorSize, "Anchor: metadata too large");
            
            // Create anchor data
            AnchorData memory anchor = AnchorData({
                snapshotHash: snapshotHash,
                submitter: msg.sender,
                timestamp: block.timestamp,
                chain: chains[i],
                version: versions[i],
                metadata: metadataArray[i],
                gasUsed: 0,
                blockNumber: block.number,
                transactionHash: bytes32(0),
                exists: true,
                expirationTime: expirationTimes[i]
            });
            
            // Store anchor
            anchors[snapshotHash] = anchor;
            
            // Update submitter tracking
            submitterAnchors[msg.sender].push(snapshotHash);
            submitterAnchorCount[msg.sender]++;
            
            // Update global statistics
            totalAnchors++;
        }
        
        // Create batch data
        BatchData memory batch = BatchData({
            batchId: batchId,
            submitter: msg.sender,
            timestamp: block.timestamp,
            snapshotHashes: snapshotHashes,
            totalGasUsed: batchGasUsed,
            exists: true
        });
        
        batches[batchId] = batch;
        totalBatches++;
        
        // Emit batch event
        emit AnchorBatchSubmitted(
            batchId,
            msg.sender,
            block.timestamp,
            snapshotHashes.length,
            batchGasUsed
        );
    }

    /**
     * @dev Update metadata for an existing anchor
     * @param snapshotHash The snapshot hash to update
     * @param newMetadata New metadata bytes
     */
    function updateAnchorMetadata(
        bytes32 snapshotHash,
        bytes calldata newMetadata
    ) external anchorExists(snapshotHash) anchorNotExpired(snapshotHash) {
        AnchorData storage anchor = anchors[snapshotHash];
        require(anchor.submitter == msg.sender, "Anchor: not the submitter");
        require(newMetadata.length <= maxAnchorSize, "Anchor: metadata too large");
        
        anchor.metadata = newMetadata;
        
        emit AnchorMetadataUpdated(
            snapshotHash,
            msg.sender,
            block.timestamp,
            newMetadata
        );
    }

    /**
     * @dev Get anchor data by snapshot hash
     * @param snapshotHash The snapshot hash to query
     * @return Anchor data struct
     */
    function getAnchor(bytes32 snapshotHash) external view returns (AnchorData memory) {
        require(anchors[snapshotHash].exists, "Anchor: anchor does not exist");
        return anchors[snapshotHash];
    }

    /**
     * @dev Get batch data by batch ID
     * @param batchId The batch ID to query
     * @return Batch data struct
     */
    function getBatch(bytes32 batchId) external view returns (BatchData memory) {
        require(batches[batchId].exists, "Anchor: batch does not exist");
        return batches[batchId];
    }

    /**
     * @dev Get all anchors submitted by an address
     * @param submitter The submitter address
     * @return Array of snapshot hashes
     */
    function getSubmitterAnchors(address submitter) external view returns (bytes32[] memory) {
        return submitterAnchors[submitter];
    }

    /**
     * @dev Check if a snapshot hash is anchored
     * @param snapshotHash The snapshot hash to check
     * @return True if anchored, false otherwise
     */
    function isAnchored(bytes32 snapshotHash) external view returns (bool) {
        if (!anchors[snapshotHash].exists) {
            return false;
        }
        
        // Check if expired
        if (anchors[snapshotHash].expirationTime > 0 && 
            block.timestamp >= anchors[snapshotHash].expirationTime) {
            return false;
        }
        
        return true;
    }

    /**
     * @dev Get contract statistics
     * @return Total anchors, total batches, total gas used
     */
    function getStats() external view returns (uint256, uint256, uint256) {
        return (totalAnchors, totalBatches, totalGasUsed);
    }

    /**
     * @dev Update anchor gas usage and transaction hash (called by owner)
     * @param snapshotHash The snapshot hash to update
     * @param gasUsed The gas used for the transaction
     * @param transactionHash The transaction hash
     */
    function updateAnchorTransaction(
        bytes32 snapshotHash,
        uint256 gasUsed,
        bytes32 transactionHash
    ) external onlyOwner anchorExists(snapshotHash) {
        AnchorData storage anchor = anchors[snapshotHash];
        anchor.gasUsed = gasUsed;
        anchor.transactionHash = transactionHash;
        
        // Update global gas statistics
        totalGasUsed += gasUsed;
    }

    /**
     * @dev Clean up expired anchors (called by anyone)
     * @param snapshotHashes Array of expired snapshot hashes to clean up
     */
    function cleanupExpiredAnchors(bytes32[] calldata snapshotHashes) external {
        uint256 cleanedCount = 0;
        
        for (uint256 i = 0; i < snapshotHashes.length; i++) {
            bytes32 snapshotHash = snapshotHashes[i];
            
            if (anchors[snapshotHash].exists && 
                anchors[snapshotHash].expirationTime > 0 && 
                block.timestamp >= anchors[snapshotHash].expirationTime) {
                
                // Emit expiration event
                emit AnchorExpired(
                    snapshotHash,
                    anchors[snapshotHash].submitter,
                    block.timestamp,
                    anchors[snapshotHash].expirationTime
                );
                
                // Remove from submitter tracking
                address submitter = anchors[snapshotHash].submitter;
                submitterAnchorCount[submitter]--;
                
                // Note: We don't remove from submitterAnchors array to maintain indices
                // In production, you might want to implement a more sophisticated cleanup
                
                cleanedCount++;
            }
        }
        
        // Update total anchors
        totalAnchors -= cleanedCount;
    }

    // Owner functions
    /**
     * @dev Set maximum anchor size
     * @param newMaxSize New maximum size in bytes
     */
    function setMaxAnchorSize(uint256 newMaxSize) external onlyOwner {
        require(newMaxSize > 0, "Anchor: invalid max size");
        maxAnchorSize = newMaxSize;
    }

    /**
     * @dev Set maximum batch size
     * @param newMaxBatchSize New maximum batch size
     */
    function setMaxBatchSize(uint256 newMaxBatchSize) external onlyOwner {
        require(newMaxBatchSize > 0, "Anchor: invalid max batch size");
        maxBatchSize = newMaxBatchSize;
    }

    /**
     * @dev Set default expiration time
     * @param newExpirationTime New default expiration time in seconds
     */
    function setDefaultExpirationTime(uint256 newExpirationTime) external onlyOwner {
        defaultExpirationTime = newExpirationTime;
    }

    /**
     * @dev Pause or unpause the contract
     * @param newPaused New pause state
     */
    function setPaused(bool newPaused) external onlyOwner {
        paused = newPaused;
    }

    /**
     * @dev Transfer ownership
     * @param newOwner New owner address
     */
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Anchor: invalid new owner");
        owner = newOwner;
    }

    /**
     * @dev Emergency function to recover stuck ETH
     */
    function emergencyWithdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "Anchor: no ETH to withdraw");
        
        (bool success, ) = owner.call{value: balance}("");
        require(success, "Anchor: ETH transfer failed");
    }

    // View functions for gas estimation
    /**
     * @dev Estimate gas for anchoring a single snapshot
     * @param metadataSize Size of metadata in bytes
     * @return Estimated gas cost
     */
    function estimateAnchorGas(uint256 metadataSize) external pure returns (uint256) {
        require(metadataSize <= 128, "Anchor: metadata too large");
        
        // Base cost: 21,000 gas for transaction
        uint256 baseCost = 21000;
        
        // Storage cost: ~20,000 gas for new storage slot
        uint256 storageCost = 20000;
        
        // Event emission cost: ~3,000 gas
        uint256 eventCost = 3000;
        
        // Metadata processing cost: ~100 gas per byte
        uint256 metadataCost = metadataSize * 100;
        
        return baseCost + storageCost + eventCost + metadataCost;
    }

    /**
     * @dev Estimate gas for batch anchoring
     * @param batchSize Number of anchors in batch
     * @param totalMetadataSize Total size of all metadata in bytes
     * @return Estimated gas cost
     */
    function estimateBatchGas(uint256 batchSize, uint256 totalMetadataSize) external pure returns (uint256) {
        require(batchSize > 0 && batchSize <= 100, "Anchor: invalid batch size");
        require(totalMetadataSize <= batchSize * 128, "Anchor: metadata too large");
        
        // Base cost: 21,000 gas for transaction
        uint256 baseCost = 21000;
        
        // Storage cost: ~20,000 gas per new storage slot
        uint256 storageCost = batchSize * 20000;
        
        // Batch storage cost: ~20,000 gas
        uint256 batchStorageCost = 20000;
        
        // Event emissions: ~3,000 gas per anchor + batch event
        uint256 eventCost = (batchSize * 3000) + 3000;
        
        // Metadata processing cost: ~100 gas per byte
        uint256 metadataCost = totalMetadataSize * 100;
        
        // Loop processing cost: ~500 gas per iteration
        uint256 loopCost = batchSize * 500;
        
        return baseCost + storageCost + batchStorageCost + eventCost + metadataCost + loopCost;
    }

    // Fallback and receive functions
    fallback() external {
        revert("Anchor: function not found");
    }

    receive() external payable {
        revert("Anchor: contract does not accept ETH");
    }
}
