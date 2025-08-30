// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

/**
 * @title AttestationRegistry
 * @dev Registry for managing digital attestations with issue/revoke/expiry functionality
 * @notice Provides a decentralized way to issue, verify, and manage attestations
 */
contract AttestationRegistry is Ownable, ReentrancyGuard {
    using Counters for Counters.Counter;
    using Strings for uint256;

    // ============ STRUCTS ============

    /**
     * @dev Attestation data structure
     */
    struct Attestation {
        uint256 id;                    // Unique attestation ID
        address issuer;                // Address that issued the attestation
        address subject;               // Address being attested
        bytes32 schemaId;              // Schema identifier for the attestation
        bytes data;                    // Attestation data (encoded)
        uint256 issuedAt;              // Timestamp when issued
        uint256 expiresAt;             // Timestamp when expires (0 = never)
        bool revoked;                  // Whether attestation is revoked
        uint256 revokedAt;             // Timestamp when revoked (0 = not revoked)
        address revokedBy;             // Address that revoked the attestation
        string uri;                    // URI to additional metadata
        bytes32[] tags;                // Tags for categorization
        uint256 version;               // Version of the attestation
    }

    /**
     * @dev Schema data structure
     */
    struct Schema {
        bytes32 id;                    // Unique schema identifier
        string name;                    // Human-readable schema name
        string description;             // Schema description
        bytes32[] fields;              // Field identifiers
        string[] fieldTypes;            // Field type definitions
        bool required;                  // Whether schema is required
        uint256 createdAt;              // Timestamp when created
        address createdBy;              // Address that created the schema
        bool deprecated;                // Whether schema is deprecated
        uint256 deprecatedAt;           // Timestamp when deprecated
        string version;                 // Schema version
    }

    /**
     * @dev Issuer data structure
     */
    struct Issuer {
        address addr;                   // Issuer address
        string name;                    // Human-readable name
        string description;             // Issuer description
        string uri;                     // URI to issuer metadata
        bool active;                    // Whether issuer is active
        uint256 registeredAt;           // Timestamp when registered
        uint256 totalAttestations;      // Total attestations issued
        bytes32[] authorizedSchemas;    // Schemas issuer can use
        mapping(bytes32 => bool) isAuthorizedForSchema; // Schema authorization
    }

    // ============ STATE VARIABLES ============

    Counters.Counter private _attestationIds;
    Counters.Counter private _schemaIds;

    // Mappings
    mapping(uint256 => Attestation) public attestations;
    mapping(bytes32 => Schema) public schemas;
    mapping(address => Issuer) public issuers;
    mapping(address => uint256[]) public subjectAttestations;
    mapping(address => uint256[]) public issuerAttestations;
    mapping(bytes32 => uint256[]) public schemaAttestations;
    mapping(bytes32 => bool) public schemaExists;
    mapping(address => bool) public issuerExists;
    mapping(bytes32 => mapping(address => bool)) public issuerSchemaAuthorization;

    // Arrays for enumeration
    uint256[] public allAttestationIds;
    bytes32[] public allSchemaIds;
    address[] public allIssuerAddresses;

    // Configuration
    uint256 public minAttestationLifetime = 1 days;
    uint256 public maxAttestationLifetime = 365 days;
    uint256 public attestationFee = 0;
    bool public attestationFeeEnabled = false;
    uint256 public maxTagsPerAttestation = 10;
    uint256 public maxAttestationDataSize = 1024; // 1KB

    // ============ EVENTS ============

    /**
     * @dev Emitted when a new attestation is issued
     */
    event AttestationIssued(
        uint256 indexed id,
        address indexed issuer,
        address indexed subject,
        bytes32 schemaId,
        uint256 issuedAt,
        uint256 expiresAt,
        string uri,
        bytes32[] tags,
        uint256 version
    );

    /**
     * @dev Emitted when an attestation is revoked
     */
    event AttestationRevoked(
        uint256 indexed id,
        address indexed issuer,
        address indexed subject,
        uint256 revokedAt,
        address revokedBy,
        string reason
    );

    /**
     * @dev Emitted when an attestation expires
     */
    event AttestationExpired(
        uint256 indexed id,
        address indexed issuer,
        address indexed subject,
        uint256 expiredAt
    );

    /**
     * @dev Emitted when a new schema is created
     */
    event SchemaCreated(
        bytes32 indexed id,
        string name,
        string description,
        bytes32[] fields,
        string[] fieldTypes,
        address indexed createdBy,
        string version
    );

    /**
     * @dev Emitted when a schema is deprecated
     */
    event SchemaDeprecated(
        bytes32 indexed id,
        address indexed deprecatedBy,
        uint256 deprecatedAt,
        string reason
    );

    /**
     * @dev Emitted when an issuer is registered
     */
    event IssuerRegistered(
        address indexed addr,
        string name,
        string description,
        string uri,
        address indexed registeredBy
    );

    /**
     * @dev Emitted when an issuer is updated
     */
    event IssuerUpdated(
        address indexed addr,
        string name,
        string description,
        string uri,
        address indexed updatedBy
    );

    /**
     * @dev Emitted when an issuer is deactivated
     */
    event IssuerDeactivated(
        address indexed addr,
        address indexed deactivatedBy,
        uint256 deactivatedAt,
        string reason
    );

    /**
     * @dev Emitted when issuer schema authorization is granted
     */
    event IssuerSchemaAuthorized(
        address indexed issuer,
        bytes32 indexed schemaId,
        address indexed authorizedBy
    );

    /**
     * @dev Emitted when issuer schema authorization is revoked
     */
    event IssuerSchemaRevoked(
        address indexed issuer,
        bytes32 indexed schemaId,
        address indexed revokedBy
    );

    /**
     * @dev Emitted when attestation data is updated
     */
    event AttestationUpdated(
        uint256 indexed id,
        bytes data,
        string uri,
        bytes32[] tags,
        uint256 version,
        address indexed updatedBy
    );

    // ============ MODIFIERS ============

    /**
     * @dev Modifier to check if attestation exists
     */
    modifier attestationExists(uint256 attestationId) {
        require(attestations[attestationId].id != 0, "Attestation does not exist");
        _;
    }

    /**
     * @dev Modifier to check if schema exists
     */
    modifier schemaExists(bytes32 schemaId) {
        require(schemaExists[schemaId], "Schema does not exist");
        _;
    }

    /**
     * @dev Modifier to check if issuer exists
     */
    modifier issuerExists(address issuerAddr) {
        require(issuerExists[issuerAddr], "Issuer does not exist");
        _;
    }

    /**
     * @dev Modifier to check if caller is authorized for schema
     */
    modifier authorizedForSchema(bytes32 schemaId) {
        require(
            issuerSchemaAuthorization[schemaId][msg.sender] || msg.sender == owner(),
            "Not authorized for schema"
        );
        _;
    }

    /**
     * @dev Modifier to check if attestation is not expired
     */
    modifier notExpired(uint256 attestationId) {
        Attestation storage attestation = attestations[attestationId];
        require(
            attestation.expiresAt == 0 || block.timestamp < attestation.expiresAt,
            "Attestation expired"
        );
        _;
    }

    /**
     * @dev Modifier to check if attestation is not revoked
     */
    modifier notRevoked(uint256 attestationId) {
        require(!attestations[attestationId].revoked, "Attestation revoked");
        _;
    }

    // ============ CONSTRUCTOR ============

    constructor() {
        // Initialize with default schema
        _createDefaultSchema();
    }

    // ============ CORE FUNCTIONS ============

    /**
     * @dev Issue a new attestation
     * @param subject Address being attested
     * @param schemaId Schema identifier
     * @param data Attestation data
     * @param expiresAt Expiration timestamp (0 = never)
     * @param uri URI to additional metadata
     * @param tags Tags for categorization
     */
    function issueAttestation(
        address subject,
        bytes32 schemaId,
        bytes calldata data,
        uint256 expiresAt,
        string calldata uri,
        bytes32[] calldata tags
    ) external nonReentrant authorizedForSchema(schemaId) returns (uint256) {
        require(subject != address(0), "Invalid subject address");
        require(data.length <= maxAttestationDataSize, "Data too large");
        require(tags.length <= maxTagsPerAttestation, "Too many tags");
        
        if (expiresAt > 0) {
            require(
                expiresAt > block.timestamp + minAttestationLifetime,
                "Expiration too soon"
            );
            require(
                expiresAt <= block.timestamp + maxAttestationLifetime,
                "Expiration too far"
            );
        }

        if (attestationFeeEnabled && attestationFee > 0) {
            require(msg.value >= attestationFee, "Insufficient fee");
        }

        _attestationIds.increment();
        uint256 attestationId = _attestationIds.current();

        Attestation storage attestation = attestations[attestationId];
        attestation.id = attestationId;
        attestation.issuer = msg.sender;
        attestation.subject = subject;
        attestation.schemaId = schemaId;
        attestation.data = data;
        attestation.issuedAt = block.timestamp;
        attestation.expiresAt = expiresAt;
        attestation.revoked = false;
        attestation.uri = uri;
        attestation.tags = tags;
        attestation.version = 1;

        // Update mappings and arrays
        subjectAttestations[subject].push(attestationId);
        issuerAttestations[msg.sender].push(attestationId);
        schemaAttestations[schemaId].push(attestationId);
        allAttestationIds.push(attestationId);

        // Update issuer stats
        issuers[msg.sender].totalAttestations++;

        emit AttestationIssued(
            attestationId,
            msg.sender,
            subject,
            schemaId,
            block.timestamp,
            expiresAt,
            uri,
            tags,
            1
        );

        return attestationId;
    }

    /**
     * @dev Revoke an attestation
     * @param attestationId ID of attestation to revoke
     * @param reason Reason for revocation
     */
    function revokeAttestation(
        uint256 attestationId,
        string calldata reason
    ) external attestationExists(attestationId) notExpired(attestationId) notRevoked(attestationId) {
        Attestation storage attestation = attestations[attestationId];
        
        require(
            msg.sender == attestation.issuer || msg.sender == owner(),
            "Not authorized to revoke"
        );

        attestation.revoked = true;
        attestation.revokedAt = block.timestamp;
        attestation.revokedBy = msg.sender;

        emit AttestationRevoked(
            attestationId,
            attestation.issuer,
            attestation.subject,
            block.timestamp,
            msg.sender,
            reason
        );
    }

    /**
     * @dev Update attestation data
     * @param attestationId ID of attestation to update
     * @param data New attestation data
     * @param uri New URI
     * @param tags New tags
     */
    function updateAttestation(
        uint256 attestationId,
        bytes calldata data,
        string calldata uri,
        bytes32[] calldata tags
    ) external attestationExists(attestationId) notExpired(attestationId) notRevoked(attestationId) {
        Attestation storage attestation = attestations[attestationId];
        
        require(
            msg.sender == attestation.issuer || msg.sender == owner(),
            "Not authorized to update"
        );
        require(data.length <= maxAttestationDataSize, "Data too large");
        require(tags.length <= maxTagsPerAttestation, "Too many tags");

        attestation.data = data;
        attestation.uri = uri;
        attestation.tags = tags;
        attestation.version++;

        emit AttestationUpdated(
            attestationId,
            data,
            uri,
            tags,
            attestation.version,
            msg.sender
        );
    }

    // ============ SCHEMA MANAGEMENT ============

    /**
     * @dev Create a new schema
     * @param name Schema name
     * @param description Schema description
     * @param fields Field identifiers
     * @param fieldTypes Field type definitions
     * @param version Schema version
     */
    function createSchema(
        string calldata name,
        string calldata description,
        bytes32[] calldata fields,
        string[] calldata fieldTypes,
        string calldata version
    ) external returns (bytes32) {
        require(fields.length > 0, "Schema must have fields");
        require(fields.length == fieldTypes.length, "Fields and types mismatch");
        require(bytes(name).length > 0, "Name cannot be empty");

        bytes32 schemaId = keccak256(abi.encodePacked(name, version, block.timestamp));
        
        Schema storage schema = schemas[schemaId];
        schema.id = schemaId;
        schema.name = name;
        schema.description = description;
        schema.fields = fields;
        schema.fieldTypes = fieldTypes;
        schema.required = false;
        schema.createdAt = block.timestamp;
        schema.createdBy = msg.sender;
        schema.deprecated = false;
        schema.version = version;

        schemaExists[schemaId] = true;
        allSchemaIds.push(schemaId);

        emit SchemaCreated(
            schemaId,
            name,
            description,
            fields,
            fieldTypes,
            msg.sender,
            version
        );

        return schemaId;
    }

    /**
     * @dev Deprecate a schema
     * @param schemaId Schema identifier
     * @param reason Reason for deprecation
     */
    function deprecateSchema(
        bytes32 schemaId,
        string calldata reason
    ) external schemaExists(schemaId) {
        require(
            msg.sender == schemas[schemaId].createdBy || msg.sender == owner(),
            "Not authorized to deprecate"
        );

        Schema storage schema = schemas[schemaId];
        schema.deprecated = true;
        schema.deprecatedAt = block.timestamp;

        emit SchemaDeprecated(
            schemaId,
            msg.sender,
            block.timestamp,
            reason
        );
    }

    // ============ ISSUER MANAGEMENT ============

    /**
     * @dev Register a new issuer
     * @param name Issuer name
     * @param description Issuer description
     * @param uri URI to issuer metadata
     */
    function registerIssuer(
        string calldata name,
        string calldata description,
        string calldata uri
    ) external returns (bool) {
        require(!issuerExists[msg.sender], "Issuer already registered");
        require(bytes(name).length > 0, "Name cannot be empty");

        Issuer storage issuer = issuers[msg.sender];
        issuer.addr = msg.sender;
        issuer.name = name;
        issuer.description = description;
        issuer.uri = uri;
        issuer.active = true;
        issuer.registeredAt = block.timestamp;
        issuer.totalAttestations = 0;

        issuerExists[msg.sender] = true;
        allIssuerAddresses.push(msg.sender);

        emit IssuerRegistered(
            msg.sender,
            name,
            description,
            uri,
            msg.sender
        );

        return true;
    }

    /**
     * @dev Update issuer information
     * @param name New issuer name
     * @param description New issuer description
     * @param uri New URI
     */
    function updateIssuer(
        string calldata name,
        string calldata description,
        string calldata uri
    ) external issuerExists(msg.sender) {
        require(bytes(name).length > 0, "Name cannot be empty");

        Issuer storage issuer = issuers[msg.sender];
        issuer.name = name;
        issuer.description = description;
        issuer.uri = uri;

        emit IssuerUpdated(
            msg.sender,
            name,
            description,
            uri,
            msg.sender
        );
    }

    /**
     * @dev Deactivate an issuer
     * @param reason Reason for deactivation
     */
    function deactivateIssuer(string calldata reason) external issuerExists(msg.sender) {
        issuers[msg.sender].active = false;

        emit IssuerDeactivated(
            msg.sender,
            msg.sender,
            block.timestamp,
            reason
        );
    }

    /**
     * @dev Authorize issuer for schema
     * @param issuer Address of issuer
     * @param schemaId Schema identifier
     */
    function authorizeIssuerForSchema(
        address issuer,
        bytes32 schemaId
    ) external onlyOwner schemaExists(schemaId) {
        require(issuerExists[issuer], "Issuer does not exist");
        
        issuerSchemaAuthorization[schemaId][issuer] = true;
        issuers[issuer].authorizedSchemas.push(schemaId);
        issuers[issuer].isAuthorizedForSchema[schemaId] = true;

        emit IssuerSchemaAuthorized(
            issuer,
            schemaId,
            msg.sender
        );
    }

    /**
     * @dev Revoke issuer schema authorization
     * @param issuer Address of issuer
     * @param schemaId Schema identifier
     */
    function revokeIssuerSchemaAuthorization(
        address issuer,
        bytes32 schemaId
    ) external onlyOwner schemaExists(schemaId) {
        issuerSchemaAuthorization[schemaId][issuer] = false;
        issuers[issuer].isAuthorizedForSchema[schemaId] = false;

        emit IssuerSchemaRevoked(
            issuer,
            schemaId,
            msg.sender
        );
    }

    // ============ QUERY FUNCTIONS ============

    /**
     * @dev Get attestation by ID
     * @param attestationId Attestation identifier
     * @return Attestation data
     */
    function getAttestation(uint256 attestationId) external view returns (Attestation memory) {
        return attestations[attestationId];
    }

    /**
     * @dev Get attestations for a subject
     * @param subject Subject address
     * @return Array of attestation IDs
     */
    function getSubjectAttestations(address subject) external view returns (uint256[] memory) {
        return subjectAttestations[subject];
    }

    /**
     * @dev Get attestations issued by an issuer
     * @param issuer Issuer address
     * @return Array of attestation IDs
     */
    function getIssuerAttestations(address issuer) external view returns (uint256[] memory) {
        return issuerAttestations[issuer];
    }

    /**
     * @dev Get attestations for a schema
     * @param schemaId Schema identifier
     * @return Array of attestation IDs
     */
    function getSchemaAttestations(bytes32 schemaId) external view returns (uint256[] memory) {
        return schemaAttestations[schemaId];
    }

    /**
     * @dev Get schema by ID
     * @param schemaId Schema identifier
     * @return Schema data
     */
    function getSchema(bytes32 schemaId) external view returns (Schema memory) {
        return schemas[schemaId];
    }

    /**
     * @dev Get issuer by address
     * @param issuerAddr Issuer address
     * @return Issuer data
     */
    function getIssuer(address issuerAddr) external view returns (Issuer memory) {
        return issuers[issuerAddr];
    }

    /**
     * @dev Check if attestation is valid (not expired and not revoked)
     * @param attestationId Attestation identifier
     * @return True if valid
     */
    function isAttestationValid(uint256 attestationId) external view returns (bool) {
        Attestation storage attestation = attestations[attestationId];
        
        if (attestation.id == 0) return false;
        if (attestation.revoked) return false;
        if (attestation.expiresAt > 0 && block.timestamp >= attestation.expiresAt) return false;
        
        return true;
    }

    /**
     * @dev Get total attestation count
     * @return Total number of attestations
     */
    function getTotalAttestationCount() external view returns (uint256) {
        return allAttestationIds.length;
    }

    /**
     * @dev Get total schema count
     * @return Total number of schemas
     */
    function getTotalSchemaCount() external view returns (uint256) {
        return allSchemaIds.length;
    }

    /**
     * @dev Get total issuer count
     * @return Total number of issuers
     */
    function getTotalIssuerCount() external view returns (uint256) {
        return allIssuerAddresses.length;
    }

    // ============ ADMIN FUNCTIONS ============

    /**
     * @dev Set attestation fee
     * @param fee New fee amount
     */
    function setAttestationFee(uint256 fee) external onlyOwner {
        attestationFee = fee;
    }

    /**
     * @dev Enable/disable attestation fee
     * @param enabled Whether fee is enabled
     */
    function setAttestationFeeEnabled(bool enabled) external onlyOwner {
        attestationFeeEnabled = enabled;
    }

    /**
     * @dev Set minimum attestation lifetime
     * @param lifetime New minimum lifetime
     */
    function setMinAttestationLifetime(uint256 lifetime) external onlyOwner {
        minAttestationLifetime = lifetime;
    }

    /**
     * @dev Set maximum attestation lifetime
     * @param lifetime New maximum lifetime
     */
    function setMaxAttestationLifetime(uint256 lifetime) external onlyOwner {
        maxAttestationLifetime = lifetime;
    }

    /**
     * @dev Set maximum tags per attestation
     * @param maxTags New maximum tags
     */
    function setMaxTagsPerAttestation(uint256 maxTags) external onlyOwner {
        maxTagsPerAttestation = maxTags;
    }

    /**
     * @dev Set maximum attestation data size
     * @param maxSize New maximum size
     */
    function setMaxAttestationDataSize(uint256 maxSize) external onlyOwner {
        maxAttestationDataSize = maxSize;
    }

    /**
     * @dev Withdraw collected fees
     */
    function withdrawFees() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No fees to withdraw");
        
        (bool success, ) = payable(owner()).call{value: balance}("");
        require(success, "Withdrawal failed");
    }

    // ============ INTERNAL FUNCTIONS ============

    /**
     * @dev Create default schema for basic attestations
     */
    function _createDefaultSchema() internal {
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        
        Schema storage schema = schemas[defaultSchemaId];
        schema.id = defaultSchemaId;
        schema.name = "Default Attestation";
        schema.description = "Basic attestation schema for general use";
        schema.fields = new bytes32[](2);
        schema.fields[0] = keccak256("type");
        schema.fields[1] = keccak256("value");
        schema.fieldTypes = new string[](2);
        schema.fieldTypes[0] = "string";
        schema.fieldTypes[1] = "string";
        schema.required = true;
        schema.createdAt = block.timestamp;
        schema.createdBy = address(this);
        schema.deprecated = false;
        schema.version = "1.0.0";

        schemaExists[defaultSchemaId] = true;
        allSchemaIds.push(defaultSchemaId);

        emit SchemaCreated(
            defaultSchemaId,
            "Default Attestation",
            "Basic attestation schema for general use",
            schema.fields,
            schema.fieldTypes,
            address(this),
            "1.0.0"
        );
    }

    // ============ FALLBACK FUNCTIONS ============

    /**
     * @dev Fallback function to receive ETH
     */
    receive() external payable {}

    /**
     * @dev Fallback function
     */
    fallback() external payable {}
}
