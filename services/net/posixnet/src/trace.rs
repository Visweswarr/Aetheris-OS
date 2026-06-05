//! End-to-End Tracing for Aetheris OS Networking
//! 
//! This module provides OpenTelemetry-compatible tracing across all networking
//! layers, from syscalls through the Rust broker to Go CLI and TypeScript bridge.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Trace context for correlating operations across layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }

    /// Create a child span context
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
        }
    }

    /// Add baggage to the context
    pub fn add_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }

    /// Get baggage from the context
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Span data for OpenTelemetry format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time: u64, // nanoseconds since epoch
    pub end_time: Option<u64>,
    pub duration: Option<u64>, // nanoseconds
    pub status: SpanStatus,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
    pub links: Vec<SpanLink>,
}

/// Span kind enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

/// Span status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanStatus {
    pub code: StatusCode,
    pub message: Option<String>,
}

/// Status code enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusCode {
    Unset,
    Ok,
    Error,
}

/// Attribute value for spans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    StringArray(Vec<String>),
    BoolArray(Vec<bool>),
    IntArray(Vec<i64>),
    FloatArray(Vec<f64>),
}

/// Span event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: u64,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Span link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    pub trace_id: String,
    pub span_id: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Trace manager for handling spans and exporting traces
pub struct TraceManager {
    spans: Arc<RwLock<Vec<Span>>>,
    config: TraceConfig,
    exporter: Option<Box<dyn TraceExporter + Send + Sync>>,
}

/// Trace configuration
#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub enabled: bool,
    pub sample_rate: f64, // 0.0 to 1.0
    pub max_spans: usize,
    pub export_interval: Duration,
    pub output_file: Option<String>,
    pub service_name: String,
    pub service_version: String,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 1.0,
            max_spans: 10000,
            export_interval: Duration::from_secs(30),
            output_file: Some("traces/network_traces.json".to_string()),
            service_name: "aetheris-net".to_string(),
            service_version: "1.0.0".to_string(),
        }
    }
}

/// Trace exporter trait
pub trait TraceExporter: Send + Sync {
    fn export(&self, spans: &[Span]) -> Result<(), Box<dyn std::error::Error>>;
}

/// JSON file trace exporter
pub struct JsonFileExporter {
    output_file: String,
}

impl JsonFileExporter {
    pub fn new(output_file: String) -> Self {
        Self { output_file }
    }
}

impl TraceExporter for JsonFileExporter {
    fn export(&self, spans: &[Span]) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.output_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write spans to JSON file
        let mut file = File::create(&self.output_file)?;
        let json = serde_json::to_string_pretty(spans)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
}

/// OpenTelemetry JSON exporter
pub struct OpenTelemetryExporter {
    output_file: String,
}

impl OpenTelemetryExporter {
    pub fn new(output_file: String) -> Self {
        Self { output_file }
    }
}

impl TraceExporter for OpenTelemetryExporter {
    fn export(&self, spans: &[Span]) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.output_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Convert to OpenTelemetry format
        let otel_spans: Vec<OpenTelemetrySpan> = spans.iter().map(|span| span.into()).collect();
        
        // Write to JSON file
        let mut file = File::create(&self.output_file)?;
        let json = serde_json::to_string_pretty(&otel_spans)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
}

/// OpenTelemetry span format
#[derive(Debug, Serialize, Deserialize)]
struct OpenTelemetrySpan {
    #[serde(rename = "traceId")]
    trace_id: String,
    #[serde(rename = "spanId")]
    span_id: String,
    #[serde(rename = "parentSpanId", skip_serializing_if = "Option::is_none")]
    parent_span_id: Option<String>,
    name: String,
    kind: i32,
    #[serde(rename = "startTimeUnixNano")]
    start_time_unix_nano: String,
    #[serde(rename = "endTimeUnixNano", skip_serializing_if = "Option::is_none")]
    end_time_unix_nano: Option<String>,
    #[serde(rename = "durationNano", skip_serializing_if = "Option::is_none")]
    duration_nano: Option<String>,
    status: OpenTelemetryStatus,
    attributes: Vec<OpenTelemetryAttribute>,
    events: Vec<OpenTelemetryEvent>,
    links: Vec<OpenTelemetryLink>,
}

