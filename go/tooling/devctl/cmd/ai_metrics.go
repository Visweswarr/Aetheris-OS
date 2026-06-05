// Package devctl provides CLI commands for AI Core Service metrics functionality
//
// This module provides the devctl ai metrics command for viewing and managing
// AI Core Service telemetry and performance metrics, extending the existing
// metrics functionality with protobuf integration and real-time capabilities.

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

// AI Metrics commands (extending existing functionality)
var aiMetricsRealTimeCmd = &cobra.Command{
	Use:   "realtime",
	Short: "Show real-time metrics",
	Long:  "Display real-time metrics from the AI Core Service with live updates",
	RunE:  runAiMetricsRealTime,
}

var aiMetricsHistoryCmd = &cobra.Command{
	Use:   "history",
	Short: "Show metrics history",
	Long:  "Display historical metrics data from the AI Core Service",
	RunE:  runAiMetricsHistory,
}

var aiMetricsCompareCmd = &cobra.Command{
	Use:   "compare [baseline-file]",
	Short: "Compare metrics with baseline",
	Long:  "Compare current metrics with a baseline file to detect regressions",
	Args:  cobra.ExactArgs(1),
	RunE:  runAiMetricsCompare,
}

var aiMetricsAlertCmd = &cobra.Command{
	Use:   "alert",
	Short: "Configure metrics alerts",
	Long:  "Configure alerting thresholds for AI Core Service metrics",
	RunE:  runAiMetricsAlert,
}

// Metrics configuration
type MetricsConfig struct {
	CapToken      string
	SocketPath    string
	Timeout       time.Duration
	OutputFormat  string
	TimeRange     string
	MetricsType   string
	IncludeHistory bool
	UpdateInterval time.Duration
	BaselineFile   string
	AlertThresholds map[string]float64
}

// Metrics comparison result
type MetricsComparison struct {
	BaselineFile    string                 `json:"baseline_file"`
	CurrentMetrics  AiCoreMetrics          `json:"current_metrics"`
	BaselineMetrics AiCoreMetrics          `json:"baseline_metrics"`
	Differences     map[string]interface{} `json:"differences"`
	Regressions     []string               `json:"regressions"`
	Improvements    []string               `json:"improvements"`
	OverallStatus   string                 `json:"overall_status"`
}

// runAiMetricsRealTime executes the real-time metrics command
func runAiMetricsRealTime(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getMetricsConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get metrics config: %w", err)
	}

	// Get update interval
	updateInterval, _ := cmd.Flags().GetDuration("update-interval")
	if updateInterval == 0 {
		updateInterval = 5 * time.Second
	}

	fmt.Println("📊 AI Core Service Real-Time Metrics")
	fmt.Println("====================================")
	fmt.Printf("Update interval: %s\n", updateInterval)
	fmt.Println("Press Ctrl+C to stop")
	fmt.Println()

	// Start real-time monitoring
	ticker := time.NewTicker(updateInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			// Get current metrics
			metrics, err := getAiCoreMetricsRealTime(config)
			if err != nil {
				fmt.Printf("❌ Error getting metrics: %v\n", err)
				continue
			}

			// Clear screen and display metrics
			fmt.Print("\033[2J\033[H") // Clear screen and move cursor to top
			displayRealTimeMetrics(metrics)
		}
	}
}

// runAiMetricsHistory executes the metrics history command
func runAiMetricsHistory(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getMetricsConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get metrics config: %w", err)
	}

	// Get historical metrics
	history, err := getAiCoreMetricsHistory(config)
	if err != nil {
		return fmt.Errorf("failed to get metrics history: %w", err)
	}

	// Output history based on format
	switch config.OutputFormat {
	case "json":
		return outputMetricsHistoryJSON(history)
	case "table":
		return outputMetricsHistoryTable(history)
	case "chart":
		return outputMetricsHistoryChart(history)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiMetricsCompare executes the metrics comparison command
func runAiMetricsCompare(cmd *cobra.Command, args []string) error {
	baselineFile := args[0]
	
	// Get configuration from flags
	config, err := getMetricsConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get metrics config: %w", err)
	}

	config.BaselineFile = baselineFile

	// Perform comparison
	comparison, err := compareMetricsWithBaseline(config)
	if err != nil {
		return fmt.Errorf("failed to compare metrics: %w", err)
	}

	// Output comparison based on format
	switch config.OutputFormat {
	case "json":
		return outputMetricsComparisonJSON(comparison)
	case "table":
		return outputMetricsComparisonTable(comparison)
	default:
		return fmt.Errorf("unsupported output format: %s", config.OutputFormat)
	}
}

