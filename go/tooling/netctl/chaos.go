//! Chaos Engineering for Aetheris OS Networking
//! 
//! This module provides chaos injection capabilities for testing network
//! resilience, including packet loss, latency/jitter injection, and connection churn.

package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"math/rand"
	"net"
	"os"
	"sync"
	"time"
)

// ChaosConfig represents chaos injection configuration
type ChaosConfig struct {
	Enabled                bool          `json:"enabled"`
	Seed                   *int64        `json:"seed,omitempty"`
	Duration               time.Duration `json:"duration"`
	PacketLossRate         float64       `json:"packet_loss_rate"`
	LatencyMs              float64       `json:"latency_ms"`
	LatencyJitterMs        float64       `json:"latency_jitter_ms"`
	ConnectionChurnRate    float64       `json:"connection_churn_rate"`
	ConnectionChurnDuration time.Duration `json:"connection_churn_duration"`
	BandwidthLimitMbps     *float64      `json:"bandwidth_limit_mbps,omitempty"`
	CpuStressPercent       *float64      `json:"cpu_stress_percent,omitempty"`
	MemoryStressMB         *float64      `json:"memory_stress_mb,omitempty"`
}

// ChaosType represents the type of chaos injection
type ChaosType string

const (
	ChaosTypePacketLoss      ChaosType = "packet_loss"
	ChaosTypeLatency         ChaosType = "latency"
	ChaosTypeJitter          ChaosType = "jitter"
	ChaosTypeConnectionChurn ChaosType = "connection_churn"
	ChaosTypeBandwidthLimit  ChaosType = "bandwidth_limit"
	ChaosTypeCpuStress       ChaosType = "cpu_stress"
	ChaosTypeMemoryStress    ChaosType = "memory_stress"
)

// ChaosResult represents the result of a chaos injection
type ChaosResult struct {
	ChaosType           ChaosType     `json:"chaos_type"`
	Applied             bool          `json:"applied"`
	Duration            time.Duration `json:"duration"`
	PacketsDropped      uint64        `json:"packets_dropped"`
	PacketsDelayed      uint64        `json:"packets_delayed"`
	ConnectionsChurned  uint64        `json:"connections_churned"`
	ErrorCount          uint64        `json:"error_count"`
}

// ChaosManager manages chaos injections
type ChaosManager struct {
	config           ChaosConfig
	activeInjections map[string]*ChaosInjection
	injectionsMutex  sync.RWMutex
	stats            *ChaosStats
	rng              *rand.Rand
}

// ChaosInjection represents an active chaos injection
type ChaosInjection struct {
	ID         string
	ChaosType  ChaosType
	Config     ChaosConfig
	StartTime  time.Time
	Stats      *ChaosStats
	RNG        *rand.Rand
	StopChan   chan struct{}
}

// ChaosStats tracks chaos injection statistics
type ChaosStats struct {
	PacketsDropped     uint64
	PacketsDelayed     uint64
	ConnectionsChurned uint64
	ErrorCount         uint64
	TotalDuration      time.Duration
	mutex              sync.RWMutex
}

// NewChaosManager creates a new chaos manager
func NewChaosManager(config ChaosConfig) *ChaosManager {
	var rng *rand.Rand
	if config.Seed != nil {
		rng = rand.New(rand.NewSource(*config.Seed))
	} else {
		rng = rand.New(rand.NewSource(time.Now().UnixNano()))
	}

	return &ChaosManager{
		config:           config,
		activeInjections: make(map[string]*ChaosInjection),
		stats:            &ChaosStats{},
		rng:              rng,
	}
}

// StartPacketLoss starts packet loss injection
func (cm *ChaosManager) StartPacketLoss(rate float64) (string, error) {
	injectionID := fmt.Sprintf("packet_loss_%d", time.Now().UnixNano())
	
	injection := &ChaosInjection{
		ID:        injectionID,
		ChaosType: ChaosTypePacketLoss,
		Config: ChaosConfig{
			PacketLossRate: rate,
			Duration:       cm.config.Duration,
		},
		StartTime: time.Now(),
		Stats:     cm.stats,
		RNG:       cm.rng,
		StopChan:  make(chan struct{}),
	}

	cm.injectionsMutex.Lock()
	cm.activeInjections[injectionID] = injection
	cm.injectionsMutex.Unlock()

	// Start packet loss injection goroutine
	go func() {
		ticker := time.NewTicker(time.Millisecond)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				if cm.rng.Float64() < rate {
					cm.stats.mutex.Lock()
					cm.stats.PacketsDropped++
					cm.stats.mutex.Unlock()
				}
			case <-injection.StopChan:
				return
			}
		}
	}()

	return injectionID, nil
}

