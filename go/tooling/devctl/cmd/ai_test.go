// Package devctl provides tests for AI Core Service CLI commands
//
// This module provides comprehensive tests for the AI Core Service CLI commands,
// including chat, toolcall, metrics, and captoken functionality.

package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	// Import generated protobuf types
	ai_core "github.com/aetheris-os/go/tooling/ai_core"
)

// TestAI Chat Commands
func TestAiChatCommand(t *testing.T) {
	tests := []struct {
		name     string
		args     []string
		flags    map[string]string
		expected string
		wantErr  bool
	}{
		{
			name:     "basic chat command",
			args:     []string{"Hello, AI!"},
			flags:    map[string]string{"format": "text"},
			expected: "Hello! I'm the AI Core Service",
			wantErr:  false,
		},
		{
			name:     "chat with JSON output",
			args:     []string{"What is the capital of France?"},
			flags:    map[string]string{"format": "json"},
			expected: "Paris",
			wantErr:  false,
		},
		{
			name:     "chat with custom model",
			args:     []string{"Test message"},
			flags:    map[string]string{"model": "gpt-4", "temperature": "0.5"},
			expected: "I understand you're asking about",
			wantErr:  false,
		},
		{
			name:     "chat with streaming",
			args:     []string{"Streaming test"},
			flags:    map[string]string{"stream": "true"},
			expected: "I understand you're asking about",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create a new command instance
			cmd := &cobra.Command{}
			cmd.Flags().String("format", "text", "")
			cmd.Flags().String("model", "gpt-3.5-turbo", "")
			cmd.Flags().Float64("temperature", 0.7, "")
			cmd.Flags().Bool("stream", false, "")

			// Set flags
			for flag, value := range tt.flags {
				cmd.Flags().Set(flag, value)
			}

			// Capture output
			var buf bytes.Buffer
			cmd.SetOutput(&buf)

			// Run the command
			err := runAiChat(cmd, tt.args)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				output := buf.String()
				assert.Contains(t, output, tt.expected)
			}
		})
	}
}

func TestAiChatInteractive(t *testing.T) {
	// Test interactive chat command setup
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "text", "")
	cmd.Flags().String("model", "gpt-3.5-turbo", "")

	// Test that the command can be created without error
	assert.NotNil(t, aiChatInteractiveCmd)
	assert.Equal(t, "interactive", aiChatInteractiveCmd.Use)
}

func TestAiChatFile(t *testing.T) {
	// Create a temporary file
	tmpFile, err := os.CreateTemp("", "test_chat_*.txt")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	// Write test content
	testContent := "This is a test message from file"
	_, err = tmpFile.WriteString(testContent)
	require.NoError(t, err)
	tmpFile.Close()

	// Test file command
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "text", "")
	cmd.Flags().String("model", "gpt-3.5-turbo", "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err = runAiChatFile(cmd, []string{tmpFile.Name()})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "I understand you're asking about")
}

// Test AI Toolcall Commands
func TestAiToolcallList(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "table", "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err := runAiToolcallList(cmd, []string{})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "Available Tools")
	assert.Contains(t, output, "open_file")
	assert.Contains(t, output, "search_files")
	assert.Contains(t, output, "create_note")
}

func TestAiToolcallDescribe(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "text", "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err := runAiToolcallDescribe(cmd, []string{"open_file"})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "Tool: open_file")
	assert.Contains(t, output, "Parameters:")
	assert.Contains(t, output, "path")
}

