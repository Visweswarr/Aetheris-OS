package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// VaultResult represents the result of a vault operation
type VaultResult struct {
	Tool    string `json:"tool"`
	Op      string `json:"op"`
	Success bool   `json:"success"`
	Error   string `json:"error,omitempty"`
	Data    string `json:"data,omitempty"`
}

// VaultOptions represents command line options
type VaultOptions struct {
	Operation string
	ID        string
	Kind      string
	File      string
	CapToken  string
	Verbose   bool
	JSON      bool
	Help      bool
}

// Mock vault entry for testing
type MockVaultEntry struct {
	ID          string            `json:"id"`
	Kind        string            `json:"kind"`
	SubjectDID  string            `json:"subject_did"`
	Size        int64             `json:"size"`
	CreatedAt   string            `json:"created_at"`
	UpdatedAt   string            `json:"updated_at"`
	Tags        []string          `json:"tags"`
	Meta        map[string]string `json:"meta"`
	Revoked     bool              `json:"revoked"`
}

// Mock capability token for testing
type MockCapToken struct {
	Version      int           `json:"version"`
	IssuerDID    string        `json:"issuer_did"`
	SubjectDID   string        `json:"subject_did"`
	Capabilities []Capability  `json:"capabilities"`
	IssuedAt     string        `json:"issued_at"`
	ExpiresAt    string        `json:"expires_at"`
	Signature    string        `json:"signature"`
	Nonce        string        `json:"nonce"`
}

// Capability represents a vault operation capability
type Capability struct {
	Operation string `json:"operation"`
	Resource  string `json:"resource"`
	ExpiresAt string `json:"expires_at,omitempty"`
}

func main() {
	opts := parseFlags()
	
	if opts.Help {
		showHelp()
		return
	}

	result := executeVaultOperation(opts)
	outputResult(result, opts.JSON)
}

func parseFlags() VaultOptions {
	var opts VaultOptions
	
	flag.StringVar(&opts.Operation, "op", "", "Operation: add, list, show, delete")
	flag.StringVar(&opts.ID, "id", "", "Entry ID")
	flag.StringVar(&opts.Kind, "kind", "", "Entry kind: key, document, credential, secret, backup")
	flag.StringVar(&opts.File, "file", "", "File path for add operation")
	flag.StringVar(&opts.CapToken, "cap", "", "Capability token file")
	flag.BoolVar(&opts.Verbose, "verbose", false, "Verbose output")
	flag.BoolVar(&opts.JSON, "json", false, "Output JSON format")
	flag.BoolVar(&opts.Help, "help", false, "Show help")
	
	flag.Parse()
	
	// Handle legacy flags for backward compatibility
	if opts.Operation == "" {
		if len(flag.Args()) > 0 {
			opts.Operation = flag.Args()[0]
		}
	}
	
	return opts
}

func showHelp() {
	fmt.Println(`NGFS Vault - Personal Data Vault CLI

Usage: ngfs-vault [OPTIONS] <OPERATION>

Operations:
  add     - Add a new vault entry
  list    - List vault entries
  show    - Show vault entry content
  delete  - Delete a vault entry (logical tombstone)

Options:
  -op string     Operation: add, list, show, delete
  -id string     Entry ID
  -kind string   Entry kind: key, document, credential, secret, backup
  -file string   File path for add operation
  -cap string   Capability token file
  -verbose      Verbose output
  -json         Output JSON format
  -help         Show this help

Examples:
  ngfs-vault add --id my-key --kind key --file secret.pem --cap token.json
  ngfs-vault list --cap token.json
  ngfs-vault show --id my-key --cap token.json
  ngfs-vault delete --id my-key --cap token.json

Capability Token:
  The --cap flag specifies a JSON file containing a valid CapToken v2
  that grants the necessary capabilities for the requested operation.

Output:
  By default, outputs human-readable text. Use --json for machine-readable
  JSON output suitable for CI integration.
`)
}