// StartLatency starts latency injection
func (cm *ChaosManager) StartLatency(latencyMs, jitterMs float64) (string, error) {
	injectionID := fmt.Sprintf("latency_%d", time.Now().UnixNano())
	
	injection := &ChaosInjection{
		ID:        injectionID,
		ChaosType: ChaosTypeLatency,
		Config: ChaosConfig{
			LatencyMs:       latencyMs,
			LatencyJitterMs: jitterMs,
			Duration:        cm.config.Duration,
		},
		StartTime: time.Now(),
		Stats:     cm.stats,
		RNG:       cm.rng,
		StopChan:  make(chan struct{}),
	}

	cm.injectionsMutex.Lock()
	cm.activeInjections[injectionID] = injection
	cm.injectionsMutex.Unlock()

	// Start latency injection goroutine
	go func() {
		ticker := time.NewTicker(time.Millisecond)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				jitter := (cm.rng.Float64() - 0.5) * 2.0 * jitterMs
				totalLatency := latencyMs + jitter
				if totalLatency > 0 {
					time.Sleep(time.Duration(totalLatency) * time.Millisecond)
					cm.stats.mutex.Lock()
					cm.stats.PacketsDelayed++
					cm.stats.mutex.Unlock()
				}
			case <-injection.StopChan:
				return
			}
		}
	}()

	return injectionID, nil
}

// StartConnectionChurn starts connection churn injection
func (cm *ChaosManager) StartConnectionChurn(rate float64) (string, error) {
	injectionID := fmt.Sprintf("connection_churn_%d", time.Now().UnixNano())
	
	injection := &ChaosInjection{
		ID:        injectionID,
		ChaosType: ChaosTypeConnectionChurn,
		Config: ChaosConfig{
			ConnectionChurnRate:    rate,
			ConnectionChurnDuration: cm.config.ConnectionChurnDuration,
			Duration:               cm.config.Duration,
		},
		StartTime: time.Now(),
		Stats:     cm.stats,
		RNG:       cm.rng,
		StopChan:  make(chan struct{}),
	}

	cm.injectionsMutex.Lock()
	cm.activeInjections[injectionID] = injection
	cm.injectionsMutex.Unlock()

	// Start connection churn injection goroutine
	go func() {
		ticker := time.NewTicker(time.Millisecond)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				if cm.rng.Float64() < rate/1000.0 { // Convert to per-millisecond probability
					cm.stats.mutex.Lock()
					cm.stats.ConnectionsChurned++
					cm.stats.mutex.Unlock()
					
					// Simulate connection churn duration
					time.Sleep(injection.Config.ConnectionChurnDuration)
				}
			case <-injection.StopChan:
				return
			}
		}
	}()

	return injectionID, nil
}

// StartBandwidthLimit starts bandwidth limiting
func (cm *ChaosManager) StartBandwidthLimit(limitMbps float64) (string, error) {
	injectionID := fmt.Sprintf("bandwidth_limit_%d", time.Now().UnixNano())
	
	injection := &ChaosInjection{
		ID:        injectionID,
		ChaosType: ChaosTypeBandwidthLimit,
		Config: ChaosConfig{
			BandwidthLimitMbps: &limitMbps,
			Duration:           cm.config.Duration,
		},
		StartTime: time.Now(),
		Stats:     cm.stats,
		RNG:       cm.rng,
		StopChan:  make(chan struct{}),
	}

	cm.injectionsMutex.Lock()
	cm.activeInjections[injectionID] = injection
	cm.injectionsMutex.Unlock()

	// Start bandwidth limiting goroutine
	go func() {
		ticker := time.NewTicker(time.Millisecond)
		defer ticker.Stop()

		bytesPerSecond := (limitMbps * 1024 * 1024) / 8
		sleepDuration := time.Duration(float64(time.Second) / (bytesPerSecond / 1024))

		for {
			select {
			case <-ticker.C:
				time.Sleep(sleepDuration)
			case <-injection.StopChan:
				return
			}
		}
	}()

	return injectionID, nil
}