func TestAiToolcallExecute(t *testing.T) {
	tests := []struct {
		name     string
		toolName string
		flags    map[string]string
		expected string
		wantErr  bool
	}{
		{
			name:     "execute open_file tool",
			toolName: "open_file",
			flags:    map[string]string{"param-path": "/tmp/test.txt", "param-mode": "read"},
			expected: "File opened successfully",
			wantErr:  false,
		},
		{
			name:     "execute search_files tool",
			toolName: "search_files",
			flags:    map[string]string{"param-query": "*.go", "param-directory": "./src"},
			expected: "Search completed",
			wantErr:  false,
		},
		{
			name:     "execute create_note tool",
			toolName: "create_note",
			flags:    map[string]string{"param-title": "Test Note", "param-content": "Test content"},
			expected: "Note created successfully",
			wantErr:  false,
		},
		{
			name:     "execute get_system_time tool",
			toolName: "get_system_time",
			flags:    map[string]string{"param-format": "iso"},
			expected: "Current system time",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cmd := &cobra.Command{}
			cmd.Flags().String("format", "text", "")
			cmd.Flags().String("cap-token", "", "")
			cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
			cmd.Flags().Duration("timeout", 30*time.Second, "")

			// Add parameter flags
			for flag, value := range tt.flags {
				cmd.Flags().String(flag, value, "")
			}

			var buf bytes.Buffer
			cmd.SetOutput(&buf)

			err := runAiToolcallExecute(cmd, []string{tt.toolName})
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				output := buf.String()
				assert.Contains(t, output, tt.expected)
			}
		})
	}
}

// Test AI Metrics Commands
func TestAiMetricsRealTime(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().Duration("update-interval", 1*time.Second, "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	// Test that the command can be created without error
	assert.NotNil(t, aiMetricsRealTimeCmd)
	assert.Equal(t, "realtime", aiMetricsRealTimeCmd.Use)
}

func TestAiMetricsHistory(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "table", "")
	cmd.Flags().String("time-range", "1h", "")
	cmd.Flags().String("type", "all", "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err := runAiMetricsHistory(cmd, []string{})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "Metrics History")
}

func TestAiMetricsCompare(t *testing.T) {
	// Create a temporary baseline file
	tmpFile, err := os.CreateTemp("", "test_baseline_*.json")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	// Write baseline metrics
	baselineMetrics := AiCoreMetrics{
		RequestsTotal:   1000,
		RequestsSuccess: 950,
		RequestsError:   50,
		LatencyP95Ms:    100.0,
		TokensPerSecond: 15.0,
		MemoryUsageMB:   2000.0,
		CPUUsagePercent: 50.0,
		Timestamp:       uint64(time.Now().Unix()),
	}

	jsonData, err := json.Marshal(baselineMetrics)
	require.NoError(t, err)
	_, err = tmpFile.Write(jsonData)
	require.NoError(t, err)
	tmpFile.Close()

	// Test comparison
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "table", "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err = runAiMetricsCompare(cmd, []string{tmpFile.Name()})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "Metrics Comparison")
}

func TestAiMetricsAlert(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().Float64("alert-latency-p95", 200.0, "")
	cmd.Flags().Float64("alert-tokens-per-second", 10.0, "")
	cmd.Flags().Float64("alert-memory-usage", 4000.0, "")
	cmd.Flags().Float64("alert-cpu-usage", 80.0, "")
	cmd.Flags().Float64("alert-error-rate", 10.0, "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err := runAiMetricsAlert(cmd, []string{})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "Metrics Alerts Configuration")
}

// Test AI CapToken Commands
func TestAiCapTokenGenerate(t *testing.T) {
	tests := []struct {
		name     string
		flags    map[string]string
		expected string
		wantErr  bool
	}{
		{
			name: "generate basic token",
			flags: map[string]string{
				"scope":  "ai:chat,ai:tool.list",
				"format": "text",
			},
			expected: "CapToken Generated",
			wantErr:  false,
		},
		{
			name: "generate token with JSON output",
			flags: map[string]string{
				"scope":  "ai:chat",
				"format": "json",
			},
			expected: "token",
			wantErr:  false,
		},
		{
			name: "generate token with custom expiration",
			flags: map[string]string{
				"scope":      "ai:chat",
				"expires-at": time.Now().Add(2 * time.Hour).Format(time.RFC3339),
				"format":     "text",
			},
			expected: "CapToken Generated",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cmd := &cobra.Command{}
			cmd.Flags().StringSlice("scope", []string{}, "")
			cmd.Flags().String("expires-at", "", "")
			cmd.Flags().String("issuer", "devctl", "")
			cmd.Flags().String("subject", "user", "")
			cmd.Flags().String("audience", "ai-core-service", "")
			cmd.Flags().String("format", "text", "")
			cmd.Flags().String("save", "", "")

			// Set flags
			for flag, value := range tt.flags {
				cmd.Flags().Set(flag, value)
			}

			var buf bytes.Buffer
			cmd.SetOutput(&buf)

			err := runAiCapTokenGenerate(cmd, []string{})
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				output := buf.String()
				assert.Contains(t, output, tt.expected)
			}
		})
	}
}

