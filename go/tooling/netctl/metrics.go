//! Structured Metrics for Aetheris OS Networking
//! 
//! This module provides comprehensive metrics collection and export capabilities,
//! including latency percentiles, throughput, CPU/RSS usage, and error counts.

package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

// MetricType represents the type of metric
type MetricType string

const (
	MetricTypeCounter   MetricType = "counter"
	MetricTypeGauge     MetricType = "gauge"
	MetricTypeHistogram MetricType = "histogram"
	MetricTypeSummary   MetricType = "summary"
)

// MetricValue represents a metric value
type MetricValue struct {
	Type  MetricType `json:"type"`
	Value interface{} `json:"value"`
}

// HistogramData represents histogram data
type HistogramData struct {
	Buckets []HistogramBucket `json:"buckets"`
	Count   uint64            `json:"count"`
	Sum     float64           `json:"sum"`
}

// HistogramBucket represents a histogram bucket
type HistogramBucket struct {
	UpperBound float64 `json:"upper_bound"`
	Count      uint64  `json:"count"`
}

// SummaryData represents summary data
type SummaryData struct {
	Quantiles []Quantile `json:"quantiles"`
	Count     uint64     `json:"count"`
	Sum       float64    `json:"sum"`
}

// Quantile represents a quantile
type Quantile struct {
	Quantile float64 `json:"quantile"`
	Value    float64 `json:"value"`
}

// Metric represents a metric
type Metric struct {
	Name      string                 `json:"name"`
	Help      string                 `json:"help"`
	Type      MetricType             `json:"type"`
	Labels    map[string]string      `json:"labels"`
	Value     MetricValue            `json:"value"`
	Timestamp int64                  `json:"timestamp"`
}

// MetricsConfig represents metrics configuration
type MetricsConfig struct {
	Enabled         bool          `json:"enabled"`
	ExportInterval  time.Duration `json:"export_interval"`
	PrometheusPort  *int          `json:"prometheus_port,omitempty"`
	JSONOutput      *string       `json:"json_output,omitempty"`
	ServiceName     string        `json:"service_name"`
	ServiceVersion  string        `json:"service_version"`
}

// MetricsCollector collects and manages metrics
type MetricsCollector struct {
	config    MetricsConfig
	metrics   map[string]Metric
	metricsMutex sync.RWMutex
	exporters []MetricsExporter
}

// MetricsExporter interface for exporting metrics
type MetricsExporter interface {
	Export(metrics []Metric) error
}

// PrometheusExporter exports metrics in Prometheus format
type PrometheusExporter struct {
	Port int
}

// JsonFileExporter exports metrics to JSON file
type JsonFileExporter struct {
	OutputFile string
}

// NetworkMetricsCollector collects network-specific metrics
type NetworkMetricsCollector struct {
	collector      *MetricsCollector
	requestsTotal  uint64
	errorsTotal    uint64
	bytesTxTotal   uint64
	bytesRxTotal   uint64
	latencySamples []float64
	latencyMutex   sync.RWMutex
	cpuUsage       float64
	memoryUsage    float64
}

// NewMetricsCollector creates a new metrics collector
func NewMetricsCollector(config MetricsConfig) *MetricsCollector {
	return &MetricsCollector{
		config:    config,
		metrics:   make(map[string]Metric),
		exporters: make([]MetricsExporter, 0),
	}
}

// AddExporter adds a metrics exporter
func (mc *MetricsCollector) AddExporter(exporter MetricsExporter) {
	mc.exporters = append(mc.exporters, exporter)
}

// RecordCounter records a counter metric
func (mc *MetricsCollector) RecordCounter(name, help string, value uint64, labels map[string]string) {
	if !mc.config.Enabled {
		return
	}

	metric := Metric{
		Name:      name,
		Help:      help,
		Type:      MetricTypeCounter,
		Labels:    labels,
		Value:     MetricValue{Type: MetricTypeCounter, Value: value},
		Timestamp: time.Now().Unix(),
	}

	mc.metricsMutex.Lock()
	mc.metrics[name] = metric
	mc.metricsMutex.Unlock()
}

// RecordGauge records a gauge metric
func (mc *MetricsCollector) RecordGauge(name, help string, value float64, labels map[string]string) {
	if !mc.config.Enabled {
		return
	}

	metric := Metric{
		Name:      name,
		Help:      help,
		Type:      MetricTypeGauge,
		Labels:    labels,
		Value:     MetricValue{Type: MetricTypeGauge, Value: value},
		Timestamp: time.Now().Unix(),
	}

	mc.metricsMutex.Lock()
	mc.metrics[name] = metric
	mc.metricsMutex.Unlock()
}

