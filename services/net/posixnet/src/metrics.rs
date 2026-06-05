//! Structured Metrics for Aetheris OS Networking
//! 
//! This module provides comprehensive metrics collection and export capabilities,
//! including latency percentiles, throughput, CPU/RSS usage, and error counts.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(HistogramData),
    Summary(SummaryData),
}

/// Histogram data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramData {
    pub buckets: Vec<HistogramBucket>,
    pub count: u64,
    pub sum: f64,
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Summary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryData {
    pub quantiles: Vec<Quantile>,
    pub count: u64,
    pub sum: f64,
}

/// Quantile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    pub quantile: f64,
    pub value: f64,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
    pub value: MetricValue,
    pub timestamp: u64,
}

/// Metric type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metrics collector for aggregating and managing metrics
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    config: MetricsConfig,
    exporters: Vec<Box<dyn MetricsExporter + Send + Sync>>,
}

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub export_interval: Duration,
    pub prometheus_port: Option<u16>,
    pub json_output: Option<String>,
    pub service_name: String,
    pub service_version: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_interval: Duration::from_secs(15),
            prometheus_port: Some(9090),
            json_output: Some("metrics/network_metrics.json".to_string()),
            service_name: "aetheris-net".to_string(),
            service_version: "1.0.0".to_string(),
        }
    }
}

/// Metrics exporter trait
pub trait MetricsExporter: Send + Sync {
    fn export(&self, metrics: &[Metric]) -> Result<(), Box<dyn std::error::Error>>;
}

/// Prometheus text format exporter
pub struct PrometheusExporter {
    port: u16,
}

impl PrometheusExporter {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl MetricsExporter for PrometheusExporter {
    fn export(&self, metrics: &[Metric]) -> Result<(), Box<dyn std::error::Error>> {
        let mut output = String::new();
        
        // Group metrics by type
        let mut counters = Vec::new();
        let mut gauges = Vec::new();
        let mut histograms = Vec::new();
        let mut summaries = Vec::new();
        
        for metric in metrics {
            match metric.metric_type {
                MetricType::Counter => counters.push(metric),
                MetricType::Gauge => gauges.push(metric),
                MetricType::Histogram => histograms.push(metric),
                MetricType::Summary => summaries.push(metric),
            }
        }
        
        // Export counters
        for metric in counters {
            if let MetricValue::Counter(value) = &metric.value {
                output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
                output.push_str(&format!("# TYPE {} counter\n", metric.name));
                
                let labels_str = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<String> = metric.labels.iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", labels.join(","))
                };
                
                output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
            }
        }
        
        // Export gauges
        for metric in gauges {
            if let MetricValue::Gauge(value) = &metric.value {
                output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
                output.push_str(&format!("# TYPE {} gauge\n", metric.name));
                
                let labels_str = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<String> = metric.labels.iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", labels.join(","))
                };
                
                output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
            }
        }
        
        // Export histograms
        for metric in histograms {
            if let MetricValue::Histogram(hist_data) = &metric.value {
                output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
                output.push_str(&format!("# TYPE {} histogram\n", metric.name));
                
                let labels_str = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<String> = metric.labels.iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", labels.join(","))
                };
                
                // Export buckets
                for bucket in &hist_data.buckets {
                    output.push_str(&format!("{}_bucket{}le=\"{}\" {}\n", 
                        metric.name, labels_str, bucket.upper_bound, bucket.count));
                }
                
                // Export count and sum
                output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, hist_data.count));
                output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, hist_data.sum));
            }
        }
        
        // Export summaries
        for metric in summaries {
            if let MetricValue::Summary(summary_data) = &metric.value {
                output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
                output.push_str(&format!("# TYPE {} summary\n", metric.name));
                
                let labels_str = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<String> = metric.labels.iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", labels.join(","))
                };
                
                // Export quantiles
                for quantile in &summary_data.quantiles {
                    output.push_str(&format!("{}{{quantile=\"{}\"}} {}\n", 
                        metric.name, quantile.quantile, quantile.value));
                }
                
                // Export count and sum
                output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, summary_data.count));
                output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, summary_data.sum));
            }
        }
        
        // In a real implementation, this would start an HTTP server
        // For now, we'll just log the output
        log::info!("Prometheus metrics:\n{}", output);
        
        Ok(())
    }
}