func TestAiCapTokenValidate(t *testing.T) {
	// First generate a token
	generateCmd := &cobra.Command{}
	generateCmd.Flags().StringSlice("scope", []string{}, "")
	generateCmd.Flags().String("expires-at", "", "")
	generateCmd.Flags().String("issuer", "devctl", "")
	generateCmd.Flags().String("subject", "user", "")
	generateCmd.Flags().String("audience", "ai-core-service", "")
	generateCmd.Flags().String("format", "text", "")
	generateCmd.Flags().String("save", "", "")

	generateCmd.Flags().Set("scope", "ai:chat")
	generateCmd.Flags().Set("format", "text")

	var generateBuf bytes.Buffer
	generateCmd.SetOutput(&generateBuf)

	err := runAiCapTokenGenerate(generateCmd, []string{})
	require.NoError(t, err)

	// Extract token from output (this is a simplified approach)
	generateOutput := generateBuf.String()
	lines := strings.Split(generateOutput, "\n")
	var token string
	for _, line := range lines {
		if strings.HasPrefix(line, "Token: ") {
			token = strings.TrimPrefix(line, "Token: ")
			break
		}
	}
	require.NotEmpty(t, token)

	// Now validate the token
	validateCmd := &cobra.Command{}
	validateCmd.Flags().String("format", "text", "")

	var validateBuf bytes.Buffer
	validateCmd.SetOutput(&validateBuf)

	err = runAiCapTokenValidate(validateCmd, []string{token})
	assert.NoError(t, err)
	output := validateBuf.String()
	assert.Contains(t, output, "CapToken Information")
}

func TestAiCapTokenList(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "table", "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	err := runAiCapTokenList(cmd, []string{})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "Active CapTokens")
}

func TestAiCapTokenRevoke(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "text", "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	// Use a mock token for testing
	mockToken := "eyJ0b2tlbiI6Im1vY2tfdG9rZW4xIn0="
	err := runAiCapTokenRevoke(cmd, []string{mockToken})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "CapToken revoked successfully")
}

func TestAiCapTokenInfo(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "text", "")

	var buf bytes.Buffer
	cmd.SetOutput(&buf)

	// Use a mock token for testing
	mockToken := "eyJ0b2tlbiI6Im1vY2tfdG9rZW4xIn0="
	err := runAiCapTokenInfo(cmd, []string{mockToken})
	assert.NoError(t, err)
	output := buf.String()
	assert.Contains(t, output, "CapToken Information")
}

// Test Configuration Functions
func TestGetChatConfig(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("model", "gpt-3.5-turbo", "")
	cmd.Flags().Float64("temperature", 0.7, "")
	cmd.Flags().Uint32("max-tokens", 1000, "")
	cmd.Flags().Float64("top-p", 1.0, "")
	cmd.Flags().StringSlice("stop", []string{}, "")
	cmd.Flags().Bool("stream", true, "")
	cmd.Flags().StringSlice("context", []string{}, "")
	cmd.Flags().String("session-id", "", "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")
	cmd.Flags().String("format", "text", "")
	cmd.Flags().Bool("include-metadata", false, "")

	config, err := getChatConfig(cmd)
	assert.NoError(t, err)
	assert.Equal(t, "gpt-3.5-turbo", config.Model)
	assert.Equal(t, 0.7, config.Temperature)
	assert.Equal(t, uint32(1000), config.MaxTokens)
	assert.Equal(t, 1.0, config.TopP)
	assert.True(t, config.Stream)
	assert.Equal(t, "/tmp/ai_core.sock", config.SocketPath)
	assert.Equal(t, 30*time.Second, config.Timeout)
	assert.Equal(t, "text", config.OutputFormat)
}

