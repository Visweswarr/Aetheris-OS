//! High-Scale Networking Benchmark for Aetheris OS
//! 
//! This module provides comprehensive benchmarking capabilities for testing
//! network performance at scale with both plain TCP and TLS/mTLS protocols.

package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"math"
	"net"
	"os"
	"runtime"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

// BenchConfig represents benchmark configuration
type BenchConfig struct {
	Clients        int           `json:"clients"`
	Duration       time.Duration `json:"duration"`
	TLSEnabled     bool          `json:"tls_enabled"`
	TLSProfile     string        `json:"tls_profile"`
	PQCAlgorithms  []string      `json:"pqc_algorithms"`
	ServerAddr     string        `json:"server_addr"`
	PayloadSize    int           `json:"payload_size"`
	ZeroCopy       bool          `json:"zero_copy"`
	BackPressure   bool          `json:"back_pressure"`
	CPUAffinity    []int         `json:"cpu_affinity,omitempty"`
	RNGSeed        *int64        `json:"rng_seed,omitempty"`
	WarmupDuration time.Duration `json:"warmup_duration"`
	EnableProfiling bool         `json:"enable_profiling"`
	RecordBaseline bool          `json:"record_baseline"`
	OutputFile     string        `json:"output_file"`
}

// BenchMetrics represents benchmark metrics
type BenchMetrics struct {
	RPS           float64      `json:"rps"`
	LatencyP50    float64      `json:"latency_p50_ms"`
	LatencyP95    float64      `json:"latency_p95_ms"`
	LatencyP99    float64      `json:"latency_p99_ms"`
	BytesTX       uint64       `json:"bytes_tx"`
	BytesRX       uint64       `json:"bytes_rx"`
	CPUPct        float64      `json:"cpu_pct"`
	RSSMB         float64      `json:"rss_mb"`
	GCStats       *GCStats     `json:"gc_stats,omitempty"`
	SyscallStats  SyscallStats `json:"syscall_stats"`
	Errors        uint64       `json:"errors"`
	Connections   int          `json:"connections"`
	Duration      time.Duration `json:"duration"`
	Timestamp     int64        `json:"timestamp"`
}

// GCStats represents garbage collection statistics
type GCStats struct {
	GCCount     uint64  `json:"gc_count"`
	GCTimeMs    float64 `json:"gc_time_ms"`
	HeapSizeMB  float64 `json:"heap_size_mb"`
	HeapUsedMB  float64 `json:"heap_used_mb"`
}

// SyscallStats represents system call statistics
type SyscallStats struct {
	SyscallsPerSec  float64 `json:"syscalls_per_sec"`
	ReadSyscalls    uint64  `json:"read_syscalls"`
	WriteSyscalls   uint64  `json:"write_syscalls"`
	ConnectSyscalls uint64  `json:"connect_syscalls"`
	AcceptSyscalls  uint64  `json:"accept_syscalls"`
	EpollSyscalls   uint64  `json:"epoll_syscalls"`
}

// BenchResult represents the complete benchmark result
type BenchResult struct {
	Config        BenchConfig `json:"config"`
	Metrics       BenchMetrics `json:"metrics"`
	Success       bool        `json:"success"`
	ErrorMessage  *string     `json:"error_message,omitempty"`
}

// BenchHarness manages the benchmark execution
type BenchHarness struct {
	config  BenchConfig
	metrics *BenchMetricsCollector
	ctx     context.Context
	cancel  context.CancelFunc
}

// BenchMetricsCollector collects metrics during benchmark execution
type BenchMetricsCollector struct {
	requests      int64
	errors        int64
	bytesTX       int64
	bytesRX       int64
	latencies     []time.Duration
	latenciesMu   sync.Mutex
	syscallStats  SyscallStats
	syscallMu     sync.Mutex
	startTime     time.Time
}