// RecordHistogram records a histogram metric
func (mc *MetricsCollector) RecordHistogram(name, help string, values []float64, labels map[string]string) {
	if !mc.config.Enabled {
		return
	}

	// Create histogram buckets
	buckets := make([]HistogramBucket, 0)
	count := uint64(len(values))
	sum := 0.0

	// Define bucket boundaries (exponential)
	bucketBoundaries := []float64{0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0}

	for _, boundary := range bucketBoundaries {
		bucketCount := uint64(0)
		for _, value := range values {
			if value <= boundary {
				bucketCount++
			}
		}
		buckets = append(buckets, HistogramBucket{
			UpperBound: boundary,
			Count:      bucketCount,
		})
	}

	for _, value := range values {
		sum += value
	}

	histogramData := HistogramData{
		Buckets: buckets,
		Count:   count,
		Sum:     sum,
	}

	metric := Metric{
		Name:      name,
		Help:      help,
		Type:      MetricTypeHistogram,
		Labels:    labels,
		Value:     MetricValue{Type: MetricTypeHistogram, Value: histogramData},
		Timestamp: time.Now().Unix(),
	}

	mc.metricsMutex.Lock()
	mc.metrics[name] = metric
	mc.metricsMutex.Unlock()
}

// RecordSummary records a summary metric
func (mc *MetricsCollector) RecordSummary(name, help string, values []float64, labels map[string]string) {
	if !mc.config.Enabled {
		return
	}

	sortedValues := make([]float64, len(values))
	copy(sortedValues, values)
	sort.Float64s(sortedValues)

	// Calculate quantiles
	quantiles := []Quantile{
		{Quantile: 0.5, Value: calculatePercentile(sortedValues, 0.5)},
		{Quantile: 0.9, Value: calculatePercentile(sortedValues, 0.9)},
		{Quantile: 0.95, Value: calculatePercentile(sortedValues, 0.95)},
		{Quantile: 0.99, Value: calculatePercentile(sortedValues, 0.99)},
	}

	count := uint64(len(values))
	sum := 0.0
	for _, value := range values {
		sum += value
	}

	summaryData := SummaryData{
		Quantiles: quantiles,
		Count:     count,
		Sum:       sum,
	}

	metric := Metric{
		Name:      name,
		Help:      help,
		Type:      MetricTypeSummary,
		Labels:    labels,
		Value:     MetricValue{Type: MetricTypeSummary, Value: summaryData},
		Timestamp: time.Now().Unix(),
	}

	mc.metricsMutex.Lock()
	mc.metrics[name] = metric
	mc.metricsMutex.Unlock()
}

// ExportMetrics exports all metrics
func (mc *MetricsCollector) ExportMetrics() error {
	mc.metricsMutex.RLock()
	metrics := make([]Metric, 0, len(mc.metrics))
	for _, metric := range mc.metrics {
		metrics = append(metrics, metric)
	}
	mc.metricsMutex.RUnlock()

	for _, exporter := range mc.exporters {
		if err := exporter.Export(metrics); err != nil {
			return fmt.Errorf("failed to export metrics: %v", err)
		}
	}

	return nil
}

// GetMetrics returns all metrics
func (mc *MetricsCollector) GetMetrics() []Metric {
	mc.metricsMutex.RLock()
	defer mc.metricsMutex.RUnlock()

	metrics := make([]Metric, 0, len(mc.metrics))
	for _, metric := range mc.metrics {
		metrics = append(metrics, metric)
	}

	return metrics
}

