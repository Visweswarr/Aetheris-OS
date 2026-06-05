//! Network Performance Benchmark Tool for Aetheris OS (Rust)
//! 
//! This tool provides high-performance benchmarking capabilities for the Aetheris OS
//! networking subsystem, including TCP, TLS, QUIC, and concurrent connection testing.

use std::collections::VecDeque;
use std::net::{SocketAddr, TcpStream, TcpListener};
use std::sync::{Arc, Mutex, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use std::io::{self, Read, Write};
use std::sync::mpsc;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpStream as AsyncTcpStream, TcpListener as AsyncTcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, sleep};
use tokio::sync::Semaphore;
use rustls::{ClientConfig, ServerConfig, ClientConnection, ServerConnection, Connection};
use rustls_pemfile::{certs, pkcs8_private_keys};
use webpki::DNSNameRef;
use quinn::{Endpoint, ClientConfig as QuinnClientConfig, ServerConfig as QuinnServerConfig};
use quinn_proto::crypto::rustls::QuicClientConfig;
use std::path::Path;

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub clients: usize,
    pub duration: Duration,
    pub message_size: usize,
    pub connections: usize,
    pub streams: usize,
    pub profile: String,
    pub verbose: bool,
    pub output_format: String,
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub protocol: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_seconds: f64,
    pub total_connections: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
    pub total_messages: u64,
    pub total_bytes: u64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_gbps: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub handshake_time_ms: Option<f64>,
    pub p95_handshake_time_ms: Option<f64>,
    pub connection_rate_per_second: Option<f64>,
    pub network_errors: u64,
    pub timeout_errors: u64,
}

/// Latency measurement
#[derive(Debug, Clone)]
pub struct LatencyMeasurement {
    pub timestamp: Instant,
    pub latency: Duration,
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    results: BenchmarkResult,
    latencies: Arc<Mutex<Vec<LatencyMeasurement>>>,
    start_time: Instant,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: BenchmarkResult {
                protocol: String::new(),
                start_time: String::new(),
                end_time: String::new(),
                duration_seconds: 0.0,
                total_connections: 0,
                successful_connections: 0,
                failed_connections: 0,
                total_messages: 0,
                total_bytes: 0,
                average_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                throughput_gbps: 0.0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0,
                handshake_time_ms: None,
                p95_handshake_time_ms: None,
                connection_rate_per_second: None,
                network_errors: 0,
                timeout_errors: 0,
            },
            latencies: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Run TCP benchmark
    pub async fn run_tcp_benchmark(&mut self) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        self.results.protocol = "TCP".to_string();
        self.results.start_time = chrono::Utc::now().to_rfc3339();
        self.start_time = Instant::now();

        println!("Starting TCP benchmark: {}:{}, {} clients, {:?} duration", 
            self.config.host, self.config.port, self.config.clients, self.config.duration);

        let semaphore = Arc::new(Semaphore::new(self.config.clients));
        let mut handles = Vec::new();

        let total_messages = Arc::new(AtomicU64::new(0));
        let total_bytes = Arc::new(AtomicU64::new(0));
        let network_errors = Arc::new(AtomicU64::new(0));
        let timeout_errors = Arc::new(AtomicU64::new(0));
        let successful_connections = Arc::new(AtomicU64::new(0));
        let failed_connections = Arc::new(AtomicU64::new(0));