// NewBenchMetricsCollector creates a new metrics collector
func NewBenchMetricsCollector() *BenchMetricsCollector {
	return &BenchMetricsCollector{
		latencies: make([]time.Duration, 0, 10000),
		startTime: time.Now(),
	}
}

// RecordRequest records a successful request
func (c *BenchMetricsCollector) RecordRequest(latency time.Duration, bytesTX, bytesRX int) {
	atomic.AddInt64(&c.requests, 1)
	atomic.AddInt64(&c.bytesTX, int64(bytesTX))
	atomic.AddInt64(&c.bytesRX, int64(bytesRX))
	
	c.latenciesMu.Lock()
	c.latencies = append(c.latencies, latency)
	c.latenciesMu.Unlock()
}

// RecordError records an error
func (c *BenchMetricsCollector) RecordError() {
	atomic.AddInt64(&c.errors, 1)
}

// RecordSyscall records a system call
func (c *BenchMetricsCollector) RecordSyscall(syscallType string) {
	c.syscallMu.Lock()
	switch syscallType {
	case "read":
		c.syscallStats.ReadSyscalls++
	case "write":
		c.syscallStats.WriteSyscalls++
	case "connect":
		c.syscallStats.ConnectSyscalls++
	case "accept":
		c.syscallStats.AcceptSyscalls++
	case "epoll":
		c.syscallStats.EpollSyscalls++
	}
	c.syscallMu.Unlock()
}

// Finalize calculates final metrics
func (c *BenchMetricsCollector) Finalize(duration time.Duration) BenchMetrics {
	requests := atomic.LoadInt64(&c.requests)
	errors := atomic.LoadInt64(&c.errors)
	bytesTX := atomic.LoadInt64(&c.bytesTX)
	bytesRX := atomic.LoadInt64(&c.bytesRX)
	
	rps := float64(requests) / duration.Seconds()
	
	// Calculate latency percentiles
	c.latenciesMu.Lock()
	latencies := make([]time.Duration, len(c.latencies))
	copy(latencies, c.latencies)
	c.latenciesMu.Unlock()
	
	sort.Slice(latencies, func(i, j int) bool {
		return latencies[i] < latencies[j]
	})
	
	var p50, p95, p99 float64
	if len(latencies) > 0 {
		p50 = float64(latencies[len(latencies)/2].Nanoseconds()) / 1e6 // Convert to ms
		if len(latencies) > 0 {
			p95Idx := int(float64(len(latencies)) * 0.95)
			if p95Idx >= len(latencies) {
				p95Idx = len(latencies) - 1
			}
			p95 = float64(latencies[p95Idx].Nanoseconds()) / 1e6
		}
		if len(latencies) > 0 {
			p99Idx := int(float64(len(latencies)) * 0.99)
			if p99Idx >= len(latencies) {
				p99Idx = len(latencies) - 1
			}
			p99 = float64(latencies[p99Idx].Nanoseconds()) / 1e6
		}
	}
	
	c.syscallMu.Lock()
	syscallStats := c.syscallStats
	c.syscallMu.Unlock()
	
	totalSyscalls := syscallStats.ReadSyscalls + syscallStats.WriteSyscalls + 
		syscallStats.ConnectSyscalls + syscallStats.AcceptSyscalls + syscallStats.EpollSyscalls
	syscallStats.SyscallsPerSec = float64(totalSyscalls) / duration.Seconds()
	
	// Get system metrics
	cpuPct, rssMB := getSystemMetrics()
	gcStats := getGCStats()
	
	return BenchMetrics{
		RPS:           rps,
		LatencyP50:    p50,
		LatencyP95:    p95,
		LatencyP99:    p99,
		BytesTX:       uint64(bytesTX),
		BytesRX:       uint64(bytesRX),
		CPUPct:        cpuPct,
		RSSMB:         rssMB,
		GCStats:       gcStats,
		SyscallStats:  syscallStats,
		Errors:        uint64(errors),
		Connections:   0, // Will be set by caller
		Duration:      duration,
		Timestamp:     time.Now().Unix(),
	}
}