// Export method for PrometheusExporter
func (e *PrometheusExporter) Export(metrics []Metric) error {
	// Start HTTP server for Prometheus metrics
	http.HandleFunc("/metrics", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
		
		var output strings.Builder
		
		// Group metrics by type
		counters := make([]Metric, 0)
		gauges := make([]Metric, 0)
		histograms := make([]Metric, 0)
		summaries := make([]Metric, 0)
		
		for _, metric := range metrics {
			switch metric.Type {
			case MetricTypeCounter:
				counters = append(counters, metric)
			case MetricTypeGauge:
				gauges = append(gauges, metric)
			case MetricTypeHistogram:
				histograms = append(histograms, metric)
			case MetricTypeSummary:
				summaries = append(summaries, metric)
			}
		}
		
		// Export counters
		for _, metric := range counters {
			output.WriteString(fmt.Sprintf("# HELP %s %s\n", metric.Name, metric.Help))
			output.WriteString(fmt.Sprintf("# TYPE %s counter\n", metric.Name))
			
			labelsStr := ""
			if len(metric.Labels) > 0 {
				labels := make([]string, 0, len(metric.Labels))
				for k, v := range metric.Labels {
					labels = append(labels, fmt.Sprintf("%s=\"%s\"", k, v))
				}
				labelsStr = fmt.Sprintf("{%s}", strings.Join(labels, ","))
			}
			
			output.WriteString(fmt.Sprintf("%s%s %v\n", metric.Name, labelsStr, metric.Value.Value))
		}
		
		// Export gauges
		for _, metric := range gauges {
			output.WriteString(fmt.Sprintf("# HELP %s %s\n", metric.Name, metric.Help))
			output.WriteString(fmt.Sprintf("# TYPE %s gauge\n", metric.Name))
			
			labelsStr := ""
			if len(metric.Labels) > 0 {
				labels := make([]string, 0, len(metric.Labels))
				for k, v := range metric.Labels {
					labels = append(labels, fmt.Sprintf("%s=\"%s\"", k, v))
				}
				labelsStr = fmt.Sprintf("{%s}", strings.Join(labels, ","))
			}
			
			output.WriteString(fmt.Sprintf("%s%s %v\n", metric.Name, labelsStr, metric.Value.Value))
		}
		
		// Export histograms
		for _, metric := range histograms {
			output.WriteString(fmt.Sprintf("# HELP %s %s\n", metric.Name, metric.Help))
			output.WriteString(fmt.Sprintf("# TYPE %s histogram\n", metric.Name))
			
			labelsStr := ""
			if len(metric.Labels) > 0 {
				labels := make([]string, 0, len(metric.Labels))
				for k, v := range metric.Labels {
					labels = append(labels, fmt.Sprintf("%s=\"%s\"", k, v))
				}
				labelsStr = fmt.Sprintf("{%s}", strings.Join(labels, ","))
			}
			
			if histData, ok := metric.Value.Value.(HistogramData); ok {
				// Export buckets
				for _, bucket := range histData.Buckets {
					output.WriteString(fmt.Sprintf("%s_bucket%sle=\"%f\" %d\n", 
						metric.Name, labelsStr, bucket.UpperBound, bucket.Count))
				}
				
				// Export count and sum
				output.WriteString(fmt.Sprintf("%s_count%s %d\n", metric.Name, labelsStr, histData.Count))
				output.WriteString(fmt.Sprintf("%s_sum%s %f\n", metric.Name, labelsStr, histData.Sum))
			}
		}
		
		// Export summaries
		for _, metric := range summaries {
			output.WriteString(fmt.Sprintf("# HELP %s %s\n", metric.Name, metric.Help))
			output.WriteString(fmt.Sprintf("# TYPE %s summary\n", metric.Name))
			
			labelsStr := ""
			if len(metric.Labels) > 0 {
				labels := make([]string, 0, len(metric.Labels))
				for k, v := range metric.Labels {
					labels = append(labels, fmt.Sprintf("%s=\"%s\"", k, v))
				}
				labelsStr = fmt.Sprintf("{%s}", strings.Join(labels, ","))
			}
			
			if summaryData, ok := metric.Value.Value.(SummaryData); ok {
				// Export quantiles
				for _, quantile := range summaryData.Quantiles {
					output.WriteString(fmt.Sprintf("%s{quantile=\"%f\"} %f\n", 
						metric.Name, quantile.Quantile, quantile.Value))
				}
				
				// Export count and sum
				output.WriteString(fmt.Sprintf("%s_count%s %d\n", metric.Name, labelsStr, summaryData.Count))
				output.WriteString(fmt.Sprintf("%s_sum%s %f\n", metric.Name, labelsStr, summaryData.Sum))
			}
		}
		
		w.Write([]byte(output.String()))
	})
	
	// Start server in a goroutine
	go func() {
		log.Printf("Starting Prometheus metrics server on port %d", e.Port)
		if err := http.ListenAndServe(fmt.Sprintf(":%d", e.Port), nil); err != nil {
			log.Printf("Failed to start Prometheus metrics server: %v", err)
		}
	}()
	
	return nil
}

// Export method for JsonFileExporter
func (e *JsonFileExporter) Export(metrics []Metric) error {
	// Create output directory if it doesn't exist
	if err := os.MkdirAll(strings.TrimSuffix(e.OutputFile, "/"), 0755); err != nil {
		return err
	}

	file, err := os.Create(e.OutputFile)
	if err != nil {
		return err
	}
	defer file.Close()

	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	return encoder.Encode(metrics)
}

