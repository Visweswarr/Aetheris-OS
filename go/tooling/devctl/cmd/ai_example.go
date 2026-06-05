// Package devctl provides examples for AI Core Service CLI commands
//
// This module provides comprehensive examples demonstrating the usage of
// AI Core Service CLI commands including chat, toolcall, metrics, and captoken.

package main

import (
	"fmt"
	"os"
	"time"
)

// ExampleAICommands demonstrates the usage of AI Core Service CLI commands
func ExampleAICommands() {
	fmt.Println("🤖 AI Core Service CLI Examples")
	fmt.Println("===============================")
	fmt.Println()

	// Example 1: Basic Chat
	fmt.Println("📝 Example 1: Basic Chat")
	fmt.Println("------------------------")
	fmt.Println("Command: devctl ai chat \"What is the capital of France?\"")
	fmt.Println("Output:")
	fmt.Println("AI: The capital of France is Paris.")
	fmt.Println()

	// Example 2: Interactive Chat
	fmt.Println("💬 Example 2: Interactive Chat")
	fmt.Println("-----------------------------")
	fmt.Println("Command: devctl ai chat interactive")
	fmt.Println("Output:")
	fmt.Println("🤖 AI Core Service Interactive Chat")
	fmt.Println("===================================")
	fmt.Println("Type your messages and press Enter. Type 'quit', 'exit', or 'bye' to end the session.")
	fmt.Println()
	fmt.Println("You: Hello!")
	fmt.Println("AI: Hello! I'm the AI Core Service. How can I help you today?")
	fmt.Println()
	fmt.Println("You: Help me with something")
	fmt.Println("AI: I can help you with various tasks including answering questions, summarizing content, generating text, and more. What would you like to know?")
	fmt.Println()

	// Example 3: Chat with Custom Parameters
	fmt.Println("⚙️  Example 3: Chat with Custom Parameters")
	fmt.Println("------------------------------------------")
	fmt.Println("Command: devctl ai chat \"Summarize this document\" --model gpt-4 --temperature 0.3 --max-tokens 500")
	fmt.Println("Output:")
	fmt.Println("AI: Here's a summary of the requested content: [This is a mock response from the AI Core Service. In a real implementation, this would be the actual AI-generated summary.]")
	fmt.Println()

	// Example 4: Chat from File
	fmt.Println("📄 Example 4: Chat from File")
	fmt.Println("----------------------------")
	fmt.Println("Command: devctl ai chat file input.txt")
	fmt.Println("Output:")
	fmt.Println("AI: I understand you're asking about: \"[file content]\". This is a mock response from the AI Core Service.")
	fmt.Println()

	// Example 5: List Available Tools
	fmt.Println("🔧 Example 5: List Available Tools")
	fmt.Println("----------------------------------")
	fmt.Println("Command: devctl ai toolcall list")
	fmt.Println("Output:")
	fmt.Println("🔧 Available Tools")
	fmt.Println("==================")
	fmt.Println()
	fmt.Println("📋 open_file")
	fmt.Println("   Description: Open and read a file from the filesystem")
	fmt.Println("   Version: 1.0.0")
	fmt.Println("   Capabilities: file.read, file.write")
	fmt.Println()
	fmt.Println("📋 search_files")
	fmt.Println("   Description: Search for files matching a pattern")
	fmt.Println("   Version: 1.0.0")
	fmt.Println("   Capabilities: file.search")
	fmt.Println()

	// Example 6: Describe a Tool
	fmt.Println("📋 Example 6: Describe a Tool")
	fmt.Println("-----------------------------")
	fmt.Println("Command: devctl ai toolcall describe open_file")
	fmt.Println("Output:")
	fmt.Println("📋 Tool: open_file")
	fmt.Println("Description: Open and read a file from the filesystem")
	fmt.Println("Version: 1.0.0")
	fmt.Println("Author: Aetheris OS Team")
	fmt.Println("Capabilities: file.read, file.write")
	fmt.Println()
	fmt.Println("Parameters:")
	fmt.Println("  path (string): Path to the file to open [required]")
	fmt.Println("  mode (string): File access mode (read, write, append) [default: read]")
	fmt.Println()

	// Example 7: Execute a Tool
	fmt.Println("⚡ Example 7: Execute a Tool")
	fmt.Println("----------------------------")
	fmt.Println("Command: devctl ai toolcall execute open_file --param-path /tmp/test.txt --param-mode read")
	fmt.Println("Output:")
	fmt.Println("✅ Tool call successful: open_file")
	fmt.Println("File opened successfully: /tmp/test.txt")
	fmt.Println("Content: [Mock file content for /tmp/test.txt]")
	fmt.Println()

	// Example 8: Execute Multiple Tools
	fmt.Println("🔄 Example 8: Execute Multiple Tools")
	fmt.Println("------------------------------------")
	fmt.Println("Command: devctl ai toolcall execute search_files --param-query \"*.go\" --param-directory ./src")
	fmt.Println("Output:")
	fmt.Println("✅ Tool call successful: search_files")
	fmt.Println("Search completed in ./src for '*.go'")
	fmt.Println("Found 3 files:")
	fmt.Println("- file1.go")
	fmt.Println("- file2.go")
	fmt.Println("- file3.go")
	fmt.Println()

	// Example 9: Show Metrics
	fmt.Println("📊 Example 9: Show Metrics")
	fmt.Println("--------------------------")
	fmt.Println("Command: devctl ai metrics show")
	fmt.Println("Output:")
	fmt.Println("📊 AI Core Service Metrics")
	fmt.Println("==========================")
	fmt.Println()
	fmt.Println("📈 Request Metrics:")
	fmt.Println("  Total requests:     1250")
	fmt.Println("  Successful:         1180 (94.4%)")
	fmt.Println("  Errors:             70 (5.6%)")
	fmt.Println()
	fmt.Println("⏱️  Latency Metrics:")
	fmt.Println("  P50 (median):       45.2 ms")
	fmt.Println("  P95:                125.8 ms")
	fmt.Println("  P99:                250.3 ms")
	fmt.Println("  Maximum:            500.1 ms")
	fmt.Println()

	// Example 10: Real-time Metrics
	fmt.Println("📈 Example 10: Real-time Metrics")
	fmt.Println("--------------------------------")
	fmt.Println("Command: devctl ai metrics realtime --update-interval 2s")
	fmt.Println("Output:")
	fmt.Println("📊 AI Core Service Real-Time Metrics - 14:30:25")
	fmt.Println("==================================================")
	fmt.Println("📈 Requests: 1250 total, 1180 success (94.4%), 70 errors (5.6%)")
	fmt.Println("⏱️  Latency: P50=45.2ms, P95=125.8ms, P99=250.3ms, Max=500.1ms")
	fmt.Println("🔤 Tokens: 12.5/sec, 15680 total (8920 in, 6760 out)")
	fmt.Println("🔧 Tools: 340 calls, 315 success (92.6%), 25 errors (7.4%)")
	fmt.Println("💻 System: CPU=23.4%, Memory=1024.8MB, Disk=5120.2MB")
	fmt.Println("👥 Sessions: 3 active, 45 total, 180.5s avg duration")
	fmt.Println()

	// Example 11: Metrics History
	fmt.Println("📅 Example 11: Metrics History")
	fmt.Println("-------------------------------")
	fmt.Println("Command: devctl ai metrics history --time-range 24h --format table")
	fmt.Println("Output:")
	fmt.Println("📊 AI Core Service Metrics History")
	fmt.Println("==================================")
	fmt.Println()
	fmt.Println("📅 2024-01-15 14:00:00")
	fmt.Println("  Requests: 1250 total, 1180 success, 70 errors")
	fmt.Println("  Latency: P95=125.8ms, P99=250.3ms")
	fmt.Println("  Tokens: 12.5/sec, 15680 total")
	fmt.Println("  System: CPU=23.4%, Memory=1024.8MB")
	fmt.Println()

	// Example 12: Metrics Comparison
	fmt.Println("🔍 Example 12: Metrics Comparison")
	fmt.Println("---------------------------------")
	fmt.Println("Command: devctl ai metrics compare baseline.json")
	fmt.Println("Output:")
	fmt.Println("📊 AI Core Service Metrics Comparison")
	fmt.Println("=====================================")
	fmt.Println("Baseline: baseline.json")
	fmt.Println("Status: GOOD")
	fmt.Println()
	fmt.Println("Metric Differences:")
	fmt.Println("------------------")
	fmt.Println("RequestsTotal    : 1000 -> 1250 (+25.0%)")
	fmt.Println("LatencyP95Ms     : 100.0 -> 125.8 (+25.8%)")
	fmt.Println("TokensPerSecond  : 15.0 -> 12.5 (-16.7%)")
	fmt.Println("MemoryUsageMB    : 2000.0 -> 1024.8 (-48.8%)")
	fmt.Println("CPUUsagePercent  : 50.0 -> 23.4 (-53.2%)")
	fmt.Println()
	fmt.Println("✅ Improvements:")
	fmt.Println("  - MemoryUsageMB")
	fmt.Println("  - CPUUsagePercent")
	fmt.Println()

	// Example 13: Configure Metrics Alerts
	fmt.Println("🔔 Example 13: Configure Metrics Alerts")
	fmt.Println("---------------------------------------")
	fmt.Println("Command: devctl ai metrics alert --alert-latency-p95 200 --alert-memory-usage 4000 --alert-cpu-usage 80")
	fmt.Println("Output:")
	fmt.Println("🔔 Metrics Alerts Configuration")
	fmt.Println("===============================")
	fmt.Println("Configured alert thresholds:")
	fmt.Println("  latency-p95: 200.00")
	fmt.Println("  memory-usage: 4000.00")
	fmt.Println("  cpu-usage: 80.00")
	fmt.Println("✅ Alert configuration updated successfully")
	fmt.Println()

	// Example 14: Generate CapToken
	fmt.Println("🔑 Example 14: Generate CapToken")
	fmt.Println("--------------------------------")
	fmt.Println("Command: devctl ai captoken generate --scope \"ai:chat,ai:tool.open_file\" --expires-at \"2024-01-16T14:30:00Z\"")
	fmt.Println("Output:")
	fmt.Println("🔑 CapToken Generated")
	fmt.Println("====================")
	fmt.Println("Token: eyJ0b2tlbiI6ImdlbmVyYXRlZF90b2tlbiJ9...")
	fmt.Println("Scopes: ai:chat, ai:tool.open_file")
	fmt.Println("Expires: 2024-01-16T14:30:00Z")
	fmt.Println("Issuer: devctl")
	fmt.Println("Subject: user")
	fmt.Println("Audience: ai-core-service")
	fmt.Println()

	// Example 15: Validate CapToken
	fmt.Println("✅ Example 15: Validate CapToken")
	fmt.Println("--------------------------------")
	fmt.Println("Command: devctl ai captoken validate eyJ0b2tlbiI6InZhbGlkX3Rva2VuIn0...")
	fmt.Println("Output:")
	fmt.Println("🔍 CapToken Information")
	fmt.Println("=======================")
	fmt.Println("Token: eyJ0b2tlbiI6InZhbGlkX3Rva2VuIn0...")
	fmt.Println("Valid: true")
	fmt.Println("Expired: false")
	fmt.Println("Scopes: ai:chat, ai:tool.open_file")
	fmt.Println("Expires: 2024-01-16T14:30:00Z")
	fmt.Println("Issuer: devctl")
	fmt.Println("Subject: user")
	fmt.Println("Audience: ai-core-service")
	fmt.Println("Created: 2024-01-15T14:30:00Z")
	fmt.Println("✅ This token is valid")
	fmt.Println()

	// Example 16: List CapTokens
	fmt.Println("📋 Example 16: List CapTokens")
	fmt.Println("-----------------------------")
	fmt.Println("Command: devctl ai captoken list")
	fmt.Println("Output:")
	fmt.Println("📋 Active CapTokens")
	fmt.Println("===================")
	fmt.Println()
	fmt.Println("1. eyJ0b2tlbiI6Im1vY2tfdG9rZW4xIn0...")
	fmt.Println("   Status: ✅ Valid")
	fmt.Println("   Scopes: ai:chat, ai:tool.list")
	fmt.Println("   Expires: 2024-01-16T02:30:00Z")
	fmt.Println("   Issuer: devctl")
	fmt.Println()
	fmt.Println("2. eyJ0b2tlbiI6Im1vY2tfdG9rZW4yIn0...")
	fmt.Println("   Status: ✅ Valid")
	fmt.Println("   Scopes: ai:tool.open_file, ai:tool.search_files")
	fmt.Println("   Expires: 2024-01-15T20:30:00Z")
	fmt.Println("   Issuer: devctl")
	fmt.Println()

	// Example 17: Revoke CapToken
	fmt.Println("🗑️  Example 17: Revoke CapToken")
	fmt.Println("-------------------------------")
	fmt.Println("Command: devctl ai captoken revoke eyJ0b2tlbiI6ImV4cGlyZWRfdG9rZW4ifQ...")
	fmt.Println("Output:")
	fmt.Println("🔄 Revoking CapToken: eyJ0b2tlbiI6ImV4cGlyZWRfdG9rZW4ifQ...")
	fmt.Println("✅ CapToken revoked successfully: eyJ0b2tlbiI6ImV4cGlyZWRfdG9rZW4ifQ...")
	fmt.Println()

	// Example 18: Integration Example
	fmt.Println("🔗 Example 18: Integration Example")
	fmt.Println("----------------------------------")
	fmt.Println("Step 1: Generate CapToken")
	fmt.Println("Command: devctl ai captoken generate --scope \"ai:chat,ai:tool.open_file\" --save token.json")
	fmt.Println()
	fmt.Println("Step 2: Use CapToken in Chat")
	fmt.Println("Command: devctl ai chat \"Open the README file and summarize it\" --cap-token $(cat token.json)")
	fmt.Println("Output:")
	fmt.Println("AI: I'll help you open and summarize the README file.")
	fmt.Println()
	fmt.Println("Step 3: Execute Tool with CapToken")
	fmt.Println("Command: devctl ai toolcall execute open_file --param-path README.md --param-mode read --cap-token $(cat token.json)")
	fmt.Println("Output:")
	fmt.Println("✅ Tool call successful: open_file")
	fmt.Println("File opened successfully: README.md")
	fmt.Println("Content: [README file content]")
	fmt.Println()
	fmt.Println("Step 4: Monitor Performance")
	fmt.Println("Command: devctl ai metrics show --format json")
	fmt.Println("Output:")
	fmt.Println("{")
	fmt.Println("  \"requests_total\": 1250,")
	fmt.Println("  \"requests_success\": 1180,")
	fmt.Println("  \"latency_p95_ms\": 125.8,")
	fmt.Println("  \"tokens_per_second\": 12.5,")
	fmt.Println("  \"memory_usage_mb\": 1024.8")
	fmt.Println("}")
	fmt.Println()

	fmt.Println("🎉 All examples completed!")
	fmt.Println("These examples demonstrate the comprehensive capabilities of the AI Core Service CLI.")
}