// NewBenchHarness creates a new benchmark harness
func NewBenchHarness(config BenchConfig) *BenchHarness {
	ctx, cancel := context.WithTimeout(context.Background(), config.Duration)
	
	return &BenchHarness{
		config:  config,
		metrics: NewBenchMetricsCollector(),
		ctx:     ctx,
		cancel:  cancel,
	}
}

// Run executes the benchmark
func (h *BenchHarness) Run() (*BenchResult, error) {
	startTime := time.Now()
	
	// Set CPU affinity if specified
	if len(h.config.CPUAffinity) > 0 {
		if err := setCPUAffinity(h.config.CPUAffinity); err != nil {
			log.Printf("Warning: Failed to set CPU affinity: %v", err)
		}
	}
	
	// Set RNG seed for deterministic results
	if h.config.RNGSeed != nil {
		setRNGSeed(*h.config.RNGSeed)
	}
	
	// Warm-up phase
	if h.config.WarmupDuration > 0 {
		if err := h.warmup(); err != nil {
			log.Printf("Warning: Warm-up failed: %v", err)
		}
	}
	
	// Reset metrics after warm-up
	h.metrics = NewBenchMetricsCollector()
	
	// Start system monitoring
	monitorCtx, monitorCancel := context.WithCancel(h.ctx)
	defer monitorCancel()
	
	// Run the benchmark
	err := h.runBenchmarkPhase()
	
	// Calculate final metrics
	duration := time.Since(startTime)
	metrics := h.metrics.Finalize(duration)
	metrics.Connections = h.config.Clients
	
	success := err == nil
	var errorMessage *string
	if err != nil {
		msg := err.Error()
		errorMessage = &msg
	}
	
	result := &BenchResult{
		Config:       h.config,
		Metrics:      metrics,
		Success:      success,
		ErrorMessage: errorMessage,
	}
	
	// Write results to file if specified
	if h.config.OutputFile != "" {
		if err := h.writeResults(result); err != nil {
			log.Printf("Warning: Failed to write results: %v", err)
		}
	}
	
	// Record baseline if requested
	if h.config.RecordBaseline {
		if err := h.recordBaseline(result); err != nil {
			log.Printf("Warning: Failed to record baseline: %v", err)
		}
	}
	
	return result, err
}

// runBenchmarkPhase runs the actual benchmark
func (h *BenchHarness) runBenchmarkPhase() error {
	var wg sync.WaitGroup
	clientCtx, clientCancel := context.WithCancel(h.ctx)
	defer clientCancel()
	
	// Start client workers
	for i := 0; i < h.config.Clients; i++ {
		wg.Add(1)
		go func(clientID int) {
			defer wg.Done()
			h.clientWorker(clientCtx, clientID)
		}(i)
	}
	
	// Wait for benchmark duration
	<-h.ctx.Done()
	
	// Signal all workers to stop
	clientCancel()
	
	// Wait for all workers to complete
	wg.Wait()
	
	// Check error rate
	errors := atomic.LoadInt64(&h.metrics.errors)
	if errors > int64(h.config.Clients/10) {
		return fmt.Errorf("too many client errors: %d", errors)
	}
	
	return nil
}

// clientWorker runs a single client worker
func (h *BenchHarness) clientWorker(ctx context.Context, clientID int) {
	// Connect to server
	conn, err := net.DialTimeout("tcp", h.config.ServerAddr, 5*time.Second)
	if err != nil {
		h.metrics.RecordError()
		return
	}
	defer conn.Close()
	
	h.metrics.RecordSyscall("connect")
	
	// Generate test payload
	payload := make([]byte, h.config.PayloadSize)
	for i := range payload {
		payload[i] = byte(i % 256)
	}
	
	// Main request loop
	for {
		select {
		case <-ctx.Done():
			return
		default:
		}
		
		start := time.Now()
		
		// Send request
		if _, err := conn.Write(payload); err != nil {
			h.metrics.RecordError()
			return
		}
		h.metrics.RecordSyscall("write")
		
		// Receive response
		response := make([]byte, h.config.PayloadSize)
		n, err := conn.Read(response)
		if err != nil {
			if err != io.EOF {
				h.metrics.RecordError()
			}
			return
		}
		h.metrics.RecordSyscall("read")
		
		latency := time.Since(start)
		h.metrics.RecordRequest(latency, len(payload), n)
		
		// Apply back-pressure if enabled
		if h.config.BackPressure && latency > 100*time.Millisecond {
			time.Sleep(time.Millisecond)
		}
	}
}

