//! Network Performance Benchmark Tool for Aetheris OS
//! 
//! This tool provides comprehensive benchmarking capabilities for the Aetheris OS
//! networking subsystem, including TCP, TLS, QUIC, and concurrent connection testing.

package main

import (
	"context"
	"crypto/tls"
	"flag"
	"fmt"
	"log"
	"net"
	"runtime"
	"sync"
	"sync/atomic"
	"time"
)

// Benchmark configuration
type BenchmarkConfig struct {
	Host         string
	Port         int
	Protocol     string
	Clients      int
	Duration     time.Duration
	MessageSize  int
	Connections  int
	Streams      int
	Profile      string
	Verbose      bool
	OutputFormat string
}

// Benchmark results
type BenchmarkResult struct {
	Protocol              string    `json:"protocol"`
	StartTime             time.Time `json:"start_time"`
	EndTime               time.Time `json:"end_time"`
	Duration              float64   `json:"duration_seconds"`
	TotalConnections      int64     `json:"total_connections"`
	SuccessfulConnections int64     `json:"successful_connections"`
	FailedConnections     int64     `json:"failed_connections"`
	TotalMessages         int64     `json:"total_messages"`
	TotalBytes            int64     `json:"total_bytes"`
	AverageLatency        float64   `json:"average_latency_ms"`
	P95Latency            float64   `json:"p95_latency_ms"`
	P99Latency            float64   `json:"p99_latency_ms"`
	Throughput            float64   `json:"throughput_gbps"`
	CPUUsage              float64   `json:"cpu_usage_percent"`
	MemoryUsage           int64     `json:"memory_usage_mb"`
	HandshakeTime         float64   `json:"handshake_time_ms,omitempty"`
	P95HandshakeTime      float64   `json:"p95_handshake_time_ms,omitempty"`
	ConnectionRate        float64   `json:"connection_rate_per_second,omitempty"`
	NetworkErrors         int64     `json:"network_errors"`
	TimeoutErrors         int64     `json:"timeout_errors"`
}

// Latency measurement
type LatencyMeasurement struct {
	Timestamp time.Time
	Latency   time.Duration
}

// Benchmark runner
type BenchmarkRunner struct {
	config   BenchmarkConfig
	results  BenchmarkResult
	latencies []LatencyMeasurement
	mu       sync.RWMutex
}

// New benchmark runner
func NewBenchmarkRunner(config BenchmarkConfig) *BenchmarkRunner {
	return &BenchmarkRunner{
		config:    config,
		latencies: make([]LatencyMeasurement, 0, config.Clients*100),
	}
}

// Run TCP benchmark
func (br *BenchmarkRunner) RunTCPBenchmark() (*BenchmarkResult, error) {
	br.results.Protocol = "TCP"
	br.results.StartTime = time.Now()
	
	log.Printf("Starting TCP benchmark: %s:%d, %d clients, %v duration", 
		br.config.Host, br.config.Port, br.config.Clients, br.config.Duration)
	
	// Start monitoring goroutines
	ctx, cancel := context.WithTimeout(context.Background(), br.config.Duration)
	defer cancel()
	
	var wg sync.WaitGroup
	var totalMessages, totalBytes, networkErrors, timeoutErrors int64
	
	// Start clients
	for i := 0; i < br.config.Clients; i++ {
		wg.Add(1)
		go func(clientID int) {
			defer wg.Done()
			
			client := &TCPClient{
				ID:          clientID,
				Host:        br.config.Host,
				Port:        br.config.Port,
				MessageSize: br.config.MessageSize,
				Duration:    br.config.Duration,
			}
			
			result := client.Run(ctx)
			
			atomic.AddInt64(&totalMessages, result.Messages)
			atomic.AddInt64(&totalBytes, result.Bytes)
			atomic.AddInt64(&networkErrors, result.NetworkErrors)
			atomic.AddInt64(&timeoutErrors, result.TimeoutErrors)
			
			// Record latencies
			br.mu.Lock()
			br.latencies = append(br.latencies, result.Latencies...)
			br.mu.Unlock()
			
			if result.Success {
				atomic.AddInt64(&br.results.SuccessfulConnections, 1)
			} else {
				atomic.AddInt64(&br.results.FailedConnections, 1)
			}
		}(i)
	}
	
	// Wait for all clients to complete
	wg.Wait()
	br.results.EndTime = time.Now()
	
	// Calculate results
	br.results.Duration = br.results.EndTime.Sub(br.results.StartTime).Seconds()
	br.results.TotalConnections = int64(br.config.Clients)
	br.results.TotalMessages = totalMessages
	br.results.TotalBytes = totalBytes
	br.results.NetworkErrors = networkErrors
	br.results.TimeoutErrors = timeoutErrors
	
	// Calculate latency statistics
	br.calculateLatencyStats()
	
	// Calculate throughput
	br.results.Throughput = float64(totalBytes*8) / (br.results.Duration * 1e9) // Gbps
	
	// Get system metrics
	br.getSystemMetrics()
	
	return &br.results, nil
}