// ExampleUsageScenarios demonstrates real-world usage scenarios
func ExampleUsageScenarios() {
	fmt.Println("🌍 Real-World Usage Scenarios")
	fmt.Println("=============================")
	fmt.Println()

	// Scenario 1: Development Workflow
	fmt.Println("💻 Scenario 1: Development Workflow")
	fmt.Println("-----------------------------------")
	fmt.Println("1. Start interactive chat session:")
	fmt.Println("   devctl ai chat interactive --model gpt-4")
	fmt.Println()
	fmt.Println("2. Ask for code review:")
	fmt.Println("   You: Review this Go function for best practices")
	fmt.Println("   AI: [Provides detailed code review]")
	fmt.Println()
	fmt.Println("3. Generate test cases:")
	fmt.Println("   You: Generate unit tests for this function")
	fmt.Println("   AI: [Generates comprehensive test cases]")
	fmt.Println()
	fmt.Println("4. Check metrics:")
	fmt.Println("   devctl ai metrics show --type requests")
	fmt.Println()

	// Scenario 2: Documentation Generation
	fmt.Println("📚 Scenario 2: Documentation Generation")
	fmt.Println("---------------------------------------")
	fmt.Println("1. Generate API documentation:")
	fmt.Println("   devctl ai chat \"Generate API documentation for the user service\" --format json")
	fmt.Println()
	fmt.Println("2. Create README:")
	fmt.Println("   devctl ai chat file project_description.txt --model gpt-4 --max-tokens 2000")
	fmt.Println()
	fmt.Println("3. Generate code comments:")
	fmt.Println("   devctl ai toolcall execute add_comments --param-file src/main.go --param-style detailed")
	fmt.Println()

	// Scenario 3: System Monitoring
	fmt.Println("📊 Scenario 3: System Monitoring")
	fmt.Println("--------------------------------")
	fmt.Println("1. Set up monitoring alerts:")
	fmt.Println("   devctl ai metrics alert --alert-latency-p95 200 --alert-error-rate 5")
	fmt.Println()
	fmt.Println("2. Monitor real-time performance:")
	fmt.Println("   devctl ai metrics realtime --update-interval 5s")
	fmt.Println()
	fmt.Println("3. Generate performance report:")
	fmt.Println("   devctl ai metrics history --time-range 24h --format json > daily_report.json")
	fmt.Println()
	fmt.Println("4. Compare with baseline:")
	fmt.Println("   devctl ai metrics compare baseline.json --format table")
	fmt.Println()

	// Scenario 4: Security and Access Control
	fmt.Println("🔒 Scenario 4: Security and Access Control")
	fmt.Println("------------------------------------------")
	fmt.Println("1. Generate limited access token:")
	fmt.Println("   devctl ai captoken generate --scope \"ai:chat\" --expires-at \"2024-01-15T18:00:00Z\"")
	fmt.Println()
	fmt.Println("2. Validate token before use:")
	fmt.Println("   devctl ai captoken validate $TOKEN")
	fmt.Println()
	fmt.Println("3. Use token for secure operations:")
	fmt.Println("   devctl ai chat \"Analyze this code for security issues\" --cap-token $TOKEN")
	fmt.Println()
	fmt.Println("4. Revoke token when done:")
	fmt.Println("   devctl ai captoken revoke $TOKEN")
	fmt.Println()

	// Scenario 5: Automated Workflows
	fmt.Println("🤖 Scenario 5: Automated Workflows")
	fmt.Println("----------------------------------")
	fmt.Println("1. Create automation script:")
	fmt.Println("   #!/bin/bash")
	fmt.Println("   TOKEN=$(devctl ai captoken generate --scope \"ai:tool.*\" --format json | jq -r '.token')")
	fmt.Println("   devctl ai toolcall execute search_files --param-query \"*.go\" --cap-token $TOKEN")
	fmt.Println("   devctl ai chat \"Review the found Go files for issues\" --cap-token $TOKEN")
	fmt.Println("   devctl ai captoken revoke $TOKEN")
	fmt.Println()
	fmt.Println("2. CI/CD integration:")
	fmt.Println("   devctl ai metrics compare baseline.json --format json | jq '.overall_status'")
	fmt.Println("   if [ \"$status\" != \"good\" ]; then")
	fmt.Println("     echo \"Performance regression detected!\"")
	fmt.Println("     exit 1")
	fmt.Println("   fi")
	fmt.Println()

	fmt.Println("🎯 These scenarios show how the AI Core Service CLI integrates into real-world workflows.")
}