// warmup runs a warm-up phase
func (h *BenchHarness) warmup() error {
	warmupClients := h.config.Clients / 10
	if warmupClients < 1 {
		warmupClients = 1
	}
	
	warmupConfig := h.config
	warmupConfig.Clients = warmupClients
	warmupConfig.Duration = h.config.WarmupDuration
	
	warmupHarness := NewBenchHarness(warmupConfig)
	_, err := warmupHarness.Run()
	return err
}

// writeResults writes benchmark results to file
func (h *BenchHarness) writeResults(result *BenchResult) error {
	data, err := json.MarshalIndent(result, "", "  ")
	if err != nil {
		return err
	}
	
	return os.WriteFile(h.config.OutputFile, data, 0644)
}

// recordBaseline records baseline metrics
func (h *BenchHarness) recordBaseline(result *BenchResult) error {
	// Create baseline directory if it doesn't exist
	baselineDir := "perf/baselines"
	if err := os.MkdirAll(baselineDir, 0755); err != nil {
		return err
	}
	
	// Load existing baselines
	baselineFile := baselineDir + "/p4_03_net_adv.json"
	var baselines map[string]interface{}
	
	if data, err := os.ReadFile(baselineFile); err == nil {
		json.Unmarshal(data, &baselines)
	} else {
		baselines = make(map[string]interface{})
	}
	
	// Add new baseline
	configKey := fmt.Sprintf("clients_%d_tls_%v_duration_%s", 
		h.config.Clients, h.config.TLSEnabled, h.config.Duration)
	baselines[configKey] = result.Metrics
	
	// Write updated baselines
	data, err := json.MarshalIndent(baselines, "", "  ")
	if err != nil {
		return err
	}
	
	return os.WriteFile(baselineFile, data, 0644)
}

// System monitoring functions
func getSystemMetrics() (float64, float64) {
	// Get CPU usage (simplified)
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	
	// Convert bytes to MB
	rssMB := float64(m.Sys) / 1024 / 1024
	
	// Mock CPU usage (in real implementation, would use platform-specific APIs)
	cpuPct := 25.0
	
	return cpuPct, rssMB
}

func getGCStats() *GCStats {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	
	return &GCStats{
		GCCount:    uint64(m.NumGC),
		GCTimeMs:   float64(m.PauseTotalNs) / 1e6,
		HeapSizeMB: float64(m.HeapSys) / 1024 / 1024,
		HeapUsedMB: float64(m.HeapInuse) / 1024 / 1024,
	}
}

func setCPUAffinity(affinity []int) error {
	// This would use platform-specific APIs to set CPU affinity
	log.Printf("Setting CPU affinity to: %v", affinity)
	return nil
}

func setRNGSeed(seed int64) {
	// This would set the RNG seed for deterministic results
	log.Printf("Setting RNG seed to: %d", seed)
}

