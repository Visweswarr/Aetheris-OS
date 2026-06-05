// Package devctl provides CLI commands for AI Core Service chat functionality
//
// This module provides the devctl ai chat command for interacting with the AI Core Service,
// including streaming responses, context management, and CapToken integration.

package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	"github.com/spf13/cobra"
	"google.golang.org/protobuf/proto"

	// Import generated protobuf types
	ai_core "github.com/aetheris-os/go/tooling/ai_core"
)

// AI Chat commands
var aiChatCmd = &cobra.Command{
	Use:   "chat [message]",
	Short: "Chat with AI Core Service",
	Long: `Send a message to the AI Core Service and receive a streaming response.
	
Examples:
  devctl ai chat "Summarize README"
  devctl ai chat "What is the capital of France?"
  devctl ai chat --interactive
  devctl ai chat --file input.txt`,
	Args: cobra.MaximumNArgs(1),
	RunE: runAiChat,
}

// AI Chat interactive command
var aiChatInteractiveCmd = &cobra.Command{
	Use:   "interactive",
	Short: "Start interactive chat session",
	Long:  "Start an interactive chat session with the AI Core Service",
	RunE:  runAiChatInteractive,
}

// AI Chat from file command
var aiChatFileCmd = &cobra.Command{
	Use:   "file [input-file]",
	Short: "Send message from file",
	Long:  "Send a message to the AI Core Service from a text file",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiChatFile,
}

// Chat configuration
type ChatConfig struct {
	Model           string
	Temperature     float64
	MaxTokens       uint32
	TopP            float64
	StopSequences   []string
	Stream          bool
	Context         []string
	SessionID       string
	CapToken        string
	SocketPath      string
	Timeout         time.Duration
	OutputFormat    string
	IncludeMetadata bool
}

// Chat session state
type ChatSession struct {
	SessionID    string
	Context      []string
	MessageCount int
	StartTime    time.Time
	Config       ChatConfig
}

// Global chat session
var chatSession *ChatSession

// runAiChat executes the main chat command
func runAiChat(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getChatConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get chat config: %w", err)
	}

	// Get message from args or stdin
	var message string
	if len(args) > 0 {
		message = args[0]
	} else {
		// Read from stdin
		scanner := bufio.NewScanner(os.Stdin)
		var lines []string
		for scanner.Scan() {
			lines = append(lines, scanner.Text())
		}
		if err := scanner.Err(); err != nil {
			return fmt.Errorf("failed to read from stdin: %w", err)
		}
		message = strings.Join(lines, "\n")
	}

	if message == "" {
		return fmt.Errorf("no message provided")
	}

	// Initialize chat session if needed
	if chatSession == nil {
		chatSession = &ChatSession{
			SessionID:    generateSessionID(),
			Context:      config.Context,
			MessageCount: 0,
			StartTime:    time.Now(),
			Config:       config,
		}
	}

	// Send chat request
	return sendChatRequest(message, config)
}

// runAiChatInteractive executes the interactive chat command
func runAiChatInteractive(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getChatConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get chat config: %w", err)
	}

	// Initialize chat session
	chatSession = &ChatSession{
		SessionID:    generateSessionID(),
		Context:      config.Context,
		MessageCount: 0,
		StartTime:    time.Now(),
		Config:       config,
	}

	fmt.Println("🤖 AI Core Service Interactive Chat")
	fmt.Println("===================================")
	fmt.Println("Type your messages and press Enter. Type 'quit', 'exit', or 'bye' to end the session.")
	fmt.Println("Type 'clear' to clear context, 'context' to show context, 'help' for more commands.")
	fmt.Println()

	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print("You: ")
		if !scanner.Scan() {
			break
		}

		input := strings.TrimSpace(scanner.Text())
		if input == "" {
			continue
		}

		// Handle special commands
		switch strings.ToLower(input) {
		case "quit", "exit", "bye":
			fmt.Println("👋 Goodbye!")
			return nil
		case "clear":
			chatSession.Context = []string{}
			fmt.Println("🧹 Context cleared")
			continue
		case "context":
			displayContext(chatSession.Context)
			continue
		case "help":
			displayHelp()
			continue
		case "session":
			displaySessionInfo(chatSession)
			continue
		}

		// Send message to AI Core Service
		fmt.Print("AI: ")
		err := sendChatRequest(input, config)
		if err != nil {
			fmt.Printf("❌ Error: %v\n", err)
			continue
		}
		fmt.Println()

		// Update session
		chatSession.MessageCount++
		chatSession.Context = append(chatSession.Context, fmt.Sprintf("User: %s", input))
	}

	return scanner.Err()
}