func TestGetToolcallConfig(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")
	cmd.Flags().String("format", "text", "")
	cmd.Flags().String("session-id", "", "")
	cmd.Flags().Bool("include-result", true, "")
	cmd.Flags().Bool("async", false, "")

	config, err := getToolcallConfig(cmd)
	assert.NoError(t, err)
	assert.Equal(t, "/tmp/ai_core.sock", config.SocketPath)
	assert.Equal(t, 30*time.Second, config.Timeout)
	assert.Equal(t, "text", config.OutputFormat)
	assert.True(t, config.IncludeResult)
	assert.False(t, config.Async)
}

func TestGetMetricsConfig(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")
	cmd.Flags().String("format", "table", "")
	cmd.Flags().String("time-range", "1h", "")
	cmd.Flags().String("type", "all", "")
	cmd.Flags().Bool("include-history", false, "")

	config, err := getMetricsConfig(cmd)
	assert.NoError(t, err)
	assert.Equal(t, "/tmp/ai_core.sock", config.SocketPath)
	assert.Equal(t, 30*time.Second, config.Timeout)
	assert.Equal(t, "table", config.OutputFormat)
	assert.Equal(t, "1h", config.TimeRange)
	assert.Equal(t, "all", config.MetricsType)
	assert.False(t, config.IncludeHistory)
}

func TestGetCapTokenConfig(t *testing.T) {
	cmd := &cobra.Command{}
	cmd.Flags().StringSlice("scope", []string{}, "")
	cmd.Flags().String("expires-at", "", "")
	cmd.Flags().String("issuer", "devctl", "")
	cmd.Flags().String("subject", "user", "")
	cmd.Flags().String("audience", "ai-core-service", "")
	cmd.Flags().String("format", "text", "")
	cmd.Flags().String("save", "", "")

	cmd.Flags().Set("scope", "ai:chat,ai:tool.list")

	config, err := getCapTokenConfig(cmd)
	assert.NoError(t, err)
	assert.Equal(t, []string{"ai:chat", "ai:tool.list"}, config.Scopes)
	assert.Equal(t, "devctl", config.Issuer)
	assert.Equal(t, "user", config.Subject)
	assert.Equal(t, "ai-core-service", config.Audience)
	assert.Equal(t, "text", config.OutputFormat)
	assert.NotNil(t, config.PrivateKey)
	assert.NotNil(t, config.PublicKey)
}

// Test Utility Functions
func TestGenerateSessionID(t *testing.T) {
	sessionID1 := generateSessionID()
	sessionID2 := generateSessionID()

	assert.NotEmpty(t, sessionID1)
	assert.NotEmpty(t, sessionID2)
	assert.NotEqual(t, sessionID1, sessionID2)
	assert.True(t, strings.HasPrefix(sessionID1, "session_"))
	assert.True(t, strings.HasPrefix(sessionID2, "session_"))
}

func TestGenerateMessageID(t *testing.T) {
	messageID1 := generateMessageID()
	messageID2 := generateMessageID()

	assert.NotEmpty(t, messageID1)
	assert.NotEmpty(t, messageID2)
	assert.NotEqual(t, messageID1, messageID2)
	assert.True(t, strings.HasPrefix(messageID1, "msg_"))
	assert.True(t, strings.HasPrefix(messageID2, "msg_"))
}

func TestGenerateMockResponse(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{
			input:    "Summarize this document",
			expected: "summary",
		},
		{
			input:    "What is the capital of France?",
			expected: "Paris",
		},
		{
			input:    "Hello there",
			expected: "Hello! I'm the AI Core Service",
		},
		{
			input:    "Help me with something",
			expected: "I can help you with various tasks",
		},
		{
			input:    "Random question",
			expected: "I understand you're asking about",
		},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			response := generateMockResponse(tt.input)
			assert.Contains(t, response, tt.expected)
		})
	}
}