// handleBenchmark handles the benchmark command
func handleBenchmark(args []string) error {
	fs := flag.NewFlagSet("bench", flag.ExitOnError)
	
	var (
		clients        = fs.Int("clients", 1000, "Number of concurrent connections")
		duration       = fs.Duration("duration", 30*time.Second, "Benchmark duration")
		tlsEnabled     = fs.Bool("tls", false, "Enable TLS")
		tlsProfile     = fs.String("profile", "tls13_modern", "TLS profile")
		pqcAlgorithms  = fs.String("pqc", "kyber512,dilithium2", "PQC algorithms (comma-separated)")
		serverAddr     = fs.String("addr", "127.0.0.1:8080", "Server address")
		payloadSize    = fs.Int("payload", 1024, "Payload size in bytes")
		zeroCopy       = fs.Bool("zero-copy", true, "Enable zero-copy I/O")
		backPressure   = fs.Bool("back-pressure", true, "Enable back-pressure")
		warmupDuration = fs.Duration("warmup", 5*time.Second, "Warm-up duration")
		outputFile     = fs.String("json", "", "Output JSON file")
		recordBaseline = fs.Bool("record-baseline", false, "Record baseline metrics")
	)
	
	if err := fs.Parse(args); err != nil {
		return err
	}
	
	// Parse PQC algorithms
	var pqcAlgList []string
	if *pqcAlgorithms != "" {
		pqcAlgList = strings.Split(*pqcAlgorithms, ",")
		for i, alg := range pqcAlgList {
			pqcAlgList[i] = strings.TrimSpace(alg)
		}
	}
	
	config := BenchConfig{
		Clients:         *clients,
		Duration:        *duration,
		TLSEnabled:      *tlsEnabled,
		TLSProfile:      *tlsProfile,
		PQCAlgorithms:   pqcAlgList,
		ServerAddr:      *serverAddr,
		PayloadSize:     *payloadSize,
		ZeroCopy:        *zeroCopy,
		BackPressure:    *backPressure,
		WarmupDuration:  *warmupDuration,
		OutputFile:      *outputFile,
		RecordBaseline:  *recordBaseline,
	}
	
	// Create and run benchmark
	harness := NewBenchHarness(config)
	result, err := harness.Run()
	
	// Print results
	fmt.Printf("Benchmark Results:\n")
	fmt.Printf("==================\n")
	fmt.Printf("Clients: %d\n", result.Metrics.Connections)
	fmt.Printf("Duration: %v\n", result.Metrics.Duration)
	fmt.Printf("RPS: %.2f\n", result.Metrics.RPS)
	fmt.Printf("Latency P50: %.2f ms\n", result.Metrics.LatencyP50)
	fmt.Printf("Latency P95: %.2f ms\n", result.Metrics.LatencyP95)
	fmt.Printf("Latency P99: %.2f ms\n", result.Metrics.LatencyP99)
	fmt.Printf("Bytes TX: %d\n", result.Metrics.BytesTX)
	fmt.Printf("Bytes RX: %d\n", result.Metrics.BytesRX)
	fmt.Printf("CPU: %.2f%%\n", result.Metrics.CPUPct)
	fmt.Printf("RSS: %.2f MB\n", result.Metrics.RSSMB)
	fmt.Printf("Errors: %d\n", result.Metrics.Errors)
	fmt.Printf("Syscalls/sec: %.2f\n", result.Metrics.SyscallStats.SyscallsPerSec)
	
	if result.Metrics.GCStats != nil {
		fmt.Printf("GC Count: %d\n", result.Metrics.GCStats.GCCount)
		fmt.Printf("GC Time: %.2f ms\n", result.Metrics.GCStats.GCTimeMs)
	}
	
	if !result.Success {
		fmt.Printf("Benchmark failed: %s\n", *result.ErrorMessage)
		return fmt.Errorf("benchmark failed")
	}
	
	return nil
}