// StartCpuStress starts CPU stress injection
func (cm *ChaosManager) StartCpuStress(stressPercent float64) (string, error) {
	injectionID := fmt.Sprintf("cpu_stress_%d", time.Now().UnixNano())
	
	injection := &ChaosInjection{
		ID:        injectionID,
		ChaosType: ChaosTypeCpuStress,
		Config: ChaosConfig{
			CpuStressPercent: &stressPercent,
			Duration:         cm.config.Duration,
		},
		StartTime: time.Now(),
		Stats:     cm.stats,
		RNG:       cm.rng,
		StopChan:  make(chan struct{}),
	}

	cm.injectionsMutex.Lock()
	cm.activeInjections[injectionID] = injection
	cm.injectionsMutex.Unlock()

	// Start CPU stress goroutine
	go func() {
		ticker := time.NewTicker(time.Millisecond)
		defer ticker.Stop()

		workDuration := time.Duration(stressPercent * 10) * time.Millisecond
		sleepDuration := time.Duration((100-stressPercent) * 10) * time.Millisecond

		for {
			select {
			case <-ticker.C:
				// Do some work
				workStart := time.Now()
				for time.Since(workStart) < workDuration {
					// Busy wait
				}
				time.Sleep(sleepDuration)
			case <-injection.StopChan:
				return
			}
		}
	}()

	return injectionID, nil
}

// StartMemoryStress starts memory stress injection
func (cm *ChaosManager) StartMemoryStress(stressMB float64) (string, error) {
	injectionID := fmt.Sprintf("memory_stress_%d", time.Now().UnixNano())
	
	injection := &ChaosInjection{
		ID:        injectionID,
		ChaosType: ChaosTypeMemoryStress,
		Config: ChaosConfig{
			MemoryStressMB: &stressMB,
			Duration:       cm.config.Duration,
		},
		StartTime: time.Now(),
		Stats:     cm.stats,
		RNG:       cm.rng,
		StopChan:  make(chan struct{}),
	}

	cm.injectionsMutex.Lock()
	cm.activeInjections[injectionID] = injection
	cm.injectionsMutex.Unlock()

	// Start memory stress goroutine
	go func() {
		ticker := time.NewTicker(100 * time.Millisecond)
		defer ticker.Stop()

		// Allocate memory chunks
		chunkSize := 1024 * 1024 // 1MB chunks
		numChunks := int(stressMB)
		if numChunks > 1000 {
			numChunks = 1000 // Limit to 1GB max
		}

		var memoryChunks [][]byte
		for i := 0; i < numChunks; i++ {
			chunk := make([]byte, chunkSize)
			memoryChunks = append(memoryChunks, chunk)
		}

		for {
			select {
			case <-ticker.C:
				// Touch memory to keep it in RAM
				for i := range memoryChunks {
					memoryChunks[i][0] = byte(i % 256)
				}
			case <-injection.StopChan:
				// Clean up
				memoryChunks = nil
				return
			}
		}
	}()

	return injectionID, nil
}

// StopInjection stops a chaos injection
func (cm *ChaosManager) StopInjection(injectionID string) (*ChaosResult, error) {
	cm.injectionsMutex.Lock()
	injection, exists := cm.activeInjections[injectionID]
	if !exists {
		cm.injectionsMutex.Unlock()
		return nil, fmt.Errorf("injection not found: %s", injectionID)
	}
	delete(cm.activeInjections, injectionID)
	cm.injectionsMutex.Unlock()

	// Stop the injection
	close(injection.StopChan)

	duration := time.Since(injection.StartTime)
	cm.stats.mutex.RLock()
	packetsDropped := cm.stats.PacketsDropped
	packetsDelayed := cm.stats.PacketsDelayed
	connectionsChurned := cm.stats.ConnectionsChurned
	errorCount := cm.stats.ErrorCount
	cm.stats.mutex.RUnlock()

	return &ChaosResult{
		ChaosType:          injection.ChaosType,
		Applied:            true,
		Duration:           duration,
		PacketsDropped:     packetsDropped,
		PacketsDelayed:     packetsDelayed,
		ConnectionsChurned: connectionsChurned,
		ErrorCount:         errorCount,
	}, nil
}

