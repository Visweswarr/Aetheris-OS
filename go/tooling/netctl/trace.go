//! End-to-End Tracing for Aetheris OS Networking
//! 
//! This module provides OpenTelemetry-compatible tracing across all networking
//! layers, from syscalls through the Rust broker to Go CLI and TypeScript bridge.

package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"sync"
	"time"
	"context"
	"strings"
)

// TraceContext represents a trace context for correlating operations
type TraceContext struct {
	TraceID       string            `json:"trace_id"`
	SpanID        string            `json:"span_id"`
	ParentSpanID  *string           `json:"parent_span_id,omitempty"`
	Baggage       map[string]string `json:"baggage"`
}

// SpanKind represents the kind of span
type SpanKind int

const (
	SpanKindInternal SpanKind = iota
	SpanKindServer
	SpanKindClient
	SpanKindProducer
	SpanKindConsumer
)

// StatusCode represents the status code of a span
type StatusCode int

const (
	StatusCodeUnset StatusCode = iota
	StatusCodeOk
	StatusCodeError
)

// AttributeValue represents an attribute value
type AttributeValue struct {
	Type  string      `json:"type"`
	Value interface{} `json:"value"`
}

// SpanEvent represents a span event
type SpanEvent struct {
	Name       string                    `json:"name"`
	Timestamp  int64                     `json:"timestamp"`
	Attributes map[string]AttributeValue `json:"attributes"`
}

// SpanLink represents a span link
type SpanLink struct {
	TraceID    string                    `json:"trace_id"`
	SpanID     string                    `json:"span_id"`
	Attributes map[string]AttributeValue `json:"attributes"`
}

// Span represents a span in OpenTelemetry format
type Span struct {
	TraceID       string                    `json:"trace_id"`
	SpanID        string                    `json:"span_id"`
	ParentSpanID  *string                   `json:"parent_span_id,omitempty"`
	Name          string                    `json:"name"`
	Kind          SpanKind                  `json:"kind"`
	StartTime     int64                     `json:"start_time"`
	EndTime       *int64                    `json:"end_time,omitempty"`
	Duration      *int64                    `json:"duration,omitempty"`
	Status        SpanStatus                `json:"status"`
	Attributes    map[string]AttributeValue `json:"attributes"`
	Events        []SpanEvent               `json:"events"`
	Links         []SpanLink                `json:"links"`
}

// SpanStatus represents the status of a span
type SpanStatus struct {
	Code    StatusCode `json:"code"`
	Message *string    `json:"message,omitempty"`
}

// TraceConfig represents trace configuration
type TraceConfig struct {
	Enabled       bool          `json:"enabled"`
	SampleRate    float64       `json:"sample_rate"`
	MaxSpans      int           `json:"max_spans"`
	ExportInterval time.Duration `json:"export_interval"`
	OutputFile    string        `json:"output_file"`
	ServiceName   string        `json:"service_name"`
	ServiceVersion string       `json:"service_version"`
}

// TraceManager manages traces and spans
type TraceManager struct {
	config        TraceConfig
	spans         []Span
	spansMutex    sync.RWMutex
	exporters     []TraceExporter
}

// TraceExporter interface for exporting traces
type TraceExporter interface {
	Export(spans []Span) error
}

// JsonFileExporter exports traces to JSON file
type JsonFileExporter struct {
	OutputFile string
}

// OpenTelemetryExporter exports traces in OpenTelemetry format
type OpenTelemetryExporter struct {
	OutputFile string
}

// NewTraceManager creates a new trace manager
func NewTraceManager(config TraceConfig) *TraceManager {
	return &TraceManager{
		config:    config,
		spans:     make([]Span, 0),
		exporters: make([]TraceExporter, 0),
	}
}

// AddExporter adds a trace exporter
func (tm *TraceManager) AddExporter(exporter TraceExporter) {
	tm.exporters = append(tm.exporters, exporter)
}

// StartSpan starts a new span
func (tm *TraceManager) StartSpan(name string, kind SpanKind, ctx *TraceContext) *SpanHandle {
	if !tm.config.Enabled {
		return &SpanHandle{disabled: true}
	}

	spanID := generateSpanID()
	startTime := time.Now().UnixNano()

	span := Span{
		TraceID:    ctx.TraceID,
		SpanID:     spanID,
		ParentSpanID: ctx.ParentSpanID,
		Name:       name,
		Kind:       kind,
		StartTime:  startTime,
		Status:     SpanStatus{Code: StatusCodeUnset},
		Attributes: make(map[string]AttributeValue),
		Events:     make([]SpanEvent, 0),
		Links:      make([]SpanLink, 0),
	}

	tm.spansMutex.Lock()
	tm.spans = append(tm.spans, span)
	tm.spansMutex.Unlock()

	return &SpanHandle{
		spanID:  spanID,
		manager: tm,
	}
}