// runAiMetricsAlert executes the metrics alert configuration command
func runAiMetricsAlert(cmd *cobra.Command, args []string) error {
	// Get configuration from flags
	config, err := getMetricsConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get metrics config: %w", err)
	}

	// Get alert configuration
	alertConfig, err := getAlertConfig(cmd)
	if err != nil {
		return fmt.Errorf("failed to get alert config: %w", err)
	}

	// Configure alerts
	return configureMetricsAlerts(config, alertConfig)
}

// getMetricsConfig extracts configuration from command flags
func getMetricsConfig(cmd *cobra.Command) (MetricsConfig, error) {
	capToken, _ := cmd.Flags().GetString("cap-token")
	socketPath, _ := cmd.Flags().GetString("socket")
	timeout, _ := cmd.Flags().GetDuration("timeout")
	outputFormat, _ := cmd.Flags().GetString("format")
	timeRange, _ := cmd.Flags().GetString("time-range")
	metricsType, _ := cmd.Flags().GetString("type")
	includeHistory, _ := cmd.Flags().GetBool("include-history")

	// Set defaults
	if socketPath == "" {
		socketPath = "/tmp/ai_core.sock"
	}
	if timeout == 0 {
		timeout = 30 * time.Second
	}
	if outputFormat == "" {
		outputFormat = "table"
	}
	if timeRange == "" {
		timeRange = "1h"
	}
	if metricsType == "" {
		metricsType = "all"
	}

	return MetricsConfig{
		CapToken:       capToken,
		SocketPath:     socketPath,
		Timeout:        timeout,
		OutputFormat:   outputFormat,
		TimeRange:      timeRange,
		MetricsType:    metricsType,
		IncludeHistory: includeHistory,
	}, nil
}

// getAlertConfig extracts alert configuration from command flags
func getAlertConfig(cmd *cobra.Command) (map[string]float64, error) {
	alerts := make(map[string]float64)

	// Get alert thresholds from flags
	cmd.Flags().VisitAll(func(flag *cobra.Flag) {
		if strings.HasPrefix(flag.Name, "alert-") {
			metricName := strings.TrimPrefix(flag.Name, "alert-")
			if threshold, err := cmd.Flags().GetFloat64(flag.Name); err == nil {
				alerts[metricName] = threshold
			}
		}
	})

	return alerts, nil
}