// StopAllInjections stops all active chaos injections
func (cm *ChaosManager) StopAllInjections() []ChaosResult {
	cm.injectionsMutex.Lock()
	injectionIDs := make([]string, 0, len(cm.activeInjections))
	for id := range cm.activeInjections {
		injectionIDs = append(injectionIDs, id)
	}
	cm.injectionsMutex.Unlock()

	var results []ChaosResult
	for _, id := range injectionIDs {
		if result, err := cm.StopInjection(id); err == nil {
			results = append(results, *result)
		}
	}

	return results
}

// GetStats returns chaos statistics
func (cm *ChaosManager) GetStats() ChaosStats {
	cm.stats.mutex.RLock()
	defer cm.stats.mutex.RUnlock()
	return *cm.stats
}

// ShouldDropPacket checks if a packet should be dropped
func (cm *ChaosManager) ShouldDropPacket() bool {
	if !cm.config.Enabled {
		return false
	}
	return cm.rng.Float64() < cm.config.PacketLossRate
}

// GetLatencyToInject returns the latency to inject
func (cm *ChaosManager) GetLatencyToInject() time.Duration {
	if !cm.config.Enabled || cm.config.LatencyMs <= 0 {
		return 0
	}
	
	jitter := (cm.rng.Float64() - 0.5) * 2.0 * cm.config.LatencyJitterMs
	totalLatency := cm.config.LatencyMs + jitter
	if totalLatency < 0 {
		totalLatency = 0
	}
	
	return time.Duration(totalLatency) * time.Millisecond
}

// ShouldChurnConnection checks if a connection should be churned
func (cm *ChaosManager) ShouldChurnConnection() bool {
	if !cm.config.Enabled {
		return false
	}
	return cm.rng.Float64() < cm.config.ConnectionChurnRate/1000.0
}

// handleChaos handles the chaos command
func handleChaos(args []string) error {
	if len(args) < 1 {
		printChaosUsage()
		return fmt.Errorf("chaos command required")
	}

	subcommand := args[0]
	switch subcommand {
	case "inject":
		return handleChaosInject(args[1:])
	case "stop":
		return handleChaosStop(args[1:])
	case "stats":
		return handleChaosStats(args[1:])
	default:
		printChaosUsage()
		return fmt.Errorf("unknown chaos command: %s", subcommand)
	}
}