func executeVaultOperation(opts VaultOptions) VaultResult {
	// Validate operation
	if opts.Operation == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "error",
			Success: false,
			Error:   "No operation specified",
		}
	}

	// Validate capability token
	if opts.CapToken == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      opts.Operation,
			Success: false,
			Error:   "Capability token required (--cap)",
		}
	}

	// Load capability token
	capToken, err := loadCapToken(opts.CapToken)
	if err != nil {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      opts.Operation,
			Success: false,
			Error:   fmt.Sprintf("Failed to load capability token: %v", err),
		}
	}

	// Validate capability token
	if err := validateCapToken(capToken); err != nil {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      opts.Operation,
			Success: false,
			Error:   fmt.Sprintf("Invalid capability token: %v", err),
		}
	}

	// Execute operation
	switch opts.Operation {
	case "add":
		return executeAdd(opts, capToken)
	case "list":
		return executeList(opts, capToken)
	case "show":
		return executeShow(opts, capToken)
	case "delete":
		return executeDelete(opts, capToken)
	default:
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      opts.Operation,
			Success: false,
			Error:   fmt.Sprintf("Unknown operation: %s", opts.Operation),
		}
	}
}

func executeAdd(opts VaultOptions, capToken *MockCapToken) VaultResult {
	// Validate required fields
	if opts.ID == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "add",
			Success: false,
			Error:   "Entry ID required (--id)",
		}
	}

	if opts.Kind == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "add",
			Success: false,
			Error:   "Entry kind required (--kind)",
		}
	}

	if opts.File == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "add",
			Success: false,
			Error:   "File path required (--file)",
		}
	}

	// Validate entry kind
	if !isValidEntryKind(opts.Kind) {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "add",
			Success: false,
			Error:   fmt.Sprintf("Invalid entry kind: %s", opts.Kind),
		}
	}

	// Check if user has write capability
	if !hasCapability(capToken, "write", "*") {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "add",
			Success: false,
			Error:   "Insufficient capabilities: write access required",
		}
	}

	// Read file content
	content, err := ioutil.ReadFile(opts.File)
	if err != nil {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "add",
			Success: false,
			Error:   fmt.Sprintf("Failed to read file: %v", err),
		}
	}

	// Create mock vault entry
	entry := MockVaultEntry{
		ID:         opts.ID,
		Kind:       opts.Kind,
		SubjectDID: capToken.SubjectDID,
		Size:       int64(len(content)),
		CreatedAt:  time.Now().UTC().Format(time.RFC3339),
		UpdatedAt:  time.Now().UTC().Format(time.RFC3339),
		Tags:       []string{},
		Meta: map[string]string{
			"filename": filepath.Base(opts.File),
			"path":     opts.File,
		},
		Revoked: false,
	}

	// In real implementation, this would:
	// 1. Encrypt the content using EncEnvelopeV1
	// 2. Store in NGFS CAS
	// 3. Update vault index
	// 4. Audit log the operation

	if opts.Verbose {
		fmt.Printf("Adding vault entry:\n")
		fmt.Printf("  ID: %s\n", entry.ID)
		fmt.Printf("  Kind: %s\n", entry.Kind)
		fmt.Printf("  Size: %d bytes\n", entry.Size)
		fmt.Printf("  Subject DID: %s\n", entry.SubjectDID)
		fmt.Printf("  Created: %s\n", entry.CreatedAt)
	}

	return VaultResult{
		Tool:    "ngfs-vault",
		Op:      "add",
		Success: true,
		Data:    fmt.Sprintf("Entry '%s' added successfully", opts.ID),
	}
}

func executeList(opts VaultOptions, capToken *MockCapToken) VaultResult {
	// Check if user has list capability
	if !hasCapability(capToken, "list", "*") {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "list",
			Success: false,
			Error:   "Insufficient capabilities: list access required",
		}
	}

	// Mock vault entries for testing
	entries := []MockVaultEntry{
		{
			ID:         "my-key",
			Kind:       "key",
			SubjectDID: capToken.SubjectDID,
			Size:       2048,
			CreatedAt:  "2024-01-01T12:00:00Z",
			UpdatedAt:  "2024-01-01T12:00:00Z",
			Tags:       []string{"private", "rsa"},
			Meta:       map[string]string{"algorithm": "RSA-2048"},
			Revoked:    false,
		},
		{
			ID:         "my-document",
			Kind:       "document",
			SubjectDID: capToken.SubjectDID,
			Size:       1024,
			CreatedAt:  "2024-01-02T10:00:00Z",
			UpdatedAt:  "2024-01-02T10:00:00Z",
			Tags:       []string{"personal", "notes"},
			Meta:       map[string]string{"type": "text"},
			Revoked:    false,
		},
	}

	if opts.Verbose {
		fmt.Printf("Vault entries:\n")
		for _, entry := range entries {
			fmt.Printf("  %s (%s) - %d bytes - %s\n", 
				entry.ID, entry.Kind, entry.Size, entry.CreatedAt)
		}
	}

	// Convert to JSON for output
	entriesJSON, _ := json.Marshal(entries)
	
	return VaultResult{
		Tool:    "ngfs-vault",
		Op:      "list",
		Success: true,
		Data:    string(entriesJSON),
	}
}