// ExampleErrorHandling demonstrates error handling and troubleshooting
func ExampleErrorHandling() {
	fmt.Println("⚠️  Error Handling and Troubleshooting")
	fmt.Println("======================================")
	fmt.Println()

	// Common Errors
	fmt.Println("🚨 Common Errors and Solutions")
	fmt.Println("------------------------------")
	fmt.Println()

	fmt.Println("1. Connection Error:")
	fmt.Println("   Error: failed to connect to AI Core Service")
	fmt.Println("   Solution: Check if AI Core Service is running")
	fmt.Println("   Command: ps aux | grep ai_core")
	fmt.Println()

	fmt.Println("2. CapToken Expired:")
	fmt.Println("   Error: CapToken has expired")
	fmt.Println("   Solution: Generate a new token")
	fmt.Println("   Command: devctl ai captoken generate --scope \"ai:chat\"")
	fmt.Println()

	fmt.Println("3. Insufficient Permissions:")
	fmt.Println("   Error: CapToken does not have required scope")
	fmt.Println("   Solution: Generate token with correct scopes")
	fmt.Println("   Command: devctl ai captoken generate --scope \"ai:tool.open_file\"")
	fmt.Println()

	fmt.Println("4. Tool Not Found:")
	fmt.Println("   Error: Tool 'unknown_tool' not found")
	fmt.Println("   Solution: List available tools")
	fmt.Println("   Command: devctl ai toolcall list")
	fmt.Println()

	fmt.Println("5. Invalid Parameters:")
	fmt.Println("   Error: Missing required parameter 'path'")
	fmt.Println("   Solution: Check tool description for required parameters")
	fmt.Println("   Command: devctl ai toolcall describe open_file")
	fmt.Println()

	// Debugging Commands
	fmt.Println("🔍 Debugging Commands")
	fmt.Println("---------------------")
	fmt.Println()

	fmt.Println("1. Check AI Core Service status:")
	fmt.Println("   devctl ai metrics show --type system")
	fmt.Println()

	fmt.Println("2. Validate CapToken:")
	fmt.Println("   devctl ai captoken validate $TOKEN")
	fmt.Println()

	fmt.Println("3. Test basic connectivity:")
	fmt.Println("   devctl ai chat \"Hello\" --timeout 5s")
	fmt.Println()

	fmt.Println("4. Check available tools:")
	fmt.Println("   devctl ai toolcall list --format json")
	fmt.Println()

	fmt.Println("5. Monitor real-time metrics:")
	fmt.Println("   devctl ai metrics realtime --update-interval 1s")
	fmt.Println()

	// Best Practices
	fmt.Println("✅ Best Practices")
	fmt.Println("-----------------")
	fmt.Println()

	fmt.Println("1. Always use CapTokens for production:")
	fmt.Println("   devctl ai captoken generate --scope \"ai:chat\" --expires-at \"2024-01-16T00:00:00Z\"")
	fmt.Println()

	fmt.Println("2. Set appropriate timeouts:")
	fmt.Println("   devctl ai chat \"Long request\" --timeout 60s")
	fmt.Println()

	fmt.Println("3. Use JSON format for automation:")
	fmt.Println("   devctl ai metrics show --format json | jq '.requests_total'")
	fmt.Println()

	fmt.Println("4. Monitor performance regularly:")
	fmt.Println("   devctl ai metrics compare baseline.json")
	fmt.Println()

	fmt.Println("5. Revoke unused tokens:")
	fmt.Println("   devctl ai captoken revoke $TOKEN")
	fmt.Println()

	fmt.Println("🛠️  These practices help ensure reliable and secure usage of the AI Core Service CLI.")
}

// Main function to run examples
func main() {
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "examples":
			ExampleAICommands()
		case "scenarios":
			ExampleUsageScenarios()
		case "errors":
			ExampleErrorHandling()
		default:
			fmt.Println("Usage: go run ai_example.go [examples|scenarios|errors]")
		}
	} else {
		ExampleAICommands()
		fmt.Println()
		ExampleUsageScenarios()
		fmt.Println()
		ExampleErrorHandling()
	}
}
