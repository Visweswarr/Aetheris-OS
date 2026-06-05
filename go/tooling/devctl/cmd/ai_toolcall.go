// Package devctl provides CLI commands for AI Core Service tool calling functionality
//
// This module provides the devctl ai toolcall command for executing tool calls
// through the AI Core Service, including tool discovery, execution, and result handling.

package main

import (
	"context"
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

// AI Toolcall commands
var aiToolcallCmd = &cobra.Command{
	Use:   "toolcall",
	Short: "Execute tool calls through AI Core Service",
	Long: `Execute tool calls through the AI Core Service.
	
Examples:
  devctl ai toolcall open_file --path /tmp/test.txt --mode read
  devctl ai toolcall search_files --query "*.go" --directory ./src
  devctl ai toolcall create_note --title "Meeting Notes" --content "Discussion points..."
  devctl ai toolcall list --format json
  devctl ai toolcall describe open_file`,
}

// AI Toolcall list command
var aiToolcallListCmd = &cobra.Command{
	Use:   "list",
	Short: "List available tools",
	Long:  "List all available tools that can be called through the AI Core Service",
	RunE:  runAiToolcallList,
}

// AI Toolcall describe command
var aiToolcallDescribeCmd = &cobra.Command{
	Use:   "describe [tool-name]",
	Short: "Describe a specific tool",
	Long:  "Get detailed information about a specific tool including parameters and capabilities",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiToolcallDescribe,
}

// AI Toolcall execute command
var aiToolcallExecuteCmd = &cobra.Command{
	Use:   "execute [tool-name]",
	Short: "Execute a tool call",
	Long:  "Execute a specific tool call with the provided parameters",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiToolcallExecute,
}

// Toolcall configuration
type ToolcallConfig struct {
	ToolName      string
	Parameters    map[string]interface{}
	CapToken      string
	SocketPath    string
	Timeout       time.Duration
	OutputFormat  string
	SessionID     string
	IncludeResult bool
	Async         bool
}

// Tool information
type ToolInfo struct {
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	Parameters  map[string]interface{} `json:"parameters"`
	Capabilities []string              `json:"capabilities"`
	Version     string                 `json:"version"`
	Author      string                 `json:"author"`
}

// runAiToolcallList executes the list tools command
func runAiToolcallList(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getToolcallConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get toolcall config: %w", err)
	}

	// Get available tools
	tools, err := getAvailableTools(config)
	if err != nil {
		return fmt.Errorf("failed to get available tools: %w", err)
	}

	// Output tools based on format
	switch config.OutputFormat {
	case "json":
		return outputToolsJSON(tools)
	case "table":
		return outputToolsTable(tools)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiToolcallDescribe executes the describe tool command
func runAiToolcallDescribe(cmd *cobra.Command, args []string) error {
	toolName := args[0]
	
	// Get configuration from flags
	config, err := getToolcallConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get toolcall config: %w", err)
	}

	// Get tool information
	toolInfo, err := getToolInfo(toolName, config)
	if err != nil {
		return fmt.Errorf("failed to get tool info: %w", err)
	}

	// Output tool information
	switch config.OutputFormat {
	case "json":
		return outputToolInfoJSON(toolInfo)
	case "text":
		return outputToolInfoText(toolInfo)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiToolcallExecute executes the execute tool command
func runAiToolcallExecute(cmd *cobra.Command, args []string) error {
	toolName := args[0]
	
	// Get configuration from flags
	config, err := getToolcallConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get toolcall config: %w", err)
	}

	// Get parameters from flags
	parameters, err := getToolParameters(cmd)
	if err != nil {
		return fmt.Errorf("failed to get tool parameters: %w", err)
	}

	config.ToolName = toolName
	config.Parameters = parameters

	// Execute tool call
	return executeToolCall(config)
}

// getToolcallConfig extracts configuration from command flags
func getToolcallConfig(cmd *cobra.Command) (ToolcallConfig, error) {
	capToken, _ := cmd.Flags().GetString("cap-token")
	socketPath, _ := cmd.Flags().GetString("socket")
	timeout, _ := cmd.Flags().GetDuration("timeout")
	outputFormat, _ := cmd.Flags().GetString("format")
	sessionID, _ := cmd.Flags().GetString("session-id")
	includeResult, _ := cmd.Flags().GetBool("include-result")
	async, _ := cmd.Flags().GetBool("async")

	// Set defaults
	if socketPath == "" {
		socketPath = "/tmp/ai_core.sock"
	}
	if timeout == 0 {
		timeout = 30 * time.Second
	}
	if outputFormat == "" {
		outputFormat = "text"
	}

	return ToolcallConfig{
		CapToken:      capToken,
		SocketPath:    socketPath,
		Timeout:       timeout,
		OutputFormat:  outputFormat,
		SessionID:     sessionID,
		IncludeResult: includeResult,
		Async:         async,
	}, nil
}

// getToolParameters extracts tool parameters from command flags
func getToolParameters(cmd *cobra.Command) (map[string]interface{}, error) {
	parameters := make(map[string]interface{})

	// Get all flags that start with "param-"
	cmd.Flags().VisitAll(func(flag *cobra.Flag) {
		if strings.HasPrefix(flag.Name, "param-") {
			paramName := strings.TrimPrefix(flag.Name, "param-")
			paramValue := flag.Value.String()
			parameters[paramName] = paramValue
		}
	})

	return parameters, nil
}

// getAvailableTools retrieves available tools from AI Core Service
func getAvailableTools(config ToolcallConfig) ([]ToolInfo, error) {
	// Create tool list request
	request := &ai_core.ToolListRequest{
		IncludeParameters: true,
		IncludeCapabilities: true,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		SessionId: config.SessionID,
		MessageType: &ai_core.AiCoreMessage_ToolListRequest{
			ToolListRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{"ai:tool.list"},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC (simulated for now)
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	return simulateToolListResponse(ctx, aiMessage, config)
}

// getToolInfo retrieves information about a specific tool
func getToolInfo(toolName string, config ToolcallConfig) (ToolInfo, error) {
	// Create tool info request
	request := &ai_core.ToolInfoRequest{
		ToolName: toolName,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		SessionId: config.SessionID,
		MessageType: &ai_core.AiCoreMessage_ToolInfoRequest{
			ToolInfoRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{"ai:tool.info"},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC (simulated for now)
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	return simulateToolInfoResponse(ctx, aiMessage, config)
}

// executeToolCall executes a tool call through the AI Core Service
func executeToolCall(config ToolcallConfig) error {
	// Convert parameters to protobuf format
	parameters := make(map[string]*ai_core.ParameterValue)
	for key, value := range config.Parameters {
		paramValue := &ai_core.ParameterValue{
			Value: &ai_core.ParameterValue_StringValue{
				StringValue: fmt.Sprintf("%v", value),
			},
		}
		parameters[key] = paramValue
	}

	// Create tool call request
	request := &ai_core.ToolCallRequest{
		ToolName:   config.ToolName,
		Parameters: parameters,
		Async:      config.Async,
		SessionId:  config.SessionID,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		SessionId: config.SessionID,
		MessageType: &ai_core.AiCoreMessage_ToolCallRequest{
			ToolCallRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{fmt.Sprintf("ai:tool.%s", config.ToolName)},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	return simulateToolCallResponse(ctx, aiMessage, config)
}

// simulateToolListResponse simulates a tool list response
func simulateToolListResponse(ctx context.Context, message *ai_core.AiCoreMessage, config ToolcallConfig) ([]ToolInfo, error) {
	// Simulate processing delay
	time.Sleep(50 * time.Millisecond)

	// Mock available tools
	tools := []ToolInfo{
		{
			Name:        "open_file",
			Description: "Open and read a file from the filesystem",
			Parameters: map[string]interface{}{
				"path": map[string]interface{}{
					"type":        "string",
					"description": "Path to the file to open",
					"required":    true,
				},
				"mode": map[string]interface{}{
					"type":        "string",
					"description": "File access mode (read, write, append)",
					"required":    false,
					"default":     "read",
				},
			},
			Capabilities: []string{"file.read", "file.write"},
			Version:      "1.0.0",
			Author:       "Aetheris OS Team",
		},
		{
			Name:        "search_files",
			Description: "Search for files matching a pattern",
			Parameters: map[string]interface{}{
				"query": map[string]interface{}{
					"type":        "string",
					"description": "Search query or pattern",
					"required":    true,
				},
				"directory": map[string]interface{}{
					"type":        "string",
					"description": "Directory to search in",
					"required":    false,
					"default":     ".",
				},
				"recursive": map[string]interface{}{
					"type":        "boolean",
					"description": "Search recursively in subdirectories",
					"required":    false,
					"default":     true,
				},
			},
			Capabilities: []string{"file.search"},
			Version:      "1.0.0",
			Author:       "Aetheris OS Team",
		},
		{
			Name:        "create_note",
			Description: "Create a new note or document",
			Parameters: map[string]interface{}{
				"title": map[string]interface{}{
					"type":        "string",
					"description": "Title of the note",
					"required":    true,
				},
				"content": map[string]interface{}{
					"type":        "string",
					"description": "Content of the note",
					"required":    true,
				},
				"format": map[string]interface{}{
					"type":        "string",
					"description": "Note format (markdown, text, html)",
					"required":    false,
					"default":     "markdown",
				},
			},
			Capabilities: []string{"note.create", "note.write"},
			Version:      "1.0.0",
			Author:       "Aetheris OS Team",
		},
		{
			Name:        "get_system_time",
			Description: "Get the current system time",
			Parameters: map[string]interface{}{
				"format": map[string]interface{}{
					"type":        "string",
					"description": "Time format (iso, unix, readable)",
					"required":    false,
					"default":     "iso",
				},
			},
			Capabilities: []string{"system.time"},
			Version:      "1.0.0",
			Author:       "Aetheris OS Team",
		},
	}

	return tools, nil
}

// simulateToolInfoResponse simulates a tool info response
func simulateToolInfoResponse(ctx context.Context, message *ai_core.AiCoreMessage, config ToolcallConfig) (ToolInfo, error) {
	toolName := message.GetToolInfoRequest().ToolName
	
	// Get available tools
	tools, err := simulateToolListResponse(ctx, message, config)
	if err != nil {
		return ToolInfo{}, err
	}

	// Find the requested tool
	for _, tool := range tools {
		if tool.Name == toolName {
			return tool, nil
		}
	}

	return ToolInfo{}, fmt.Errorf("tool not found: %s", toolName)
}

// simulateToolCallResponse simulates a tool call response
func simulateToolCallResponse(ctx context.Context, message *ai_core.AiCoreMessage, config ToolcallConfig) error {
	// Simulate processing delay
	time.Sleep(100 * time.Millisecond)

	toolName := message.GetToolCallRequest().ToolName
	parameters := message.GetToolCallRequest().Parameters

	// Create mock response based on tool
	var result string
	var success bool

	switch toolName {
	case "open_file":
		path := parameters["path"].GetStringValue()
		result = fmt.Sprintf("File opened successfully: %s\nContent: [Mock file content for %s]", path, path)
		success = true
	case "search_files":
		query := parameters["query"].GetStringValue()
		directory := parameters["directory"].GetStringValue()
		result = fmt.Sprintf("Search completed in %s for '%s'\nFound 3 files:\n- file1.go\n- file2.go\n- file3.go", directory, query)
		success = true
	case "create_note":
		title := parameters["title"].GetStringValue()
		content := parameters["content"].GetStringValue()
		result = fmt.Sprintf("Note created successfully:\nTitle: %s\nContent: %s\nFile saved to: notes/%s.md", title, content, strings.ToLower(strings.ReplaceAll(title, " ", "_")))
		success = true
	case "get_system_time":
		result = fmt.Sprintf("Current system time: %s", time.Now().Format(time.RFC3339))
		success = true
	default:
		result = fmt.Sprintf("Unknown tool: %s", toolName)
		success = false
	}

	// Output result based on format
	switch config.OutputFormat {
	case "json":
		response := map[string]interface{}{
			"tool":    toolName,
			"success": success,
			"result":  result,
			"timestamp": time.Now().Unix(),
		}
		jsonData, err := json.MarshalIndent(response, "", "  ")
		if err != nil {
			return fmt.Errorf("failed to marshal response: %w", err)
		}
		fmt.Println(string(jsonData))
	case "text":
		if success {
			fmt.Printf("✅ Tool call successful: %s\n", toolName)
		} else {
			fmt.Printf("❌ Tool call failed: %s\n", toolName)
		}
		fmt.Println(result)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}

	return nil
}

// outputToolsJSON outputs tools in JSON format
func outputToolsJSON(tools []ToolInfo) error {
	jsonData, err := json.MarshalIndent(tools, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal tools: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputToolsTable outputs tools in table format
func outputToolsTable(tools []ToolInfo) error {
	fmt.Println("🔧 Available Tools")
	fmt.Println("==================")
	fmt.Println()
	
	for _, tool := range tools {
		fmt.Printf("📋 %s\n", tool.Name)
		fmt.Printf("   Description: %s\n", tool.Description)
		fmt.Printf("   Version: %s\n", tool.Version)
		fmt.Printf("   Capabilities: %s\n", strings.Join(tool.Capabilities, ", "))
		fmt.Println()
	}
	
	return nil
}

// outputToolInfoJSON outputs tool info in JSON format
func outputToolInfoJSON(toolInfo ToolInfo) error {
	jsonData, err := json.MarshalIndent(toolInfo, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal tool info: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputToolInfoText outputs tool info in text format
func outputToolInfoText(toolInfo ToolInfo) error {
	fmt.Printf("📋 Tool: %s\n", toolInfo.Name)
	fmt.Printf("Description: %s\n", toolInfo.Description)
	fmt.Printf("Version: %s\n", toolInfo.Version)
	fmt.Printf("Author: %s\n", toolInfo.Author)
	fmt.Printf("Capabilities: %s\n", strings.Join(toolInfo.Capabilities, ", "))
	fmt.Println()
	
	fmt.Println("Parameters:")
	for paramName, paramInfo := range toolInfo.Parameters {
		paramMap := paramInfo.(map[string]interface{})
		fmt.Printf("  %s (%s): %s", paramName, paramMap["type"], paramMap["description"])
		if required, ok := paramMap["required"].(bool); ok && required {
			fmt.Print(" [required]")
		}
		if defaultValue, ok := paramMap["default"]; ok {
			fmt.Printf(" [default: %v]", defaultValue)
		}
		fmt.Println()
	}
	
	return nil
}

func init() {
	// Add toolcall commands to AI command
	aiCmd.AddCommand(aiToolcallCmd)
	aiToolcallCmd.AddCommand(aiToolcallListCmd)
	aiToolcallCmd.AddCommand(aiToolcallDescribeCmd)
	aiToolcallCmd.AddCommand(aiToolcallExecuteCmd)

	// Common flags for all toolcall commands
	aiToolcallCmd.PersistentFlags().String("cap-token", "", "Capability token for authentication")
	aiToolcallCmd.PersistentFlags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiToolcallCmd.PersistentFlags().Duration("timeout", 30*time.Second, "Request timeout")
	aiToolcallCmd.PersistentFlags().String("format", "text", "Output format (text, json)")
	aiToolcallCmd.PersistentFlags().String("session-id", "", "Session ID for conversation")
	aiToolcallCmd.PersistentFlags().Bool("include-result", true, "Include tool execution result")
	aiToolcallCmd.PersistentFlags().Bool("async", false, "Execute tool call asynchronously")

	// List command flags
	aiToolcallListCmd.Flags().String("format", "table", "Output format (table, json)")

	// Describe command flags
	aiToolcallDescribeCmd.Flags().String("format", "text", "Output format (text, json)")

	// Execute command flags
	aiToolcallExecuteCmd.Flags().String("param-path", "", "File path parameter")
	aiToolcallExecuteCmd.Flags().String("param-mode", "", "File mode parameter")
	aiToolcallExecuteCmd.Flags().String("param-query", "", "Search query parameter")
	aiToolcallExecuteCmd.Flags().String("param-directory", "", "Directory parameter")
	aiToolcallExecuteCmd.Flags().Bool("param-recursive", false, "Recursive search parameter")
	aiToolcallExecuteCmd.Flags().String("param-title", "", "Note title parameter")
	aiToolcallExecuteCmd.Flags().String("param-content", "", "Note content parameter")
	aiToolcallExecuteCmd.Flags().String("param-format", "", "Format parameter")
}