func executeShow(opts VaultOptions, capToken *MockCapToken) VaultResult {
	// Validate required fields
	if opts.ID == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "show",
			Success: false,
			Error:   "Entry ID required (--id)",
		}
	}

	// Check if user has read capability
	if !hasCapability(capToken, "read", opts.ID) {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "show",
			Success: false,
			Error:   "Insufficient capabilities: read access required",
		}
	}

	// Mock entry retrieval
	entry := MockVaultEntry{
		ID:         opts.ID,
		Kind:       "key",
		SubjectDID: capToken.SubjectDID,
		Size:       2048,
		CreatedAt:  "2024-01-01T12:00:00Z",
		UpdatedAt:  "2024-01-01T12:00:00Z",
		Tags:       []string{"private", "rsa"},
		Meta:       map[string]string{"algorithm": "RSA-2048"},
		Revoked:    false,
	}

	if opts.Verbose {
		fmt.Printf("Entry details:\n")
		fmt.Printf("  ID: %s\n", entry.ID)
		fmt.Printf("  Kind: %s\n", entry.Kind)
		fmt.Printf("  Size: %d bytes\n", entry.Size)
		fmt.Printf("  Subject DID: %s\n", entry.SubjectDID)
		fmt.Printf("  Created: %s\n", entry.CreatedAt)
		fmt.Printf("  Tags: %s\n", strings.Join(entry.Tags, ", "))
		fmt.Printf("  Meta: %v\n", entry.Meta)
	}

	// In real implementation, this would:
	// 1. Retrieve encrypted content from NGFS CAS
	// 2. Decrypt using EncEnvelopeV1
	// 3. Return plaintext content
	// 4. Audit log the access

	// Mock decrypted content
	content := fmt.Sprintf("Mock decrypted content for entry '%s'", opts.ID)

	return VaultResult{
		Tool:    "ngfs-vault",
		Op:      "show",
		Success: true,
		Data:    content,
	}
}

func executeDelete(opts VaultOptions, capToken *MockCapToken) VaultResult {
	// Validate required fields
	if opts.ID == "" {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "delete",
			Success: false,
			Error:   "Entry ID required (--id)",
		}
	}

	// Check if user has delete capability
	if !hasCapability(capToken, "delete", opts.ID) {
		return VaultResult{
			Tool:    "ngfs-vault",
			Op:      "delete",
			Success: false,
			Error:   "Insufficient capabilities: delete access required",
		}
	}

	if opts.Verbose {
		fmt.Printf("Deleting entry: %s\n", opts.ID)
		fmt.Printf("  This will mark the entry as revoked (logical tombstone)\n")
		fmt.Printf("  The encrypted content remains in NGFS CAS\n")
	}

	// In real implementation, this would:
	// 1. Mark entry as revoked in vault index
	// 2. Update entry metadata
	// 3. Audit log the operation
	// 4. Keep encrypted content in CAS (no physical deletion)

	return VaultResult{
		Tool:    "ngfs-vault",
		Op:      "delete",
		Success: true,
		Data:    fmt.Sprintf("Entry '%s' marked as revoked", opts.ID),
	}
}

func loadCapToken(filepath string) (*MockCapToken, error) {
	data, err := ioutil.ReadFile(filepath)
	if err != nil {
		return nil, fmt.Errorf("failed to read capability token file: %v", err)
	}

	var capToken MockCapToken
	if err := json.Unmarshal(data, &capToken); err != nil {
		return nil, fmt.Errorf("failed to parse capability token: %v", err)
	}

	return &capToken, nil
}

func validateCapToken(capToken *MockCapToken) error {
	// Basic validation
	if capToken.Version != 2 {
		return fmt.Errorf("invalid token version: expected 2, got %d", capToken.Version)
	}

	if capToken.IssuerDID == "" {
		return fmt.Errorf("missing issuer DID")
	}

	if capToken.SubjectDID == "" {
		return fmt.Errorf("missing subject DID")
	}

	if len(capToken.Capabilities) == 0 {
		return fmt.Errorf("no capabilities specified")
	}

	// Check expiration
	if capToken.ExpiresAt != "" {
		expiresAt, err := time.Parse(time.RFC3339, capToken.ExpiresAt)
		if err != nil {
			return fmt.Errorf("invalid expiration timestamp: %v", err)
		}

		if time.Now().UTC().After(expiresAt) {
			return fmt.Errorf("capability token has expired")
		}
	}

	return nil
}