// ExportTraces exports all traces
func (tm *TraceManager) ExportTraces() error {
	tm.spansMutex.RLock()
	spans := make([]Span, len(tm.spans))
	copy(spans, tm.spans)
	tm.spansMutex.RUnlock()

	for _, exporter := range tm.exporters {
		if err := exporter.Export(spans); err != nil {
			return fmt.Errorf("failed to export traces: %v", err)
		}
	}

	return nil
}

// ClearSpans clears all spans
func (tm *TraceManager) ClearSpans() {
	tm.spansMutex.Lock()
	tm.spans = tm.spans[:0]
	tm.spansMutex.Unlock()
}

// SpanHandle represents a handle to a span
type SpanHandle struct {
	spanID   string
	manager  *TraceManager
	disabled bool
}

// AddAttribute adds an attribute to the span
func (sh *SpanHandle) AddAttribute(key string, value interface{}) {
	if sh.disabled {
		return
	}

	sh.manager.spansMutex.Lock()
	defer sh.manager.spansMutex.Unlock()

	for i := range sh.manager.spans {
		if sh.manager.spans[i].SpanID == sh.spanID {
			attrValue := AttributeValue{
				Type:  getAttributeType(value),
				Value: value,
			}
			sh.manager.spans[i].Attributes[key] = attrValue
			break
		}
	}
}

// AddEvent adds an event to the span
func (sh *SpanHandle) AddEvent(name string, attributes map[string]interface{}) {
	if sh.disabled {
		return
	}

	sh.manager.spansMutex.Lock()
	defer sh.manager.spansMutex.Unlock()

	event := SpanEvent{
		Name:       name,
		Timestamp:  time.Now().UnixNano(),
		Attributes: make(map[string]AttributeValue),
	}

	for k, v := range attributes {
		event.Attributes[k] = AttributeValue{
			Type:  getAttributeType(v),
			Value: v,
		}
	}

	for i := range sh.manager.spans {
		if sh.manager.spans[i].SpanID == sh.spanID {
			sh.manager.spans[i].Events = append(sh.manager.spans[i].Events, event)
			break
		}
	}
}

// SetStatus sets the status of the span
func (sh *SpanHandle) SetStatus(code StatusCode, message *string) {
	if sh.disabled {
		return
	}

	sh.manager.spansMutex.Lock()
	defer sh.manager.spansMutex.Unlock()

	for i := range sh.manager.spans {
		if sh.manager.spans[i].SpanID == sh.spanID {
			sh.manager.spans[i].Status = SpanStatus{
				Code:    code,
				Message: message,
			}
			break
		}
	}
}

// Finish finishes the span
func (sh *SpanHandle) Finish() {
	if sh.disabled {
		return
	}

	endTime := time.Now().UnixNano()

	sh.manager.spansMutex.Lock()
	defer sh.manager.spansMutex.Unlock()

	for i := range sh.manager.spans {
		if sh.manager.spans[i].SpanID == sh.spanID {
			sh.manager.spans[i].EndTime = &endTime
			duration := endTime - sh.manager.spans[i].StartTime
			sh.manager.spans[i].Duration = &duration
			break
		}
	}
}

// Export method for JsonFileExporter
func (e *JsonFileExporter) Export(spans []Span) error {
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
	return encoder.Encode(spans)
}

// Export method for OpenTelemetryExporter
func (e *OpenTelemetryExporter) Export(spans []Span) error {
	// Create output directory if it doesn't exist
	if err := os.MkdirAll(strings.TrimSuffix(e.OutputFile, "/"), 0755); err != nil {
		return err
	}

	file, err := os.Create(e.OutputFile)
	if err != nil {
		return err
	}
	defer file.Close()

	// Convert to OpenTelemetry format
	otelSpans := make([]OpenTelemetrySpan, len(spans))
	for i, span := range spans {
		otelSpans[i] = convertToOpenTelemetry(span)
	}

	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	return encoder.Encode(otelSpans)
}

// OpenTelemetrySpan represents a span in OpenTelemetry format
type OpenTelemetrySpan struct {
	TraceID           string                    `json:"traceId"`
	SpanID            string                    `json:"spanId"`
	ParentSpanID      *string                   `json:"parentSpanId,omitempty"`
	Name              string                    `json:"name"`
	Kind              int                       `json:"kind"`
	StartTimeUnixNano string                    `json:"startTimeUnixNano"`
	EndTimeUnixNano   *string                   `json:"endTimeUnixNano,omitempty"`
	DurationNano      *string                   `json:"durationNano,omitempty"`
	Status            OpenTelemetryStatus       `json:"status"`
	Attributes        []OpenTelemetryAttribute  `json:"attributes"`
	Events            []OpenTelemetryEvent      `json:"events"`
	Links             []OpenTelemetryLink       `json:"links"`
}