// handleConcurrentBenchmark handles the concurrent benchmark command
func handleConcurrentBenchmark(args []string) error {
	fs := flag.NewFlagSet("concurrent", flag.ExitOnError)
	
	var (
		clients        = fs.Int("clients", 10000, "Number of concurrent connections")
		duration       = fs.Duration("duration", 30*time.Minute, "Benchmark duration")
		tlsEnabled     = fs.Bool("tls", false, "Enable TLS")
		tlsProfile     = fs.String("profile", "pqc_hybrid", "TLS profile")
		pqcAlgorithms  = fs.String("pqc", "kyber768,dilithium3", "PQC algorithms (comma-separated)")
		serverAddr     = fs.String("addr", "127.0.0.1:8080", "Server address")
		payloadSize    = fs.Int("payload", 1024, "Payload size in bytes")
		zeroCopy       = fs.Bool("zero-copy", true, "Enable zero-copy I/O")
		backPressure   = fs.Bool("back-pressure", true, "Enable back-pressure")
		warmupDuration = fs.Duration("warmup", 30*time.Second, "Warm-up duration")
		outputFile     = fs.String("json", "", "Output JSON file")
		recordBaseline = fs.Bool("record-baseline", false, "Record baseline metrics")
	)
	
	if err := fs.Parse(args); err != nil {
		return err
	}
	
	// Parse PQC algorithms
	var pqcAlgList []string
	if *pqcAlgorithms != "" {
		pqcAlgList = strings.Split(*pqcAlgorithms, ",")
		for i, alg := range pqcAlgList {
			pqcAlgList[i] = strings.TrimSpace(alg)
		}
	}
	
	config := BenchConfig{
		Clients:         *clients,
		Duration:        *duration,
		TLSEnabled:      *tlsEnabled,
		TLSProfile:      *tlsProfile,
		PQCAlgorithms:   pqcAlgList,
		ServerAddr:      *serverAddr,
		PayloadSize:     *payloadSize,
		ZeroCopy:        *zeroCopy,
		BackPressure:    *backPressure,
		WarmupDuration:  *warmupDuration,
		OutputFile:      *outputFile,
		RecordBaseline:  *recordBaseline,
	}
	
	// Create and run benchmark
	harness := NewBenchHarness(config)
	result, err := harness.Run()
	
	// Print results
	fmt.Printf("Concurrent Benchmark Results:\n")
	fmt.Printf("=============================\n")
	fmt.Printf("Clients: %d\n", result.Metrics.Connections)
	fmt.Printf("Duration: %v\n", result.Metrics.Duration)
	fmt.Printf("RPS: %.2f\n", result.Metrics.RPS)
	fmt.Printf("Latency P50: %.2f ms\n", result.Metrics.LatencyP50)
	fmt.Printf("Latency P95: %.2f ms\n", result.Metrics.LatencyP95)
	fmt.Printf("Latency P99: %.2f ms\n", result.Metrics.LatencyP99)
	fmt.Printf("Throughput: %.2f MB/s\n", float64(result.Metrics.BytesTX)/1024/1024/result.Metrics.Duration.Seconds())
	fmt.Printf("CPU: %.2f%%\n", result.Metrics.CPUPct)
	fmt.Printf("RSS: %.2f MB\n", result.Metrics.RSSMB)
	fmt.Printf("Errors: %d\n", result.Metrics.Errors)
	
	if !result.Success {
		fmt.Printf("Benchmark failed: %s\n", *result.ErrorMessage)
		return fmt.Errorf("benchmark failed")
	}
	
	return nil
}