func hasCapability(capToken *MockCapToken, operation, resource string) bool {
	for _, cap := range capToken.Capabilities {
		if cap.Operation == operation {
			// Check resource pattern matching
			if cap.Resource == "*" || cap.Resource == resource {
				return true
			}
			
			// Simple wildcard matching
			if strings.HasSuffix(cap.Resource, "*") {
				prefix := cap.Resource[:len(cap.Resource)-1]
				if strings.HasPrefix(resource, prefix) {
					return true
				}
			}
		}
	}
	return false
}

func isValidEntryKind(kind string) bool {
	validKinds := []string{"key", "document", "credential", "secret", "backup"}
	for _, valid := range validKinds {
		if kind == valid {
			return true
		}
	}
	return false
}

func outputResult(result VaultResult, jsonOutput bool) {
	if jsonOutput {
		// Output JSON for machine consumption
		jsonData, _ := json.Marshal(result)
		fmt.Println(string(jsonData))
	} else {
		// Output human-readable text
		if result.Success {
			fmt.Printf("✅ %s: %s\n", result.Op, result.Data)
		} else {
			fmt.Printf("❌ %s failed: %s\n", result.Op, result.Error)
			os.Exit(1)
		}
	}
}

// Test functions for CI integration
func RunTests() error {
	fmt.Println("Running NGFS vault tests...")
	
	// Test capability token validation
	if err := testCapTokenValidation(); err != nil {
		return fmt.Errorf("capability token validation test failed: %v", err)
	}
	
	// Test entry kind validation
	if err := testEntryKindValidation(); err != nil {
		return fmt.Errorf("entry kind validation test failed: %v", err)
	}
	
	// Test capability checking
	if err := testCapabilityChecking(); err != nil {
		return fmt.Errorf("capability checking test failed: %v", err)
	}
	
	fmt.Println("All tests passed!")
	return nil
}

func testCapTokenValidation() error {
	// Test valid token
	validToken := &MockCapToken{
		Version:      2,
		IssuerDID:    "did:example:issuer",
		SubjectDID:   "did:example:subject",
		Capabilities: []Capability{{Operation: "read", Resource: "*"}},
		IssuedAt:     time.Now().UTC().Format(time.RFC3339),
	}
	
	if err := validateCapToken(validToken); err != nil {
		return fmt.Errorf("valid token failed validation: %v", err)
	}
	
	// Test invalid version
	invalidToken := *validToken
	invalidToken.Version = 1
	if err := validateCapToken(&invalidToken); err == nil {
		return fmt.Errorf("invalid version should have failed validation")
	}
	
	return nil
}

func testEntryKindValidation() error {
	validKinds := []string{"key", "document", "credential", "secret", "backup"}
	for _, kind := range validKinds {
		if !isValidEntryKind(kind) {
			return fmt.Errorf("valid kind '%s' failed validation", kind)
		}
	}
	
	invalidKinds := []string{"invalid", "unknown", ""}
	for _, kind := range invalidKinds {
		if isValidEntryKind(kind) {
			return fmt.Errorf("invalid kind '%s' passed validation", kind)
		}
	}
	
	return nil
}

func testCapabilityChecking() error {
	capToken := &MockCapToken{
		Capabilities: []Capability{
			{Operation: "read", Resource: "*"},
			{Operation: "write", Resource: "documents/*"},
			{Operation: "delete", Resource: "temp-*"},
		},
	}
	
	// Test wildcard access
	if !hasCapability(capToken, "read", "any-resource") {
		return fmt.Errorf("wildcard read capability not working")
	}
	
	// Test pattern matching
	if !hasCapability(capToken, "write", "documents/notes.txt") {
		return fmt.Errorf("pattern write capability not working")
	}
	
	if !hasCapability(capToken, "delete", "temp-file") {
		return fmt.Errorf("prefix delete capability not working")
	}
	
	// Test denied operations
	if hasCapability(capToken, "admin", "system") {
		return fmt.Errorf("admin capability should not be granted")
	}
	
	return nil
}