/// OpenTelemetry status format
#[derive(Debug, Serialize, Deserialize)]
struct OpenTelemetryStatus {
    code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// OpenTelemetry attribute format
#[derive(Debug, Serialize, Deserialize)]
struct OpenTelemetryAttribute {
    key: String,
    value: OpenTelemetryAttributeValue,
}

/// OpenTelemetry attribute value format
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum OpenTelemetryAttributeValue {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "int")]
    Int(i64),
    #[serde(rename = "double")]
    Double(f64),
    #[serde(rename = "stringArray")]
    StringArray(Vec<String>),
    #[serde(rename = "boolArray")]
    BoolArray(Vec<bool>),
    #[serde(rename = "intArray")]
    IntArray(Vec<i64>),
    #[serde(rename = "doubleArray")]
    DoubleArray(Vec<f64>),
}

/// OpenTelemetry event format
#[derive(Debug, Serialize, Deserialize)]
struct OpenTelemetryEvent {
    name: String,
    #[serde(rename = "timeUnixNano")]
    time_unix_nano: String,
    attributes: Vec<OpenTelemetryAttribute>,
}

/// OpenTelemetry link format
#[derive(Debug, Serialize, Deserialize)]
struct OpenTelemetryLink {
    #[serde(rename = "traceId")]
    trace_id: String,
    #[serde(rename = "spanId")]
    span_id: String,
    attributes: Vec<OpenTelemetryAttribute>,
}