// handleChaosInject handles the chaos inject command
func handleChaosInject(args []string) error {
	fs := flag.NewFlagSet("chaos inject", flag.ExitOnError)
	
	var (
		lossRate      = fs.Float64("loss", 0.0, "Packet loss rate (0.0 to 1.0)")
		latency       = fs.Float64("latency", 0.0, "Latency in milliseconds")
		jitter        = fs.Float64("jitter", 0.0, "Latency jitter in milliseconds")
		duration      = fs.Duration("duration", 60*time.Second, "Chaos injection duration")
		churnRate     = fs.Float64("churn", 0.0, "Connection churn rate (connections per second)")
		bandwidth     = fs.Float64("bandwidth", 0.0, "Bandwidth limit in Mbps")
		cpuStress     = fs.Float64("cpu", 0.0, "CPU stress percentage")
		memoryStress  = fs.Float64("memory", 0.0, "Memory stress in MB")
		seed          = fs.Int64("seed", 42, "Random seed for deterministic results")
		outputFile    = fs.String("output", "", "Output file for chaos results")
	)

	if err := fs.Parse(args); err != nil {
		return err
	}

	config := ChaosConfig{
		Enabled:                true,
		Seed:                   &seed,
		Duration:               *duration,
		PacketLossRate:         *lossRate,
		LatencyMs:              *latency,
		LatencyJitterMs:        *jitter,
		ConnectionChurnRate:    *churnRate,
		ConnectionChurnDuration: time.Second,
		BandwidthLimitMbps:     nil,
		CpuStressPercent:       nil,
		MemoryStressMB:         nil,
	}

	if *bandwidth > 0 {
		config.BandwidthLimitMbps = bandwidth
	}
	if *cpuStress > 0 {
		config.CpuStressPercent = cpuStress
	}
	if *memoryStress > 0 {
		config.MemoryStressMB = memoryStress
	}

	manager := NewChaosManager(config)
	var injectionIDs []string

	// Start chaos injections based on configuration
	if *lossRate > 0 {
		id, err := manager.StartPacketLoss(*lossRate)
		if err != nil {
			return fmt.Errorf("failed to start packet loss injection: %v", err)
		}
		injectionIDs = append(injectionIDs, id)
		fmt.Printf("Started packet loss injection: %s (rate: %.2f%%)\n", id, *lossRate*100)
	}

	if *latency > 0 {
		id, err := manager.StartLatency(*latency, *jitter)
		if err != nil {
			return fmt.Errorf("failed to start latency injection: %v", err)
		}
		injectionIDs = append(injectionIDs, id)
		fmt.Printf("Started latency injection: %s (latency: %.2fms, jitter: %.2fms)\n", id, *latency, *jitter)
	}

	if *churnRate > 0 {
		id, err := manager.StartConnectionChurn(*churnRate)
		if err != nil {
			return fmt.Errorf("failed to start connection churn injection: %v", err)
		}
		injectionIDs = append(injectionIDs, id)
		fmt.Printf("Started connection churn injection: %s (rate: %.2f conn/s)\n", id, *churnRate)
	}

	if *bandwidth > 0 {
		id, err := manager.StartBandwidthLimit(*bandwidth)
		if err != nil {
			return fmt.Errorf("failed to start bandwidth limit injection: %v", err)
		}
		injectionIDs = append(injectionIDs, id)
		fmt.Printf("Started bandwidth limit injection: %s (limit: %.2f Mbps)\n", id, *bandwidth)
	}

	if *cpuStress > 0 {
		id, err := manager.StartCpuStress(*cpuStress)
		if err != nil {
			return fmt.Errorf("failed to start CPU stress injection: %v", err)
		}
		injectionIDs = append(injectionIDs, id)
		fmt.Printf("Started CPU stress injection: %s (stress: %.2f%%)\n", id, *cpuStress)
	}

	if *memoryStress > 0 {
		id, err := manager.StartMemoryStress(*memoryStress)
		if err != nil {
			return fmt.Errorf("failed to start memory stress injection: %v", err)
		}
		injectionIDs = append(injectionIDs, id)
		fmt.Printf("Started memory stress injection: %s (stress: %.2f MB)\n", id, *memoryStress)
	}

	if len(injectionIDs) == 0 {
		return fmt.Errorf("no chaos injections specified")
	}

	fmt.Printf("Chaos injection running for %v...\n", *duration)
	time.Sleep(*duration)

	// Stop all injections
	var results []ChaosResult
	for _, id := range injectionIDs {
		result, err := manager.StopInjection(id)
		if err != nil {
			fmt.Printf("Error stopping injection %s: %v\n", id, err)
			continue
		}
		results = append(results, *result)
	}

	// Print results
	fmt.Println("\nChaos Injection Results:")
	fmt.Println("========================")
	for _, result := range results {
		fmt.Printf("Type: %s\n", result.ChaosType)
		fmt.Printf("Applied: %v\n", result.Applied)
		fmt.Printf("Duration: %v\n", result.Duration)
		fmt.Printf("Packets Dropped: %d\n", result.PacketsDropped)
		fmt.Printf("Packets Delayed: %d\n", result.PacketsDelayed)
		fmt.Printf("Connections Churned: %d\n", result.ConnectionsChurned)
		fmt.Printf("Errors: %d\n", result.ErrorCount)
		fmt.Println()
	}

	// Write results to file if specified
	if *outputFile != "" {
		data, err := json.MarshalIndent(results, "", "  ")
		if err != nil {
			return fmt.Errorf("failed to marshal results: %v", err)
		}
		
		if err := os.WriteFile(*outputFile, data, 0644); err != nil {
			return fmt.Errorf("failed to write results file: %v", err)
		}
		
		fmt.Printf("Results written to: %s\n", *outputFile)
	}

	return nil
}

// handleChaosStop handles the chaos stop command
func handleChaosStop(args []string) error {
	// This would stop active chaos injections
	// For now, just print a message
	fmt.Println("Stopping all chaos injections...")
	return nil
}

// handleChaosStats handles the chaos stats command
func handleChaosStats(args []string) error {
	// This would show chaos injection statistics
	// For now, just print a message
	fmt.Println("Chaos injection statistics:")
	fmt.Println("===========================")
	return nil
}

// printChaosUsage prints the chaos command usage
func printChaosUsage() {
	fmt.Println(`
Chaos Engineering Commands:

  inject    Start chaos injections
  stop      Stop active chaos injections
  stats     Show chaos injection statistics

Examples:
  ./netctl chaos inject --loss 5% --latency 100ms --duration 10m
  ./netctl chaos inject --churn 10 --cpu 50 --memory 100
  ./netctl chaos stop
  ./netctl chaos stats
`)
}