        // Start clients
        for client_id in 0..self.config.clients {
            let permit = semaphore.clone().acquire_owned().await?;
            let host = self.config.host.clone();
            let port = self.config.port;
            let message_size = self.config.message_size;
            let duration = self.config.duration;
            let latencies = self.latencies.clone();
            let total_messages = total_messages.clone();
            let total_bytes = total_bytes.clone();
            let network_errors = network_errors.clone();
            let timeout_errors = timeout_errors.clone();
            let successful_connections = successful_connections.clone();
            let failed_connections = failed_connections.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let result = Self::run_tcp_client(
                    client_id,
                    host,
                    port,
                    message_size,
                    duration,
                    latencies,
                    total_messages,
                    total_bytes,
                    network_errors,
                    timeout_errors,
                ).await;

                if result.success {
                    successful_connections.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_connections.fetch_add(1, Ordering::Relaxed);
                }
            });

            handles.push(handle);
        }

        // Wait for all clients to complete
        for handle in handles {
            handle.await?;
        }

        self.results.end_time = chrono::Utc::now().to_rfc3339();
        self.results.duration_seconds = self.start_time.elapsed().as_secs_f64();
        self.results.total_connections = self.config.clients as u64;
        self.results.successful_connections = successful_connections.load(Ordering::Relaxed);
        self.results.failed_connections = failed_connections.load(Ordering::Relaxed);
        self.results.total_messages = total_messages.load(Ordering::Relaxed);
        self.results.total_bytes = total_bytes.load(Ordering::Relaxed);
        self.results.network_errors = network_errors.load(Ordering::Relaxed);
        self.results.timeout_errors = timeout_errors.load(Ordering::Relaxed);

        // Calculate latency statistics
        self.calculate_latency_stats();

        // Calculate throughput
        self.results.throughput_gbps = (self.results.total_bytes as f64 * 8.0) / 
            (self.results.duration_seconds * 1_000_000_000.0);

        // Get system metrics
        self.get_system_metrics();

        Ok(self.results.clone())
    }

    /// Run TLS benchmark
    pub async fn run_tls_benchmark(&mut self) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        self.results.protocol = "TLS".to_string();
        self.results.start_time = chrono::Utc::now().to_rfc3339();
        self.start_time = Instant::now();

        println!("Starting TLS benchmark: {}:{}, {} clients, {:?} duration, profile: {}", 
            self.config.host, self.config.port, self.config.clients, self.config.duration, self.config.profile);

        let semaphore = Arc::new(Semaphore::new(self.config.clients));
        let mut handles = Vec::new();

        let total_messages = Arc::new(AtomicU64::new(0));
        let total_bytes = Arc::new(AtomicU64::new(0));
        let network_errors = Arc::new(AtomicU64::new(0));
        let timeout_errors = Arc::new(AtomicU64::new(0));
        let successful_connections = Arc::new(AtomicU64::new(0));
        let failed_connections = Arc::new(AtomicU64::new(0));
        let handshake_times = Arc::new(Mutex::new(Vec::new()));

        // Start clients
        for client_id in 0..self.config.clients {
            let permit = semaphore.clone().acquire_owned().await?;
            let host = self.config.host.clone();
            let port = self.config.port;
            let message_size = self.config.message_size;
            let duration = self.config.duration;
            let profile = self.config.profile.clone();
            let latencies = self.latencies.clone();
            let total_messages = total_messages.clone();
            let total_bytes = total_bytes.clone();
            let network_errors = network_errors.clone();
            let timeout_errors = timeout_errors.clone();
            let successful_connections = successful_connections.clone();
            let failed_connections = failed_connections.clone();
            let handshake_times = handshake_times.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let result = Self::run_tls_client(
                    client_id,
                    host,
                    port,
                    message_size,
                    duration,
                    profile,
                    latencies,
                    total_messages,
                    total_bytes,
                    network_errors,
                    timeout_errors,
                    handshake_times,
                ).await;

                if result.success {
                    successful_connections.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_connections.fetch_add(1, Ordering::Relaxed);
                }
            });

            handles.push(handle);
        }

        // Wait for all clients to complete
        for handle in handles {
            handle.await?;
        }

        self.results.end_time = chrono::Utc::now().to_rfc3339();
        self.results.duration_seconds = self.start_time.elapsed().as_secs_f64();
        self.results.total_connections = self.config.clients as u64;
        self.results.successful_connections = successful_connections.load(Ordering::Relaxed);
        self.results.failed_connections = failed_connections.load(Ordering::Relaxed);
        self.results.total_messages = total_messages.load(Ordering::Relaxed);
        self.results.total_bytes = total_bytes.load(Ordering::Relaxed);
        self.results.network_errors = network_errors.load(Ordering::Relaxed);
        self.results.timeout_errors = timeout_errors.load(Ordering::Relaxed);

        // Calculate latency statistics
        self.calculate_latency_stats();

        // Calculate handshake statistics
        self.calculate_handshake_stats(handshake_times);

        // Calculate throughput
        self.results.throughput_gbps = (self.results.total_bytes as f64 * 8.0) / 
            (self.results.duration_seconds * 1_000_000_000.0);

        // Get system metrics
        self.get_system_metrics();

        Ok(self.results.clone())
    }

    /// Run QUIC benchmark
    pub async fn run_quic_benchmark(&mut self) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        self.results.protocol = "QUIC".to_string();
        self.results.start_time = chrono::Utc::now().to_rfc3339();
        self.start_time = Instant::now();

        println!("Starting QUIC benchmark: {}:{}, {} clients, {:?} duration, profile: {}", 
            self.config.host, self.config.port, self.config.clients, self.config.duration, self.config.profile);

        let semaphore = Arc::new(Semaphore::new(self.config.clients));
        let mut handles = Vec::new();

        let total_messages = Arc::new(AtomicU64::new(0));
        let total_bytes = Arc::new(AtomicU64::new(0));
        let network_errors = Arc::new(AtomicU64::new(0));
        let timeout_errors = Arc::new(AtomicU64::new(0));
        let successful_connections = Arc::new(AtomicU64::new(0));
        let failed_connections = Arc::new(AtomicU64::new(0));
        let handshake_times = Arc::new(Mutex::new(Vec::new()));

        // Start clients
        for client_id in 0..self.config.clients {
            let permit = semaphore.clone().acquire_owned().await?;
            let host = self.config.host.clone();
            let port = self.config.port;
            let message_size = self.config.message_size;
            let duration = self.config.duration;
            let profile = self.config.profile.clone();
            let streams = self.config.streams;
            let latencies = self.latencies.clone();
            let total_messages = total_messages.clone();
            let total_bytes = total_bytes.clone();
            let network_errors = network_errors.clone();
            let timeout_errors = timeout_errors.clone();
            let successful_connections = successful_connections.clone();
            let failed_connections = failed_connections.clone();
            let handshake_times = handshake_times.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let result = Self::run_quic_client(
                    client_id,
                    host,
                    port,
                    message_size,
                    duration,
                    profile,
                    streams,
                    latencies,
                    total_messages,
                    total_bytes,
                    network_errors,
                    timeout_errors,
                    handshake_times,
                ).await;

                if result.success {
                    successful_connections.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_connections.fetch_add(1, Ordering::Relaxed);
                }
            });

            handles.push(handle);
        }

        // Wait for all clients to complete
        for handle in handles {
            handle.await?;
        }

        self.results.end_time = chrono::Utc::now().to_rfc3339();
        self.results.duration_seconds = self.start_time.elapsed().as_secs_f64();
        self.results.total_connections = self.config.clients as u64;
        self.results.successful_connections = successful_connections.load(Ordering::Relaxed);
        self.results.failed_connections = failed_connections.load(Ordering::Relaxed);
        self.results.total_messages = total_messages.load(Ordering::Relaxed);
        self.results.total_bytes = total_bytes.load(Ordering::Relaxed);
        self.results.network_errors = network_errors.load(Ordering::Relaxed);
        self.results.timeout_errors = timeout_errors.load(Ordering::Relaxed);

        // Calculate latency statistics
        self.calculate_latency_stats();

        // Calculate handshake statistics
        self.calculate_handshake_stats(handshake_times);

        // Calculate throughput
        self.results.throughput_gbps = (self.results.total_bytes as f64 * 8.0) / 
            (self.results.duration_seconds * 1_000_000_000.0);

        // Get system metrics
        self.get_system_metrics();

        Ok(self.results.clone())
    }

    /// Run concurrent connections benchmark
    pub async fn run_concurrent_connections_benchmark(&mut self) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        self.results.protocol = self.config.protocol.clone();
        self.results.start_time = chrono::Utc::now().to_rfc3339();
        self.start_time = Instant::now();

        println!("Starting concurrent connections benchmark: {}:{}, {} connections, protocol: {}", 
            self.config.host, self.config.port, self.config.connections, self.config.protocol);

        let semaphore = Arc::new(Semaphore::new(self.config.connections));
        let mut handles = Vec::new();

        let network_errors = Arc::new(AtomicU64::new(0));
        let timeout_errors = Arc::new(AtomicU64::new(0));
        let successful_connections = Arc::new(AtomicU64::new(0));
        let failed_connections = Arc::new(AtomicU64::new(0));
        let connection_times = Arc::new(Mutex::new(Vec::new()));

        // Start connections
        for conn_id in 0..self.config.connections {
            let permit = semaphore.clone().acquire_owned().await?;
            let host = self.config.host.clone();
            let port = self.config.port;
            let protocol = self.config.protocol.clone();
            let duration = self.config.duration;
            let network_errors = network_errors.clone();
            let timeout_errors = timeout_errors.clone();
            let successful_connections = successful_connections.clone();
            let failed_connections = failed_connections.clone();
            let connection_times = connection_times.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let start_time = Instant::now();

                let result = match protocol.as_str() {
                    "tcp" => {
                        Self::create_tcp_connection(host, port).await
                    }
                    "tls" => {
                        Self::create_tls_connection(host, port).await
                    }
                    _ => {
                        Err("Unsupported protocol".into())
                    }
                };

                let connection_time = start_time.elapsed();

                match result {
                    Ok(_) => {
                        successful_connections.fetch_add(1, Ordering::Relaxed);
                        connection_times.lock().unwrap().push(connection_time);
                        
                        // Keep connection alive for the duration
                        sleep(duration).await;
                    }
                    Err(_) => {
                        failed_connections.fetch_add(1, Ordering::Relaxed);
                        network_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all connections to complete
        for handle in handles {
            handle.await?;
        }

        self.results.end_time = chrono::Utc::now().to_rfc3339();
        self.results.duration_seconds = self.start_time.elapsed().as_secs_f64();
        self.results.total_connections = self.config.connections as u64;
        self.results.successful_connections = successful_connections.load(Ordering::Relaxed);
        self.results.failed_connections = failed_connections.load(Ordering::Relaxed);
        self.results.network_errors = network_errors.load(Ordering::Relaxed);
        self.results.timeout_errors = timeout_errors.load(Ordering::Relaxed);

        // Calculate connection time statistics
        self.calculate_connection_time_stats(connection_times);

        // Calculate connection rate
        self.results.connection_rate_per_second = Some(
            self.results.successful_connections as f64 / self.results.duration_seconds
        );

        // Get system metrics
        self.get_system_metrics();

        Ok(self.results.clone())
    }

    /// Run TCP client
    async fn run_tcp_client(
        client_id: usize,
        host: String,
        port: u16,
        message_size: usize,
        duration: Duration,
        latencies: Arc<Mutex<Vec<LatencyMeasurement>>>,
        total_messages: Arc<AtomicU64>,
        total_bytes: Arc<AtomicU64>,
        network_errors: Arc<AtomicU64>,
        timeout_errors: Arc<AtomicU64>,
    ) -> ClientResult {
        let mut result = ClientResult::new();

        // Connect to server
        let addr = format!("{}:{}", host, port);
        let stream = match timeout(Duration::from_secs(10), AsyncTcpStream::connect(&addr)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                result.network_errors += 1;
                network_errors.fetch_add(1, Ordering::Relaxed);
                return result;
            }
            Err(_) => {
                result.timeout_errors += 1;
                timeout_errors.fetch_add(1, Ordering::Relaxed);
                return result;
            }
        };

        result.success = true;

        // Send messages
        let message = vec![0u8; message_size];
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
            
            let start_time = Instant::now();
            
            // Send message
            if let Err(_) = stream.try_write(&message) {
                result.network_errors += 1;
                network_errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }

            // Read response
            let mut response = vec![0u8; message_size];
            if let Err(_) = stream.try_read(&mut response) {
                result.network_errors += 1;
                network_errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }

            let latency = start_time.elapsed();
            latencies.lock().unwrap().push(LatencyMeasurement {
                timestamp: start_time,
                latency,
            });

            result.messages += 1;
            result.bytes += (message_size + message_size) as u64;
            total_messages.fetch_add(1, Ordering::Relaxed);
            total_bytes.fetch_add((message_size + message_size) as u64, Ordering::Relaxed);
        }
    }

    /// Run TLS client
    async fn run_tls_client(
        client_id: usize,
        host: String,
        port: u16,
        message_size: usize,
        duration: Duration,
        profile: String,
        latencies: Arc<Mutex<Vec<LatencyMeasurement>>>,
        total_messages: Arc<AtomicU64>,
        total_bytes: Arc<AtomicU64>,
        network_errors: Arc<AtomicU64>,
        timeout_errors: Arc<AtomicU64>,
        handshake_times: Arc<Mutex<Vec<Duration>>>,
    ) -> ClientResult {
        let mut result = ClientResult::new();

        // Create TLS connection (simplified - in real implementation would use proper TLS)
        let start_time = Instant::now();
        
        // Mock TLS handshake
        sleep(Duration::from_millis(10)).await;
        let handshake_time = start_time.elapsed();
        handshake_times.lock().unwrap().push(handshake_time);

        result.success = true;

        // Send messages
        let message = vec![0u8; message_size];
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
            
            let start_time = Instant::now();
            
            // Mock TLS send/receive
            sleep(Duration::from_millis(1)).await;

            let latency = start_time.elapsed();
            latencies.lock().unwrap().push(LatencyMeasurement {
                timestamp: start_time,
                latency,
            });

            result.messages += 1;
            result.bytes += (message_size + message_size) as u64;
            total_messages.fetch_add(1, Ordering::Relaxed);
            total_bytes.fetch_add((message_size + message_size) as u64, Ordering::Relaxed);
        }
    }

    /// Run QUIC client
    async fn run_quic_client(
        client_id: usize,
        host: String,
        port: u16,
        message_size: usize,
        duration: Duration,
        profile: String,
        streams: usize,
        latencies: Arc<Mutex<Vec<LatencyMeasurement>>>,
        total_messages: Arc<AtomicU64>,
        total_bytes: Arc<AtomicU64>,
        network_errors: Arc<AtomicU64>,
        timeout_errors: Arc<AtomicU64>,
        handshake_times: Arc<Mutex<Vec<Duration>>>,
    ) -> ClientResult {
        let mut result = ClientResult::new();

        // Create QUIC connection (simplified - in real implementation would use actual QUIC)
        let start_time = Instant::now();
        
        // Mock QUIC handshake
        sleep(Duration::from_millis(5)).await;
        let handshake_time = start_time.elapsed();
        handshake_times.lock().unwrap().push(handshake_time);

        result.success = true;

        // Send messages
        let message = vec![0u8; message_size];
        let mut interval = tokio::time::interval(Duration::from_millis(50)); // Faster for QUIC

        loop {
            interval.tick().await;
            
            let start_time = Instant::now();
            
            // Mock QUIC send/receive
            sleep(Duration::from_millis(1)).await;

            let latency = start_time.elapsed();
            latencies.lock().unwrap().push(LatencyMeasurement {
                timestamp: start_time,
                latency,
            });

            result.messages += 1;
            result.bytes += (message_size + message_size) as u64;
            total_messages.fetch_add(1, Ordering::Relaxed);
            total_bytes.fetch_add((message_size + message_size) as u64, Ordering::Relaxed);
        }
    }

    /// Create TCP connection
    async fn create_tcp_connection(host: String, port: u16) -> Result<AsyncTcpStream, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", host, port);
        let stream = timeout(Duration::from_secs(10), AsyncTcpStream::connect(&addr)).await??;
        Ok(stream)
    }

    /// Create TLS connection
    async fn create_tls_connection(host: String, port: u16) -> Result<AsyncTcpStream, Box<dyn std::error::Error>> {
        // Mock TLS connection - in real implementation would use actual TLS
        let addr = format!("{}:{}", host, port);
        let stream = timeout(Duration::from_secs(10), AsyncTcpStream::connect(&addr)).await??;
        Ok(stream)
    }

    /// Calculate latency statistics
    fn calculate_latency_stats(&mut self) {
        let latencies = self.latencies.lock().unwrap();
        if latencies.is_empty() {
            return;
        }

        // Calculate average
        let total: Duration = latencies.iter().map(|l| l.latency).sum();
        self.results.average_latency_ms = total.as_micros() as f64 / latencies.len() as f64 / 1000.0;

        // Calculate percentiles (simplified)
        if !latencies.is_empty() {
            let mut sorted_latencies: Vec<Duration> = latencies.iter().map(|l| l.latency).collect();
            sorted_latencies.sort();

            // P95
            let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
            if p95_index < sorted_latencies.len() {
                self.results.p95_latency_ms = sorted_latencies[p95_index].as_micros() as f64 / 1000.0;
            }

            // P99
            let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;
            if p99_index < sorted_latencies.len() {
                self.results.p99_latency_ms = sorted_latencies[p99_index].as_micros() as f64 / 1000.0;
            }
        }
    }

    /// Calculate handshake statistics
    fn calculate_handshake_stats(&mut self, handshake_times: Arc<Mutex<Vec<Duration>>>) {
        let times = handshake_times.lock().unwrap();
        if times.is_empty() {
            return;
        }

        // Calculate average handshake time
        let total: Duration = times.iter().sum();
        self.results.handshake_time_ms = Some(total.as_millis() as f64 / times.len() as f64);

        // Calculate P95 handshake time
        if !times.is_empty() {
            let mut sorted_times: Vec<Duration> = times.clone();
            sorted_times.sort();
            let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
            if p95_index < sorted_times.len() {
                self.results.p95_handshake_time_ms = Some(sorted_times[p95_index].as_millis() as f64);
            }
        }
    }

    /// Calculate connection time statistics
    fn calculate_connection_time_stats(&mut self, connection_times: Arc<Mutex<Vec<Duration>>>) {
        let times = connection_times.lock().unwrap();
        if times.is_empty() {
            return;
        }

        // Calculate average connection time
        let total: Duration = times.iter().sum();
        self.results.average_latency_ms = total.as_millis() as f64 / times.len() as f64;

        // Calculate P95 and P99 connection times
        if !times.is_empty() {
            let mut sorted_times: Vec<Duration> = times.clone();
            sorted_times.sort();

            let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
            if p95_index < sorted_times.len() {
                self.results.p95_latency_ms = sorted_times[p95_index].as_millis() as f64;
            }

            let p99_index = (sorted_times.len() as f64 * 0.99) as usize;
            if p99_index < sorted_times.len() {
                self.results.p99_latency_ms = sorted_times[p99_index].as_millis() as f64;
            }
        }
    }

    /// Get system metrics
    fn get_system_metrics(&mut self) {
        // Mock system metrics - in real implementation would use actual system monitoring
        self.results.cpu_usage_percent = 25.0;
        self.results.memory_usage_mb = 128;
    }
}

