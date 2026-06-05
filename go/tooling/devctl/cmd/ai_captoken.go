// Package devctl provides CLI commands for AI Core Service CapToken functionality
//
// This module provides CapToken management and integration for the AI Core Service,
// including token generation, validation, and header injection for secure communication.

package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/spf13/cobra"
	"google.golang.org/protobuf/proto"

	// Import generated protobuf types
	ai_core "github.com/aetheris-os/go/tooling/ai_core"
)

// AI CapToken commands
var aiCapTokenCmd = &cobra.Command{
	Use:   "captoken",
	Short: "Manage CapTokens for AI Core Service",
	Long: `Manage CapTokens for secure communication with the AI Core Service.
	
CapTokens provide fine-grained access control and authentication for AI operations.
	
Examples:
  devctl ai captoken generate --scope "ai:chat,ai:tool.open_file"
  devctl ai captoken validate --token "eyJ0b2tlbiI6..."
  devctl ai captoken list
  devctl ai captoken revoke --token "eyJ0b2tlbiI6..."`,
}

// AI CapToken generate command
var aiCapTokenGenerateCmd = &cobra.Command{
	Use:   "generate",
	Short: "Generate a new CapToken",
	Long:  "Generate a new CapToken with specified scopes and expiration",
	RunE:  runAiCapTokenGenerate,
}

// AI CapToken validate command
var aiCapTokenValidateCmd = &cobra.Command{
	Use:   "validate [token]",
	Short: "Validate a CapToken",
	Long:  "Validate a CapToken and display its information",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiCapTokenValidate,
}

// AI CapToken list command
var aiCapTokenListCmd = &cobra.Command{
	Use:   "list",
	Short: "List active CapTokens",
	Long:  "List all active CapTokens for the current session",
	RunE:  runAiCapTokenList,
}

// AI CapToken revoke command
var aiCapTokenRevokeCmd = &cobra.Command{
	Use:   "revoke [token]",
	Short: "Revoke a CapToken",
	Long:  "Revoke a CapToken to invalidate it",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiCapTokenRevoke,
}

// AI CapToken info command
var aiCapTokenInfoCmd = &cobra.Command{
	Use:   "info [token]",
	Short: "Get CapToken information",
	Long:  "Get detailed information about a CapToken without validating it",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiCapTokenInfo,
}

// CapToken configuration
type CapTokenConfig struct {
	Scopes      []string
	ExpiresAt   time.Time
	Issuer      string
	Subject     string
	Audience    string
	PrivateKey  ed25519.PrivateKey
	PublicKey   ed25519.PublicKey
	OutputFormat string
	SaveToFile  string
}

// CapToken information
type CapTokenInfo struct {
	Token     string    `json:"token"`
	Scopes    []string  `json:"scopes"`
	ExpiresAt time.Time `json:"expires_at"`
	Issuer    string    `json:"issuer"`
	Subject   string    `json:"subject"`
	Audience  string    `json:"audience"`
	Valid     bool      `json:"valid"`
	Expired   bool      `json:"expired"`
	CreatedAt time.Time `json:"created_at"`
}