/// JSON file exporter
pub struct JsonFileExporter {
    output_file: String,
}

impl JsonFileExporter {
    pub fn new(output_file: String) -> Self {
        Self { output_file }
    }
}

impl MetricsExporter for JsonFileExporter {
    fn export(&self, metrics: &[Metric]) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.output_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write metrics to JSON file
        let mut file = std::fs::File::create(&self.output_file)?;
        let json = serde_json::to_string_pretty(metrics)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
            exporters: Vec::new(),
        }
    }

    /// Add an exporter
    pub fn add_exporter(&mut self, exporter: Box<dyn MetricsExporter + Send + Sync>) {
        self.exporters.push(exporter);
    }

    /// Record a counter metric
    pub async fn record_counter(
        &self,
        name: String,
        help: String,
        value: u64,
        labels: HashMap<String, String>,
    ) {
        if !self.config.enabled {
            return;
        }

        let metric = Metric {
            name: name.clone(),
            help,
            metric_type: MetricType::Counter,
            labels,
            value: MetricValue::Counter(value),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        self.metrics.write().await.insert(name, metric);
    }

    /// Record a gauge metric
    pub async fn record_gauge(
        &self,
        name: String,
        help: String,
        value: f64,
        labels: HashMap<String, String>,
    ) {
        if !self.config.enabled {
            return;
        }

        let metric = Metric {
            name: name.clone(),
            help,
            metric_type: MetricType::Gauge,
            labels,
            value: MetricValue::Gauge(value),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        self.metrics.write().await.insert(name, metric);
    }

    /// Record a histogram metric
    pub async fn record_histogram(
        &self,
        name: String,
        help: String,
        values: Vec<f64>,
        labels: HashMap<String, String>,
    ) {
        if !self.config.enabled {
            return;
        }

        // Create histogram buckets
        let mut buckets = Vec::new();
        let mut count = 0u64;
        let mut sum = 0.0;

        // Define bucket boundaries (exponential)
        let bucket_boundaries = vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0, f64::INFINITY];

        for &boundary in &bucket_boundaries {
            let bucket_count = values.iter().filter(|&&v| v <= boundary).count() as u64;
            buckets.push(HistogramBucket {
                upper_bound: boundary,
                count: bucket_count,
            });
        }

        for &value in &values {
            count += 1;
            sum += value;
        }

        let metric = Metric {
            name: name.clone(),
            help,
            metric_type: MetricType::Histogram,
            labels,
            value: MetricValue::Histogram(HistogramData {
                buckets,
                count,
                sum,
            }),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        self.metrics.write().await.insert(name, metric);
    }

    /// Record a summary metric
    pub async fn record_summary(
        &self,
        name: String,
        help: String,
        values: Vec<f64>,
        labels: HashMap<String, String>,
    ) {
        if !self.config.enabled {
            return;
        }

        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Calculate quantiles
        let quantiles = vec![
            Quantile { quantile: 0.5, value: self.percentile(&sorted_values, 0.5) },
            Quantile { quantile: 0.9, value: self.percentile(&sorted_values, 0.9) },
            Quantile { quantile: 0.95, value: self.percentile(&sorted_values, 0.95) },
            Quantile { quantile: 0.99, value: self.percentile(&sorted_values, 0.99) },
        ];

        let count = values.len() as u64;
        let sum = values.iter().sum();

        let metric = Metric {
            name: name.clone(),
            help,
            metric_type: MetricType::Summary,
            labels,
            value: MetricValue::Summary(SummaryData {
                quantiles,
                count,
                sum,
            }),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        self.metrics.write().await.insert(name, metric);
    }

    /// Calculate percentile
    fn percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile * (sorted_values.len() - 1) as f64) as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    /// Export metrics to all exporters
    pub async fn export_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metrics: Vec<Metric> = self.metrics.read().await.values().cloned().collect();
        
        for exporter in &self.exporters {
            exporter.export(&metrics)?;
        }
        
        Ok(())
    }

    /// Start the metrics collection loop
    pub async fn start_collection_loop(&self) {
        let mut interval = interval(self.config.export_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.export_metrics().await {
                log::error!("Failed to export metrics: {}", e);
            }
        }
    }

    /// Get all metrics as a vector
    pub async fn get_metrics(&self) -> Vec<Metric> {
        self.metrics.read().await.values().cloned().collect()
    }
}