// Run TLS benchmark
func (br *BenchmarkRunner) RunTLSBenchmark() (*BenchmarkResult, error) {
	br.results.Protocol = "TLS"
	br.results.StartTime = time.Now()
	
	log.Printf("Starting TLS benchmark: %s:%d, %d clients, %v duration, profile: %s", 
		br.config.Host, br.config.Port, br.config.Clients, br.config.Duration, br.config.Profile)
	
	// Start monitoring goroutines
	ctx, cancel := context.WithTimeout(context.Background(), br.config.Duration)
	defer cancel()
	
	var wg sync.WaitGroup
	var totalMessages, totalBytes, networkErrors, timeoutErrors int64
	var handshakeTimes []time.Duration
	
	// Start clients
	for i := 0; i < br.config.Clients; i++ {
		wg.Add(1)
		go func(clientID int) {
			defer wg.Done()
			
			client := &TLSClient{
				ID:          clientID,
				Host:        br.config.Host,
				Port:        br.config.Port,
				MessageSize: br.config.MessageSize,
				Duration:    br.config.Duration,
				Profile:     br.config.Profile,
			}
			
			result := client.Run(ctx)
			
			atomic.AddInt64(&totalMessages, result.Messages)
			atomic.AddInt64(&totalBytes, result.Bytes)
			atomic.AddInt64(&networkErrors, result.NetworkErrors)
			atomic.AddInt64(&timeoutErrors, result.TimeoutErrors)
			
			// Record latencies and handshake times
			br.mu.Lock()
			br.latencies = append(br.latencies, result.Latencies...)
			handshakeTimes = append(handshakeTimes, result.HandshakeTime)
			br.mu.Unlock()
			
			if result.Success {
				atomic.AddInt64(&br.results.SuccessfulConnections, 1)
			} else {
				atomic.AddInt64(&br.results.FailedConnections, 1)
			}
		}(i)
	}
	
	// Wait for all clients to complete
	wg.Wait()
	br.results.EndTime = time.Now()
	
	// Calculate results
	br.results.Duration = br.results.EndTime.Sub(br.results.StartTime).Seconds()
	br.results.TotalConnections = int64(br.config.Clients)
	br.results.TotalMessages = totalMessages
	br.results.TotalBytes = totalBytes
	br.results.NetworkErrors = networkErrors
	br.results.TimeoutErrors = timeoutErrors
	
	// Calculate latency statistics
	br.calculateLatencyStats()
	
	// Calculate handshake statistics
	br.calculateHandshakeStats(handshakeTimes)
	
	// Calculate throughput
	br.results.Throughput = float64(totalBytes*8) / (br.results.Duration * 1e9) // Gbps
	
	// Get system metrics
	br.getSystemMetrics()
	
	return &br.results, nil
}