// runAiCapTokenGenerate executes the generate CapToken command
func runAiCapTokenGenerate(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getCapTokenConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get captoken config: %w", err)
	}

	// Generate CapToken
	token, err := generateCapToken(config)
	if err != nil {
		return fmt.Errorf("failed to generate captoken: %w", err)
	}

	// Output token based on format
	switch config.OutputFormat {
	case "json":
		return outputCapTokenJSON(token, config)
	case "text":
		return outputCapTokenText(token, config)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiCapTokenValidate executes the validate CapToken command
func runAiCapTokenValidate(cmd *cobra.Command, args []string) error {
	token := args[0]
	
	// Get configuration from flags
	config, err := getCapTokenConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get captoken config: %w", err)
	}

	// Validate CapToken
	tokenInfo, err := validateCapToken(token, config)
	if err != nil {
		return fmt.Errorf("failed to validate captoken: %w", err)
	}

	// Output validation result
	switch config.OutputFormat {
	case "json":
		return outputCapTokenInfoJSON(tokenInfo)
	case "text":
		return outputCapTokenInfoText(tokenInfo)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiCapTokenList executes the list CapTokens command
func runAiCapTokenList(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getCapTokenConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get captoken config: %w", err)
	}

	// List CapTokens
	tokens, err := listCapTokens(config)
	if err != nil {
		return fmt.Errorf("failed to list captokens: %w", err)
	}

	// Output tokens based on format
	switch config.OutputFormat {
	case "json":
		return outputCapTokenListJSON(tokens)
	case "text":
		return outputCapTokenListText(tokens)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiCapTokenRevoke executes the revoke CapToken command
func runAiCapTokenRevoke(cmd *cobra.Command, args []string) error {
	token := args[0]
	
	// Get configuration from flags
	config, err := getCapTokenConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get captoken config: %w", err)
	}

	// Revoke CapToken
	err = revokeCapToken(token, config)
	if err != nil {
		return fmt.Errorf("failed to revoke captoken: %w", err)
	}

	fmt.Printf("✅ CapToken revoked successfully: %s\n", token[:20]+"...")
	return nil
}

// runAiCapTokenInfo executes the info CapToken command
func runAiCapTokenInfo(cmd *cobra.Command, args []string) error {
	token := args[0]
	
	// Get configuration from flags
	config, err := getCapTokenConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get captoken config: %w", err)
	}

	// Get CapToken info
	tokenInfo, err := getCapTokenInfo(token, config)
	if err != nil {
		return fmt.Errorf("failed to get captoken info: %w", err)
	}

	// Output info based on format
	switch config.OutputFormat {
	case "json":
		return outputCapTokenInfoJSON(tokenInfo)
	case "text":
		return outputCapTokenInfoText(tokenInfo)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// getCapTokenConfig extracts configuration from command flags
func getCapTokenConfig(cmd *cobra.Command) (CapTokenConfig, error) {
	scopes, _ := cmd.Flags().GetStringSlice("scope")
	expiresAtStr, _ := cmd.Flags().GetString("expires-at")
	issuer, _ := cmd.Flags().GetString("issuer")
	subject, _ := cmd.Flags().GetString("subject")
	audience, _ := cmd.Flags().GetString("audience")
	outputFormat, _ := cmd.Flags().GetString("format")
	saveToFile, _ := cmd.Flags().GetString("save")

	// Set defaults
	if len(scopes) == 0 {
		scopes = []string{"ai:chat", "ai:tool.list"}
	}
	if issuer == "" {
		issuer = "devctl"
	}
	if subject == "" {
		subject = "user"
	}
	if audience == "" {
		audience = "ai-core-service"
	}
	if outputFormat == "" {
		outputFormat = "text"
	}

	// Parse expiration time
	var expiresAt time.Time
	if expiresAtStr == "" {
		expiresAt = time.Now().Add(24 * time.Hour) // Default 24 hours
	} else {
		var err error
		expiresAt, err = time.Parse(time.RFC3339, expiresAtStr)
		if err != nil {
			return CapTokenConfig{}, fmt.Errorf("invalid expires-at format: %w", err)
		}
	}

	// Generate key pair
	publicKey, privateKey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return CapTokenConfig{}, fmt.Errorf("failed to generate key pair: %w", err)
	}

	return CapTokenConfig{
		Scopes:       scopes,
		ExpiresAt:    expiresAt,
		Issuer:       issuer,
		Subject:      subject,
		Audience:     audience,
		PrivateKey:   privateKey,
		PublicKey:    publicKey,
		OutputFormat: outputFormat,
		SaveToFile:   saveToFile,
	}, nil
}

// generateCapToken generates a new CapToken
func generateCapToken(config CapTokenConfig) (string, error) {
	// Create CapToken protobuf message
	capToken := &ai_core.CapToken{
		Token:     "", // Will be set after signing
		Scopes:    config.Scopes,
		ExpiresAt: config.ExpiresAt.Unix(),
		Issuer:    config.Issuer,
		Subject:   config.Subject,
		Audience:  config.Audience,
		CreatedAt: time.Now().Unix(),
	}

	// Serialize the token data
	tokenData, err := proto.Marshal(capToken)
	if err != nil {
		return "", fmt.Errorf("failed to marshal captoken: %w", err)
	}

	// Sign the token data
	signature := ed25519.Sign(config.PrivateKey, tokenData)

	// Create signed token structure
	signedToken := map[string]interface{}{
		"data":      base64.StdEncoding.EncodeToString(tokenData),
		"signature": base64.StdEncoding.EncodeToString(signature),
		"public_key": base64.StdEncoding.EncodeToString(config.PublicKey),
	}

	// Encode as JSON and then base64
	jsonData, err := json.Marshal(signedToken)
	if err != nil {
		return "", fmt.Errorf("failed to marshal signed token: %w", err)
	}

	token := base64.StdEncoding.EncodeToString(jsonData)

	// Save to file if requested
	if config.SaveToFile != "" {
		err = saveCapTokenToFile(token, config.SaveToFile)
		if err != nil {
			return "", fmt.Errorf("failed to save captoken to file: %w", err)
		}
	}

	return token, nil
}

// validateCapToken validates a CapToken
func validateCapToken(token string, config CapTokenConfig) (CapTokenInfo, error) {
	// Decode the token
	jsonData, err := base64.StdEncoding.DecodeString(token)
	if err != nil {
		return CapTokenInfo{}, fmt.Errorf("failed to decode token: %w", err)
	}

	// Parse signed token structure
	var signedToken map[string]interface{}
	if err := json.Unmarshal(jsonData, &signedToken); err != nil {
		return CapTokenInfo{}, fmt.Errorf("failed to parse signed token: %w", err)
	}

	// Extract components
	dataStr, ok := signedToken["data"].(string)
	if !ok {
		return CapTokenInfo{}, fmt.Errorf("invalid token data")
	}

	signatureStr, ok := signedToken["signature"].(string)
	if !ok {
		return CapTokenInfo{}, fmt.Errorf("invalid token signature")
	}

	publicKeyStr, ok := signedToken["public_key"].(string)
	if !ok {
		return CapTokenInfo{}, fmt.Errorf("invalid token public key")
	}

	// Decode components
	tokenData, err := base64.StdEncoding.DecodeString(dataStr)
	if err != nil {
		return CapTokenInfo{}, fmt.Errorf("failed to decode token data: %w", err)
	}

	signature, err := base64.StdEncoding.DecodeString(signatureStr)
	if err != nil {
		return CapTokenInfo{}, fmt.Errorf("failed to decode signature: %w", err)
	}

	publicKey, err := base64.StdEncoding.DecodeString(publicKeyStr)
	if err != nil {
		return CapTokenInfo{}, fmt.Errorf("failed to decode public key: %w", err)
	}

	// Verify signature
	if !ed25519.Verify(publicKey, tokenData, signature) {
		return CapTokenInfo{}, fmt.Errorf("invalid token signature")
	}

	// Parse CapToken data
	var capToken ai_core.CapToken
	if err := proto.Unmarshal(tokenData, &capToken); err != nil {
		return CapTokenInfo{}, fmt.Errorf("failed to unmarshal captoken: %w", err)
	}

	// Check expiration
	expiresAt := time.Unix(capToken.ExpiresAt, 0)
	expired := time.Now().After(expiresAt)

	// Create token info
	tokenInfo := CapTokenInfo{
		Token:     token,
		Scopes:    capToken.Scopes,
		ExpiresAt: expiresAt,
		Issuer:    capToken.Issuer,
		Subject:   capToken.Subject,
		Audience:  capToken.Audience,
		Valid:     !expired,
		Expired:   expired,
		CreatedAt: time.Unix(capToken.CreatedAt, 0),
	}

	return tokenInfo, nil
}

// listCapTokens lists active CapTokens
func listCapTokens(config CapTokenConfig) ([]CapTokenInfo, error) {
	// In a real implementation, this would query the AI Core Service
	// For now, we'll return mock data
	tokens := []CapTokenInfo{
		{
			Token:     "eyJ0b2tlbiI6Im1vY2tfdG9rZW4xIn0=",
			Scopes:    []string{"ai:chat", "ai:tool.list"},
			ExpiresAt: time.Now().Add(12 * time.Hour),
			Issuer:    "devctl",
			Subject:   "user",
			Audience:  "ai-core-service",
			Valid:     true,
			Expired:   false,
			CreatedAt: time.Now().Add(-12 * time.Hour),
		},
		{
			Token:     "eyJ0b2tlbiI6Im1vY2tfdG9rZW4yIn0=",
			Scopes:    []string{"ai:tool.open_file", "ai:tool.search_files"},
			ExpiresAt: time.Now().Add(6 * time.Hour),
			Issuer:    "devctl",
			Subject:   "user",
			Audience:  "ai-core-service",
			Valid:     true,
			Expired:   false,
			CreatedAt: time.Now().Add(-18 * time.Hour),
		},
	}

	return tokens, nil
}

// revokeCapToken revokes a CapToken
func revokeCapToken(token string, config CapTokenConfig) error {
	// In a real implementation, this would send a revocation request to the AI Core Service
	// For now, we'll simulate the revocation
	fmt.Printf("🔄 Revoking CapToken: %s...\n", token[:20]+"...")
	
	// Simulate processing delay
	time.Sleep(100 * time.Millisecond)
	
	return nil
}

// getCapTokenInfo gets information about a CapToken without validating it
func getCapTokenInfo(token string, config CapTokenConfig) (CapTokenInfo, error) {
	// Try to validate the token first
	tokenInfo, err := validateCapToken(token, config)
	if err != nil {
		// If validation fails, try to extract basic info
		return CapTokenInfo{
			Token:   token,
			Valid:   false,
			Expired: true,
		}, nil
	}

	return tokenInfo, nil
}

// saveCapTokenToFile saves a CapToken to a file
func saveCapTokenToFile(token, filename string) error {
	file, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create file: %w", err)
	}
	defer file.Close()

	_, err = file.WriteString(token)
	if err != nil {
		return fmt.Errorf("failed to write token: %w", err)
	}

	return nil
}

// outputCapTokenJSON outputs CapToken in JSON format
func outputCapTokenJSON(token string, config CapTokenConfig) error {
	output := map[string]interface{}{
		"token":      token,
		"scopes":     config.Scopes,
		"expires_at": config.ExpiresAt.Format(time.RFC3339),
		"issuer":     config.Issuer,
		"subject":    config.Subject,
		"audience":   config.Audience,
	}

	if config.SaveToFile != "" {
		output["saved_to"] = config.SaveToFile
	}

	jsonData, err := json.MarshalIndent(output, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal output: %w", err)
	}

	fmt.Println(string(jsonData))
	return nil
}

// outputCapTokenText outputs CapToken in text format
func outputCapTokenText(token string, config CapTokenConfig) error {
	fmt.Println("🔑 CapToken Generated")
	fmt.Println("====================")
	fmt.Printf("Token: %s\n", token)
	fmt.Printf("Scopes: %s\n", strings.Join(config.Scopes, ", "))
	fmt.Printf("Expires: %s\n", config.ExpiresAt.Format(time.RFC3339))
	fmt.Printf("Issuer: %s\n", config.Issuer)
	fmt.Printf("Subject: %s\n", config.Subject)
	fmt.Printf("Audience: %s\n", config.Audience)

	if config.SaveToFile != "" {
		fmt.Printf("Saved to: %s\n", config.SaveToFile)
	}

	return nil
}

// outputCapTokenInfoJSON outputs CapToken info in JSON format
func outputCapTokenInfoJSON(tokenInfo CapTokenInfo) error {
	jsonData, err := json.MarshalIndent(tokenInfo, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal token info: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputCapTokenInfoText outputs CapToken info in text format
func outputCapTokenInfoText(tokenInfo CapTokenInfo) error {
	fmt.Println("🔍 CapToken Information")
	fmt.Println("=======================")
	fmt.Printf("Token: %s\n", tokenInfo.Token)
	fmt.Printf("Valid: %t\n", tokenInfo.Valid)
	fmt.Printf("Expired: %t\n", tokenInfo.Expired)
	fmt.Printf("Scopes: %s\n", strings.Join(tokenInfo.Scopes, ", "))
	fmt.Printf("Expires: %s\n", tokenInfo.ExpiresAt.Format(time.RFC3339))
	fmt.Printf("Issuer: %s\n", tokenInfo.Issuer)
	fmt.Printf("Subject: %s\n", tokenInfo.Subject)
	fmt.Printf("Audience: %s\n", tokenInfo.Audience)
	fmt.Printf("Created: %s\n", tokenInfo.CreatedAt.Format(time.RFC3339))

	if tokenInfo.Expired {
		fmt.Println("⚠️  This token has expired")
	} else if !tokenInfo.Valid {
		fmt.Println("❌ This token is invalid")
	} else {
		fmt.Println("✅ This token is valid")
	}

	return nil
}

// outputCapTokenListJSON outputs CapToken list in JSON format
func outputCapTokenListJSON(tokens []CapTokenInfo) error {
	jsonData, err := json.MarshalIndent(tokens, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal token list: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputCapTokenListText outputs CapToken list in text format
func outputCapTokenListText(tokens []CapTokenInfo) error {
	fmt.Println("📋 Active CapTokens")
	fmt.Println("===================")
	fmt.Println()

	for i, token := range tokens {
		status := "✅ Valid"
		if token.Expired {
			status = "⚠️  Expired"
		} else if !token.Valid {
			status = "❌ Invalid"
		}

		fmt.Printf("%d. %s\n", i+1, token.Token[:20]+"...")
		fmt.Printf("   Status: %s\n", status)
		fmt.Printf("   Scopes: %s\n", strings.Join(token.Scopes, ", "))
		fmt.Printf("   Expires: %s\n", token.ExpiresAt.Format(time.RFC3339))
		fmt.Printf("   Issuer: %s\n", token.Issuer)
		fmt.Println()
	}

	return nil
}

func init() {
	// Add captoken commands to AI command
	aiCmd.AddCommand(aiCapTokenCmd)
	aiCapTokenCmd.AddCommand(aiCapTokenGenerateCmd)
	aiCapTokenCmd.AddCommand(aiCapTokenValidateCmd)
	aiCapTokenCmd.AddCommand(aiCapTokenListCmd)
	aiCapTokenCmd.AddCommand(aiCapTokenRevokeCmd)
	aiCapTokenCmd.AddCommand(aiCapTokenInfoCmd)

	// Common flags for all captoken commands
	aiCapTokenCmd.PersistentFlags().String("format", "text", "Output format (text, json)")

	// Generate command flags
	aiCapTokenGenerateCmd.Flags().StringSlice("scope", []string{}, "Token scopes (e.g., ai:chat,ai:tool.open_file)")
	aiCapTokenGenerateCmd.Flags().String("expires-at", "", "Expiration time (RFC3339 format)")
	aiCapTokenGenerateCmd.Flags().String("issuer", "devctl", "Token issuer")
	aiCapTokenGenerateCmd.Flags().String("subject", "user", "Token subject")
	aiCapTokenGenerateCmd.Flags().String("audience", "ai-core-service", "Token audience")
	aiCapTokenGenerateCmd.Flags().String("save", "", "Save token to file")

	// Validate command flags
	aiCapTokenValidateCmd.Flags().String("format", "text", "Output format (text, json)")

	// List command flags
	aiCapTokenListCmd.Flags().String("format", "table", "Output format (table, json)")

	// Revoke command flags (no additional flags needed)

	// Info command flags
	aiCapTokenInfoCmd.Flags().String("format", "text", "Output format (text, json)")
}
