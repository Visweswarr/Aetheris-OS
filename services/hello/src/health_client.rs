use clap::{App, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tonic::{transport::Channel, Request};
use tonic_health::{
    health_client::HealthClient, HealthCheckRequest, HealthCheckResponse,
    HealthCheckResponse_ServingStatus,
};
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct HealthConfig {
    server_url: String,
    timeout_seconds: u64,
    check_interval: u64,
    max_failures: u32,
    verbose: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            server_url: "http://[::1]:50051".to_string(),
            timeout_seconds: 10,
            check_interval: 5,
            max_failures: 3,
            verbose: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum HealthError {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Service unhealthy after {0} consecutive failures")]
    ServiceUnhealthy(u32),
    
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

struct HealthClient {
    client: HealthClient<Channel>,
    config: HealthConfig,
    failure_count: u32,
}

impl HealthClient {
    async fn new(config: HealthConfig) -> Result<Self, HealthError> {
        let channel = Channel::from_shared(config.server_url.clone())
            .map_err(|e| HealthError::InvalidArgument(format!("Invalid URL: {}", e)))?
            .connect_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .connect()
            .await
            .map_err(|e| {
                error!("Failed to connect to server: {}", e);
                HealthError::Transport(e)
            })?;
        
        let client = HealthClient::new(channel);
        
        Ok(Self {
            client,
            config,
            failure_count: 0,
        })
    }
    
    async fn check_health(&mut self) -> Result<bool, HealthError> {
        let request = Request::new(HealthCheckRequest {
            service: "".to_string(),
        });
        
        match self.client.check(request).await {
            Ok(response) => {
                let response = response.into_inner();
                let status = response.status;
                
                match status {
                    HealthCheckResponse_ServingStatus::Serving => {
                        if self.config.verbose {
                            info!("✅ Service is healthy (SERVING)");
                        }
                        self.failure_count = 0;
                        Ok(true)
                    }
                    HealthCheckResponse_ServingStatus::NotServing => {
                        warn!("❌ Service is unhealthy (NOT_SERVING)");
                        self.failure_count += 1;
                        Ok(false)
                    }
                    HealthCheckResponse_ServingStatus::ServiceUnknown => {
                        warn!("❓ Service status unknown (SERVICE_UNKNOWN)");
                        self.failure_count += 1;
                        Ok(false)
                    }
                    _ => {
                        warn!("❓ Service status unknown ({:?})", status);
                        self.failure_count += 1;
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                self.failure_count += 1;
                
                if self.failure_count >= self.config.max_failures {
                    return Err(HealthError::ServiceUnhealthy(self.failure_count));
                }
                
                Ok(false)
            }
        }
    }
    
    async fn watch_health(&mut self) -> Result<(), HealthError> {
        let request = Request::new(HealthCheckRequest {
            service: "".to_string(),
        });
        
        match self.client.watch(request).await {
            Ok(mut response) => {
                info!("🔍 Starting health watch...");
                
                while let Some(health_response) = response.message().await? {
                    let status = health_response.status;
                    let timestamp = std::time::SystemTime::now();
                    
                    match status {
                        HealthCheckResponse_ServingStatus::Serving => {
                            if self.config.verbose {
                                info!("✅ [{}] Service is healthy (SERVING)", 
                                    chrono::DateTime::<chrono::Utc>::from(timestamp).format("%H:%M:%S"));
                            }
                            self.failure_count = 0;
                        }
                        HealthCheckResponse_ServingStatus::NotServing => {
                            warn!("❌ [{}] Service is unhealthy (NOT_SERVING)", 
                                chrono::DateTime::<chrono::Utc>::from(timestamp).format("%H:%M:%S"));
                            self.failure_count += 1;
                        }
                        HealthCheckResponse_ServingStatus::ServiceUnknown => {
                            warn!("❓ [{}] Service status unknown (SERVICE_UNKNOWN)", 
                                chrono::DateTime::<chrono::DateTime::<chrono::Utc>>::from(timestamp).format("%H:%M:%S"));
                            self.failure_count += 1;
                        }
                        _ => {
                            warn!("❓ [{}] Service status unknown ({:?})", 
                                chrono::DateTime::<chrono::Utc>::from(timestamp).format("%H:%M:%S"), status);
                            self.failure_count += 1;
                        }
                    }
                    
                    if self.failure_count >= self.config.max_failures {
                        return Err(HealthError::ServiceUnhealthy(self.failure_count));
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_secs(self.config.check_interval)).await;
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Health watch failed: {}", e);
                Err(HealthError::Grpc(e))
            }
        }
    }
    
    async fn continuous_health_check(&mut self) -> Result<(), HealthError> {
        info!("🔄 Starting continuous health checks every {} seconds", self.config.check_interval);
        info!("⏱️  Timeout: {} seconds", self.config.timeout_seconds);
        info!("🚫 Max failures: {}", self.config.max_failures);
        
        loop {
            match self.check_health().await {
                Ok(healthy) => {
                    if !healthy && self.config.verbose {
                        warn!("⚠️  Health check failed ({} failures)", self.failure_count);
                    }
                }
                Err(e) => {
                    error!("🚨 Health check error: {}", e);
                    return Err(e);
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_secs(self.config.check_interval)).await;
        }
    }
    
    fn get_status_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        summary.insert("failure_count".to_string(), self.failure_count.to_string());
        summary.insert("max_failures".to_string(), self.config.max_failures.to_string());
        summary.insert("check_interval".to_string(), format!("{}s", self.config.check_interval));
        summary.insert("timeout".to_string(), format!("{}s", self.config.timeout_seconds));
        summary.insert("server_url".to_string(), self.config.server_url.clone());
        
        let status = if self.failure_count == 0 {
            "HEALTHY"
        } else if self.failure_count < self.config.max_failures {
            "DEGRADED"
        } else {
            "UNHEALTHY"
        };
        
        summary.insert("overall_status".to_string(), status.to_string());
        
        summary
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = App::new("Hello Service Health Client")
        .version("1.0")
        .author("Polymera OS Team")
        .about("Health check client for Hello Service")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .value_name("URL")
                .help("Server URL (default: http://[::1]:50051)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("SECONDS")
                .help("Timeout in seconds (default: 10)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("SECONDS")
                .help("Check interval in seconds (default: 5)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max-failures")
                .short("m")
                .long("max-failures")
                .value_name("COUNT")
                .help("Maximum consecutive failures (default: 3)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enable verbose output"),
        )
        .subcommand(
            SubCommand::with_name("check")
                .about("Perform a single health check"),
        )
        .subcommand(
            SubCommand::with_name("watch")
                .about("Watch health status continuously"),
        )
        .subcommand(
            SubCommand::with_name("monitor")
                .about("Monitor health with continuous checks"),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Show current health status"),
        )
        .get_matches();
    
    // Parse configuration
    let mut config = HealthConfig::default();
    
    if let Some(server) = matches.value_of("server") {
        config.server_url = server.to_string();
    }
    
    if let Some(timeout) = matches.value_of("timeout") {
        config.timeout_seconds = timeout.parse().unwrap_or(10);
    }
    
    if let Some(interval) = matches.value_of("interval") {
        config.check_interval = interval.parse().unwrap_or(5);
    }
    
    if let Some(max_failures) = matches.value_of("max-failures") {
        config.max_failures = max_failures.parse().unwrap_or(3);
    }
    
    config.verbose = matches.is_present("verbose");
    
    // Create client
    let mut client = HealthClient::new(config).await?;
    
    // Handle commands
    match matches.subcommand() {
        ("check", Some(_)) => {
            let healthy = client.check_health().await?;
            if healthy {
                println!("✅ Service is healthy");
            } else {
                println!("❌ Service is unhealthy");
            }
        }
        
        ("watch", Some(_)) => {
            client.watch_health().await?;
        }
        
        ("monitor", Some(_)) => {
            client.continuous_health_check().await?;
        }
        
        ("status", Some(_)) => {
            let summary = client.get_status_summary();
            println!("📊 Health Status Summary:");
            for (key, value) in summary {
                println!("  {}: {}", key, value);
            }
        }
        
        _ => {
            // Default to single check
            let healthy = client.check_health().await?;
            if healthy {
                println!("✅ Service is healthy");
            } else {
                println!("❌ Service is unhealthy");
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.server_url, "http://[::1]:50051");
        assert_eq!(config.timeout_seconds, 10);
        assert_eq!(config.check_interval, 5);
        assert_eq!(config.max_failures, 3);
        assert!(!config.verbose);
    }
    
    #[test]
    fn test_health_error_display() {
        let error = HealthError::InvalidArgument("test".to_string());
        assert_eq!(error.to_string(), "Invalid argument: test");
    }
}