/// Network-specific metrics collector
pub struct NetworkMetricsCollector {
    collector: Arc<MetricsCollector>,
    
    // Atomic counters for high-frequency updates
    requests_total: AtomicU64,
    errors_total: AtomicU64,
    bytes_tx_total: AtomicU64,
    bytes_rx_total: AtomicU64,
    
    // Latency samples for histogram
    latency_samples: Arc<RwLock<Vec<f64>>>,
    
    // System metrics
    cpu_usage: AtomicUsize,
    memory_usage: AtomicUsize,
}

impl NetworkMetricsCollector {
    /// Create a new network metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        let mut collector = MetricsCollector::new(config);
        
        // Add exporters
        if let Some(port) = collector.config.prometheus_port {
            collector.add_exporter(Box::new(PrometheusExporter::new(port)));
        }
        
        if let Some(ref output_file) = collector.config.json_output {
            collector.add_exporter(Box::new(JsonFileExporter::new(output_file.clone())));
        }
        
        Self {
            collector: Arc::new(collector),
            requests_total: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            bytes_tx_total: AtomicU64::new(0),
            bytes_rx_total: AtomicU64::new(0),
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            cpu_usage: AtomicUsize::new(0),
            memory_usage: AtomicUsize::new(0),
        }
    }

    /// Record a request
    pub fn record_request(&self, latency_ms: f64, bytes_tx: usize, bytes_rx: usize) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.bytes_tx_total.fetch_add(bytes_tx as u64, Ordering::Relaxed);
        self.bytes_rx_total.fetch_add(bytes_rx as u64, Ordering::Relaxed);
        
        // Add latency sample
        let samples = self.latency_samples.clone();
        tokio::spawn(async move {
            samples.write().await.push(latency_ms);
        });
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Update system metrics
    pub fn update_system_metrics(&self, cpu_pct: f64, memory_mb: f64) {
        self.cpu_usage.store((cpu_pct * 100.0) as usize, Ordering::Relaxed);
        self.memory_usage.store((memory_mb * 1024.0 * 1024.0) as usize, Ordering::Relaxed);
    }

    /// Export network metrics
    pub async fn export_network_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let requests = self.requests_total.load(Ordering::Relaxed);
        let errors = self.errors_total.load(Ordering::Relaxed);
        let bytes_tx = self.bytes_tx_total.load(Ordering::Relaxed);
        let bytes_rx = self.bytes_rx_total.load(Ordering::Relaxed);
        let cpu_pct = self.cpu_usage.load(Ordering::Relaxed) as f64 / 100.0;
        let memory_mb = self.memory_usage.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0);

        // Record counter metrics
        self.collector.record_counter(
            "network_requests_total".to_string(),
            "Total number of network requests".to_string(),
            requests,
            HashMap::new(),
        ).await;

        self.collector.record_counter(
            "network_errors_total".to_string(),
            "Total number of network errors".to_string(),
            errors,
            HashMap::new(),
        ).await;

        self.collector.record_counter(
            "network_bytes_tx_total".to_string(),
            "Total bytes transmitted".to_string(),
            bytes_tx,
            HashMap::new(),
        ).await;

        self.collector.record_counter(
            "network_bytes_rx_total".to_string(),
            "Total bytes received".to_string(),
            bytes_rx,
            HashMap::new(),
        ).await;

        // Record gauge metrics
        self.collector.record_gauge(
            "network_cpu_usage_percent".to_string(),
            "CPU usage percentage".to_string(),
            cpu_pct,
            HashMap::new(),
        ).await;

        self.collector.record_gauge(
            "network_memory_usage_mb".to_string(),
            "Memory usage in MB".to_string(),
            memory_mb,
            HashMap::new(),
        ).await;

        // Record latency histogram
        let latency_samples = self.latency_samples.read().await;
        if !latency_samples.is_empty() {
            self.collector.record_histogram(
                "network_latency_ms".to_string(),
                "Network latency in milliseconds".to_string(),
                latency_samples.clone(),
                HashMap::new(),
            ).await;
        }

        // Export all metrics
        self.collector.export_metrics().await?;

        Ok(())
    }

    /// Start the metrics collection loop
    pub async fn start_collection_loop(&self) {
        let collector = self.collector.clone();
        let network_collector = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(15));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = network_collector.export_network_metrics().await {
                    log::error!("Failed to export network metrics: {}", e);
                }
            }
        });
        
        collector.start_collection_loop().await;
    }
}