// OpenTelemetryStatus represents status in OpenTelemetry format
type OpenTelemetryStatus struct {
	Code    int     `json:"code"`
	Message *string `json:"message,omitempty"`
}

// OpenTelemetryAttribute represents an attribute in OpenTelemetry format
type OpenTelemetryAttribute struct {
	Key   string                 `json:"key"`
	Value OpenTelemetryAttrValue `json:"value"`
}

// OpenTelemetryAttrValue represents an attribute value in OpenTelemetry format
type OpenTelemetryAttrValue struct {
	Type  string      `json:"type"`
	Value interface{} `json:"value"`
}

// OpenTelemetryEvent represents an event in OpenTelemetry format
type OpenTelemetryEvent struct {
	Name       string                    `json:"name"`
	TimeUnixNano string                  `json:"timeUnixNano"`
	Attributes []OpenTelemetryAttribute  `json:"attributes"`
}

// OpenTelemetryLink represents a link in OpenTelemetry format
type OpenTelemetryLink struct {
	TraceID    string                   `json:"traceId"`
	SpanID     string                   `json:"spanId"`
	Attributes []OpenTelemetryAttribute `json:"attributes"`
}

// convertToOpenTelemetry converts a span to OpenTelemetry format
func convertToOpenTelemetry(span Span) OpenTelemetrySpan {
	otelSpan := OpenTelemetrySpan{
		TraceID:           span.TraceID,
		SpanID:            span.SpanID,
		ParentSpanID:      span.ParentSpanID,
		Name:              span.Name,
		Kind:              int(span.Kind),
		StartTimeUnixNano: fmt.Sprintf("%d", span.StartTime),
		Status: OpenTelemetryStatus{
			Code:    int(span.Status.Code),
			Message: span.Status.Message,
		},
		Attributes: make([]OpenTelemetryAttribute, 0),
		Events:     make([]OpenTelemetryEvent, 0),
		Links:      make([]OpenTelemetryLink, 0),
	}

	if span.EndTime != nil {
		endTimeStr := fmt.Sprintf("%d", *span.EndTime)
		otelSpan.EndTimeUnixNano = &endTimeStr
	}

	if span.Duration != nil {
		durationStr := fmt.Sprintf("%d", *span.Duration)
		otelSpan.DurationNano = &durationStr
	}

	// Convert attributes
	for k, v := range span.Attributes {
		otelSpan.Attributes = append(otelSpan.Attributes, OpenTelemetryAttribute{
			Key: k,
			Value: OpenTelemetryAttrValue{
				Type:  v.Type,
				Value: v.Value,
			},
		})
	}

	// Convert events
	for _, event := range span.Events {
		otelEvent := OpenTelemetryEvent{
			Name:         event.Name,
			TimeUnixNano: fmt.Sprintf("%d", event.Timestamp),
			Attributes:   make([]OpenTelemetryAttribute, 0),
		}

		for k, v := range event.Attributes {
			otelEvent.Attributes = append(otelEvent.Attributes, OpenTelemetryAttribute{
				Key: k,
				Value: OpenTelemetryAttrValue{
					Type:  v.Type,
					Value: v.Value,
				},
			})
		}

		otelSpan.Events = append(otelSpan.Events, otelEvent)
	}

	// Convert links
	for _, link := range span.Links {
		otelLink := OpenTelemetryLink{
			TraceID:    link.TraceID,
			SpanID:     link.SpanID,
			Attributes: make([]OpenTelemetryAttribute, 0),
		}

		for k, v := range link.Attributes {
			otelLink.Attributes = append(otelLink.Attributes, OpenTelemetryAttribute{
				Key: k,
				Value: OpenTelemetryAttrValue{
					Type:  v.Type,
					Value: v.Value,
				},
			})
		}

		otelSpan.Links = append(otelSpan.Links, otelLink)
	}

	return otelSpan
}

// Helper functions
func generateSpanID() string {
	return fmt.Sprintf("%d", time.Now().UnixNano())
}

