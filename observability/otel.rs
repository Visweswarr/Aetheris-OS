use opentelemetry::{
    global,
    sdk::{
        trace::{self, BatchSpanProcessor, TracerProvider},
        Resource,
    },
    trace::{Span, Tracer},
    KeyValue,
};
use opentelemetry_otlp::{ExportConfig, Protocol, SpanExporterBuilder};
use std::time::Duration;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// OpenTelemetry configuration for Polymera OS services
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// Service name for identification
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Service namespace
    pub service_namespace: String,
    /// OTLP endpoint for telemetry export
    pub otlp_endpoint: Option<String>,
    /// OTLP protocol (grpc or http)
    pub otlp_protocol: Protocol,
    /// Batch processing configuration
    pub batch_config: BatchConfig,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Enable local development mode
    pub dev_mode: bool,
}

/// Batch processing configuration for spans
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of spans per batch
    pub max_export_batch_size: usize,
    /// Maximum time to wait before exporting
    pub max_concurrent_exports: usize,
    /// Maximum time to wait before exporting
    pub scheduled_delay: Duration,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "polymera-service".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            service_namespace: "polymera-os".to_string(),
            otlp_endpoint: None,
            otlp_protocol: Protocol::Grpc,
            batch_config: BatchConfig::default(),
            sampling_rate: 1.0,
            dev_mode: false,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_export_batch_size: 512,
            max_concurrent_exports: 5,
            scheduled_delay: Duration::from_secs(5),
        }
    }
}

/// Initialize OpenTelemetry with the given configuration
pub async fn init_otel(config: OtelConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing OpenTelemetry for service: {}", config.service_name);

    // Create resource with service information
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", config.service_version.clone()),
        KeyValue::new("service.namespace", config.service_namespace.clone()),
        KeyValue::new("deployment.environment", if config.dev_mode { "development" } else { "production" }),
    ]);

    // Configure OTLP exporter
    let export_config = ExportConfig {
        endpoint: config.otlp_endpoint.unwrap_or_else(|| {
            if config.dev_mode {
                "http://localhost:4317".to_string()
            } else {
                "http://otel-collector:4317".to_string()
            }
        }),
        protocol: config.otlp_protocol,
        timeout: Duration::from_secs(30),
    };

    // Build span exporter
    let span_exporter = SpanExporterBuilder::from(export_config)
        .build_span_exporter()
        .map_err(|e| format!("Failed to build span exporter: {}", e))?;

    // Create tracer provider
    let provider = TracerProvider::builder()
        .with_resource(resource)
        .with_sampler(trace::Sampler::ParentBased(Box::new(
            trace::Sampler::TraceIdRatioBased(config.sampling_rate)
        )))
        .with_span_processor(BatchSpanProcessor::builder(span_exporter, opentelemetry::runtime::Tokio)
            .with_max_queue_size(config.batch_config.max_export_batch_size)
            .with_max_concurrent_exports(config.batch_config.max_concurrent_exports)
            .with_scheduled_delay(config.batch_config.scheduled_delay)
            .build())
        .build();

    // Set global tracer provider
    global::set_tracer_provider(provider);

    // Initialize tracing subscriber with OpenTelemetry layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(
        global::tracer(&config.service_name)
    );

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("OpenTelemetry initialized successfully");
    Ok(())
}

/// Shutdown OpenTelemetry gracefully
pub async fn shutdown_otel() {
    info!("Shutting down OpenTelemetry");
    global::shutdown_tracer_provider();
}

/// Create a tracer for a specific component
pub fn get_tracer(component: &str) -> Tracer {
    global::tracer(component)
}

/// Create a span for a specific operation
pub fn create_span(tracer: &Tracer, operation: &str) -> Span {
    tracer.start(operation)
}

/// Add attributes to the current span
pub fn add_span_attributes(attributes: Vec<KeyValue>) {
    if let Some(span) = tracing::Span::current().id() {
        span.record("attributes", &attributes);
    }
}

/// Record an event in the current span
pub fn record_event(name: &str, attributes: Vec<KeyValue>) {
    if let Some(span) = tracing::Span::current().id() {
        span.record("event", &name);
        span.record("attributes", &attributes);
    }
}

/// Privacy-aware span creation that redacts sensitive data
pub fn create_private_span(tracer: &Tracer, operation: &str, redacted_data: &str) -> Span {
    let mut span = tracer.start(operation);
    span.set_attribute(KeyValue::new("data.redacted", true));
    span.set_attribute(KeyValue::new("data.type", "redacted"));
    span.set_attribute(KeyValue::new("data.hash", format!("{:x}", md5::compute(redacted_data))));
    span
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_otel_config_default() {
        let config = OtelConfig::default();
        assert_eq!(config.service_name, "polymera-service");
        assert_eq!(config.sampling_rate, 1.0);
        assert!(!config.dev_mode);
    }

    #[tokio::test]
    async fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_export_batch_size, 512);
        assert_eq!(config.max_concurrent_exports, 5);
        assert_eq!(config.scheduled_delay, Duration::from_secs(5));
    }

    #[test]
    fn test_tracer_creation() {
        let tracer = get_tracer("test-component");
        assert_eq!(tracer.name(), "test-component");
    }

    #[test]
    fn test_span_creation() {
        let tracer = get_tracer("test-component");
        let span = create_span(&tracer, "test-operation");
        assert_eq!(span.name(), "test-operation");
    }
}