impl From<&Span> for OpenTelemetrySpan {
    fn from(span: &Span) -> Self {
        Self {
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            parent_span_id: span.parent_span_id.clone(),
            name: span.name.clone(),
            kind: match span.kind {
                SpanKind::Internal => 0,
                SpanKind::Server => 1,
                SpanKind::Client => 2,
                SpanKind::Producer => 3,
                SpanKind::Consumer => 4,
            },
            start_time_unix_nano: span.start_time.to_string(),
            end_time_unix_nano: span.end_time.map(|t| t.to_string()),
            duration_nano: span.duration.map(|d| d.to_string()),
            status: OpenTelemetryStatus {
                code: match span.status.code {
                    StatusCode::Unset => 0,
                    StatusCode::Ok => 1,
                    StatusCode::Error => 2,
                },
                message: span.status.message.clone(),
            },
            attributes: span.attributes.iter().map(|(k, v)| OpenTelemetryAttribute {
                key: k.clone(),
                value: match v {
                    AttributeValue::String(s) => OpenTelemetryAttributeValue::String(s.clone()),
                    AttributeValue::Bool(b) => OpenTelemetryAttributeValue::Bool(*b),
                    AttributeValue::Int(i) => OpenTelemetryAttributeValue::Int(*i),
                    AttributeValue::Float(f) => OpenTelemetryAttributeValue::Double(*f),
                    AttributeValue::StringArray(sa) => OpenTelemetryAttributeValue::StringArray(sa.clone()),
                    AttributeValue::BoolArray(ba) => OpenTelemetryAttributeValue::BoolArray(ba.clone()),
                    AttributeValue::IntArray(ia) => OpenTelemetryAttributeValue::IntArray(ia.clone()),
                    AttributeValue::FloatArray(fa) => OpenTelemetryAttributeValue::DoubleArray(fa.clone()),
                },
            }).collect(),
            events: span.events.iter().map(|e| OpenTelemetryEvent {
                name: e.name.clone(),
                time_unix_nano: e.timestamp.to_string(),
                attributes: e.attributes.iter().map(|(k, v)| OpenTelemetryAttribute {
                    key: k.clone(),
                    value: match v {
                        AttributeValue::String(s) => OpenTelemetryAttributeValue::String(s.clone()),
                        AttributeValue::Bool(b) => OpenTelemetryAttributeValue::Bool(*b),
                        AttributeValue::Int(i) => OpenTelemetryAttributeValue::Int(*i),
                        AttributeValue::Float(f) => OpenTelemetryAttributeValue::Double(*f),
                        AttributeValue::StringArray(sa) => OpenTelemetryAttributeValue::StringArray(sa.clone()),
                        AttributeValue::BoolArray(ba) => OpenTelemetryAttributeValue::BoolArray(ba.clone()),
                        AttributeValue::IntArray(ia) => OpenTelemetryAttributeValue::IntArray(ia.clone()),
                        AttributeValue::FloatArray(fa) => OpenTelemetryAttributeValue::DoubleArray(fa.clone()),
                    },
                }).collect(),
            }).collect(),
            links: span.links.iter().map(|l| OpenTelemetryLink {
                trace_id: l.trace_id.clone(),
                span_id: l.span_id.clone(),
                attributes: l.attributes.iter().map(|(k, v)| OpenTelemetryAttribute {
                    key: k.clone(),
                    value: match v {
                        AttributeValue::String(s) => OpenTelemetryAttributeValue::String(s.clone()),
                        AttributeValue::Bool(b) => OpenTelemetryAttributeValue::Bool(*b),
                        AttributeValue::Int(i) => OpenTelemetryAttributeValue::Int(*i),
                        AttributeValue::Float(f) => OpenTelemetryAttributeValue::Double(*f),
                        AttributeValue::StringArray(sa) => OpenTelemetryAttributeValue::StringArray(sa.clone()),
                        AttributeValue::BoolArray(ba) => OpenTelemetryAttributeValue::BoolArray(ba.clone()),
                        AttributeValue::IntArray(ia) => OpenTelemetryAttributeValue::IntArray(ia.clone()),
                        AttributeValue::FloatArray(fa) => OpenTelemetryAttributeValue::DoubleArray(fa.clone()),
                    },
                }).collect(),
            }).collect(),
        }
    }
}

impl TraceManager {
    /// Create a new trace manager
    pub fn new(config: TraceConfig) -> Self {
        Self {
            spans: Arc::new(RwLock::new(Vec::new())),
            config,
            exporter: None,
        }
    }

    /// Set the trace exporter
    pub fn set_exporter(&mut self, exporter: Box<dyn TraceExporter + Send + Sync>) {
        self.exporter = Some(exporter);
    }

    /// Start a new span
    pub async fn start_span(
        &self,
        name: String,
        kind: SpanKind,
        context: &TraceContext,
    ) -> Result<SpanHandle, Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(SpanHandle::disabled());
        }

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos() as u64;

        let span = Span {
            trace_id: context.trace_id.clone(),
            span_id: context.span_id.clone(),
            parent_span_id: context.parent_span_id.clone(),
            name,
            kind,
            start_time,
            end_time: None,
            duration: None,
            status: SpanStatus {
                code: StatusCode::Unset,
                message: None,
            },
            attributes: HashMap::new(),
            events: Vec::new(),
            links: Vec::new(),
        };

        let span_id = span.span_id.clone();
        self.spans.write().await.push(span);

        Ok(SpanHandle {
            span_id,
            manager: self.spans.clone(),
        })
    }

    /// Export traces
    pub async fn export_traces(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref exporter) = self.exporter {
            let spans = self.spans.read().await;
            exporter.export(&spans)?;
        }
        Ok(())
    }

    /// Clear exported spans
    pub async fn clear_spans(&self) {
        self.spans.write().await.clear();
    }
}

/// Span handle for managing individual spans
pub struct SpanHandle {
    span_id: String,
    manager: Arc<RwLock<Vec<Span>>>,
}