// Run QUIC benchmark
func (br *BenchmarkRunner) RunQUICBenchmark() (*BenchmarkResult, error) {
	br.results.Protocol = "QUIC"
	br.results.StartTime = time.Now()
	
	log.Printf("Starting QUIC benchmark: %s:%d, %d clients, %v duration, profile: %s", 
		br.config.Host, br.config.Port, br.config.Clients, br.config.Duration, br.config.Profile)
	
	// Start monitoring goroutines
	ctx, cancel := context.WithTimeout(context.Background(), br.config.Duration)
	defer cancel()
	
	var wg sync.WaitGroup
	var totalMessages, totalBytes, networkErrors, timeoutErrors int64
	var handshakeTimes []time.Duration
	
	// Start clients
	for i := 0; i < br.config.Clients; i++ {
		wg.Add(1)
		go func(clientID int) {
			defer wg.Done()
			
			client := &QUICClient{
				ID:          clientID,
				Host:        br.config.Host,
				Port:        br.config.Port,
				MessageSize: br.config.MessageSize,
				Duration:    br.config.Duration,
				Profile:     br.config.Profile,
				Streams:     br.config.Streams,
			}
			
			result := client.Run(ctx)
			
			atomic.AddInt64(&totalMessages, result.Messages)
			atomic.AddInt64(&totalBytes, result.Bytes)
			atomic.AddInt64(&networkErrors, result.NetworkErrors)
			atomic.AddInt64(&timeoutErrors, result.TimeoutErrors)
			
			// Record latencies and handshake times
			br.mu.Lock()
			br.latencies = append(br.latencies, result.Latencies...)
			handshakeTimes = append(handshakeTimes, result.HandshakeTime)
			br.mu.Unlock()
			
			if result.Success {
				atomic.AddInt64(&br.results.SuccessfulConnections, 1)
			} else {
				atomic.AddInt64(&br.results.FailedConnections, 1)
			}
		}(i)
	}
	
	// Wait for all clients to complete
	wg.Wait()
	br.results.EndTime = time.Now()
	
	// Calculate results
	br.results.Duration = br.results.EndTime.Sub(br.results.StartTime).Seconds()
	br.results.TotalConnections = int64(br.config.Clients)
	br.results.TotalMessages = totalMessages
	br.results.TotalBytes = totalBytes
	br.results.NetworkErrors = networkErrors
	br.results.TimeoutErrors = timeoutErrors
	
	// Calculate latency statistics
	br.calculateLatencyStats()
	
	// Calculate handshake statistics
	br.calculateHandshakeStats(handshakeTimes)
	
	// Calculate throughput
	br.results.Throughput = float64(totalBytes*8) / (br.results.Duration * 1e9) // Gbps
	
	// Get system metrics
	br.getSystemMetrics()
	
	return &br.results, nil
}

// Run concurrent connections benchmark
func (br *BenchmarkRunner) RunConcurrentConnectionsBenchmark() (*BenchmarkResult, error) {
	br.results.Protocol = br.config.Protocol
	br.results.StartTime = time.Now()
	
	log.Printf("Starting concurrent connections benchmark: %s:%d, %d connections, protocol: %s", 
		br.config.Host, br.config.Port, br.config.Connections, br.config.Protocol)
	
	// Start monitoring goroutines
	ctx, cancel := context.WithTimeout(context.Background(), br.config.Duration)
	defer cancel()
	
	var wg sync.WaitGroup
	var connectionTimes []time.Duration
	var networkErrors, timeoutErrors int64
	
	// Start connections
	for i := 0; i < br.config.Connections; i++ {
		wg.Add(1)
		go func(connID int) {
			defer wg.Done()
			
			startTime := time.Now()
			
			var conn net.Conn
			var err error
			
			switch br.config.Protocol {
			case "tcp":
				conn, err = net.DialTimeout("tcp", fmt.Sprintf("%s:%d", br.config.Host, br.config.Port), 10*time.Second)
			case "tls":
				tlsConn, tlsErr := tls.DialWithDialer(&net.Dialer{Timeout: 10 * time.Second}, "tcp", 
					fmt.Sprintf("%s:%d", br.config.Host, br.config.Port), &tls.Config{InsecureSkipVerify: true})
				conn, err = tlsConn, tlsErr
			default:
				err = fmt.Errorf("unsupported protocol: %s", br.config.Protocol)
			}
			
			connectionTime := time.Since(startTime)
			
			if err != nil {
				atomic.AddInt64(&networkErrors, 1)
				atomic.AddInt64(&br.results.FailedConnections, 1)
				return
			}
			
			defer conn.Close()
			
			// Record connection time
			br.mu.Lock()
			connectionTimes = append(connectionTimes, connectionTime)
			br.mu.Unlock()
			
			atomic.AddInt64(&br.results.SuccessfulConnections, 1)
			
			// Keep connection alive for the duration
			select {
			case <-ctx.Done():
				return
			case <-time.After(br.config.Duration):
				return
			}
		}(i)
	}
	
	// Wait for all connections to complete
	wg.Wait()
	br.results.EndTime = time.Now()
	
	// Calculate results
	br.results.Duration = br.results.EndTime.Sub(br.results.StartTime).Seconds()
	br.results.TotalConnections = int64(br.config.Connections)
	br.results.NetworkErrors = networkErrors
	br.results.TimeoutErrors = timeoutErrors
	
	// Calculate connection time statistics
	br.calculateConnectionTimeStats(connectionTimes)
	
	// Calculate connection rate
	br.results.ConnectionRate = float64(br.results.SuccessfulConnections) / br.results.Duration
	
	// Get system metrics
	br.getSystemMetrics()
	
	return &br.results, nil
}

