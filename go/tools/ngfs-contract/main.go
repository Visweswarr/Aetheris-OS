package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Contract execution result
type ContractResult struct {
	Test      string `json:"test"`
	Ok        bool   `json:"ok"`
	Gas       uint64 `json:"gas"`
	GasLimit  uint64 `json:"gas_limit"`
	TimeMs    uint64 `json:"time_ms"`
	Memory    uint64 `json:"memory"`
	ZKProof   bool   `json:"zk_proof"`
	Error     string `json:"error,omitempty"`
	Contract  string `json:"contract"`
	InputSize uint64 `json:"input_size"`
}

// Contract metadata
type ContractMeta struct {
	Name        string   `json:"name"`
	Description string   `json:"description,omitempty"`
	Author      string   `json:"author"`
	Category    string   `json:"category"`
	Tags        []string `json:"tags"`
	GasEstimate uint64   `json:"gas_estimate,omitempty"`
	MaxMemory   uint64   `json:"max_memory,omitempty"`
	TimeoutMs   uint64   `json:"timeout_ms,omitempty"`
}

// Contract definition
type Contract struct {
	ID        string         `json:"id"`
	WASM      []byte         `json:"wasm"`
	Meta      ContractMeta   `json:"meta"`
	ZKMode    bool           `json:"zk_mode"`
	GasLimit  uint64         `json:"gas_limit"`
	Version   string         `json:"version"`
	CreatedAt string         `json:"created_at"`
	UpdatedAt string         `json:"updated_at"`
	OwnerDID  string         `json:"owner_did"`
	Tags      []string       `json:"tags"`
}

// Execution options
type ExecutionOptions struct {
	ContractPath string
	InputPath    string
	GasLimit     uint64
	ZKMode       bool
	Verbose      bool
	JSONOutput   bool
	TimeoutMs    uint64
	Algorithm    string
}

// Mock contract sandbox for demonstration
type MockContractSandbox struct {
	gasUsed    uint64
	memoryUsed uint64
	execTime   time.Duration
	zkProof    []byte
}

func (s *MockContractSandbox) ExecuteContract(wasm []byte, input []byte, gasLimit uint64, zkMode bool) (*ContractResult, error) {
	startTime := time.Now()
	
	// Simulate contract execution
	time.Sleep(10 * time.Millisecond) // Simulate execution time
	
	// Calculate mock gas usage based on input size and WASM size
	s.gasUsed = uint64(len(input)) + uint64(len(wasm)/100)
	if s.gasUsed > gasLimit {
		return &ContractResult{
			Test:      "contract",
			Ok:        false,
			Gas:       s.gasUsed,
			GasLimit:  gasLimit,
			TimeMs:    uint64(time.Since(startTime).Milliseconds()),
			Memory:    s.memoryUsed,
			ZKProof:   false,
			Error:     "gas limit exceeded",
			Contract:  "mock-contract",
			InputSize: uint64(len(input)),
		}, nil
	}
	
	// Simulate memory usage
	s.memoryUsed = uint64(len(wasm) + len(input))
	
	// Generate mock ZK proof if requested
	if zkMode {
		s.zkProof = []byte("mock_zk_proof_data")
	}
	
	execTime := time.Since(startTime)
	
	return &ContractResult{
		Test:      "contract",
		Ok:        true,
		Gas:       s.gasUsed,
		GasLimit:  gasLimit,
		TimeMs:    uint64(execTime.Milliseconds()),
		Memory:    s.memoryUsed,
		ZKProof:   zkMode && len(s.zkProof) > 0,
		Error:     "",
		Contract:  "mock-contract",
		InputSize: uint64(len(input)),
	}, nil
}

