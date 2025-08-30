use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Opts, Registry,
    Encoder, TextEncoder,
};
use serde::{Deserialize, Serialize};

/// Metrics configuration for Polymera OS services
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics port for Prometheus scraping
    pub port: u16,
    /// Metrics path for Prometheus scraping
    pub path: String,
    /// Enable histogram buckets for latency metrics
    pub enable_histograms: bool,
    /// Custom histogram buckets for latency
    pub custom_buckets: Vec<f64>,
    /// Privacy mode - redact sensitive labels
    pub privacy_mode: PrivacyMode,
}

/// Privacy protection modes for metrics
#[derive(Debug, Clone, PartialEq)]
pub enum PrivacyMode {
    /// No privacy protection
    None,
    /// Redact sensitive labels (default)
    Redact,
    /// Hash sensitive values
    Hash,
    /// Drop sensitive metrics entirely
    Drop,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9090,
            path: "/metrics".to_string(),
            enable_histograms: true,
            custom_buckets: vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            privacy_mode: PrivacyMode::Redact,
        }
    }
}

/// RED metrics (Rate, Errors, Duration)
#[derive(Debug)]
pub struct RedMetrics {
    /// Request rate counter
    pub request_rate: CounterVec,
    /// Error rate counter
    pub error_rate: CounterVec,
    /// Request duration histogram
    pub request_duration: HistogramVec,
    /// Request duration p95 gauge
    pub request_duration_p95: GaugeVec,
    /// Request duration p99 gauge
    pub request_duration_p99: GaugeVec,
}

/// USE metrics (Utilization, Saturation, Errors)
#[derive(Debug)]
pub struct UseMetrics {
    /// Resource utilization gauge
    pub utilization: GaugeVec,
    /// Resource saturation gauge
    pub saturation: GaugeVec,
    /// Error count counter
    pub errors: CounterVec,
    /// Resource capacity gauge
    pub capacity: GaugeVec,
}

/// Business metrics for Polymera OS
#[derive(Debug)]
pub struct BusinessMetrics {
    /// Active users counter
    pub active_users: GaugeVec,
    /// Transaction volume counter
    pub transaction_volume: CounterVec,
    /// Contract deployments counter
    pub contract_deployments: CounterVec,
    /// Attestation operations counter
    pub attestation_operations: CounterVec,
}

/// Privacy-aware metrics registry
#[derive(Debug)]
pub struct PrivacyAwareRegistry {
    registry: Registry,
    red_metrics: RedMetrics,
    use_metrics: UseMetrics,
    business_metrics: BusinessMetrics,
    privacy_config: MetricsConfig,
    sensitive_labels: Arc<RwLock<HashMap<String, String>>>,
}