// getAiCoreMetricsRealTime retrieves real-time metrics from AI Core Service
func getAiCoreMetricsRealTime(config MetricsConfig) (AiCoreMetrics, error) {
	// Create metrics request
	request := &ai_core.MetricsRequest{
		IncludeHistory: config.IncludeHistory,
		TimeRange:      config.TimeRange,
		MetricsType:    config.MetricsType,
		RealTime:       true,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		MessageType: &ai_core.AiCoreMessage_MetricsRequest{
			MetricsRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{"ai:metrics.read"},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC (simulated for now)
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	return simulateMetricsResponse(ctx, aiMessage, config)
}

// getAiCoreMetricsHistory retrieves historical metrics from AI Core Service
func getAiCoreMetricsHistory(config MetricsConfig) ([]AiCoreMetrics, error) {
	// Create metrics request
	request := &ai_core.MetricsRequest{
		IncludeHistory: true,
		TimeRange:      config.TimeRange,
		MetricsType:    config.MetricsType,
		RealTime:       false,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		MessageType: &ai_core.AiCoreMessage_MetricsRequest{
			MetricsRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{"ai:metrics.read"},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC (simulated for now)
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	return simulateMetricsHistoryResponse(ctx, aiMessage, config)
}

// compareMetricsWithBaseline compares current metrics with baseline
func compareMetricsWithBaseline(config MetricsConfig) (MetricsComparison, error) {
	// Load baseline metrics
	baselineData, err := os.ReadFile(config.BaselineFile)
	if err != nil {
		return MetricsComparison{}, fmt.Errorf("failed to read baseline file: %w", err)
	}

	var baselineMetrics AiCoreMetrics
	if err := json.Unmarshal(baselineData, &baselineMetrics); err != nil {
		return MetricsComparison{}, fmt.Errorf("failed to parse baseline metrics: %w", err)
	}

	// Get current metrics
	currentMetrics, err := getAiCoreMetricsRealTime(config)
	if err != nil {
		return MetricsComparison{}, fmt.Errorf("failed to get current metrics: %w", err)
	}

	// Perform comparison
	comparison := MetricsComparison{
		BaselineFile:    config.BaselineFile,
		CurrentMetrics:  currentMetrics,
		BaselineMetrics: baselineMetrics,
		Differences:     make(map[string]interface{}),
		Regressions:     []string{},
		Improvements:    []string{},
		OverallStatus:   "unknown",
	}

	// Compare key metrics
	compareMetric("RequestsTotal", currentMetrics.RequestsTotal, baselineMetrics.RequestsTotal, &comparison)
	compareMetric("LatencyP95Ms", currentMetrics.LatencyP95Ms, baselineMetrics.LatencyP95Ms, &comparison)
	compareMetric("TokensPerSecond", currentMetrics.TokensPerSecond, baselineMetrics.TokensPerSecond, &comparison)
	compareMetric("MemoryUsageMB", currentMetrics.MemoryUsageMB, baselineMetrics.MemoryUsageMB, &comparison)
	compareMetric("CPUUsagePercent", currentMetrics.CPUUsagePercent, baselineMetrics.CPUUsagePercent, &comparison)

	// Determine overall status
	if len(comparison.Regressions) == 0 {
		comparison.OverallStatus = "good"
	} else if len(comparison.Regressions) <= 2 {
		comparison.OverallStatus = "warning"
	} else {
		comparison.OverallStatus = "critical"
	}

	return comparison, nil
}

// compareMetric compares a single metric value
func compareMetric(name string, current, baseline uint64, comparison *MetricsComparison) {
	if current > baseline {
		diff := float64(current-baseline) / float64(baseline) * 100
		comparison.Differences[name] = map[string]interface{}{
			"current":  current,
			"baseline": baseline,
			"change":   fmt.Sprintf("+%.1f%%", diff),
			"type":     "increase",
		}
		if diff > 10 { // 10% threshold for regression
			comparison.Regressions = append(comparison.Regressions, name)
		}
	} else if current < baseline {
		diff := float64(baseline-current) / float64(baseline) * 100
		comparison.Differences[name] = map[string]interface{}{
			"current":  current,
			"baseline": baseline,
			"change":   fmt.Sprintf("-%.1f%%", diff),
			"type":     "decrease",
		}
		if diff > 5 { // 5% threshold for improvement
			comparison.Improvements = append(comparison.Improvements, name)
		}
	}
}

// configureMetricsAlerts configures metrics alerts
func configureMetricsAlerts(config MetricsConfig, alertConfig map[string]float64) error {
	// Create alert configuration request
	request := &ai_core.AlertConfigRequest{
		Thresholds: alertConfig,
		Enabled:    true,
	}

	// Create AI Core message
	aiMessage := &ai_core.AiCoreMessage{
		MessageId: generateMessageID(),
		Timestamp: time.Now().Unix(),
		MessageType: &ai_core.AiCoreMessage_AlertConfigRequest{
			AlertConfigRequest: request,
		},
	}

	// Add CapToken if provided
	if config.CapToken != "" {
		capToken := &ai_core.CapToken{
			Token:     config.CapToken,
			Scope:     []string{"ai:metrics.configure"},
			ExpiresAt: time.Now().Add(24 * time.Hour).Unix(),
		}
		aiMessage.CapToken = capToken
	}

	// Send request via IPC (simulated for now)
	ctx, cancel := context.WithTimeout(context.Background(), config.Timeout)
	defer cancel()

	return simulateAlertConfigResponse(ctx, aiMessage, config)
}

// simulateMetricsResponse simulates a metrics response
func simulateMetricsResponse(ctx context.Context, message *ai_core.AiCoreMessage, config MetricsConfig) (AiCoreMetrics, error) {
	// Simulate processing delay
	time.Sleep(50 * time.Millisecond)

	// Generate mock metrics with some variation
	baseTime := time.Now().Unix()
	variation := float64(baseTime % 100) / 100.0 // 0-1 variation

	return AiCoreMetrics{
		RequestsTotal:   1250 + uint64(variation*100),
		RequestsSuccess: 1180 + uint64(variation*50),
		RequestsError:   70 + uint64(variation*20),
		
		LatencyP50Ms:    45.2 + variation*10,
		LatencyP95Ms:    125.8 + variation*20,
		LatencyP99Ms:    250.3 + variation*30,
		LatencyMaxMs:    500.1 + variation*50,
		
		TokensPerSecond: 12.5 + variation*2,
		TokensTotal:     15680 + uint64(variation*1000),
		TokensInput:     8920 + uint64(variation*500),
		TokensOutput:    6760 + uint64(variation*500),
		
		ToolCallsTotal:   340 + uint64(variation*50),
		ToolCallsSuccess: 315 + uint64(variation*30),
		ToolCallsError:   25 + uint64(variation*20),
		ToolErrorsByType: map[string]uint64{
			"open_file":     8 + uint64(variation*5),
			"search_files":  12 + uint64(variation*3),
			"create_note":   3 + uint64(variation*2),
			"unknown_tool":  2 + uint64(variation*1),
		},
		
		ModelLoadsTotal:      15 + uint64(variation*5),
		ModelInferencesTotal: 1180 + uint64(variation*100),
		ModelMemoryUsageMB:   2048.5 + variation*200,
		
		CPUUsagePercent: 23.4 + variation*10,
		MemoryUsageMB:   1024.8 + variation*100,
		DiskUsageMB:     5120.2 + variation*500,
		
		ActiveSessions:         3 + uint64(variation*2),
		SessionsTotal:          45 + uint64(variation*10),
		SessionDurationAvgSecs: 180.5 + variation*30,
		
		Timestamp: uint64(baseTime),
	}, nil
}

// simulateMetricsHistoryResponse simulates a metrics history response
func simulateMetricsHistoryResponse(ctx context.Context, message *ai_core.AiCoreMessage, config MetricsConfig) ([]AiCoreMetrics, error) {
	// Generate mock historical data
	var history []AiCoreMetrics
	now := time.Now()
	
	for i := 0; i < 10; i++ {
		timestamp := now.Add(-time.Duration(i) * 5 * time.Minute)
		variation := float64(i) / 10.0
		
		metrics := AiCoreMetrics{
			RequestsTotal:   1250 + uint64(variation*100),
			RequestsSuccess: 1180 + uint64(variation*50),
			RequestsError:   70 + uint64(variation*20),
			
			LatencyP50Ms:    45.2 + variation*10,
			LatencyP95Ms:    125.8 + variation*20,
			LatencyP99Ms:    250.3 + variation*30,
			LatencyMaxMs:    500.1 + variation*50,
			
			TokensPerSecond: 12.5 + variation*2,
			TokensTotal:     15680 + uint64(variation*1000),
			TokensInput:     8920 + uint64(variation*500),
			TokensOutput:    6760 + uint64(variation*500),
			
			ToolCallsTotal:   340 + uint64(variation*50),
			ToolCallsSuccess: 315 + uint64(variation*30),
			ToolCallsError:   25 + uint64(variation*20),
			ToolErrorsByType: map[string]uint64{
				"open_file":     8 + uint64(variation*5),
				"search_files":  12 + uint64(variation*3),
				"create_note":   3 + uint64(variation*2),
				"unknown_tool":  2 + uint64(variation*1),
			},
			
			ModelLoadsTotal:      15 + uint64(variation*5),
			ModelInferencesTotal: 1180 + uint64(variation*100),
			ModelMemoryUsageMB:   2048.5 + variation*200,
			
			CPUUsagePercent: 23.4 + variation*10,
			MemoryUsageMB:   1024.8 + variation*100,
			DiskUsageMB:     5120.2 + variation*500,
			
			ActiveSessions:         3 + uint64(variation*2),
			SessionsTotal:          45 + uint64(variation*10),
			SessionDurationAvgSecs: 180.5 + variation*30,
			
			Timestamp: uint64(timestamp.Unix()),
		}
		
		history = append(history, metrics)
	}
	
	return history, nil
}

// simulateAlertConfigResponse simulates an alert configuration response
func simulateAlertConfigResponse(ctx context.Context, message *ai_core.AiCoreMessage, config MetricsConfig) error {
	// Simulate processing delay
	time.Sleep(50 * time.Millisecond)

	fmt.Println("🔔 Metrics Alerts Configuration")
	fmt.Println("===============================")
	
	thresholds := message.GetAlertConfigRequest().Thresholds
	if len(thresholds) == 0 {
		fmt.Println("No alert thresholds configured")
		return nil
	}
	
	fmt.Println("Configured alert thresholds:")
	for metric, threshold := range thresholds {
		fmt.Printf("  %s: %.2f\n", metric, threshold)
	}
	
	fmt.Println("✅ Alert configuration updated successfully")
	return nil
}

// displayRealTimeMetrics displays real-time metrics
func displayRealTimeMetrics(metrics AiCoreMetrics) {
	fmt.Printf("📊 AI Core Service Real-Time Metrics - %s\n", time.Now().Format("15:04:05"))
	fmt.Println("==================================================")
	
	// Request metrics
	fmt.Printf("📈 Requests: %d total, %d success (%.1f%%), %d errors (%.1f%%)\n",
		metrics.RequestsTotal,
		metrics.RequestsSuccess,
		float64(metrics.RequestsSuccess)/float64(metrics.RequestsTotal)*100,
		metrics.RequestsError,
		float64(metrics.RequestsError)/float64(metrics.RequestsTotal)*100)
	
	// Latency metrics
	fmt.Printf("⏱️  Latency: P50=%.1fms, P95=%.1fms, P99=%.1fms, Max=%.1fms\n",
		metrics.LatencyP50Ms, metrics.LatencyP95Ms, metrics.LatencyP99Ms, metrics.LatencyMaxMs)
	
	// Token metrics
	fmt.Printf("🔤 Tokens: %.1f/sec, %d total (%d in, %d out)\n",
		metrics.TokensPerSecond, metrics.TokensTotal, metrics.TokensInput, metrics.TokensOutput)
	
	// Tool metrics
	fmt.Printf("🔧 Tools: %d calls, %d success (%.1f%%), %d errors (%.1f%%)\n",
		metrics.ToolCallsTotal,
		metrics.ToolCallsSuccess,
		float64(metrics.ToolCallsSuccess)/float64(metrics.ToolCallsTotal)*100,
		metrics.ToolCallsError,
		float64(metrics.ToolCallsError)/float64(metrics.ToolCallsTotal)*100)
	
	// System metrics
	fmt.Printf("💻 System: CPU=%.1f%%, Memory=%.1fMB, Disk=%.1fMB\n",
		metrics.CPUUsagePercent, metrics.MemoryUsageMB, metrics.DiskUsageMB)
	
	// Session metrics
	fmt.Printf("👥 Sessions: %d active, %d total, %.1fs avg duration\n",
		metrics.ActiveSessions, metrics.SessionsTotal, metrics.SessionDurationAvgSecs)
}

// outputMetricsHistoryJSON outputs metrics history in JSON format
func outputMetricsHistoryJSON(history []AiCoreMetrics) error {
	jsonData, err := json.MarshalIndent(history, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal history: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputMetricsHistoryTable outputs metrics history in table format
func outputMetricsHistoryTable(history []AiCoreMetrics) error {
	fmt.Println("📊 AI Core Service Metrics History")
	fmt.Println("==================================")
	fmt.Println()
	
	for i, metrics := range history {
		timestamp := time.Unix(int64(metrics.Timestamp), 0)
		fmt.Printf("📅 %s\n", timestamp.Format("2006-01-02 15:04:05"))
		fmt.Printf("  Requests: %d total, %d success, %d errors\n", metrics.RequestsTotal, metrics.RequestsSuccess, metrics.RequestsError)
		fmt.Printf("  Latency: P95=%.1fms, P99=%.1fms\n", metrics.LatencyP95Ms, metrics.LatencyP99Ms)
		fmt.Printf("  Tokens: %.1f/sec, %d total\n", metrics.TokensPerSecond, metrics.TokensTotal)
		fmt.Printf("  System: CPU=%.1f%%, Memory=%.1fMB\n", metrics.CPUUsagePercent, metrics.MemoryUsageMB)
		if i < len(history)-1 {
			fmt.Println()
		}
	}
	
	return nil
}

// outputMetricsHistoryChart outputs metrics history in chart format
func outputMetricsHistoryChart(history []AiCoreMetrics) error {
	fmt.Println("📊 AI Core Service Metrics Chart")
	fmt.Println("================================")
	fmt.Println()
	
	// Simple ASCII chart for latency
	fmt.Println("Latency P95 (ms):")
	maxLatency := 0.0
	for _, metrics := range history {
		if metrics.LatencyP95Ms > maxLatency {
			maxLatency = metrics.LatencyP95Ms
		}
	}
	
	for _, metrics := range history {
		barLength := int(metrics.LatencyP95Ms / maxLatency * 50)
		bar := strings.Repeat("█", barLength)
		fmt.Printf("%.1fms %s\n", metrics.LatencyP95Ms, bar)
	}
	
	return nil
}

// outputMetricsComparisonJSON outputs metrics comparison in JSON format
func outputMetricsComparisonJSON(comparison MetricsComparison) error {
	jsonData, err := json.MarshalIndent(comparison, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal comparison: %w", err)
	}
	fmt.Println(string(jsonData))
	return nil
}

// outputMetricsComparisonTable outputs metrics comparison in table format
func outputMetricsComparisonTable(comparison MetricsComparison) error {
	fmt.Println("📊 AI Core Service Metrics Comparison")
	fmt.Println("=====================================")
	fmt.Printf("Baseline: %s\n", comparison.BaselineFile)
	fmt.Printf("Status: %s\n", strings.ToUpper(comparison.OverallStatus))
	fmt.Println()
	
	fmt.Println("Metric Differences:")
	fmt.Println("------------------")
	for metric, diff := range comparison.Differences {
		diffMap := diff.(map[string]interface{})
		fmt.Printf("%-20s: %v -> %v (%s)\n",
			metric,
			diffMap["baseline"],
			diffMap["current"],
			diffMap["change"])
	}
	
	if len(comparison.Regressions) > 0 {
		fmt.Println()
		fmt.Println("⚠️  Regressions:")
		for _, regression := range comparison.Regressions {
			fmt.Printf("  - %s\n", regression)
		}
	}
	
	if len(comparison.Improvements) > 0 {
		fmt.Println()
		fmt.Println("✅ Improvements:")
		for _, improvement := range comparison.Improvements {
			fmt.Printf("  - %s\n", improvement)
		}
	}
	
	return nil
}

func init() {
	// Add metrics commands to existing AI metrics command
	aiMetricsCmd.AddCommand(aiMetricsRealTimeCmd)
	aiMetricsCmd.AddCommand(aiMetricsHistoryCmd)
	aiMetricsCmd.AddCommand(aiMetricsCompareCmd)
	aiMetricsCmd.AddCommand(aiMetricsAlertCmd)

	// Real-time command flags
	aiMetricsRealTimeCmd.Flags().Duration("update-interval", 5*time.Second, "Update interval for real-time metrics")
	aiMetricsRealTimeCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiMetricsRealTimeCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiMetricsRealTimeCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")

	// History command flags
	aiMetricsHistoryCmd.Flags().String("format", "table", "Output format (table, json, chart)")
	aiMetricsHistoryCmd.Flags().String("time-range", "1h", "Time range for history (1h, 24h, 7d, 30d)")
	aiMetricsHistoryCmd.Flags().String("type", "all", "Metrics type filter (requests, latency, tokens, tools, system)")
	aiMetricsHistoryCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiMetricsHistoryCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiMetricsHistoryCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")

	// Compare command flags
	aiMetricsCompareCmd.Flags().String("format", "table", "Output format (table, json)")
	aiMetricsCompareCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiMetricsCompareCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiMetricsCompareCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")

	// Alert command flags
	aiMetricsAlertCmd.Flags().Float64("alert-latency-p95", 200.0, "Alert threshold for P95 latency (ms)")
	aiMetricsAlertCmd.Flags().Float64("alert-tokens-per-second", 10.0, "Alert threshold for minimum tokens per second")
	aiMetricsAlertCmd.Flags().Float64("alert-memory-usage", 4000.0, "Alert threshold for memory usage (MB)")
	aiMetricsAlertCmd.Flags().Float64("alert-cpu-usage", 80.0, "Alert threshold for CPU usage (%)")
	aiMetricsAlertCmd.Flags().Float64("alert-error-rate", 10.0, "Alert threshold for error rate (%)")
	aiMetricsAlertCmd.Flags().String("cap-token", "", "Capability token for authentication")
	aiMetricsAlertCmd.Flags().String("socket", "/tmp/ai_core.sock", "AI Core Service socket path")
	aiMetricsAlertCmd.Flags().Duration("timeout", 30*time.Second, "Request timeout")
}