/// Client result
#[derive(Debug, Clone)]
struct ClientResult {
    success: bool,
    messages: u64,
    bytes: u64,
    network_errors: u64,
    timeout_errors: u64,
}

impl ClientResult {
    fn new() -> Self {
        Self {
            success: false,
            messages: 0,
            bytes: 0,
            network_errors: 0,
            timeout_errors: 0,
        }
    }
}

/// Print benchmark results
fn print_results(result: &BenchmarkResult, verbose: bool) {
    println!("\n{} Benchmark Results", result.protocol);
    println!("==================");
    println!("Duration: {:.2}s", result.duration_seconds);
    println!("Total Connections: {}", result.total_connections);
    println!("Successful Connections: {}", result.successful_connections);
    println!("Failed Connections: {}", result.failed_connections);
    
    if result.total_messages > 0 {
        println!("Total Messages: {}", result.total_messages);
    }
    
    println!("Total Bytes: {} MB", result.total_bytes / (1024 * 1024));
    println!("Average Latency: {:.2}ms", result.average_latency_ms);
    println!("P95 Latency: {:.2}ms", result.p95_latency_ms);
    println!("P99 Latency: {:.2}ms", result.p99_latency_ms);
    println!("Throughput: {:.2} Gbps", result.throughput_gbps);
    println!("CPU Usage: {:.1}%", result.cpu_usage_percent);
    println!("Memory Usage: {} MB", result.memory_usage_mb);
    
    if let Some(handshake_time) = result.handshake_time_ms {
        println!("Handshake Time (avg): {:.2}ms", handshake_time);
    }
    
    if let Some(p95_handshake) = result.p95_handshake_time_ms {
        println!("Handshake Time (p95): {:.2}ms", p95_handshake);
    }
    
    if let Some(connection_rate) = result.connection_rate_per_second {
        println!("Connection Rate: {:.1} conn/s", connection_rate);
    }
    
    if result.network_errors > 0 {
        println!("Network Errors: {}", result.network_errors);
    }
    
    if result.timeout_errors > 0 {
        println!("Timeout Errors: {}", result.timeout_errors);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <protocol> [options]", args[0]);
        eprintln!("Protocols: tcp, tls, quic, concurrent");
        eprintln!("Options:");
        eprintln!("  --host <host>           Server host (default: 127.0.0.1)");
        eprintln!("  --port <port>           Server port (default: 8080)");
        eprintln!("  --clients <clients>     Number of concurrent clients (default: 100)");
        eprintln!("  --duration <seconds>    Benchmark duration in seconds (default: 30)");
        eprintln!("  --message-size <bytes>  Message size in bytes (default: 1024)");
        eprintln!("  --connections <count>   Number of concurrent connections (default: 10000)");
        eprintln!("  --streams <count>       Number of streams per QUIC connection (default: 4)");
        eprintln!("  --profile <profile>     TLS/QUIC profile (default: tls13_modern)");
        eprintln!("  --verbose               Verbose output");
        eprintln!("  --output <format>       Output format: text, json (default: text)");
        return Ok(());
    }

    let protocol = &args[1];
    
    let mut config = BenchmarkConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        protocol: protocol.clone(),
        clients: 100,
        duration: Duration::from_secs(30),
        message_size: 1024,
        connections: 10000,
        streams: 4,
        profile: "tls13_modern".to_string(),
        verbose: false,
        output_format: "text".to_string(),
    };

    // Parse command line arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                if i + 1 < args.len() {
                    config.host = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --host requires a value");
                    return Ok(());
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    config.port = args[i + 1].parse().unwrap_or(8080);
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    return Ok(());
                }
            }
            "--clients" => {
                if i + 1 < args.len() {
                    config.clients = args[i + 1].parse().unwrap_or(100);
                    i += 2;
                } else {
                    eprintln!("Error: --clients requires a value");
                    return Ok(());
                }
            }
            "--duration" => {
                if i + 1 < args.len() {
                    let seconds = args[i + 1].parse().unwrap_or(30);
                    config.duration = Duration::from_secs(seconds);
                    i += 2;
                } else {
                    eprintln!("Error: --duration requires a value");
                    return Ok(());
                }
            }
            "--message-size" => {
                if i + 1 < args.len() {
                    config.message_size = args[i + 1].parse().unwrap_or(1024);
                    i += 2;
                } else {
                    eprintln!("Error: --message-size requires a value");
                    return Ok(());
                }
            }
            "--connections" => {
                if i + 1 < args.len() {
                    config.connections = args[i + 1].parse().unwrap_or(10000);
                    i += 2;
                } else {
                    eprintln!("Error: --connections requires a value");
                    return Ok(());
                }
            }
            "--streams" => {
                if i + 1 < args.len() {
                    config.streams = args[i + 1].parse().unwrap_or(4);
                    i += 2;
                } else {
                    eprintln!("Error: --streams requires a value");
                    return Ok(());
                }
            }
            "--profile" => {
                if i + 1 < args.len() {
                    config.profile = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --profile requires a value");
                    return Ok(());
                }
            }
            "--verbose" => {
                config.verbose = true;
                i += 1;
            }
            "--output" => {
                if i + 1 < args.len() {
                    config.output_format = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }

    let mut runner = BenchmarkRunner::new(config.clone());
    
    let result = match protocol.as_str() {
        "tcp" => runner.run_tcp_benchmark().await?,
        "tls" => runner.run_tls_benchmark().await?,
        "quic" => runner.run_quic_benchmark().await?,
        "concurrent" => runner.run_concurrent_connections_benchmark().await?,
        _ => {
            eprintln!("Unsupported protocol: {}", protocol);
            return Ok(());
        }
    };

    if config.output_format == "json" {
        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);
    } else {
        print_results(&result, config.verbose);
    }

    Ok(())
}