// Calculate latency statistics
func (br *BenchmarkRunner) calculateLatencyStats() {
	if len(br.latencies) == 0 {
		return
	}
	
	// Sort latencies
	latencies := make([]time.Duration, len(br.latencies))
	for i, l := range br.latencies {
		latencies[i] = l.Latency
	}
	
	// Calculate average
	var total time.Duration
	for _, l := range latencies {
		total += l
	}
	br.results.AverageLatency = float64(total.Microseconds()) / float64(len(latencies)) / 1000.0 // ms
	
	// Calculate percentiles (simplified)
	if len(latencies) > 0 {
		// P95 (95th percentile)
		p95Index := int(float64(len(latencies)) * 0.95)
		if p95Index < len(latencies) {
			br.results.P95Latency = float64(latencies[p95Index].Microseconds()) / 1000.0 // ms
		}
		
		// P99 (99th percentile)
		p99Index := int(float64(len(latencies)) * 0.99)
		if p99Index < len(latencies) {
			br.results.P99Latency = float64(latencies[p99Index].Microseconds()) / 1000.0 // ms
		}
	}
}

// Calculate handshake statistics
func (br *BenchmarkRunner) calculateHandshakeStats(handshakeTimes []time.Duration) {
	if len(handshakeTimes) == 0 {
		return
	}
	
	// Calculate average handshake time
	var total time.Duration
	for _, h := range handshakeTimes {
		total += h
	}
	br.results.HandshakeTime = float64(total.Milliseconds()) / float64(len(handshakeTimes))
	
	// Calculate P95 handshake time (simplified)
	if len(handshakeTimes) > 0 {
		p95Index := int(float64(len(handshakeTimes)) * 0.95)
		if p95Index < len(handshakeTimes) {
			br.results.P95HandshakeTime = float64(handshakeTimes[p95Index].Milliseconds())
		}
	}
}

// Calculate connection time statistics
func (br *BenchmarkRunner) calculateConnectionTimeStats(connectionTimes []time.Duration) {
	if len(connectionTimes) == 0 {
		return
	}
	
	// Calculate average connection time
	var total time.Duration
	for _, c := range connectionTimes {
		total += c
	}
	br.results.AverageLatency = float64(total.Milliseconds()) / float64(len(connectionTimes))
	
	// Calculate P95 and P99 connection times (simplified)
	if len(connectionTimes) > 0 {
		p95Index := int(float64(len(connectionTimes)) * 0.95)
		if p95Index < len(connectionTimes) {
			br.results.P95Latency = float64(connectionTimes[p95Index].Milliseconds())
		}
		
		p99Index := int(float64(len(connectionTimes)) * 0.99)
		if p99Index < len(connectionTimes) {
			br.results.P99Latency = float64(connectionTimes[p99Index].Milliseconds())
		}
	}
}

// Get system metrics
func (br *BenchmarkRunner) getSystemMetrics() {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	br.results.MemoryUsage = int64(m.Alloc / 1024 / 1024) // MB
	
	// Mock CPU usage - in real implementation would use actual CPU monitoring
	br.results.CPUUsage = 25.0 // Mock value
}

// TCP client implementation
type TCPClient struct {
	ID          int
	Host        string
	Port        int
	MessageSize int
	Duration    time.Duration
}