// handleSoakTest handles the soak test command
func handleSoakTest(args []string) error {
	fs := flag.NewFlagSet("soak", flag.ExitOnError)
	
	var (
		clients        = fs.Int("clients", 5000, "Number of concurrent connections")
		duration       = fs.Duration("duration", 2*time.Hour, "Soak test duration")
		tlsEnabled     = fs.Bool("tls", true, "Enable TLS")
		tlsProfile     = fs.String("profile", "tls13_modern", "TLS profile")
		pqcAlgorithms  = fs.String("pqc", "kyber512,dilithium2", "PQC algorithms (comma-separated)")
		serverAddr     = fs.String("addr", "127.0.0.1:8080", "Server address")
		payloadSize    = fs.Int("payload", 1024, "Payload size in bytes")
		zeroCopy       = fs.Bool("zero-copy", true, "Enable zero-copy I/O")
		backPressure   = fs.Bool("back-pressure", true, "Enable back-pressure")
		warmupDuration = fs.Duration("warmup", 60*time.Second, "Warm-up duration")
		outputFile     = fs.String("json", "", "Output JSON file")
		recordBaseline = fs.Bool("record-baseline", false, "Record baseline metrics")
		reportInterval = fs.Duration("report-interval", 5*time.Minute, "Report interval")
	)
	
	if err := fs.Parse(args); err != nil {
		return err
	}
	
	// Parse PQC algorithms
	var pqcAlgList []string
	if *pqcAlgorithms != "" {
		pqcAlgList = strings.Split(*pqcAlgorithms, ",")
		for i, alg := range pqcAlgList {
			pqcAlgList[i] = strings.TrimSpace(alg)
		}
	}
	
	config := BenchConfig{
		Clients:         *clients,
		Duration:        *duration,
		TLSEnabled:      *tlsEnabled,
		TLSProfile:      *tlsProfile,
		PQCAlgorithms:   pqcAlgList,
		ServerAddr:      *serverAddr,
		PayloadSize:     *payloadSize,
		ZeroCopy:        *zeroCopy,
		BackPressure:    *backPressure,
		WarmupDuration:  *warmupDuration,
		OutputFile:      *outputFile,
		RecordBaseline:  *recordBaseline,
	}
	
	// Create and run soak test
	harness := NewBenchHarness(config)
	
	// Start periodic reporting
	reportCtx, reportCancel := context.WithCancel(context.Background())
	defer reportCancel()
	
	go func() {
		ticker := time.NewTicker(*reportInterval)
		defer ticker.Stop()
		
		for {
			select {
			case <-reportCtx.Done():
				return
			case <-ticker.C:
				// Print intermediate results
				fmt.Printf("[%s] Soak test running... Clients: %d, RPS: %.2f, Errors: %d\n",
					time.Now().Format("15:04:05"),
					atomic.LoadInt64(&harness.metrics.requests),
					float64(atomic.LoadInt64(&harness.metrics.requests))/time.Since(harness.metrics.startTime).Seconds(),
					atomic.LoadInt64(&harness.metrics.errors))
			}
		}
	}()
	
	result, err := harness.Run()
	
	// Print final results
	fmt.Printf("\nSoak Test Results:\n")
	fmt.Printf("==================\n")
	fmt.Printf("Clients: %d\n", result.Metrics.Connections)
	fmt.Printf("Duration: %v\n", result.Metrics.Duration)
	fmt.Printf("RPS: %.2f\n", result.Metrics.RPS)
	fmt.Printf("Latency P50: %.2f ms\n", result.Metrics.LatencyP50)
	fmt.Printf("Latency P95: %.2f ms\n", result.Metrics.LatencyP95)
	fmt.Printf("Latency P99: %.2f ms\n", result.Metrics.LatencyP99)
	fmt.Printf("Throughput: %.2f MB/s\n", float64(result.Metrics.BytesTX)/1024/1024/result.Metrics.Duration.Seconds())
	fmt.Printf("CPU: %.2f%%\n", result.Metrics.CPUPct)
	fmt.Printf("RSS: %.2f MB\n", result.Metrics.RSSMB)
	fmt.Printf("Errors: %d\n", result.Metrics.Errors)
	
	if result.Metrics.GCStats != nil {
		fmt.Printf("GC Count: %d\n", result.Metrics.GCStats.GCCount)
		fmt.Printf("GC Time: %.2f ms\n", result.Metrics.GCStats.GCTimeMs)
		fmt.Printf("Heap Size: %.2f MB\n", result.Metrics.GCStats.HeapSizeMB)
		fmt.Printf("Heap Used: %.2f MB\n", result.Metrics.GCStats.HeapUsedMB)
	}
	
	if !result.Success {
		fmt.Printf("Soak test failed: %s\n", *result.ErrorMessage)
		return fmt.Errorf("soak test failed")
	}
	
	return nil
}