impl Clone for NetworkMetricsCollector {
    fn clone(&self) -> Self {
        Self {
            collector: self.collector.clone(),
            requests_total: AtomicU64::new(self.requests_total.load(Ordering::Relaxed)),
            errors_total: AtomicU64::new(self.errors_total.load(Ordering::Relaxed)),
            bytes_tx_total: AtomicU64::new(self.bytes_tx_total.load(Ordering::Relaxed)),
            bytes_rx_total: AtomicU64::new(self.bytes_rx_total.load(Ordering::Relaxed)),
            latency_samples: self.latency_samples.clone(),
            cpu_usage: AtomicUsize::new(self.cpu_usage.load(Ordering::Relaxed)),
            memory_usage: AtomicUsize::new(self.memory_usage.load(Ordering::Relaxed)),
        }
    }
}

/// Global metrics collector instance
static mut GLOBAL_METRICS_COLLECTOR: Option<Arc<NetworkMetricsCollector>> = None;

/// Initialize the global metrics collector
pub fn init_global_metrics_collector(config: MetricsConfig) -> Result<(), Box<dyn std::error::Error>> {
    let collector = Arc::new(NetworkMetricsCollector::new(config));
    unsafe {
        GLOBAL_METRICS_COLLECTOR = Some(collector);
    }
    Ok(())
}

/// Get the global metrics collector
pub fn get_global_metrics_collector() -> Option<Arc<NetworkMetricsCollector>> {
    unsafe { GLOBAL_METRICS_COLLECTOR.clone() }
}

/// Record a request with the global metrics collector
pub fn record_request(latency_ms: f64, bytes_tx: usize, bytes_rx: usize) {
    if let Some(collector) = get_global_metrics_collector() {
        collector.record_request(latency_ms, bytes_tx, bytes_rx);
    }
}

/// Record an error with the global metrics collector
pub fn record_error() {
    if let Some(collector) = get_global_metrics_collector() {
        collector.record_error();
    }
}

/// Update system metrics with the global metrics collector
pub fn update_system_metrics(cpu_pct: f64, memory_mb: f64) {
    if let Some(collector) = get_global_metrics_collector() {
        collector.update_system_metrics(cpu_pct, memory_mb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        // Record some metrics
        collector.record_counter(
            "test_counter".to_string(),
            "Test counter".to_string(),
            42,
            HashMap::new(),
        ).await;
        
        collector.record_gauge(
            "test_gauge".to_string(),
            "Test gauge".to_string(),
            3.14,
            HashMap::new(),
        ).await;
        
        collector.record_histogram(
            "test_histogram".to_string(),
            "Test histogram".to_string(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            HashMap::new(),
        ).await;
        
        // Get metrics
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.len(), 3);
    }

    #[tokio::test]
    async fn test_network_metrics_collector() {
        let config = MetricsConfig::default();
        let collector = NetworkMetricsCollector::new(config);
        
        // Record some network metrics
        collector.record_request(10.0, 1024, 1024);
        collector.record_request(20.0, 2048, 2048);
        collector.record_error();
        
        collector.update_system_metrics(25.0, 128.0);
        
        // Export metrics
        collector.export_network_metrics().await.unwrap();
    }

    #[test]
    fn test_metric_serialization() {
        let metric = Metric {
            name: "test_metric".to_string(),
            help: "Test metric".to_string(),
            metric_type: MetricType::Counter,
            labels: HashMap::new(),
            value: MetricValue::Counter(42),
            timestamp: 1234567890,
        };
        
        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains("test_metric"));
        assert!(json.contains("42"));
    }
}