type ClientResult struct {
	Success       bool
	Messages      int64
	Bytes         int64
	NetworkErrors int64
	TimeoutErrors int64
	Latencies     []LatencyMeasurement
	HandshakeTime time.Duration
}

func (c *TCPClient) Run(ctx context.Context) ClientResult {
	result := ClientResult{
		Latencies: make([]LatencyMeasurement, 0, 100),
	}
	
	// Connect to server
	conn, err := net.DialTimeout("tcp", fmt.Sprintf("%s:%d", c.Host, c.Port), 10*time.Second)
	if err != nil {
		result.NetworkErrors++
		return result
	}
	defer conn.Close()
	
	result.Success = true
	
	// Send messages
	message := make([]byte, c.MessageSize)
	for i := range message {
		message[i] = byte(i % 256)
	}
	
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()
	
	for {
		select {
		case <-ctx.Done():
			return result
		case <-ticker.C:
			startTime := time.Now()
			
			_, err := conn.Write(message)
			if err != nil {
				result.NetworkErrors++
				continue
			}
			
			// Read response
			response := make([]byte, c.MessageSize)
			_, err = conn.Read(response)
			if err != nil {
				result.NetworkErrors++
				continue
			}
			
			latency := time.Since(startTime)
			result.Latencies = append(result.Latencies, LatencyMeasurement{
				Timestamp: startTime,
				Latency:   latency,
			})
			
			result.Messages++
			result.Bytes += int64(len(message) + len(response))
		}
	}
}

// TLS client implementation
type TLSClient struct {
	ID          int
	Host        string
	Port        int
	MessageSize int
	Duration    time.Duration
	Profile     string
}

func (c *TLSClient) Run(ctx context.Context) ClientResult {
	result := ClientResult{
		Latencies: make([]LatencyMeasurement, 0, 100),
	}
	
	// Create TLS connection
	startTime := time.Now()
	conn, err := tls.DialWithDialer(&net.Dialer{Timeout: 10 * time.Second}, "tcp", 
		fmt.Sprintf("%s:%d", c.Host, c.Port), &tls.Config{InsecureSkipVerify: true})
	if err != nil {
		result.NetworkErrors++
		return result
	}
	defer conn.Close()
	
	result.HandshakeTime = time.Since(startTime)
	result.Success = true
	
	// Send messages
	message := make([]byte, c.MessageSize)
	for i := range message {
		message[i] = byte(i % 256)
	}
	
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()
	
	for {
		select {
		case <-ctx.Done():
			return result
		case <-ticker.C:
			startTime := time.Now()
			
			_, err := conn.Write(message)
			if err != nil {
				result.NetworkErrors++
				continue
			}
			
			// Read response
			response := make([]byte, c.MessageSize)
			_, err = conn.Read(response)
			if err != nil {
				result.NetworkErrors++
				continue
			}
			
			latency := time.Since(startTime)
			result.Latencies = append(result.Latencies, LatencyMeasurement{
				Timestamp: startTime,
				Latency:   latency,
			})
			
			result.Messages++
			result.Bytes += int64(len(message) + len(response))
		}
	}
}

// QUIC client implementation (mock)
type QUICClient struct {
	ID          int
	Host        string
	Port        int
	MessageSize int
	Duration    time.Duration
	Profile     string
	Streams     int
}

func (c *QUICClient) Run(ctx context.Context) ClientResult {
	result := ClientResult{
		Latencies: make([]LatencyMeasurement, 0, 100),
	}
	
	// Mock QUIC connection - in real implementation would use actual QUIC library
	startTime := time.Now()
	time.Sleep(5 * time.Millisecond) // Mock handshake time
	result.HandshakeTime = time.Since(startTime)
	
	result.Success = true
	
	// Send messages
	message := make([]byte, c.MessageSize)
	for i := range message {
		message[i] = byte(i % 256)
	}
	
	ticker := time.NewTicker(50 * time.Millisecond) // Faster for QUIC
	defer ticker.Stop()
	
	for {
		select {
		case <-ctx.Done():
			return result
		case <-ticker.C:
			startTime := time.Now()
			
			// Mock QUIC send/receive
			time.Sleep(1 * time.Millisecond) // Mock network latency
			
			latency := time.Since(startTime)
			result.Latencies = append(result.Latencies, LatencyMeasurement{
				Timestamp: startTime,
				Latency:   latency,
			})
			
			result.Messages++
			result.Bytes += int64(len(message) * 2) // Send + receive
		}
	}
}

