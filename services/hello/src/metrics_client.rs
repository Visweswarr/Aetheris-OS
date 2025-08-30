use clap::{App, Arg, SubCommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct MetricsConfig {
    server_url: String,
    timeout_seconds: u64,
    check_interval: u64,
    max_retries: u32,
    verbose: bool,
    format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum OutputFormat {
    Text,
    Json,
    Prometheus,
    Summary,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            server_url: "http://[::1]:9090".to_string(),
            timeout_seconds: 30,
            check_interval: 10,
            max_retries: 3,
            verbose: false,
            format: OutputFormat::Summary,
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum MetricsError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
    
    #[error("Service unavailable after {0} retries")]
    ServiceUnavailable(u32),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct PrometheusMetric {
    name: String,
    help: Option<String>,
    r#type: Option<String>,
    labels: HashMap<String, String>,
    value: f64,
    timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricsSummary {
    total_metrics: usize,
    counters: Vec<String>,
    gauges: Vec<String>,
    histograms: Vec<String>,
    summary: Vec<String>,
    timestamp: String,
    service_name: String,
    version: String,
}

struct MetricsClient {
    client: Client,
    config: MetricsConfig,
    retry_count: u32,
}

impl MetricsClient {
    fn new(config: MetricsConfig) -> Result<Self, MetricsError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| MetricsError::InvalidArgument(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            client,
            config,
            retry_count: 0,
        })
    }
    
    async fn get_prometheus_metrics(&self) -> Result<String, MetricsError> {
        let url = format!("{}/metrics", self.config.server_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch metrics: {}", e);
                MetricsError::Http(e)
            })?;
        
        if !response.status().is_success() {
            return Err(MetricsError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.status().as_str())
            ));
        }
        
        let body = response.text().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            MetricsError::Http(e)
        })?;
        
        Ok(body)
    }
    
    async fn get_grpc_metrics(&self) -> Result<serde_json::Value, MetricsError> {
        let url = format!("{}/metrics", self.config.server_url.replace("9090", "50051"));
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch gRPC metrics: {}", e);
                MetricsError::Http(e)
            })?;
        
        if !response.status().is_success() {
            return Err(MetricsError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.status().as_str())
            ));
        }
        
        let body = response.json::<serde_json::Value>().await.map_err(|e| {
            error!("Failed to parse gRPC metrics: {}", e);
            MetricsError::Http(e)
        })?;
        
        Ok(body)
    }
    
    fn parse_prometheus_metrics(&self, raw_metrics: &str) -> Vec<PrometheusMetric> {
        let mut metrics = Vec::new();
        let lines: Vec<&str> = raw_metrics.lines().collect();
        
        for line in lines {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            if let Some(metric) = self.parse_metric_line(line) {
                metrics.push(metric);
            }
        }
        
        metrics
    }
    
    fn parse_metric_line(&self, line: &str) -> Option<PrometheusMetric> {
        // Simple Prometheus metric parser
        // Format: metric_name{label="value"} value timestamp
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() < 2 {
            return None;
        }
        
        let metric_part = parts[0];
        let value_part = parts[1];
        
        // Parse metric name and labels
        let (name, labels) = if metric_part.contains('{') {
            let brace_start = metric_part.find('{').unwrap();
            let brace_end = metric_part.find('}').unwrap();
            
            let name = metric_part[..brace_start].to_string();
            let labels_str = &metric_part[brace_start + 1..brace_end];
            let labels = self.parse_labels(labels_str);
            
            (name, labels)
        } else {
            (metric_part.to_string(), HashMap::new())
        };
        
        // Parse value
        let value = value_part.parse::<f64>().ok()?;
        
        // Parse timestamp if present
        let timestamp = if parts.len() > 2 {
            Some(parts[2].to_string())
        } else {
            None
        };
        
        Some(PrometheusMetric {
            name,
            help: None,
            r#type: None,
            labels,
            value,
            timestamp,
        })
    }
    
    fn parse_labels(&self, labels_str: &str) -> HashMap<String, String> {
        let mut labels = HashMap::new();
        
        for label_pair in labels_str.split(',') {
            if let Some((key, value)) = label_pair.split_once('=') {
                let clean_value = value.trim_matches('"');
                labels.insert(key.to_string(), clean_value.to_string());
            }
        }
        
        labels
    }
    
    fn generate_summary(&self, metrics: &[PrometheusMetric]) -> MetricsSummary {
        let mut counters = Vec::new();
        let mut gauges = Vec::new();
        let mut histograms = Vec::new();
        let mut summary = Vec::new();
        
        for metric in metrics {
            let metric_name = metric.name.clone();
            if metric_name.contains("total") || metric_name.contains("counter") {
                counters.push(format!("{} = {}", metric_name, metric.value));
            } else if metric_name.contains("gauge") || metric_name.contains("current") {
                gauges.push(format!("{} = {}", metric_name, metric.value));
            } else if metric_name.contains("histogram") || metric_name.contains("bucket") {
                histograms.push(format!("{} = {}", metric_name, metric.value));
            } else if metric_name.contains("summary") || metric_name.contains("quantile") {
                summary.push(format!("{} = {}", metric_name, metric.value));
            }
        }
        
        MetricsSummary {
            total_metrics: metrics.len(),
            counters,
            gauges,
            histograms,
            summary,
            timestamp: chrono::Utc::now().to_rfc3339(),
            service_name: "hello-service".to_string(),
            version: "0.1.0".to_string(),
        }
    }
    
    async fn display_metrics(&self, format: &OutputFormat) -> Result<(), MetricsError> {
        let raw_metrics = self.get_prometheus_metrics().await?;
        let metrics = self.parse_prometheus_metrics(&raw_metrics);
        
        match format {
            OutputFormat::Text => {
                println!("📊 Prometheus Metrics ({} total):", metrics.len());
                for metric in metrics {
                    println!("  {} = {} {:?}", metric.name, metric.value, metric.labels);
                }
            }
            
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&metrics).unwrap();
                println!("{}", json);
            }
            
            OutputFormat::Prometheus => {
                println!("{}", raw_metrics);
            }
            
            OutputFormat::Summary => {
                let summary = self.generate_summary(&metrics);
                println!("📊 Metrics Summary:");
                println!("  Service: {} v{}", summary.service_name, summary.version);
                println!("  Total Metrics: {}", summary.total_metrics);
                println!("  Timestamp: {}", summary.timestamp);
                
                if !summary.counters.is_empty() {
                    println!("  📈 Counters:");
                    for counter in summary.counters {
                        println!("    {}", counter);
                    }
                }
                
                if !summary.gauges.is_empty() {
                    println!("  📊 Gauges:");
                    for gauge in summary.gauges {
                        println!("    {}", gauge);
                    }
                }
                
                if !summary.histograms.is_empty() {
                    println!("  📉 Histograms:");
                    for histogram in summary.histograms {
                        println!("    {}", histogram);
                    }
                }
                
                if !summary.summary.is_empty() {
                    println!("  📋 Summary:");
                    for summary_item in summary.summary {
                        println!("    {}", summary_item);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn continuous_monitoring(&self) -> Result<(), MetricsError> {
        info!("🔄 Starting continuous metrics monitoring every {} seconds", self.config.check_interval);
        info!("⏱️  Timeout: {} seconds", self.config.timeout_seconds);
        info!("🔄 Max retries: {}", self.config.max_retries);
        
        loop {
            match self.display_metrics(&self.config.format).await {
                Ok(_) => {
                    if self.config.verbose {
                        info!("✅ Metrics collected successfully");
                    }
                    self.retry_count = 0;
                }
                Err(e) => {
                    error!("❌ Failed to collect metrics: {}", e);
                    self.retry_count += 1;
                    
                    if self.retry_count >= self.config.max_retries {
                        return Err(MetricsError::ServiceUnavailable(self.retry_count));
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(self.config.check_interval)).await;
        }
    }
    
    async fn health_check(&self) -> Result<bool, MetricsError> {
        let url = format!("{}/metrics", self.config.server_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let healthy = response.status().is_success();
                if self.config.verbose {
                    if healthy {
                        info!("✅ Metrics endpoint is healthy");
                    } else {
                        warn!("⚠️  Metrics endpoint returned status: {}", response.status());
                    }
                }
                Ok(healthy)
            }
            Err(e) => {
                error!("❌ Metrics endpoint health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = App::new("Hello Service Metrics Client")
        .version("1.0")
        .author("Polymera OS Team")
        .about("Metrics client for Hello Service")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .value_name("URL")
                .help("Server URL (default: http://[::1]:9090)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("SECONDS")
                .help("Timeout in seconds (default: 30)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("SECONDS")
                .help("Check interval in seconds (default: 10)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max-retries")
                .short("m")
                .long("max-retries")
                .value_name("COUNT")
                .help("Maximum retries (default: 3)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .value_name("FORMAT")
                .help("Output format: text, json, prometheus, summary (default: summary)")
                .takes_value(true)
                .possible_values(&["text", "json", "prometheus", "summary"]),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enable verbose output"),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get metrics once"),
        )
        .subcommand(
            SubCommand::with_name("monitor")
                .about("Monitor metrics continuously"),
        )
        .subcommand(
            SubCommand::with_name("health")
                .about("Check metrics endpoint health"),
        )
        .get_matches();
    
    // Parse configuration
    let mut config = MetricsConfig::default();
    
    if let Some(server) = matches.value_of("server") {
        config.server_url = server.to_string();
    }
    
    if let Some(timeout) = matches.value_of("timeout") {
        config.timeout_seconds = timeout.parse().unwrap_or(30);
    }
    
    if let Some(interval) = matches.value_of("interval") {
        config.check_interval = interval.parse().unwrap_or(10);
    }
    
    if let Some(max_retries) = matches.value_of("max-retries") {
        config.max_retries = max_retries.parse().unwrap_or(3);
    }
    
    if let Some(format) = matches.value_of("format") {
        config.format = match format {
            "text" => OutputFormat::Text,
            "json" => OutputFormat::Json,
            "prometheus" => OutputFormat::Prometheus,
            "summary" => OutputFormat::Summary,
            _ => OutputFormat::Summary,
        };
    }
    
    config.verbose = matches.is_present("verbose");
    
    // Create client
    let client = MetricsClient::new(config.clone())?;
    
    // Handle commands
    match matches.subcommand() {
        ("get", Some(_)) => {
            client.display_metrics(&config.format).await?;
        }
        
        ("monitor", Some(_)) => {
            client.continuous_monitoring().await?;
        }
        
        ("health", Some(_)) => {
            let healthy = client.health_check().await?;
            if healthy {
                println!("✅ Metrics endpoint is healthy");
            } else {
                println!("❌ Metrics endpoint is unhealthy");
            }
        }
        
        _ => {
            // Default to getting metrics once
            client.display_metrics(&config.format).await?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert_eq!(config.server_url, "http://[::1]:9090");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.check_interval, 10);
        assert_eq!(config.max_retries, 3);
        assert!(!config.verbose);
    }
    
    #[test]
    fn test_metrics_error_display() {
        let error = MetricsError::InvalidArgument("test".to_string());
        assert_eq!(error.to_string(), "Invalid argument: test");
    }
    
    #[test]
    fn test_parse_labels() {
        let client = MetricsClient::new(MetricsConfig::default()).unwrap();
        let labels_str = r#"method="say_hello",language="en""#;
        let labels = client.parse_labels(labels_str);
        
        assert_eq!(labels.get("method"), Some(&"say_hello".to_string()));
        assert_eq!(labels.get("language"), Some(&"en".to_string()));
    }
}