// Test Protobuf Integration
func TestProtobufMessageCreation(t *testing.T) {
	// Test ChatRequest creation
	chatRequest := &ai_core.ChatRequest{
		Message:         "Test message",
		Context:         []string{"context1", "context2"},
		Model:           "gpt-3.5-turbo",
		Temperature:     0.7,
		MaxTokens:       1000,
		TopP:            1.0,
		StopSequences:   []string{"\n\n"},
		Stream:          true,
		SessionId:       "test-session",
		IncludeMetadata: true,
	}

	assert.Equal(t, "Test message", chatRequest.Message)
	assert.Equal(t, []string{"context1", "context2"}, chatRequest.Context)
	assert.Equal(t, "gpt-3.5-turbo", chatRequest.Model)
	assert.Equal(t, 0.7, chatRequest.Temperature)
	assert.Equal(t, uint32(1000), chatRequest.MaxTokens)
	assert.Equal(t, 1.0, chatRequest.TopP)
	assert.Equal(t, []string{"\n\n"}, chatRequest.StopSequences)
	assert.True(t, chatRequest.Stream)
	assert.Equal(t, "test-session", chatRequest.SessionId)
	assert.True(t, chatRequest.IncludeMetadata)

	// Test ToolCallRequest creation
	toolCallRequest := &ai_core.ToolCallRequest{
		ToolName:   "open_file",
		Parameters: map[string]*ai_core.ParameterValue{
			"path": {
				Value: &ai_core.ParameterValue_StringValue{
					StringValue: "/tmp/test.txt",
				},
			},
		},
		Async:     false,
		SessionId: "test-session",
	}

	assert.Equal(t, "open_file", toolCallRequest.ToolName)
	assert.Equal(t, "/tmp/test.txt", toolCallRequest.Parameters["path"].GetStringValue())
	assert.False(t, toolCallRequest.Async)
	assert.Equal(t, "test-session", toolCallRequest.SessionId)

	// Test CapToken creation
	capToken := &ai_core.CapToken{
		Token:     "test-token",
		Scopes:    []string{"ai:chat", "ai:tool.list"},
		ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		Issuer:    "devctl",
		Subject:   "user",
		Audience:  "ai-core-service",
		CreatedAt: time.Now().Unix(),
	}

	assert.Equal(t, "test-token", capToken.Token)
	assert.Equal(t, []string{"ai:chat", "ai:tool.list"}, capToken.Scopes)
	assert.Equal(t, "devctl", capToken.Issuer)
	assert.Equal(t, "user", capToken.Subject)
	assert.Equal(t, "ai-core-service", capToken.Audience)
}

// Test Error Handling
func TestErrorHandling(t *testing.T) {
	// Test invalid file path
	cmd := &cobra.Command{}
	cmd.Flags().String("format", "text", "")
	cmd.Flags().String("model", "gpt-3.5-turbo", "")

	err := runAiChatFile(cmd, []string{"/nonexistent/file.txt"})
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to read file")

	// Test invalid baseline file
	cmd = &cobra.Command{}
	cmd.Flags().String("format", "table", "")
	cmd.Flags().String("cap-token", "", "")
	cmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	cmd.Flags().Duration("timeout", 30*time.Second, "")

	err = runAiMetricsCompare(cmd, []string{"/nonexistent/baseline.json"})
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to read baseline file")
}

// Benchmark Tests
func BenchmarkGenerateSessionID(b *testing.B) {
	for i := 0; i < b.N; i++ {
		generateSessionID()
	}
}

func BenchmarkGenerateMessageID(b *testing.B) {
	for i := 0; i < b.N; i++ {
		generateMessageID()
	}
}

func BenchmarkGenerateMockResponse(b *testing.B) {
	message := "What is the capital of France?"
	for i := 0; i < b.N; i++ {
		generateMockResponse(message)
	}
}