func main() {
	var options ExecutionOptions
	
	// Parse command line flags
	flag.StringVar(&options.ContractPath, "contract", "", "Path to WASM contract file")
	flag.StringVar(&options.InputPath, "input", "", "Path to input JSON file")
	flag.Uint64Var(&options.GasLimit, "gas", 1000000, "Gas limit for execution")
	flag.BoolVar(&options.ZKMode, "zk", false, "Enable ZK proof generation")
	flag.BoolVar(&options.Verbose, "verbose", false, "Enable verbose output")
	flag.BoolVar(&options.JSONOutput, "json", false, "Output in JSON format")
	flag.Uint64Var(&options.TimeoutMs, "timeout", 30000, "Execution timeout in milliseconds")
	flag.StringVar(&options.Algorithm, "algorithm", "halo2", "ZK algorithm (halo2, noir, plonk)")
	
	flag.Parse()
	
	// Validate required arguments
	if options.ContractPath == "" {
		log.Fatal("Contract path is required. Use --contract flag.")
	}
	
	if options.InputPath == "" {
		log.Fatal("Input path is required. Use --input flag.")
	}
	
	// Validate gas limit
	if options.GasLimit == 0 || options.GasLimit > 10000000 {
		log.Fatal("Gas limit must be between 1 and 10,000,000")
	}
	
	// Validate ZK algorithm
	validAlgorithms := []string{"halo2", "noir", "plonk", "custom"}
	algorithmValid := false
	for _, alg := range validAlgorithms {
		if options.Algorithm == alg {
			algorithmValid = true
			break
		}
	}
	if !algorithmValid {
		log.Fatalf("Invalid ZK algorithm: %s. Valid options: %v", options.Algorithm, validAlgorithms)
	}
	
	// Load contract
	contract, err := loadContract(options.ContractPath)
	if err != nil {
		log.Fatalf("Failed to load contract: %v", err)
	}
	
	// Load input
	input, err := loadInput(options.InputPath)
	if err != nil {
		log.Fatalf("Failed to load input: %v", err)
	}
	
	if options.Verbose {
		fmt.Printf("Contract: %s\n", contract.Meta.Name)
		fmt.Printf("Category: %s\n", contract.Meta.Category)
		fmt.Printf("Author: %s\n", contract.Meta.Author)
		fmt.Printf("WASM Size: %d bytes\n", len(contract.WASM))
		fmt.Printf("Input Size: %d bytes\n", len(input))
		fmt.Printf("Gas Limit: %d\n", options.GasLimit)
		fmt.Printf("ZK Mode: %v\n", options.ZKMode)
		fmt.Printf("ZK Algorithm: %s\n", options.Algorithm)
		fmt.Printf("Timeout: %d ms\n", options.TimeoutMs)
		fmt.Println()
	}
	
	// Execute contract
	sandbox := &MockContractSandbox{}
	result, err := sandbox.ExecuteContract(contract.WASM, input, options.GasLimit, options.ZKMode)
	if err != nil {
		log.Fatalf("Contract execution failed: %v", err)
	}
	
	// Output result
	if options.JSONOutput {
		outputJSON(result)
	} else {
		outputHuman(result, options.Verbose)
	}
	
	// Exit with appropriate code
	if !result.Ok {
		os.Exit(1)
	}
}

func loadContract(path string) (*Contract, error) {
	// Read WASM file
	wasmData, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read WASM file: %v", err)
	}
	
	// Try to load metadata from JSON file
	metaPath := strings.TrimSuffix(path, ".wasm") + ".json"
	meta := ContractMeta{
		Name:        filepath.Base(path),
		Description: "Contract loaded from " + path,
		Author:      "Unknown",
		Category:    "utility",
		Tags:        []string{"wasm", "contract"},
	}
	
	if metaData, err := ioutil.ReadFile(metaPath); err == nil {
		if err := json.Unmarshal(metaData, &meta); err != nil {
			// Use default metadata if JSON parsing fails
		}
	}
	
	contract := &Contract{
		ID:        filepath.Base(path),
		WASM:      wasmData,
		Meta:      meta,
		ZKMode:    false,
		GasLimit:  1000000,
		Version:   "1.0.0",
		CreatedAt: time.Now().Format(time.RFC3339),
		UpdatedAt: time.Now().Format(time.RFC3339),
		OwnerDID:  "did:aetheris:mock:user",
		Tags:      meta.Tags,
	}
	
	return contract, nil
}

func loadInput(path string) ([]byte, error) {
	// Try to read as JSON first
	if data, err := ioutil.ReadFile(path); err == nil {
		// Validate JSON
		var jsonData interface{}
		if err := json.Unmarshal(data, &jsonData); err == nil {
			return data, nil
		}
	}
	
	// If not valid JSON, read as raw bytes
	return ioutil.ReadFile(path)
}