// runAiChatFile executes the chat from file command
func runAiChatFile(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getChatConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get chat config: %w", err)
	}

	// Read file content
	filePath := args[0]
	content, err := os.ReadFile(filePath)
	if err != nil {
		return fmt.Errorf("failed to read file %s: %w", filePath, err)
	}

	message := strings.TrimSpace(string(content))
	if message == "" {
		return fmt.Errorf("file %s is empty", filePath)
	}

	// Initialize chat session if needed
	if chatSession == nil {
		chatSession = &ChatSession{
			SessionID:    generateSessionID(),
			Context:      config.Context,
			MessageCount: 0,
			StartTime:    time.Now(),
			Config:       config,
		}
	}

	// Send chat request
	return sendChatRequest(message, config)
}

// getChatConfig extracts configuration from command flags
func getChatConfig(cmd *cobra.Command) (ChatConfig, error) {
	model, _ := cmd.Flags().GetString("model")
	temperature, _ := cmd.Flags().GetFloat64("temperature")
	maxTokens, _ := cmd.Flags().GetUint32("max-tokens")
	topP, _ := cmd.Flags().GetFloat64("top-p")
	stopSequences, _ := cmd.Flags().GetStringSlice("stop")
	stream, _ := cmd.Flags().GetBool("stream")
	context, _ := cmd.Flags().GetStringSlice("context")
	sessionID, _ := cmd.Flags().GetString("session-id")
	capToken, _ := cmd.Flags().GetString("cap-token")
	socketPath, _ := cmd.Flags().GetString("socket")
	timeout, _ := cmd.Flags().GetDuration("timeout")
	outputFormat, _ := cmd.Flags().GetString("format")
	includeMetadata, _ := cmd.Flags().GetBool("include-metadata")

	// Set defaults
	if model == "" {
		model = "gpt-3.5-turbo"
	}
	if temperature == 0 {
		temperature = 0.7
	}
	if maxTokens == 0 {
		maxTokens = 1000
	}
	if topP == 0 {
		topP = 1.0
	}
	if socketPath == "" {
		socketPath = "/tmp/ai_core.sock"
	}
	if timeout == 0 {
		timeout = 30 * time.Second
	}
	if outputFormat == "" {
		outputFormat = "text"
	}

	return ChatConfig{
		Model:           model,
		Temperature:     temperature,
		MaxTokens:       maxTokens,
		TopP:            topP,
		StopSequences:   stopSequences,
		Stream:          stream,
		Context:         context,
		SessionID:       sessionID,
		CapToken:        capToken,
		SocketPath:      socketPath,
		Timeout:         timeout,
		OutputFormat:    outputFormat,
		IncludeMetadata: includeMetadata,
	}, nil
}

