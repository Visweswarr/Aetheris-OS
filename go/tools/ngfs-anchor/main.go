package main

import (
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"math/big"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/ethereum/go-ethereum/rpc"
)

// AnchorRequest represents a request to anchor a snapshot
type AnchorRequest struct {
	SnapshotCID string `json:"snapshot_cid"`
	TargetChain string `json:"target_chain"`
	Priority    string `json:"priority"`
	Description string `json:"description,omitempty"`
	Tags        string `json:"tags,omitempty"`
	GasLimit    uint64 `json:"gas_limit"`
	MaxFeePerGas string `json:"max_fee_per_gas,omitempty"`
	PrivateKey  string `json:"private_key"`
	RPCURL      string `json:"rpc_url"`
	ContractAddress string `json:"contract_address"`
}

// AnchorResult represents the result of an anchor submission
type AnchorResult struct {
	Success        bool   `json:"success"`
	AnchorHash     string `json:"anchor_hash,omitempty"`
	TransactionHash string `json:"transaction_hash,omitempty"`
	BlockNumber    uint64 `json:"block_number,omitempty"`
	GasUsed        uint64 `json:"gas_used,omitempty"`
	Error          string `json:"error,omitempty"`
	ChainID        uint64 `json:"chain_id,omitempty"`
	Cost           string `json:"cost,omitempty"`
}

// ChainConfig holds blockchain configuration
type ChainConfig struct {
	ChainID         uint64 `json:"chain_id"`
	Name            string `json:"name"`
	RPCURL          string `json:"rpc_url"`
	ContractAddress string `json:"contract_address"`
	GasLimit        uint64 `json:"gas_limit"`
	MaxFeePerGas    string `json:"max_fee_per_gas"`
	PriorityFee     string `json:"priority_fee"`
	Confirmations   uint64 `json:"confirmations"`
	Timeout         uint64 `json:"timeout"`
	Enabled         bool   `json:"enabled"`
}

// AnchorService handles blockchain interactions
type AnchorService struct {
	client          *ethclient.Client
	rpcClient       *rpc.Client
	config          ChainConfig
	privateKey      *ecdsa.PrivateKey
	contractAddress common.Address
}

// NewAnchorService creates a new anchor service
func NewAnchorService(config ChainConfig, privateKeyHex string) (*AnchorService, error) {
	// Connect to blockchain
	client, err := ethclient.Dial(config.RPCURL)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to blockchain: %w", err)
	}

	rpcClient, err := rpc.Dial(config.RPCURL)
	if err != nil {
		return nil, fmt.Errorf("failed to create RPC client: %w", err)
	}

	// Parse private key
	privateKey, err := crypto.HexToECDSA(strings.TrimPrefix(privateKeyHex, "0x"))
	if err != nil {
		return nil, fmt.Errorf("failed to parse private key: %w", err)
	}

	// Parse contract address
	contractAddress := common.HexToAddress(config.ContractAddress)

	return &AnchorService{
		client:          client,
		rpcClient:       rpcClient,
		config:          config,
		privateKey:      privateKey,
		contractAddress: contractAddress,
	}, nil
}

// SubmitAnchor submits an anchor to the blockchain
func (s *AnchorService) SubmitAnchor(ctx context.Context, request AnchorRequest) (*AnchorResult, error) {
	// Get account from private key
	publicKey := s.privateKey.Public()
	publicKeyECDSA, ok := publicKey.(*ecdsa.PublicKey)
	if !ok {
		return nil, fmt.Errorf("failed to get public key")
	}

	fromAddress := crypto.PubkeyToAddress(*publicKeyECDSA)

	// Get nonce
	nonce, err := s.client.PendingNonceAt(ctx, fromAddress)
	if err != nil {
		return nil, fmt.Errorf("failed to get nonce: %w", err)
	}

	// Get gas price
	gasPrice, err := s.client.SuggestGasPrice(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get gas price: %w", err)
	}

	// Parse max fee per gas if provided
	if request.MaxFeePerGas != "" {
		maxFeePerGas, ok := new(big.Int).SetString(request.MaxFeePerGas, 10)
		if !ok {
			return nil, fmt.Errorf("invalid max_fee_per_gas: %s", request.MaxFeePerGas)
		}
		gasPrice = maxFeePerGas
	}

	// Create anchor data
	anchorData, err := s.createAnchorData(request)
	if err != nil {
		return nil, fmt.Errorf("failed to create anchor data: %w", err)
	}

	// Create transaction
	tx := types.NewTransaction(
		nonce,
		s.contractAddress,
		big.NewInt(0), // No ETH transfer
		request.GasLimit,
		gasPrice,
		anchorData,
	)

	// Get chain ID
	chainID, err := s.client.NetworkID(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get chain ID: %w", err)
	}

	// Sign transaction
	signedTx, err := types.SignTx(tx, types.NewEIP155Signer(chainID), s.privateKey)
	if err != nil {
		return nil, fmt.Errorf("failed to sign transaction: %w", err)
	}

	// Send transaction
	err = s.client.SendTransaction(ctx, signedTx)
	if err != nil {
		return nil, fmt.Errorf("failed to send transaction: %w", err)
	}

	// Wait for confirmation
	receipt, err := s.waitForConfirmation(ctx, signedTx.Hash())
	if err != nil {
		return nil, fmt.Errorf("failed to get transaction receipt: %w", err)
	}

	// Check transaction status
	if receipt.Status == 0 {
		return &AnchorResult{
			Success: false,
			Error:   "transaction failed on chain",
		}, nil
	}

	// Calculate cost
	cost := new(big.Int).Mul(new(big.Int).SetUint64(receipt.GasUsed), gasPrice)
	costEth := new(big.Float).Quo(new(big.Float).SetInt(cost), new(big.Float).SetFloat64(1e18))

	return &AnchorResult{
		Success:        true,
		TransactionHash: signedTx.Hash().Hex(),
		BlockNumber:    receipt.BlockNumber.Uint64(),
		GasUsed:        receipt.GasUsed,
		ChainID:        chainID.Uint64(),
		Cost:           costEth.Text('f', 18) + " ETH",
	}, nil
}