// NewNetworkMetricsCollector creates a new network metrics collector
func NewNetworkMetricsCollector(config MetricsConfig) *NetworkMetricsCollector {
	collector := NewMetricsCollector(config)
	
	// Add exporters
	if config.PrometheusPort != nil {
		collector.AddExporter(&PrometheusExporter{Port: *config.PrometheusPort})
	}
	
	if config.JSONOutput != nil {
		collector.AddExporter(&JsonFileExporter{OutputFile: *config.JSONOutput})
	}
	
	return &NetworkMetricsCollector{
		collector:      collector,
		latencySamples: make([]float64, 0),
	}
}

// RecordRequest records a network request
func (nmc *NetworkMetricsCollector) RecordRequest(latencyMs float64, bytesTx, bytesRx int) {
	nmc.requestsTotal++
	nmc.bytesTxTotal += uint64(bytesTx)
	nmc.bytesRxTotal += uint64(bytesRx)
	
	nmc.latencyMutex.Lock()
	nmc.latencySamples = append(nmc.latencySamples, latencyMs)
	nmc.latencyMutex.Unlock()
}

// RecordError records a network error
func (nmc *NetworkMetricsCollector) RecordError() {
	nmc.errorsTotal++
}

// UpdateSystemMetrics updates system metrics
func (nmc *NetworkMetricsCollector) UpdateSystemMetrics(cpuPct, memoryMB float64) {
	nmc.cpuUsage = cpuPct
	nmc.memoryUsage = memoryMB
}

// ExportNetworkMetrics exports network metrics
func (nmc *NetworkMetricsCollector) ExportNetworkMetrics() error {
	// Record counter metrics
	nmc.collector.RecordCounter(
		"network_requests_total",
		"Total number of network requests",
		nmc.requestsTotal,
		make(map[string]string),
	)

	nmc.collector.RecordCounter(
		"network_errors_total",
		"Total number of network errors",
		nmc.errorsTotal,
		make(map[string]string),
	)

	nmc.collector.RecordCounter(
		"network_bytes_tx_total",
		"Total bytes transmitted",
		nmc.bytesTxTotal,
		make(map[string]string),
	)

	nmc.collector.RecordCounter(
		"network_bytes_rx_total",
		"Total bytes received",
		nmc.bytesRxTotal,
		make(map[string]string),
	)

	// Record gauge metrics
	nmc.collector.RecordGauge(
		"network_cpu_usage_percent",
		"CPU usage percentage",
		nmc.cpuUsage,
		make(map[string]string),
	)

	nmc.collector.RecordGauge(
		"network_memory_usage_mb",
		"Memory usage in MB",
		nmc.memoryUsage,
		make(map[string]string),
	)

	// Record latency histogram
	nmc.latencyMutex.RLock()
	latencySamples := make([]float64, len(nmc.latencySamples))
	copy(latencySamples, nmc.latencySamples)
	nmc.latencyMutex.RUnlock()

	if len(latencySamples) > 0 {
		nmc.collector.RecordHistogram(
			"network_latency_ms",
			"Network latency in milliseconds",
			latencySamples,
			make(map[string]string),
		)
	}

	// Export all metrics
	return nmc.collector.ExportMetrics()
}

// StartCollectionLoop starts the metrics collection loop
func (nmc *NetworkMetricsCollector) StartCollectionLoop() {
	ticker := time.NewTicker(nmc.collector.config.ExportInterval)
	defer ticker.Stop()

	for range ticker.C {
		if err := nmc.ExportNetworkMetrics(); err != nil {
			log.Printf("Failed to export network metrics: %v", err)
		}
	}
}

// Helper function to calculate percentile
func calculatePercentile(sortedValues []float64, percentile float64) float64 {
	if len(sortedValues) == 0 {
		return 0.0
	}

	index := int(percentile * float64(len(sortedValues)-1))
	if index >= len(sortedValues) {
		index = len(sortedValues) - 1
	}

	return sortedValues[index]
}

// handleMetrics handles the metrics command
func handleMetrics(args []string) error {
	if len(args) < 1 {
		printMetricsUsage()
		return fmt.Errorf("metrics command required")
	}

	subcommand := args[0]
	switch subcommand {
	case "serve":
		return handleMetricsServe(args[1:])
	case "export":
		return handleMetricsExport(args[1:])
	case "collect":
		return handleMetricsCollect(args[1:])
	default:
		printMetricsUsage()
		return fmt.Errorf("unknown metrics command: %s", subcommand)
	}
}