// sendChatRequest sends a chat request to the AI Core Service
func sendChatRequest(message string, config ChatConfig) error {
	// Create chat request
	request := &ai_core.ChatRequest{
		Message:         message,
		Context:         config.Context,
		Model:           config.Model,
		Temperature:     config.Temperature,
		MaxTokens:       config.MaxTokens,
		TopP:            config.TopP,
		StopSequences:   config.StopSequences,
		Stream:          config.Stream,
		SessionId:       config.SessionID,
		IncludeMetadata: config.IncludeMetadata,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		SessionId: config.SessionID,
		MessageType: &ai_core.AiCoreMessage_ChatRequest{
			ChatRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{"ai:chat"},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	// For now, simulate the response (in real implementation, this would use IPC)
	return simulateChatResponse(ctx, aiMessage, config)
}

// simulateChatResponse simulates a chat response from the AI Core Service
func simulateChatResponse(ctx context.Context, message *ai_core.AiCoreMessage, config ChatConfig) error {
	// Simulate processing delay
	time.Sleep(100 * time.Millisecond)

	// Create mock response
	response := &ai_core.ChatResponse{
		Content:        generateMockResponse(message.GetChatRequest().Message),
		TokensGenerated: 50,
		TokensInput:     uint32(len(strings.Split(message.GetChatRequest().Message, " "))),
		Model:          config.Model,
		SessionId:      config.SessionID,
		Timestamp:      time.Now().Unix(),
		Metadata: &ai_core.ResponseMetadata{
			ProcessingTimeMs: 150,
			Confidence:       0.95,
			ModelVersion:     "1.0.0",
		},
	}

	// Output response based on format
	switch config.OutputFormat {
	case "json":
		return outputJSONResponse(response)
	case "text":
		return outputTextResponse(response, config.Stream)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// generateMockResponse generates a mock response based on the input message
func generateMockResponse(message string) string {
	message = strings.ToLower(message)
	
	if strings.Contains(message, "summarize") || strings.Contains(message, "summary") {
		return "Here's a summary of the requested content: [This is a mock response from the AI Core Service. In a real implementation, this would be the actual AI-generated summary.]"
	}
	
	if strings.Contains(message, "capital") && strings.Contains(message, "france") {
		return "The capital of France is Paris."
	}
	
	if strings.Contains(message, "hello") || strings.Contains(message, "hi") {
		return "Hello! I'm the AI Core Service. How can I help you today?"
	}
	
	if strings.Contains(message, "help") {
		return "I can help you with various tasks including answering questions, summarizing content, generating text, and more. What would you like to know?"
	}
	
	// Default response
	return fmt.Sprintf("I understand you're asking about: \"%s\". This is a mock response from the AI Core Service. In a real implementation, this would be the actual AI-generated response.", message)
}

// outputJSONResponse outputs the response in JSON format
func outputJSONResponse(response *ai_core.ChatResponse) error {
	jsonData, err := json.MarshalIndent(response, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal response: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputTextResponse outputs the response in text format
func outputTextResponse(response *ai_core.ChatResponse, stream bool) error {
	if stream {
		// Simulate streaming response
		words := strings.Split(response.Content, " ")
		for i, word := range words {
			if i > 0 {
				fmt.Print(" ")
			}
			fmt.Print(word)
			time.Sleep(50 * time.Millisecond) // Simulate streaming delay
		}
	} else {
		fmt.Print(response.Content)
	}
	return nil
}

// generateSessionID generates a unique session ID
func generateSessionID() string {
	return fmt.Sprintf("session_%d", time.Now().UnixNano())
}

// generateMessageID generates a unique message ID
func generateMessageID() string {
	return fmt.Sprintf("msg_%d", time.Now().UnixNano())
}

// displayContext displays the current chat context
func displayContext(context []string) {
	if len(context) == 0 {
		fmt.Println("📝 No context available")
		return
	}
	
	fmt.Println("📝 Chat Context:")
	fmt.Println("================")
	for i, ctx := range context {
		fmt.Printf("%d. %s\n", i+1, ctx)
	}
}

// displayHelp displays help information
func displayHelp() {
	fmt.Println("🤖 AI Core Service Chat Help")
	fmt.Println("============================")
	fmt.Println("Commands:")
	fmt.Println("  quit, exit, bye  - End the chat session")
	fmt.Println("  clear           - Clear the chat context")
	fmt.Println("  context         - Show current context")
	fmt.Println("  session         - Show session information")
	fmt.Println("  help            - Show this help message")
	fmt.Println()
	fmt.Println("Features:")
	fmt.Println("  - Streaming responses")
	fmt.Println("  - Context management")
	fmt.Println("  - CapToken authentication")
	fmt.Println("  - Multiple output formats")
}

// displaySessionInfo displays session information
func displaySessionInfo(session *ChatSession) {
	fmt.Println("📊 Session Information")
	fmt.Println("=====================")
	fmt.Printf("Session ID: %s\n", session.SessionID)
	fmt.Printf("Messages: %d\n", session.MessageCount)
	fmt.Printf("Duration: %s\n", time.Since(session.StartTime).Round(time.Second))
	fmt.Printf("Context entries: %d\n", len(session.Context))
	fmt.Printf("Model: %s\n", session.Config.Model)
	fmt.Printf("Temperature: %.2f\n", session.Config.Temperature)
	fmt.Printf("Max tokens: %d\n", session.Config.MaxTokens)
}

func init() {
	// Add chat commands to AI command
	aiCmd.AddCommand(aiChatCmd)
	aiChatCmd.AddCommand(aiChatInteractiveCmd)
	aiChatCmd.AddCommand(aiChatFileCmd)

	// Chat command flags
	aiChatCmd.Flags().String("model", "gpt-3.5-turbo", "AI model to use")
	aiChatCmd.Flags().Float64("temperature", 0.7, "Sampling temperature (0.0-2.0)")
	aiChatCmd.Flags().Uint32("max-tokens", 1000, "Maximum tokens to generate")
	aiChatCmd.Flags().Float64("top-p", 1.0, "Top-p sampling parameter")
	aiChatCmd.Flags().StringSlice("stop", []string{}, "Stop sequences")
	aiChatCmd.Flags().Bool("stream", true, "Enable streaming responses")
	aiChatCmd.Flags().StringSlice("context", []string{}, "Context messages")
	aiChatCmd.Flags().String("session-id", "", "Session ID for conversation")
	aiChatCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiChatCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiChatCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")
	aiChatCmd.Flags().String("format", "text", "Output format (text, json)")
	aiChatCmd.Flags().Bool("include-metadata", false, "Include response metadata")

	// Interactive command flags
	aiChatInteractiveCmd.Flags().String("model", "gpt-3.5-turbo", "AI model to use")
	aiChatInteractiveCmd.Flags().Float64("temperature", 0.7, "Sampling temperature (0.0-2.0)")
	aiChatInteractiveCmd.Flags().Uint32("max-tokens", 1000, "Maximum tokens to generate")
	aiChatInteractiveCmd.Flags().Float64("top-p", 1.0, "Top-p sampling parameter")
	aiChatInteractiveCmd.Flags().StringSlice("stop", []string{}, "Stop sequences")
	aiChatInteractiveCmd.Flags().Bool("stream", true, "Enable streaming responses")
	aiChatInteractiveCmd.Flags().StringSlice("context", []string{}, "Context messages")
	aiChatInteractiveCmd.Flags().String("session-id", "", "Session ID for conversation")
	aiChatInteractiveCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiChatInteractiveCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiChatInteractiveCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")
	aiChatInteractiveCmd.Flags().String("format", "text", "Output format (text, json)")
	aiChatInteractiveCmd.Flags().Bool("include-metadata", false, "Include response metadata")

	// File command flags
	aiChatFileCmd.Flags().String("model", "gpt-3.5-turbo", "AI model to use")
	aiChatFileCmd.Flags().Float64("temperature", 0.7, "Sampling temperature (0.0-2.0)")
	aiChatFileCmd.Flags().Uint32("max-tokens", 1000, "Maximum tokens to generate")
	aiChatFileCmd.Flags().Float64("top-p", 1.0, "Top-p sampling parameter")
	aiChatFileCmd.Flags().StringSlice("stop", []string{}, "Stop sequences")
	aiChatFileCmd.Flags().Bool("stream", true, "Enable streaming responses")
	aiChatFileCmd.Flags().StringSlice("context", []string{}, "Context messages")
	aiChatFileCmd.Flags().String("session-id", "", "Session ID for conversation")
	aiChatFileCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiChatFileCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiChatFileCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")
	aiChatFileCmd.Flags().String("format", "text", "Output format (text, json)")
	aiChatFileCmd.Flags().Bool("include-metadata", false, "Include response metadata")
}