func getAttributeType(value interface{}) string {
	switch value.(type) {
	case string:
		return "string"
	case bool:
		return "bool"
	case int, int8, int16, int32, int64:
		return "int"
	case uint, uint8, uint16, uint32, uint64:
		return "int"
	case float32, float64:
		return "double"
	case []string:
		return "stringArray"
	case []bool:
		return "boolArray"
	case []int, []int8, []int16, []int32, []int64:
		return "intArray"
	case []uint, []uint8, []uint16, []uint32, []uint64:
		return "intArray"
	case []float32, []float64:
		return "doubleArray"
	default:
		return "string"
	}
}

// handleTrace handles the trace command
func handleTrace(args []string) error {
	if len(args) < 1 {
		printTraceUsage()
		return fmt.Errorf("trace command required")
	}

	subcommand := args[0]
	switch subcommand {
	case "start":
		return handleTraceStart(args[1:])
	case "stop":
		return handleTraceStop(args[1:])
	case "export":
		return handleTraceExport(args[1:])
	default:
		printTraceUsage()
		return fmt.Errorf("unknown trace command: %s", subcommand)
	}
}

// handleTraceStart handles the trace start command
func handleTraceStart(args []string) error {
	fs := flag.NewFlagSet("trace start", flag.ExitOnError)
	
	var (
		outputDir     = fs.String("output", "traces/", "Output directory for trace files")
		sampleRate    = fs.Float64("sample-rate", 1.0, "Sample rate (0.0 to 1.0)")
		maxSpans      = fs.Int("max-spans", 10000, "Maximum number of spans")
		exportInterval = fs.Duration("export-interval", 30*time.Second, "Export interval")
		serviceName   = fs.String("service", "aetheris-net", "Service name")
		serviceVersion = fs.String("version", "1.0.0", "Service version")
	)

	if err := fs.Parse(args); err != nil {
		return err
	}

	config := TraceConfig{
		Enabled:        true,
		SampleRate:     *sampleRate,
		MaxSpans:       *maxSpans,
		ExportInterval: *exportInterval,
		OutputFile:     *outputDir + "network_traces.json",
		ServiceName:    *serviceName,
		ServiceVersion: *serviceVersion,
	}

	manager := NewTraceManager(config)
	
	// Add exporters
	manager.AddExporter(&JsonFileExporter{OutputFile: config.OutputFile})
	manager.AddExporter(&OpenTelemetryExporter{OutputFile: *outputDir + "otel_traces.json"})

	// Start tracing
	fmt.Printf("Starting trace collection...\n")
	fmt.Printf("Output directory: %s\n", *outputDir)
	fmt.Printf("Sample rate: %.2f\n", *sampleRate)
	fmt.Printf("Max spans: %d\n", *maxSpans)
	fmt.Printf("Export interval: %v\n", *exportInterval)

	// Create a sample trace
	ctx := &TraceContext{
		TraceID:      generateSpanID(),
		SpanID:       generateSpanID(),
		ParentSpanID: nil,
		Baggage:      make(map[string]string),
	}

	span := manager.StartSpan("network_operation", SpanKindClient, ctx)
	span.AddAttribute("operation", "trace_start")
	span.AddAttribute("service", *serviceName)
	span.AddAttribute("version", *serviceVersion)
	span.AddEvent("trace_started", map[string]interface{}{
		"timestamp": time.Now().Unix(),
	})
	span.SetStatus(StatusCodeOk, nil)
	span.Finish()

	// Export traces
	if err := manager.ExportTraces(); err != nil {
		return fmt.Errorf("failed to export traces: %v", err)
	}

	fmt.Println("Trace collection started successfully")
	return nil
}

// handleTraceStop handles the trace stop command
func handleTraceStop(args []string) error {
	fmt.Println("Stopping trace collection...")
	// This would stop active trace collection
	// For now, just print a message
	return nil
}

// handleTraceExport handles the trace export command
func handleTraceExport(args []string) error {
	fs := flag.NewFlagSet("trace export", flag.ExitOnError)
	
	var (
		outputFile = fs.String("output", "traces/exported_traces.json", "Output file for exported traces")
		format     = fs.String("format", "json", "Export format (json, otel)")
	)

	if err := fs.Parse(args); err != nil {
		return err
	}

	fmt.Printf("Exporting traces to: %s\n", *outputFile)
	fmt.Printf("Format: %s\n", *format)
	
	// This would export existing traces
	// For now, just print a message
	return nil
}

// printTraceUsage prints the trace command usage
func printTraceUsage() {
	fmt.Println(`
Tracing Commands:

  start     Start trace collection
  stop      Stop trace collection
  export    Export collected traces

Examples:
  ./netctl trace start --output traces/ --sample-rate 1.0
  ./netctl trace stop
  ./netctl trace export --output traces/exported.json --format otel
`)
}
