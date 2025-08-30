// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../AttestationRegistry.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

contract AttestationRegistryTest is Test {
    using Strings for uint256;

    AttestationRegistry public registry;
    address public admin;
    address public issuer1;
    address public issuer2;
    address public subject1;
    address public subject2;
    address public user;

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

    event SchemaCreated(
        bytes32 indexed id,
        string name,
        string description,
        bytes32[] fields,
        string[] fieldTypes,
        address indexed createdBy,
        string version
    );

    event IssuerRegistered(
        address indexed addr,
        string name,
        string description,
        string uri,
        address indexed registeredBy
    );

    function setUp() public {
        admin = makeAddr("admin");
        issuer1 = makeAddr("issuer1");
        issuer2 = makeAddr("issuer2");
        subject1 = makeAddr("subject1");
        subject2 = makeAddr("subject2");
        user = makeAddr("user");

        vm.startPrank(admin);
        registry = new AttestationRegistry();
        vm.stopPrank();

        // Fund accounts
        vm.deal(issuer1, 100 ether);
        vm.deal(issuer2, 100 ether);
        vm.deal(subject1, 100 ether);
        vm.deal(subject2, 100 ether);
        vm.deal(user, 100 ether);
    }

    // ============ CONSTRUCTOR TESTS ============

    function testConstructor() public {
        assertEq(registry.owner(), admin);
        assertEq(registry.getTotalSchemaCount(), 1); // Default schema
        assertEq(registry.getTotalAttestationCount(), 0);
        assertEq(registry.getTotalIssuerCount(), 0);
    }

    // ============ SCHEMA MANAGEMENT TESTS ============

    function testCreateSchema() public {
        vm.startPrank(issuer1);
        
        bytes32[] memory fields = new bytes32[](2);
        fields[0] = keccak256("name");
        fields[1] = keccak256("age");
        
        string[] memory fieldTypes = new string[](2);
        fieldTypes[0] = "string";
        fieldTypes[1] = "uint256";
        
        vm.expectEmit(true, false, false, true);
        emit SchemaCreated(
            bytes32(0), // schemaId will be generated
            "Person",
            "Person identification schema",
            fields,
            fieldTypes,
            issuer1,
            "1.0.0"
        );
        
        bytes32 schemaId = registry.createSchema(
            "Person",
            "Person identification schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        
        assertTrue(schemaId != bytes32(0));
        assertEq(registry.getTotalSchemaCount(), 2); // Default + new
        
        vm.stopPrank();
    }

    function testCreateSchemaWithEmptyFields() public {
        vm.startPrank(issuer1);
        
        bytes32[] memory fields = new bytes32[](0);
        string[] memory fieldTypes = new string[](0);
        
        vm.expectRevert("Schema must have fields");
        registry.createSchema(
            "Empty",
            "Empty schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        
        vm.stopPrank();
    }

    function testCreateSchemaWithMismatchedFields() public {
        vm.startPrank(issuer1);
        
        bytes32[] memory fields = new bytes32[](2);
        fields[0] = keccak256("name");
        fields[1] = keccak256("age");
        
        string[] memory fieldTypes = new string[](1);
        fieldTypes[0] = "string";
        
        vm.expectRevert("Fields and types mismatch");
        registry.createSchema(
            "Mismatch",
            "Mismatched schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        
        vm.stopPrank();
    }

    function testDeprecateSchema() public {
        vm.startPrank(issuer1);
        
        // Create a schema first
        bytes32[] memory fields = new bytes32[](1);
        fields[0] = keccak256("name");
        string[] memory fieldTypes = new string[](1);
        fieldTypes[0] = "string";
        
        bytes32 schemaId = registry.createSchema(
            "Test",
            "Test schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        
        // Deprecate it
        registry.deprecateSchema(schemaId, "No longer needed");
        
        // Verify it's deprecated
        AttestationRegistry.Schema memory schema = registry.getSchema(schemaId);
        assertTrue(schema.deprecated);
        assertEq(schema.deprecatedAt, block.timestamp);
        
        vm.stopPrank();
    }

    function testDeprecateSchemaUnauthorized() public {
        vm.startPrank(issuer1);
        
        // Create a schema
        bytes32[] memory fields = new bytes32[](1);
        fields[0] = keccak256("name");
        string[] memory fieldTypes = new string[](1);
        fieldTypes[0] = "string";
        
        bytes32 schemaId = registry.createSchema(
            "Test",
            "Test schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        
        vm.stopPrank();
        
        // Try to deprecate from different account
        vm.startPrank(issuer2);
        vm.expectRevert("Not authorized to deprecate");
        registry.deprecateSchema(schemaId, "Not authorized");
        vm.stopPrank();
    }

    // ============ ISSUER MANAGEMENT TESTS ============

    function testRegisterIssuer() public {
        vm.startPrank(issuer1);
        
        vm.expectEmit(true, false, false, true);
        emit IssuerRegistered(
            issuer1,
            "Test Issuer",
            "Test issuer description",
            "https://example.com",
            issuer1
        );
        
        bool success = registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        
        assertTrue(success);
        assertEq(registry.getTotalIssuerCount(), 1);
        
        AttestationRegistry.Issuer memory issuer = registry.getIssuer(issuer1);
        assertEq(issuer.addr, issuer1);
        assertEq(issuer.name, "Test Issuer");
        assertEq(issuer.description, "Test issuer description");
        assertEq(issuer.uri, "https://example.com");
        assertTrue(issuer.active);
        
        vm.stopPrank();
    }

    function testRegisterIssuerDuplicate() public {
        vm.startPrank(issuer1);
        
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        
        vm.expectRevert("Issuer already registered");
        registry.registerIssuer(
            "Another Issuer",
            "Another description",
            "https://example2.com"
        );
        
        vm.stopPrank();
    }

    function testUpdateIssuer() public {
        vm.startPrank(issuer1);
        
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        
        registry.updateIssuer(
            "Updated Issuer",
            "Updated description",
            "https://updated.com"
        );
        
        AttestationRegistry.Issuer memory issuer = registry.getIssuer(issuer1);
        assertEq(issuer.name, "Updated Issuer");
        assertEq(issuer.description, "Updated description");
        assertEq(issuer.uri, "https://updated.com");
        
        vm.stopPrank();
    }

    function testDeactivateIssuer() public {
        vm.startPrank(issuer1);
        
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        
        registry.deactivateIssuer("No longer active");
        
        AttestationRegistry.Issuer memory issuer = registry.getIssuer(issuer1);
        assertFalse(issuer.active);
        
        vm.stopPrank();
    }

    // ============ AUTHORIZATION TESTS ============

    function testAuthorizeIssuerForSchema() public {
        vm.startPrank(admin);
        
        // Register issuer first
        vm.stopPrank();
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        // Create schema
        vm.startPrank(issuer2);
        bytes32[] memory fields = new bytes32[](1);
        fields[0] = keccak256("name");
        string[] memory fieldTypes = new string[](1);
        fieldTypes[0] = "string";
        
        bytes32 schemaId = registry.createSchema(
            "Test",
            "Test schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        vm.stopPrank();
        
        // Authorize issuer for schema
        vm.startPrank(admin);
        registry.authorizeIssuerForSchema(issuer1, schemaId);
        
        // Verify authorization
        assertTrue(registry.issuerSchemaAuthorization(schemaId, issuer1));
        
        vm.stopPrank();
    }

    function testAuthorizeIssuerForSchemaUnauthorized() public {
        vm.startPrank(issuer1);
        
        vm.expectRevert("Ownable: caller is not the owner");
        registry.authorizeIssuerForSchema(issuer2, bytes32(0));
        
        vm.stopPrank();
    }

    function testRevokeIssuerSchemaAuthorization() public {
        vm.startPrank(admin);
        
        // Register issuer first
        vm.stopPrank();
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        // Create schema
        vm.startPrank(issuer2);
        bytes32[] memory fields = new bytes32[](1);
        fields[0] = keccak256("name");
        string[] memory fieldTypes = new string[](1);
        fieldTypes[0] = "string";
        
        bytes32 schemaId = registry.createSchema(
            "Test",
            "Test schema",
            fields,
            fieldTypes,
            "1.0.0"
        );
        vm.stopPrank();
        
        // Authorize and then revoke
        vm.startPrank(admin);
        registry.authorizeIssuerForSchema(issuer1, schemaId);
        assertTrue(registry.issuerSchemaAuthorization(schemaId, issuer1));
        
        registry.revokeIssuerSchemaAuthorization(issuer1, schemaId);
        assertFalse(registry.issuerSchemaAuthorization(schemaId, issuer1));
        
        vm.stopPrank();
    }

    // ============ ATTESTATION TESTS ============

    function testIssueAttestation() public {
        // Setup: register issuer and authorize for default schema
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        // Issue attestation
        vm.startPrank(issuer1);
        
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](2);
        tags[0] = keccak256("person");
        tags[1] = keccak256("identification");
        
        vm.expectEmit(true, true, true, true);
        emit AttestationIssued(
            1, // attestationId
            issuer1,
            subject1,
            defaultSchemaId,
            block.timestamp,
            0, // expiresAt
            "https://example.com/attestation/1",
            tags,
            1 // version
        );
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0, // never expires
            "https://example.com/attestation/1",
            tags
        );
        
        assertEq(attestationId, 1);
        assertEq(registry.getTotalAttestationCount(), 1);
        
        // Verify attestation
        AttestationRegistry.Attestation memory attestation = registry.getAttestation(attestationId);
        assertEq(attestation.id, 1);
        assertEq(attestation.issuer, issuer1);
        assertEq(attestation.subject, subject1);
        assertEq(attestation.schemaId, defaultSchemaId);
        assertEq(attestation.data, data);
        assertEq(attestation.issuedAt, block.timestamp);
        assertEq(attestation.expiresAt, 0);
        assertFalse(attestation.revoked);
        assertEq(attestation.uri, "https://example.com/attestation/1");
        assertEq(attestation.tags.length, 2);
        assertEq(attestation.version, 1);
        
        vm.stopPrank();
    }

    function testIssueAttestationUnauthorized() public {
        vm.startPrank(issuer1);
        
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        vm.expectRevert("Not authorized for schema");
        registry.issueAttestation(
            subject1,
            bytes32(0),
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        vm.stopPrank();
    }

    function testIssueAttestationWithExpiry() public {
        // Setup: register issuer and authorize for default schema
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        // Issue attestation with expiry
        vm.startPrank(issuer1);
        
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 expiresAt = block.timestamp + 365 days;
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            expiresAt,
            "https://example.com/attestation/1",
            tags
        );
        
        AttestationRegistry.Attestation memory attestation = registry.getAttestation(attestationId);
        assertEq(attestation.expiresAt, expiresAt);
        
        vm.stopPrank();
    }

    function testIssueAttestationExpiryTooSoon() public {
        // Setup: register issuer and authorize for default schema
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        // Try to issue attestation with expiry too soon
        vm.startPrank(issuer1);
        
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 expiresAt = block.timestamp + 12 hours; // Less than 1 day minimum
        
        vm.expectRevert("Expiration too soon");
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            expiresAt,
            "https://example.com/attestation/1",
            tags
        );
        
        vm.stopPrank();
    }

    function testIssueAttestationExpiryTooFar() public {
        // Setup: register issuer and authorize for default schema
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        // Try to issue attestation with expiry too far
        vm.startPrank(issuer1);
        
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 expiresAt = block.timestamp + 2 * 365 days; // More than 1 year maximum
        
        vm.expectRevert("Expiration too far");
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            expiresAt,
            "https://example.com/attestation/1",
            tags
        );
        
        vm.stopPrank();
    }

    function testIssueAttestationTooManyTags() public {
        // Setup: register issuer and authorize for default schema
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        // Try to issue attestation with too many tags
        vm.startPrank(issuer1);
        
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](11); // More than max 10
        for (uint i = 0; i < 11; i++) {
            tags[i] = keccak256(abi.encodePacked("tag", i));
        }
        
        vm.expectRevert("Too many tags");
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        vm.stopPrank();
    }

    function testIssueAttestationDataTooLarge() public {
        // Setup: register issuer and authorize for default schema
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        // Try to issue attestation with data too large
        vm.startPrank(issuer1);
        
        // Create data larger than 1KB
        bytes memory data = new bytes(1025);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        vm.expectRevert("Data too large");
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        vm.stopPrank();
    }

    // ============ REVOCATION TESTS ============

    function testRevokeAttestation() public {
        // Setup: create attestation first
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        // Revoke attestation
        registry.revokeAttestation(attestationId, "Information incorrect");
        
        AttestationRegistry.Attestation memory attestation = registry.getAttestation(attestationId);
        assertTrue(attestation.revoked);
        assertEq(attestation.revokedAt, block.timestamp);
        assertEq(attestation.revokedBy, issuer1);
        
        vm.stopPrank();
    }

    function testRevokeAttestationUnauthorized() public {
        // Setup: create attestation first
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        vm.stopPrank();
        
        // Try to revoke from different account
        vm.startPrank(issuer2);
        vm.expectRevert("Not authorized to revoke");
        registry.revokeAttestation(attestationId, "Not authorized");
        vm.stopPrank();
    }

    function testRevokeAttestationAlreadyRevoked() public {
        // Setup: create attestation first
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        // Revoke once
        registry.revokeAttestation(attestationId, "First revocation");
        
        // Try to revoke again
        vm.expectRevert("Attestation already revoked");
        registry.revokeAttestation(attestationId, "Second revocation");
        
        vm.stopPrank();
    }

    // ============ UPDATE TESTS ============

    function testUpdateAttestation() public {
        // Setup: create attestation first
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        // Update attestation
        bytes memory newData = abi.encode("John Doe", 31);
        bytes32[] memory newTags = new bytes32[](2);
        newTags[0] = keccak256("person");
        newTags[1] = keccak256("updated");
        
        registry.updateAttestation(
            attestationId,
            newData,
            "https://example.com/attestation/1/updated",
            newTags
        );
        
        AttestationRegistry.Attestation memory attestation = registry.getAttestation(attestationId);
        assertEq(attestation.data, newData);
        assertEq(attestation.uri, "https://example.com/attestation/1/updated");
        assertEq(attestation.tags.length, 2);
        assertEq(attestation.version, 2);
        
        vm.stopPrank();
    }

    // ============ QUERY TESTS ============

    function testGetSubjectAttestations() public {
        // Setup: create multiple attestations
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/2",
            tags
        );
        
        vm.stopPrank();
        
        // Query attestations for subject
        uint256[] memory attestations = registry.getSubjectAttestations(subject1);
        assertEq(attestations.length, 2);
        assertEq(attestations[0], 1);
        assertEq(attestations[1], 2);
    }

    function testGetIssuerAttestations() public {
        // Setup: create attestations
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        registry.issueAttestation(
            subject2,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/2",
            tags
        );
        
        vm.stopPrank();
        
        // Query attestations for issuer
        uint256[] memory attestations = registry.getIssuerAttestations(issuer1);
        assertEq(attestations.length, 2);
        assertEq(attestations[0], 1);
        assertEq(attestations[1], 2);
    }

    function testIsAttestationValid() public {
        // Setup: create attestation
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        // Check validity
        assertTrue(registry.isAttestationValid(attestationId));
        
        // Revoke and check again
        registry.revokeAttestation(attestationId, "Test revocation");
        assertFalse(registry.isAttestationValid(attestationId));
        
        vm.stopPrank();
    }

    // ============ ADMIN TESTS ============

    function testUpdateConfig() public {
        vm.startPrank(admin);
        
        // Update various config values
        registry.setAttestationFee(1000);
        registry.setAttestationFeeEnabled(true);
        registry.setMinAttestationLifetime(2 days);
        registry.setMaxAttestationLifetime(730 days);
        registry.setMaxTagsPerAttestation(20);
        registry.setMaxAttestationDataSize(2048);
        
        // Verify updates
        assertEq(registry.attestationFee(), 1000);
        assertTrue(registry.attestationFeeEnabled());
        assertEq(registry.minAttestationLifetime(), 2 days);
        assertEq(registry.maxAttestationLifetime(), 730 days);
        assertEq(registry.maxTagsPerAttestation(), 20);
        assertEq(registry.maxAttestationDataSize(), 2048);
        
        vm.stopPrank();
    }

    function testUpdateConfigUnauthorized() public {
        vm.startPrank(user);
        
        vm.expectRevert("Ownable: caller is not the owner");
        registry.setAttestationFee(1000);
        
        vm.stopPrank();
    }

    // ============ EDGE CASES ============

    function testAttestationWithZeroSubject() public {
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        vm.expectRevert("Invalid subject address");
        registry.issueAttestation(
            address(0),
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        vm.stopPrank();
    }

    function testAttestationWithEmptyData() public {
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = "";
        bytes32[] memory tags = new bytes32[](1);
        tags[0] = keccak256("person");
        
        // Empty data should be allowed
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        assertEq(attestationId, 1);
        
        vm.stopPrank();
    }

    function testAttestationWithEmptyTags() public {
        vm.startPrank(issuer1);
        registry.registerIssuer(
            "Test Issuer",
            "Test issuer description",
            "https://example.com"
        );
        vm.stopPrank();
        
        vm.startPrank(admin);
        bytes32 defaultSchemaId = keccak256(abi.encodePacked("default", "1.0.0"));
        registry.authorizeIssuerForSchema(issuer1, defaultSchemaId);
        vm.stopPrank();
        
        vm.startPrank(issuer1);
        bytes memory data = abi.encode("John Doe", 30);
        bytes32[] memory tags = new bytes32[](0);
        
        // Empty tags should be allowed
        uint256 attestationId = registry.issueAttestation(
            subject1,
            defaultSchemaId,
            data,
            0,
            "https://example.com/attestation/1",
            tags
        );
        
        assertEq(attestationId, 1);
        
        vm.stopPrank();
    }
}