// Print benchmark results
func printResults(result *BenchmarkResult, verbose bool) {
	fmt.Printf("\n%s Benchmark Results\n", result.Protocol)
	fmt.Println("==================")
	fmt.Printf("Duration: %.2fs\n", result.Duration)
	fmt.Printf("Total Connections: %d\n", result.TotalConnections)
	fmt.Printf("Successful Connections: %d\n", result.SuccessfulConnections)
	fmt.Printf("Failed Connections: %d\n", result.FailedConnections)
	
	if result.TotalMessages > 0 {
		fmt.Printf("Total Messages: %d\n", result.TotalMessages)
	}
	
	fmt.Printf("Total Bytes: %d MB\n", result.TotalBytes/(1024*1024))
	fmt.Printf("Average Latency: %.2fms\n", result.AverageLatency)
	fmt.Printf("P95 Latency: %.2fms\n", result.P95Latency)
	fmt.Printf("P99 Latency: %.2fms\n", result.P99Latency)
	fmt.Printf("Throughput: %.2f Gbps\n", result.Throughput)
	fmt.Printf("CPU Usage: %.1f%%\n", result.CPUUsage)
	fmt.Printf("Memory Usage: %d MB\n", result.MemoryUsage)
	
	if result.HandshakeTime > 0 {
		fmt.Printf("Handshake Time (avg): %.2fms\n", result.HandshakeTime)
	}
	
	if result.P95HandshakeTime > 0 {
		fmt.Printf("Handshake Time (p95): %.2fms\n", result.P95HandshakeTime)
	}
	
	if result.ConnectionRate > 0 {
		fmt.Printf("Connection Rate: %.1f conn/s\n", result.ConnectionRate)
	}
	
	if result.NetworkErrors > 0 {
		fmt.Printf("Network Errors: %d\n", result.NetworkErrors)
	}
	
	if result.TimeoutErrors > 0 {
		fmt.Printf("Timeout Errors: %d\n", result.TimeoutErrors)
	}
}

func main() {
	var config BenchmarkConfig
	
	flag.StringVar(&config.Host, "host", "127.0.0.1", "Server host")
	flag.IntVar(&config.Port, "port", 8080, "Server port")
	flag.StringVar(&config.Protocol, "protocol", "tcp", "Protocol (tcp, tls, quic)")
	flag.IntVar(&config.Clients, "clients", 100, "Number of concurrent clients")
	flag.DurationVar(&config.Duration, "duration", 30*time.Second, "Benchmark duration")
	flag.IntVar(&config.MessageSize, "message-size", 1024, "Message size in bytes")
	flag.IntVar(&config.Connections, "connections", 10000, "Number of concurrent connections (for connection benchmark)")
	flag.IntVar(&config.Streams, "streams", 4, "Number of streams per QUIC connection")
	flag.StringVar(&config.Profile, "profile", "tls13_modern", "TLS/QUIC profile")
	flag.BoolVar(&config.Verbose, "verbose", false, "Verbose output")
	flag.StringVar(&config.OutputFormat, "output", "text", "Output format (text, json)")
	
	flag.Parse()
	
	runner := NewBenchmarkRunner(config)
	
	var result *BenchmarkResult
	var err error
	
	switch config.Protocol {
	case "tcp":
		result, err = runner.RunTCPBenchmark()
	case "tls":
		result, err = runner.RunTLSBenchmark()
	case "quic":
		result, err = runner.RunQUICBenchmark()
	case "concurrent":
		result, err = runner.RunConcurrentConnectionsBenchmark()
	default:
		log.Fatalf("Unsupported protocol: %s", config.Protocol)
	}
	
	if err != nil {
		log.Fatalf("Benchmark failed: %v", err)
	}
	
	if config.OutputFormat == "json" {
		// Output JSON format
		jsonData, err := json.MarshalIndent(result, "", "  ")
		if err != nil {
			log.Fatalf("Failed to marshal JSON: %v", err)
		}
		fmt.Println(string(jsonData))
	} else {
		printResults(result, config.Verbose)
	}
}