func outputJSON(result *ContractResult) {
	jsonData, err := json.Marshal(result)
	if err != nil {
		log.Fatalf("Failed to marshal result to JSON: %v", err)
	}
	fmt.Println(string(jsonData))
}

func outputHuman(result *ContractResult, verbose bool) {
	if result.Ok {
		fmt.Printf("✓ Contract executed successfully\n")
		fmt.Printf("  Gas Used: %d / %d\n", result.Gas, result.GasLimit)
		fmt.Printf("  Execution Time: %d ms\n", result.TimeMs)
		fmt.Printf("  Memory Used: %d bytes\n", result.Memory)
		fmt.Printf("  ZK Proof: %v\n", result.ZKProof)
		fmt.Printf("  Input Size: %d bytes\n", result.InputSize)
	} else {
		fmt.Printf("✗ Contract execution failed\n")
		fmt.Printf("  Error: %s\n", result.Error)
		fmt.Printf("  Gas Used: %d / %d\n", result.Gas, result.GasLimit)
		fmt.Printf("  Execution Time: %d ms\n", result.TimeMs)
		fmt.Printf("  Memory Used: %d bytes\n", result.Memory)
	}
	
	if verbose {
		fmt.Printf("\nDetailed Result:\n")
		fmt.Printf("  Test: %s\n", result.Test)
		fmt.Printf("  Contract: %s\n", result.Contract)
	}
}

// Run tests if requested
func RunTests() {
	fmt.Println("Running NGFS contract tests...")
	
	// Test 1: Basic contract execution
	testBasicExecution()
	
	// Test 2: Gas limit enforcement
	testGasLimitEnforcement()
	
	// Test 3: ZK proof generation
	testZKProofGeneration()
	
	// Test 4: Error handling
	testErrorHandling()
	
	fmt.Println("All tests completed successfully!")
}

func testBasicExecution() {
	sandbox := &MockContractSandbox{}
	input := []byte(`{"value": 42}`)
	wasm := []byte("mock_wasm_bytecode")
	
	result, err := sandbox.ExecuteContract(wasm, input, 1000, false)
	if err != nil {
		log.Fatalf("Test failed: %v", err)
	}
	
	if !result.Ok {
		log.Fatalf("Test failed: expected success, got failure")
	}
	
	if result.Gas == 0 {
		log.Fatalf("Test failed: expected non-zero gas usage")
	}
	
	fmt.Println("✓ Basic execution test passed")
}

func testGasLimitEnforcement() {
	sandbox := &MockContractSandbox{}
	input := []byte(`{"value": 999999}`)
	wasm := []byte("very_large_wasm_bytecode_that_exceeds_gas_limit")
	
	result, err := sandbox.ExecuteContract(wasm, input, 10, false)
	if err != nil {
		log.Fatalf("Test failed: %v", err)
	}
	
	if result.Ok {
		log.Fatalf("Test failed: expected failure due to gas limit, got success")
	}
	
	if result.Error == "" {
		log.Fatalf("Test failed: expected error message, got empty")
	}
	
	fmt.Println("✓ Gas limit enforcement test passed")
}

func testZKProofGeneration() {
	sandbox := &MockContractSandbox{}
	input := []byte(`{"value": 42}`)
	wasm := []byte("mock_wasm_bytecode")
	
	result, err := sandbox.ExecuteContract(wasm, input, 1000, true)
	if err != nil {
		log.Fatalf("Test failed: %v", err)
	}
	
	if !result.Ok {
		log.Fatalf("Test failed: expected success, got failure")
	}
	
	if !result.ZKProof {
		log.Fatalf("Test failed: expected ZK proof, got none")
	}
	
	fmt.Println("✓ ZK proof generation test passed")
}

func testErrorHandling() {
	sandbox := &MockContractSandbox{}
	
	// Test with empty input
	result, err := sandbox.ExecuteContract([]byte{}, []byte{}, 1000, false)
	if err != nil {
		log.Fatalf("Test failed: %v", err)
	}
	
	// Should still succeed with empty input
	if !result.Ok {
		log.Fatalf("Test failed: expected success with empty input, got failure")
	}
	
	fmt.Println("✓ Error handling test passed")
}