// Integration Tests
func TestIntegrationAiChatWithCapToken(t *testing.T) {
	// Generate a CapToken first
	generateCmd := &cobra.Command{}
	generateCmd.Flags().StringSlice("scope", []string{}, "")
	generateCmd.Flags().String("expires-at", "", "")
	generateCmd.Flags().String("issuer", "devctl", "")
	generateCmd.Flags().String("subject", "user", "")
	generateCmd.Flags().String("audience", "ai-core-service", "")
	generateCmd.Flags().String("format", "text", "")
	generateCmd.Flags().String("save", "", "")

	generateCmd.Flags().Set("scope", "ai:chat")

	var generateBuf bytes.Buffer
	generateCmd.SetOutput(&generateBuf)

	err := runAiCapTokenGenerate(generateCmd, []string{})
	require.NoError(t, err)

	// Extract token (simplified)
	generateOutput := generateBuf.String()
	lines := strings.Split(generateOutput, "\n")
	var token string
	for _, line := range lines {
		if strings.HasPrefix(line, "Token: ") {
			token = strings.TrimPrefix(line, "Token: ")
			break
		}
	}
	require.NotEmpty(t, token)

	// Use the token in a chat command
	chatCmd := &cobra.Command{}
	chatCmd.Flags().String("format", "text", "")
	chatCmd.Flags().String("model", "gpt-3.5-turbo", "")
	chatCmd.Flags().Float64("temperature", 0.7, "")
	chatCmd.Flags().Uint32("max-tokens", 1000, "")
	chatCmd.Flags().Float64("top-p", 1.0, "")
	chatCmd.Flags().StringSlice("stop", []string{}, "")
	chatCmd.Flags().Bool("stream", true, "")
	chatCmd.Flags().StringSlice("context", []string{}, "")
	chatCmd.Flags().String("session-id", "", "")
	chatCmd.Flags().String("cap-token", "", "")
	chatCmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	chatCmd.Flags().Duration("timeout", 30*time.Second, "")
	chatCmd.Flags().Bool("include-metadata", false, "")

	chatCmd.Flags().Set("cap-token", token)

	var chatBuf bytes.Buffer
	chatCmd.SetOutput(&chatBuf)

	err = runAiChat(chatCmd, []string{"Hello with CapToken!"})
	assert.NoError(t, err)
	output := chatBuf.String()
	assert.Contains(t, output, "Hello! I'm the AI Core Service")
}

func TestIntegrationAiToolcallWithCapToken(t *testing.T) {
	// Generate a CapToken for tool access
	generateCmd := &cobra.Command{}
	generateCmd.Flags().StringSlice("scope", []string{}, "")
	generateCmd.Flags().String("expires-at", "", "")
	generateCmd.Flags().String("issuer", "devctl", "")
	generateCmd.Flags().String("subject", "user", "")
	generateCmd.Flags().String("audience", "ai-core-service", "")
	generateCmd.Flags().String("format", "text", "")
	generateCmd.Flags().String("save", "", "")

	generateCmd.Flags().Set("scope", "ai:tool.open_file")

	var generateBuf bytes.Buffer
	generateCmd.SetOutput(&generateBuf)

	err := runAiCapTokenGenerate(generateCmd, []string{})
	require.NoError(t, err)

	// Extract token (simplified)
	generateOutput := generateBuf.String()
	lines := strings.Split(generateOutput, "\n")
	var token string
	for _, line := range lines {
		if strings.HasPrefix(line, "Token: ") {
			token = strings.TrimPrefix(line, "Token: ")
			break
		}
	}
	require.NotEmpty(t, token)

	// Use the token in a toolcall command
	toolcallCmd := &cobra.Command{}
	toolcallCmd.Flags().String("format", "text", "")
	toolcallCmd.Flags().String("cap-token", "", "")
	toolcallCmd.Flags().String("socket", "/tmp/ai_core.sock", "")
	toolcallCmd.Flags().Duration("timeout", 30*time.Second, "")
	toolcallCmd.Flags().String("session-id", "", "")
	toolcallCmd.Flags().Bool("include-result", true, "")
	toolcallCmd.Flags().Bool("async", false, "")
	toolcallCmd.Flags().String("param-path", "", "")
	toolcallCmd.Flags().String("param-mode", "", "")

	toolcallCmd.Flags().Set("cap-token", token)
	toolcallCmd.Flags().Set("param-path", "/tmp/test.txt")
	toolcallCmd.Flags().Set("param-mode", "read")

	var toolcallBuf bytes.Buffer
	toolcallCmd.SetOutput(&toolcallBuf)

	err = runAiToolcallExecute(toolcallCmd, []string{"open_file"})
	assert.NoError(t, err)
	output := toolcallBuf.String()
	assert.Contains(t, output, "Tool call successful: open_file")
}
