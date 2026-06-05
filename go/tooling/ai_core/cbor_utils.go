// CBOR utilities for AI Core Service
package ai_core

import (
	"encoding/json"
	"fmt"

	"github.com/fxamacker/cbor/v2"
)

// CborUtils provides utility functions for working with CBOR payloads
type CborUtils struct{}

// Encode encodes a Go value to CBOR bytes
func (c *CborUtils) Encode(v interface{}) ([]byte, error) {
	em, err := cbor.EncOptions{}.EncMode()
	if err != nil {
		return nil, fmt.Errorf("failed to create CBOR encoder: %w", err)
	}
	
	data, err := em.Marshal(v)
	if err != nil {
		return nil, fmt.Errorf("failed to encode to CBOR: %w", err)
	}
	
	return data, nil
}

// Decode decodes CBOR bytes to a Go value
func (c *CborUtils) Decode(data []byte, v interface{}) error {
	dm, err := cbor.DecOptions{}.DecMode()
	if err != nil {
		return fmt.Errorf("failed to create CBOR decoder: %w", err)
	}
	
	err = dm.Unmarshal(data, v)
	if err != nil {
		return fmt.Errorf("failed to decode from CBOR: %w", err)
	}
	
	return nil
}

// EncodeToJSON encodes a Go value to JSON bytes (fallback for simple cases)
func (c *CborUtils) EncodeToJSON(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}

// DecodeFromJSON decodes JSON bytes to a Go value (fallback for simple cases)
func (c *CborUtils) DecodeFromJSON(data []byte, v interface{}) error {
	return json.Unmarshal(data, v)
}

// MessageBuilder provides convenience methods for creating AI Core messages
type MessageBuilder struct{}

// NewMessageBuilder creates a new message builder
func NewMessageBuilder() *MessageBuilder {
	return &MessageBuilder{}
}

// CreatePingRequest creates a ping request message
func (m *MessageBuilder) CreatePingRequest(clientID, version string) *AiCoreMessage {
	return &AiCoreMessage{
		MessageType: &AiCoreMessage_PingRequest{
			PingRequest: &PingRequest{
				ClientId: clientID,
				Version:  version,
			},
		},
		MessageId: generateMessageID(),
		Timestamp: getCurrentTimestamp(),
		SessionId: generateSessionID(),
	}
}

// CreateChatRequest creates a chat request message
func (m *MessageBuilder) CreateChatRequest(prompt string, config *ChatConfig, conversationID string) *AiCoreMessage {
	return &AiCoreMessage{
		MessageType: &AiCoreMessage_ChatRequest{
			ChatRequest: &ChatRequest{
				Prompt:          prompt,
				Config:          config,
				Context:         []*MessageContext{},
				Stream:          false,
				ConversationId:  conversationID,
				Metadata:        make(map[string]string),
				Attachments:     []string{},
				EnableFunctionCalling: false,
				AllowedFunctions:      []string{},
			},
		},
		MessageId: generateMessageID(),
		Timestamp: getCurrentTimestamp(),
		SessionId: generateSessionID(),
	}
}

// CreateToolCallRequest creates a tool call request message
func (m *MessageBuilder) CreateToolCallRequest(toolName string, parameters map[string]string, context string) *AiCoreMessage {
	return &AiCoreMessage{
		MessageType: &AiCoreMessage_ToolCallRequest{
			ToolCallRequest: &ToolCallRequest{
				ToolName:        toolName,
				Parameters:      parameters,
				Context:         context,
				Metadata:        make(map[string]string),
				TimeoutSeconds:  30,
				Async:           false,
			},
		},
		MessageId: generateMessageID(),
		Timestamp: getCurrentTimestamp(),
		SessionId: generateSessionID(),
	}
}

// CreateErrorEnvelope creates an error envelope message
func (m *MessageBuilder) CreateErrorEnvelope(code ErrorCode, message, details string) *AiCoreMessage {
	return &AiCoreMessage{
		MessageType: &AiCoreMessage_ErrorEnvelope{
			ErrorEnvelope: &ErrorEnvelope{
				Code:              code,
				Message:           message,
				Details:           details,
				Timestamp:         getCurrentTimestamp(),
				Context:           make(map[string]string),
				StackTrace:        []string{},
				Retryable:         false,
				RetryAfterSeconds: 0,
			},
		},
		MessageId: generateMessageID(),
		Timestamp: getCurrentTimestamp(),
		SessionId: generateSessionID(),
	}
}

// Helper functions
func generateMessageID() string {
	// In a real implementation, this would generate a proper UUID
	return "msg_" + fmt.Sprintf("%d", getCurrentTimestamp())
}

func generateSessionID() string {
	// In a real implementation, this would generate a proper UUID
	return "session_" + fmt.Sprintf("%d", getCurrentTimestamp())
}

func getCurrentTimestamp() int64 {
	// In a real implementation, this would return the current Unix timestamp
	return 1234567890 // Placeholder
}

// Global instances for convenience
var (
	DefaultCborUtils    = &CborUtils{}
	DefaultMessageBuilder = NewMessageBuilder()
)