impl PrivacyAwareRegistry {
    /// Create a new privacy-aware metrics registry
    pub fn new(config: MetricsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        // Initialize RED metrics
        let red_metrics = RedMetrics {
            request_rate: CounterVec::new(
                Opts::new("polymera_requests_total", "Total number of requests")
                    .namespace("polymera")
                    .subsystem("api"),
                &["service", "endpoint", "method", "status"]
            )?,
            error_rate: CounterVec::new(
                Opts::new("polymera_errors_total", "Total number of errors")
                    .namespace("polymera")
                    .subsystem("api"),
                &["service", "endpoint", "method", "error_type"]
            )?,
            request_duration: HistogramVec::new(
                Opts::new("polymera_request_duration_seconds", "Request duration in seconds")
                    .namespace("polymera")
                    .subsystem("api")
                    .buckets(config.custom_buckets.clone()),
                &["service", "endpoint", "method"]
            )?,
            request_duration_p95: GaugeVec::new(
                Opts::new("polymera_request_duration_p95_seconds", "95th percentile request duration")
                    .namespace("polymera")
                    .subsystem("api"),
                &["service", "endpoint", "method"]
            )?,
            request_duration_p99: GaugeVec::new(
                Opts::new("polymera_request_duration_p99_seconds", "99th percentile request duration")
                    .namespace("polymera")
                    .subsystem("api"),
                &["service", "endpoint", "method"]
            )?,
        };

        // Initialize USE metrics
        let use_metrics = UseMetrics {
            utilization: GaugeVec::new(
                Opts::new("polymera_resource_utilization", "Resource utilization percentage")
                    .namespace("polymera")
                    .subsystem("resources"),
                &["resource_type", "resource_name"]
            )?,
            saturation: GaugeVec::new(
                Opts::new("polymera_resource_saturation", "Resource saturation percentage")
                    .namespace("polymera")
                    .subsystem("resources"),
                &["resource_type", "resource_name"]
            )?,
            errors: CounterVec::new(
                Opts::new("polymera_resource_errors_total", "Total resource errors")
                    .namespace("polymera")
                    .subsystem("resources"),
                &["resource_type", "resource_name", "error_type"]
            )?,
            capacity: GaugeVec::new(
                Opts::new("polymera_resource_capacity", "Resource capacity")
                    .namespace("polymera")
                    .subsystem("resources"),
                &["resource_type", "resource_name"]
            )?,
        };

        // Initialize business metrics
        let business_metrics = BusinessMetrics {
            active_users: GaugeVec::new(
                Opts::new("polymera_active_users", "Number of active users")
                    .namespace("polymera")
                    .subsystem("business"),
                &["user_type", "region"]
            )?,
            transaction_volume: CounterVec::new(
                Opts::new("polymera_transactions_total", "Total transaction volume")
                    .namespace("polymera")
                    .subsystem("business"),
                &["transaction_type", "chain", "status"]
            )?,
            contract_deployments: CounterVec::new(
                Opts::new("polymera_contract_deployments_total", "Total contract deployments")
                    .namespace("polymera")
                    .subsystem("business"),
                &["contract_type", "chain", "status"]
            )?,
            attestation_operations: CounterVec::new(
                Opts::new("polymera_attestations_total", "Total attestation operations")
                    .namespace("polymera")
                    .subsystem("business"),
                &["operation_type", "schema", "status"]
            )?,
        };

        // Register all metrics
        registry.register(Box::new(red_metrics.request_rate.clone()))?;
        registry.register(Box::new(red_metrics.error_rate.clone()))?;
        registry.register(Box::new(red_metrics.request_duration.clone()))?;
        registry.register(Box::new(red_metrics.request_duration_p95.clone()))?;
        registry.register(Box::new(red_metrics.request_duration_p99.clone()))?;
        
        registry.register(Box::new(use_metrics.utilization.clone()))?;
        registry.register(Box::new(use_metrics.saturation.clone()))?;
        registry.register(Box::new(use_metrics.errors.clone()))?;
        registry.register(Box::new(use_metrics.capacity.clone()))?;
        
        registry.register(Box::new(business_metrics.active_users.clone()))?;
        registry.register(Box::new(business_metrics.transaction_volume.clone()))?;
        registry.register(Box::new(business_metrics.contract_deployments.clone()))?;
        registry.register(Box::new(business_metrics.attestation_operations.clone()))?;

        Ok(Self {
            registry,
            red_metrics,
            use_metrics,
            business_metrics,
            privacy_config: config,
            sensitive_labels: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Record a request with privacy protection
    pub fn record_request(&self, service: &str, endpoint: &str, method: &str, status: &str, duration: Duration) {
        let labels = self.sanitize_labels(&[
            ("service", service),
            ("endpoint", endpoint),
            ("method", method),
            ("status", status),
        ]);

        // Record request rate
        self.red_metrics.request_rate
            .with_label_values(&[&labels[0], &labels[1], &labels[2], &labels[3]])
            .inc();

        // Record duration
        let duration_secs = duration.as_secs_f64();
        self.red_metrics.request_duration
            .with_label_values(&[&labels[0], &labels[1], &labels[2]])
            .observe(duration_secs);

        // Update p95 and p99 (simplified - in production use proper percentile calculation)
        self.update_percentiles(service, endpoint, method, duration_secs);
    }

    /// Record an error with privacy protection
    pub fn record_error(&self, service: &str, endpoint: &str, method: &str, error_type: &str) {
        let labels = self.sanitize_labels(&[
            ("service", service),
            ("endpoint", endpoint),
            ("method", method),
            ("error_type", error_type),
        ]);

        self.red_metrics.error_rate
            .with_label_values(&[&labels[0], &labels[1], &labels[2], &labels[3]])
            .inc();
    }

    /// Record resource utilization
    pub fn record_utilization(&self, resource_type: &str, resource_name: &str, utilization: f64) {
        let labels = self.sanitize_labels(&[
            ("resource_type", resource_type),
            ("resource_name", resource_name),
        ]);

        self.use_metrics.utilization
            .with_label_values(&[&labels[0], &labels[1]])
            .set(utilization);
    }

    /// Record business metric
    pub fn record_business_metric(&self, metric_type: BusinessMetricType, labels: &[(&str, &str)], value: f64) {
        let sanitized_labels = self.sanitize_labels(labels);
        
        match metric_type {
            BusinessMetricType::ActiveUsers => {
                if sanitized_labels.len() >= 2 {
                    self.business_metrics.active_users
                        .with_label_values(&[&sanitized_labels[0], &sanitized_labels[1]])
                        .set(value);
                }
            }
            BusinessMetricType::TransactionVolume => {
                if sanitized_labels.len() >= 3 {
                    self.business_metrics.transaction_volume
                        .with_label_values(&[&sanitized_labels[0], &sanitized_labels[1], &sanitized_labels[2]])
                        .inc_by(value as u64);
                }
            }
            BusinessMetricType::ContractDeployments => {
                if sanitized_labels.len() >= 3 {
                    self.business_metrics.contract_deployments
                        .with_label_values(&[&sanitized_labels[0], &sanitized_labels[1], &sanitized_labels[2]])
                        .inc();
                }
            }
            BusinessMetricType::AttestationOperations => {
                if sanitized_labels.len() >= 3 {
                    self.business_metrics.attestation_operations
                        .with_label_values(&[&sanitized_labels[0], &sanitized_labels[1], &sanitized_labels[2]])
                        .inc();
                }
            }
        }
    }

    /// Sanitize labels based on privacy configuration
    fn sanitize_labels(&self, labels: &[(&str, &str)]) -> Vec<String> {
        labels.iter().map(|(key, value)| {
            match self.privacy_config.privacy_mode {
                PrivacyMode::None => value.to_string(),
                PrivacyMode::Redact => {
                    if self.is_sensitive_key(key) {
                        "[REDACTED]".to_string()
                    } else {
                        value.to_string()
                    }
                }
                PrivacyMode::Hash => {
                    if self.is_sensitive_key(key) {
                        format!("{:x}", md5::compute(value))
                    } else {
                        value.to_string()
                    }
                }
                PrivacyMode::Drop => {
                    if self.is_sensitive_key(key) {
                        "".to_string()
                    } else {
                        value.to_string()
                    }
                }
            }
        }).collect()
    }

    /// Check if a label key is sensitive
    fn is_sensitive_key(&self, key: &str) -> bool {
        let sensitive_keys = [
            "user_id", "email", "address", "private_key", "secret", "token",
            "password", "api_key", "session_id", "ip_address"
        ];
        sensitive_keys.contains(&key)
    }

    /// Update percentile calculations (simplified implementation)
    fn update_percentiles(&self, service: &str, endpoint: &str, method: &str, duration: f64) {
        // In production, implement proper percentile calculation with sliding windows
        // This is a simplified version for demonstration
        
        let labels = self.sanitize_labels(&[
            ("service", service),
            ("endpoint", endpoint),
            ("method", method),
        ]);

        // For demo purposes, just set some values
        // In production, calculate actual p95 and p99 from histogram data
        self.red_metrics.request_duration_p95
            .with_label_values(&[&labels[0], &labels[1], &labels[2]])
            .set(duration * 1.1); // Simplified p95 approximation

        self.red_metrics.request_duration_p99
            .with_label_values(&[&labels[0], &labels[1], &labels[2]])
            .set(duration * 1.5); // Simplified p99 approximation
    }

    /// Get metrics in Prometheus format
    pub fn gather(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(buffer)
    }

    /// Get the registry reference
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// Business metric types
#[derive(Debug, Clone)]
pub enum BusinessMetricType {
    ActiveUsers,
    TransactionVolume,
    ContractDeployments,
    AttestationOperations,
}

/// Request timing wrapper for automatic metrics recording
pub struct RequestTimer<'a> {
    registry: &'a PrivacyAwareRegistry,
    service: String,
    endpoint: String,
    method: String,
    start_time: Instant,
}

impl<'a> RequestTimer<'a> {
    /// Create a new request timer
    pub fn new(
        registry: &'a PrivacyAwareRegistry,
        service: &str,
        endpoint: &str,
        method: &str,
    ) -> Self {
        Self {
            registry,
            service: service.to_string(),
            endpoint: endpoint.to_string(),
            method: method.to_string(),
            start_time: Instant::now(),
        }
    }

    /// Record successful request completion
    pub fn success(self) {
        let duration = self.start_time.elapsed();
        self.registry.record_request(
            &self.service,
            &self.endpoint,
            &self.method,
            "success",
            duration,
        );
    }

    /// Record failed request completion
    pub fn error(self, error_type: &str) {
        let duration = self.start_time.elapsed();
        self.registry.record_request(
            &self.service,
            &self.endpoint,
            &self.method,
            "error",
            duration,
        );
        self.registry.record_error(
            &self.service,
            &self.endpoint,
            &self.method,
            error_type,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 9090);
        assert_eq!(config.path, "/metrics");
        assert_eq!(config.privacy_mode, PrivacyMode::Redact);
    }

    #[test]
    fn test_privacy_aware_registry_creation() {
        let config = MetricsConfig::default();
        let registry = PrivacyAwareRegistry::new(config);
        assert!(registry.is_ok());
    }

    #[test]
    fn test_sensitive_key_detection() {
        let config = MetricsConfig::default();
        let registry = PrivacyAwareRegistry::new(config).unwrap();
        
        assert!(registry.is_sensitive_key("user_id"));
        assert!(registry.is_sensitive_key("email"));
        assert!(!registry.is_sensitive_key("service"));
        assert!(!registry.is_sensitive_key("endpoint"));
    }

    #[test]
    fn test_request_timer() {
        let config = MetricsConfig::default();
        let registry = PrivacyAwareRegistry::new(config).unwrap();
        
        let timer = RequestTimer::new(&registry, "test-service", "/test", "GET");
        std::thread::sleep(Duration::from_millis(10));
        timer.success();
        
        // Verify metrics were recorded
        let metrics = registry.gather().unwrap();
        let metrics_str = String::from_utf8(metrics).unwrap();
        assert!(metrics_str.contains("polymera_requests_total"));
    }
}
