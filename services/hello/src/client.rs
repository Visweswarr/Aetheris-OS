use clap::{App, Arg, SubCommand};
use hello::hello_service_client::HelloServiceClient;
use hello::{
    EchoRequest, SayHelloRequest, SayHelloStreamRequest, SayHelloStreamResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::{transport::Channel, Request, Response};
use tracing::{error, info, warn};

mod hello {
    tonic::include_proto!("hello.v1");
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientConfig {
    server_url: String,
    timeout_seconds: u64,
    retry_count: u32,
    language: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://[::1]:50051".to_string(),
            timeout_seconds: 30,
            retry_count: 3,
            language: "en".to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum ClientError {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
    
    #[error("Service unavailable")]
    ServiceUnavailable,
}

struct HelloClient {
    client: HelloServiceClient<Channel>,
    config: ClientConfig,
}

impl HelloClient {
    async fn new(config: ClientConfig) -> Result<Self, ClientError> {
        let channel = Channel::from_shared(config.server_url.clone())
            .map_err(|e| ClientError::InvalidArgument(format!("Invalid URL: {}", e)))?
            .connect_timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .connect()
            .await
            .map_err(|e| {
                error!("Failed to connect to server: {}", e);
                ClientError::ServiceUnavailable
            })?;
        
        let client = HelloServiceClient::new(channel);
        
        Ok(Self { client, config })
    }
    
    async fn say_hello(&mut self, name: &str, language: Option<&str>) -> Result<String, ClientError> {
        let lang = language.unwrap_or(&self.config.language);
        let metadata = HashMap::new();
        
        let request = SayHelloRequest {
            name: Some(name.to_string()),
            language: Some(lang.to_string()),
            metadata,
            timestamp: Some(prost_types::Timestamp::from(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            )),
        };
        
        let response = self.client.say_hello(Request::new(request)).await?;
        let response = response.into_inner();
        
        Ok(response.message)
    }
    
    async fn say_hello_stream(
        &mut self,
        name: &str,
        language: Option<&str>,
        count: i32,
        interval_ms: i32,
    ) -> Result<Vec<String>, ClientError> {
        let lang = language.unwrap_or(&self.config.language);
        
        let request = SayHelloStreamRequest {
            name: Some(name.to_string()),
            language: Some(lang.to_string()),
            count,
            interval_ms,
        };
        
        let mut response = self.client.say_hello_stream(Request::new(request)).await?;
        let response = response.into_inner();
        
        let mut messages = Vec::new();
        while let Some(message) = response.message().await? {
            messages.push(message.message);
        }
        
        Ok(messages)
    }
    
    async fn echo(
        &mut self,
        message: &str,
        uppercase: bool,
        repeat_count: i32,
    ) -> Result<String, ClientError> {
        let headers = HashMap::new();
        
        let request = EchoRequest {
            message: message.to_string(),
            uppercase,
            repeat_count,
            headers,
        };
        
        let response = self.client.echo(Request::new(request)).await?;
        let response = response.into_inner();
        
        Ok(response.message)
    }
    
    async fn health_check(&mut self) -> Result<bool, ClientError> {
        let request = tonic::Request::new(());
        
        match self.client.health_check(request).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    async fn get_metrics(&mut self) -> Result<(), ClientError> {
        let request = tonic::Request::new(());
        
        let response = self.client.get_metrics(request).await?;
        let response = response.into_inner();
        
        info!("Service: {} v{}", response.service_name, response.version);
        info!("Metrics timestamp: {:?}", response.timestamp);
        
        for metric in response.metrics {
            info!(
                "Metric: {} ({}) = {:?}",
                metric.name, metric.description, metric.value
            );
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = App::new("Hello Service Client")
        .version("1.0")
        .author("Polymera OS Team")
        .about("gRPC client for Hello Service")
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
                .help("Timeout in seconds (default: 30)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("language")
                .short("l")
                .long("language")
                .value_name("LANG")
                .help("Language for greetings (default: en)")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("hello")
                .about("Send a hello request")
                .arg(
                    Arg::with_name("name")
                        .help("Name to greet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("lang")
                        .short("l")
                        .long("language")
                        .help("Language for greeting")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("stream")
                .about("Send a streaming hello request")
                .arg(
                    Arg::with_name("name")
                        .help("Name to greet")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("count")
                        .short("c")
                        .long("count")
                        .help("Number of messages (default: 5)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("interval")
                        .short("i")
                        .long("interval")
                        .help("Interval between messages in ms (default: 1000)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("lang")
                        .short("l")
                        .long("language")
                        .help("Language for greeting")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("echo")
                .about("Echo a message")
                .arg(
                    Arg::with_name("message")
                        .help("Message to echo")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("uppercase")
                        .short("u")
                        .long("uppercase")
                        .help("Convert to uppercase"),
                )
                .arg(
                    Arg::with_name("repeat")
                        .short("r")
                        .long("repeat")
                        .help("Repeat count (default: 1)")
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("health").about("Check service health"))
        .subcommand(SubCommand::with_name("metrics").about("Get service metrics"))
        .get_matches();
    
    // Parse configuration
    let mut config = ClientConfig::default();
    
    if let Some(server) = matches.value_of("server") {
        config.server_url = server.to_string();
    }
    
    if let Some(timeout) = matches.value_of("timeout") {
        config.timeout_seconds = timeout.parse().unwrap_or(30);
    }
    
    if let Some(language) = matches.value_of("language") {
        config.language = language.to_string();
    }
    
    // Create client
    let mut client = HelloClient::new(config).await?;
    
    // Handle commands
    match matches.subcommand() {
        ("hello", Some(sub_matches)) => {
            let name = sub_matches.value_of("name").unwrap();
            let lang = sub_matches.value_of("lang");
            
            let message = client.say_hello(name, lang).await?;
            println!("{}", message);
        }
        
        ("stream", Some(sub_matches)) => {
            let name = sub_matches.value_of("name").unwrap();
            let count = sub_matches
                .value_of("count")
                .unwrap_or("5")
                .parse()
                .unwrap_or(5);
            let interval = sub_matches
                .value_of("interval")
                .unwrap_or("1000")
                .parse()
                .unwrap_or(1000);
            let lang = sub_matches.value_of("lang");
            
            let messages = client.say_hello_stream(name, lang, count, interval).await?;
            
            for (i, message) in messages.iter().enumerate() {
                println!("[{}] {}", i + 1, message);
            }
        }
        
        ("echo", Some(sub_matches)) => {
            let message = sub_matches.value_of("message").unwrap();
            let uppercase = sub_matches.is_present("uppercase");
            let repeat = sub_matches
                .value_of("repeat")
                .unwrap_or("1")
                .parse()
                .unwrap_or(1);
            
            let response = client.echo(message, uppercase, repeat).await?;
            println!("{}", response);
        }
        
        ("health", Some(_)) => {
            let healthy = client.health_check().await?;
            if healthy {
                println!("✅ Service is healthy");
            } else {
                println!("❌ Service is unhealthy");
            }
        }
        
        ("metrics", Some(_)) => {
            client.get_metrics().await?;
        }
        
        _ => {
            println!("Use --help for usage information");
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.server_url, "http://[::1]:50051");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.language, "en");
    }
    
    #[test]
    fn test_client_error_display() {
        let error = ClientError::InvalidArgument("test".to_string());
        assert_eq!(error.to_string(), "Invalid argument: test");
    }
}