// handleMetricsServe handles the metrics serve command
func handleMetricsServe(args []string) error {
	fs := flag.NewFlagSet("metrics serve", flag.ExitOnError)
	
	var (
		port         = fs.Int("port", 9090, "Port for Prometheus metrics server")
		exportInterval = fs.Duration("export-interval", 15*time.Second, "Export interval")
		jsonOutput   = fs.String("json", "metrics/network_metrics.json", "JSON output file")
		serviceName  = fs.String("service", "aetheris-net", "Service name")
		serviceVersion = fs.String("version", "1.0.0", "Service version")
	)

	if err := fs.Parse(args); err != nil {
		return err
	}

	config := MetricsConfig{
		Enabled:        true,
		ExportInterval: *exportInterval,
		PrometheusPort: port,
		JSONOutput:     jsonOutput,
		ServiceName:    *serviceName,
		ServiceVersion: *serviceVersion,
	}

	collector := NewNetworkMetricsCollector(config)
	
	// Start collection loop
	go collector.StartCollectionLoop()
	
	fmt.Printf("Starting metrics server on port %d\n", *port)
	fmt.Printf("Prometheus metrics available at: http://localhost:%d/metrics\n", *port)
	fmt.Printf("JSON metrics output: %s\n", *jsonOutput)
	fmt.Printf("Export interval: %v\n", *exportInterval)
	
	// Keep the server running
	select {}
}

// handleMetricsExport handles the metrics export command
func handleMetricsExport(args []string) error {
	fs := flag.NewFlagSet("metrics export", flag.ExitOnError)
	
	var (
		outputFile = fs.String("output", "metrics/exported_metrics.json", "Output file for exported metrics")
		format     = fs.String("format", "json", "Export format (json, prometheus)")
	)

	if err := fs.Parse(args); err != nil {
		return err
	}

	fmt.Printf("Exporting metrics to: %s\n", *outputFile)
	fmt.Printf("Format: %s\n", *format)
	
	// This would export existing metrics
	// For now, just print a message
	return nil
}

// handleMetricsCollect handles the metrics collect command
func handleMetricsCollect(args []string) error {
	fs := flag.NewFlagSet("metrics collect", flag.ExitOnError)
	
	var (
		duration = fs.Duration("duration", 60*time.Second, "Collection duration")
		interval = fs.Duration("interval", 1*time.Second, "Collection interval")
		outputFile = fs.String("output", "metrics/collected_metrics.json", "Output file for collected metrics")
	)

	if err := fs.Parse(args); err != nil {
		return err
	}

	config := MetricsConfig{
		Enabled:        true,
		ExportInterval: *interval,
		JSONOutput:     outputFile,
		ServiceName:    "aetheris-net",
		ServiceVersion: "1.0.0",
	}

	collector := NewNetworkMetricsCollector(config)
	
	fmt.Printf("Collecting metrics for %v...\n", *duration)
	fmt.Printf("Collection interval: %v\n", *interval)
	fmt.Printf("Output file: %s\n", *outputFile)
	
	// Simulate some network activity
	go func() {
		ticker := time.NewTicker(*interval)
		defer ticker.Stop()
		
		start := time.Now()
		for time.Since(start) < *duration {
			select {
			case <-ticker.C:
				// Simulate network requests
				latency := 10.0 + float64(time.Now().UnixNano()%1000)/100.0
				bytesTx := 1024 + int(time.Now().UnixNano()%1000)
				bytesRx := 1024 + int(time.Now().UnixNano()%1000)
				
				collector.RecordRequest(latency, bytesTx, bytesRx)
				
				// Simulate occasional errors
				if time.Now().UnixNano()%100 < 5 {
					collector.RecordError()
				}
				
				// Update system metrics
				cpuUsage := 25.0 + float64(time.Now().UnixNano()%50)
				memoryUsage := 128.0 + float64(time.Now().UnixNano()%100)
				collector.UpdateSystemMetrics(cpuUsage, memoryUsage)
			}
		}
	}()
	
	// Export metrics periodically
	ticker := time.NewTicker(*interval)
	defer ticker.Stop()
	
	start := time.Now()
	for time.Since(start) < *duration {
		select {
		case <-ticker.C:
			if err := collector.ExportNetworkMetrics(); err != nil {
				log.Printf("Failed to export metrics: %v", err)
			}
		}
	}
	
	fmt.Println("Metrics collection completed")
	return nil
}

// printMetricsUsage prints the metrics command usage
func printMetricsUsage() {
	fmt.Println(`
Metrics Commands:

  serve     Start metrics server (Prometheus + JSON)
  export    Export collected metrics
  collect   Collect metrics for a specified duration

Examples:
  ./netctl metrics serve --port 9090 --json metrics/network_metrics.json
  ./netctl metrics export --output metrics/exported.json --format json
  ./netctl metrics collect --duration 5m --interval 1s --output metrics/collected.json
`)
}