// createAnchorData creates the transaction data for the anchor contract
func (s *AnchorService) createAnchorData(request AnchorRequest) ([]byte, error) {
	// This is a simplified version - in production, you would:
	// 1. Use the actual contract ABI
	// 2. Encode the function call with proper parameters
	// 3. Return the encoded data

	// For now, create a simple data structure
	data := map[string]interface{}{
		"function": "anchorSnapshot",
		"snapshot_cid": request.SnapshotCID,
		"priority": request.Priority,
		"description": request.Description,
		"tags": request.Tags,
		"timestamp": time.Now().Unix(),
	}

	// Convert to JSON and then to bytes
	jsonData, err := json.Marshal(data)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal anchor data: %w", err)
	}

	return jsonData, nil
}

// waitForConfirmation waits for transaction confirmation
func (s *AnchorService) waitForConfirmation(ctx context.Context, txHash common.Hash) (*types.Receipt, error) {
	ticker := time.NewTicker(1 * time.Second)
	defer ticker.Stop()

	timeout := time.After(time.Duration(s.config.Timeout) * time.Second)

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-timeout:
			return nil, fmt.Errorf("timeout waiting for confirmation")
		case <-ticker.C:
			receipt, err := s.client.TransactionReceipt(ctx, txHash)
			if err == nil && receipt != nil {
				if receipt.Confirmations >= s.config.Confirmations {
					return receipt, nil
				}
			}
		}
	}
}

// loadConfig loads chain configuration from file
func loadConfig(configPath string) (*ChainConfig, error) {
	data, err := ioutil.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	var config ChainConfig
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("failed to parse config file: %w", err)
	}

	return &config, nil
}

// loadRequest loads anchor request from file
func loadRequest(requestPath string) (*AnchorRequest, error) {
	data, err := ioutil.ReadFile(requestPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read request file: %w", err)
	}

	var request AnchorRequest
	if err := json.Unmarshal(data, &request); err != nil {
		return nil, fmt.Errorf("failed to parse request file: %w", err)
	}

	return &request, nil
}

// saveResult saves anchor result to file
func saveResult(result *AnchorResult, outputPath string) error {
	data, err := json.MarshalIndent(result, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal result: %w", err)
	}

	if err := ioutil.WriteFile(outputPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write result file: %w", err)
	}

	return nil
}

// printResult prints anchor result to console
func printResult(result *AnchorResult, verbose bool) {
	if result.Success {
		fmt.Printf("✓ Anchor submitted successfully\n")
		fmt.Printf("  Transaction: %s\n", result.TransactionHash)
		fmt.Printf("  Block: %d\n", result.BlockNumber)
		fmt.Printf("  Gas Used: %d\n", result.GasUsed)
		fmt.Printf("  Cost: %s\n", result.Cost)
		if verbose {
			fmt.Printf("  Chain ID: %d\n", result.ChainID)
		}
	} else {
		fmt.Printf("✗ Anchor submission failed\n")
		fmt.Printf("  Error: %s\n", result.Error)
	}
}

