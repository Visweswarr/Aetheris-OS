//! Polymera OS Observability Module
//! 
//! This module provides comprehensive observability capabilities including:
//! - OpenTelemetry tracing and metrics
//! - RED/USE metrics with privacy protection
//! - Prometheus metrics export
//! - Privacy-aware data handling
//! 
//! ## Features
//! 
//! - **Tracing**: Distributed tracing with OpenTelemetry
//! - **Metrics**: RED (Rate, Errors, Duration) and USE (Utilization, Saturation, Errors) metrics
//! - **Privacy**: Built-in PII protection and data redaction
//! - **Performance**: p95/p99 latency tracking
//! - **Integration**: Prometheus and Grafana ready

pub mod otel;
pub mod metrics;
pub mod logging;

pub use otel::{
    OtelConfig, BatchConfig, init_otel, shutdown_otel,
    get_tracer, create_span, add_span_attributes, record_event,
    create_private_span
};

pub use metrics::{
    MetricsConfig, PrivacyMode, RedMetrics, UseMetrics, BusinessMetrics,
    PrivacyAwareRegistry, BusinessMetricType, RequestTimer
};

pub use logging::{
    LoggingConfig, LogLevel, LogFormat, LogOutput, PolymeraLogger,
    CorrelationContext, RedactionConfig, RedactionMode, RedactionEngine,
    RedactionRule, RedactionRuleBuilder
};

/// Re-export common observability types
pub use opentelemetry::{
    trace::{Span, Tracer},
    KeyValue,
};

pub use prometheus::Registry;

/// Initialize observability for a Polymera OS service
pub async fn init_observability(
    service_name: &str,
    service_version: &str,
    otel_config: Option<OtelConfig>,
    metrics_config: Option<MetricsConfig>,
) -> Result<(Option<opentelemetry::global::TracerProvider>, Option<PrivacyAwareRegistry>), Box<dyn std::error::Error>> {
    let mut otel_provider = None;
    let mut metrics_registry = None;

    // Initialize OpenTelemetry if enabled
    if let Some(config) = otel_config {
        let mut otel_cfg = config;
        otel_cfg.service_name = service_name.to_string();
        otel_cfg.service_version = service_version.to_string();
        
        init_otel(otel_cfg).await?;
        otel_provider = Some(opentelemetry::global::tracer_provider());
    }

    // Initialize metrics if enabled
    if let Some(config) = metrics_config {
        let mut metrics_cfg = config;
        metrics_cfg.enabled = true;
        
        let registry = PrivacyAwareRegistry::new(metrics_cfg)?;
        metrics_registry = Some(registry);
    }

    Ok((otel_provider, metrics_registry))
}

/// Shutdown observability gracefully
pub async fn shutdown_observability() {
    shutdown_otel().await;
}

/// Create a privacy-aware span for sensitive operations
pub fn create_sensitive_span(
    tracer: &Tracer,
    operation: &str,
    sensitive_data: &str,
    privacy_level: PrivacyMode,
) -> Span {
    match privacy_level {
        PrivacyMode::None => create_span(tracer, operation),
        PrivacyMode::Redact | PrivacyMode::Hash | PrivacyMode::Drop => {
            create_private_span(tracer, operation, sensitive_data)
        }
    }
}

/// Record business metrics with automatic privacy protection
pub fn record_business_metric_safe(
    registry: &PrivacyAwareRegistry,
    metric_type: BusinessMetricType,
    labels: &[(&str, &str)],
    value: f64,
) {
    registry.record_business_metric(metric_type, labels, value);
}

/// Create a request timer with automatic metrics recording
pub fn create_request_timer(
    registry: &PrivacyAwareRegistry,
    service: &str,
    endpoint: &str,
    method: &str,
) -> RequestTimer {
    RequestTimer::new(registry, service, endpoint, method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_init() {
        let otel_config = OtelConfig {
            dev_mode: true,
            ..Default::default()
        };
        
        let metrics_config = MetricsConfig {
            enabled: true,
            ..Default::default()
        };

        let result = init_observability("test-service", "1.0.0", Some(otel_config), Some(metrics_config)).await;
        assert!(result.is_ok());
        
        // Cleanup
        shutdown_observability().await;
    }

    #[test]
    fn test_sensitive_span_creation() {
        let tracer = get_tracer("test");
        let span = create_sensitive_span(
            &tracer,
            "test-operation",
            "sensitive-data",
            PrivacyMode::Redact
        );
        
        assert_eq!(span.name(), "test-operation");
    }
}