impl SpanHandle {
    /// Create a disabled span handle
    fn disabled() -> Self {
        Self {
            span_id: String::new(),
            manager: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add an attribute to the span
    pub async fn add_attribute(&self, key: String, value: AttributeValue) {
        if self.span_id.is_empty() {
            return;
        }

        let mut spans = self.manager.write().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == self.span_id) {
            span.attributes.insert(key, value);
        }
    }

    /// Add an event to the span
    pub async fn add_event(&self, name: String, attributes: HashMap<String, AttributeValue>) {
        if self.span_id.is_empty() {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let event = SpanEvent {
            name,
            timestamp,
            attributes,
        };

        let mut spans = self.manager.write().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == self.span_id) {
            span.events.push(event);
        }
    }

    /// Set the span status
    pub async fn set_status(&self, code: StatusCode, message: Option<String>) {
        if self.span_id.is_empty() {
            return;
        }

        let mut spans = self.manager.write().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == self.span_id) {
            span.status = SpanStatus { code, message };
        }
    }

    /// Finish the span
    pub async fn finish(&self) {
        if self.span_id.is_empty() {
            return;
        }

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut spans = self.manager.write().await;
        if let Some(span) = spans.iter_mut().find(|s| s.span_id == self.span_id) {
            span.end_time = Some(end_time);
            span.duration = Some(end_time - span.start_time);
        }
    }
}

/// Global trace manager instance
static mut GLOBAL_TRACE_MANAGER: Option<Arc<TraceManager>> = None;

/// Initialize the global trace manager
pub fn init_global_trace_manager(config: TraceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(TraceManager::new(config));
    unsafe {
        GLOBAL_TRACE_MANAGER = Some(manager);
    }
    Ok(())
}

/// Get the global trace manager
pub fn get_global_trace_manager() -> Option<Arc<TraceManager>> {
    unsafe { GLOBAL_TRACE_MANAGER.clone() }
}

/// Start a span with the global trace manager
pub async fn start_span(
    name: String,
    kind: SpanKind,
    context: &TraceContext,
) -> Result<SpanHandle, Box<dyn std::error::Error>> {
    if let Some(manager) = get_global_trace_manager() {
        manager.start_span(name, kind, context).await
    } else {
        Ok(SpanHandle::disabled())
    }
}

/// Export traces with the global trace manager
pub async fn export_traces() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = get_global_trace_manager() {
        manager.export_traces().await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trace_context() {
        let context = TraceContext::new();
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());
        assert!(context.parent_span_id.is_none());

        let child = context.child();
        assert_eq!(child.trace_id, context.trace_id);
        assert_ne!(child.span_id, context.span_id);
        assert_eq!(child.parent_span_id, Some(context.span_id));
    }

    #[tokio::test]
    async fn test_span_creation() {
        let config = TraceConfig::default();
        let mut manager = TraceManager::new(config);
        manager.set_exporter(Box::new(JsonFileExporter::new("test_traces.json".to_string())));

        let context = TraceContext::new();
        let span_handle = manager.start_span(
            "test_span".to_string(),
            SpanKind::Internal,
            &context,
        ).await.unwrap();

        span_handle.add_attribute(
            "test_key".to_string(),
            AttributeValue::String("test_value".to_string()),
        ).await;

        span_handle.set_status(StatusCode::Ok, None).await;
        span_handle.finish().await;

        // Export traces
        manager.export_traces().await.unwrap();
    }

    #[test]
    fn test_attribute_values() {
        let string_attr = AttributeValue::String("test".to_string());
        let bool_attr = AttributeValue::Bool(true);
        let int_attr = AttributeValue::Int(42);
        let float_attr = AttributeValue::Float(3.14);

        // Test serialization
        let json = serde_json::to_string(&string_attr).unwrap();
        assert!(json.contains("test"));
    }
}