// createDefaultConfig creates a default chain configuration
func createDefaultConfig(outputPath string) error {
	config := ChainConfig{
		ChainID:         1337,
		Name:            "local",
		RPCURL:          "http://localhost:8545",
		ContractAddress: "0x0000000000000000000000000000000000000000",
		GasLimit:        100000,
		MaxFeePerGas:    "20000000000", // 20 gwei
		PriorityFee:     "1000000000",  // 1 gwei
		Confirmations:   1,
		Timeout:         300, // 5 minutes
		Enabled:         true,
	}

	data, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	if err := ioutil.WriteFile(outputPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	fmt.Printf("Default config created: %s\n", outputPath)
	return nil
}

// createSampleRequest creates a sample anchor request
func createSampleRequest(outputPath string) error {
	request := AnchorRequest{
		SnapshotCID:    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
		TargetChain:    "local",
		Priority:       "normal",
		Description:    "Sample NGFS snapshot anchor",
		Tags:           "ngfs,snapshot,anchor",
		GasLimit:       100000,
		MaxFeePerGas:   "20000000000",
		PrivateKey:     "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
		RPCURL:         "http://localhost:8545",
		ContractAddress: "0x0000000000000000000000000000000000000000",
	}

	data, err := json.MarshalIndent(request, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal request: %w", err)
	}

	if err := ioutil.WriteFile(outputPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write request file: %w", err)
	}

	fmt.Printf("Sample request created: %s\n", outputPath)
	fmt.Printf("⚠️  Remember to update the private key and contract address!\n")
	return nil
}

// validateRequest validates anchor request parameters
func validateRequest(request *AnchorRequest) error {
	if request.SnapshotCID == "" {
		return fmt.Errorf("snapshot_cid is required")
	}

	if request.TargetChain == "" {
		return fmt.Errorf("target_chain is required")
	}

	if request.PrivateKey == "" {
		return fmt.Errorf("private_key is required")
	}

	if request.RPCURL == "" {
		return fmt.Errorf("rpc_url is required")
	}

	if request.ContractAddress == "" {
		return fmt.Errorf("contract_address is required")
	}

	if request.GasLimit == 0 {
		return fmt.Errorf("gas_limit must be greater than 0")
	}

	return nil
}

func main() {
	var (
		configPath   = flag.String("config", "", "Path to chain configuration file")
		requestPath  = flag.String("request", "", "Path to anchor request file")
		outputPath   = flag.String("output", "", "Path to save result (optional)")
		verbose      = flag.Bool("verbose", false, "Enable verbose output")
		createConfig = flag.String("create-config", "", "Create default config file at specified path")
		createSample = flag.String("create-sample", "", "Create sample request file at specified path")
		help         = flag.Bool("help", false, "Show help")
	)

	flag.Parse()

	if *help {
		fmt.Printf("NGFS Anchor Tool - Submit NGFS snapshot anchors to blockchain\n\n")
		fmt.Printf("Usage:\n")
		fmt.Printf("  %s [flags]\n\n", os.Args[0])
		fmt.Printf("Flags:\n")
		flag.PrintDefaults()
		fmt.Printf("\nExamples:\n")
		fmt.Printf("  # Create default config\n")
		fmt.Printf("  %s -create-config config.json\n\n", os.Args[0])
		fmt.Printf("  # Create sample request\n")
		fmt.Printf("  %s -create-sample request.json\n\n", os.Args[0])
		fmt.Printf("  # Submit anchor\n")
		fmt.Printf("  %s -config config.json -request request.json\n\n", os.Args[0])
		return
	}

	// Handle config creation
	if *createConfig != "" {
		if err := createDefaultConfig(*createConfig); err != nil {
			log.Fatalf("Failed to create config: %v", err)
		}
		return
	}

	// Handle sample creation
	if *createSample != "" {
		if err := createSampleRequest(*createSample); err != nil {
			log.Fatalf("Failed to create sample: %v", err)
		}
		return
	}

	// Validate required parameters
	if *configPath == "" || *requestPath == "" {
		log.Fatal("Both -config and -request are required")
	}

	// Load configuration
	config, err := loadConfig(*configPath)
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Load request
	request, err := loadRequest(*requestPath)
	if err != nil {
		log.Fatalf("Failed to load request: %v", err)
	}

	// Validate request
	if err := validateRequest(request); err != nil {
		log.Fatalf("Invalid request: %v", err)
	}

	// Override config with request values if provided
	if request.RPCURL != "" {
		config.RPCURL = request.RPCURL
	}
	if request.ContractAddress != "" {
		config.ContractAddress = request.ContractAddress
	}

	// Create anchor service
	service, err := NewAnchorService(*config, request.PrivateKey)
	if err != nil {
		log.Fatalf("Failed to create anchor service: %v", err)
	}

	// Submit anchor
	ctx := context.Background()
	result, err := service.SubmitAnchor(ctx, *request)
	if err != nil {
		log.Fatalf("Failed to submit anchor: %v", err)
	}

	// Print result
	printResult(result, *verbose)

	// Save result if output path specified
	if *outputPath != "" {
		if err := saveResult(result, *outputPath); err != nil {
			log.Printf("Warning: Failed to save result: %v", err)
		} else {
			fmt.Printf("Result saved to: %s\n", *outputPath)
		}
	}

	// Exit with error code if failed
	if !result.Success {
		os.Exit(1)
	}
}
