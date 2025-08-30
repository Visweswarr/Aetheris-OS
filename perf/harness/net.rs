//! Mesh Network RTT Harness
//! 
//! Simulates mesh network operations and generates Round Trip Time (RTT) metrics
//! for network synchronization, peer discovery, and data propagation.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use anyhow::Result;
use clap::{Arg, Command};
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::runtime;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Network simulation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
struct NetworkConfig {
    /// Number of mesh peers to simulate
    peer_count: u32,
    /// Base RTT in milliseconds
    base_rtt_ms: f64,
    /// RTT variance for simulation
    rtt_variance_ms: f64,
    /// Packet loss probability (0.0-1.0)
    packet_loss_probability: f64,
    /// Network sync interval in seconds
    sync_interval_seconds: u64,
    /// Peer discovery interval in seconds
    discovery_interval_seconds: u64,
    /// Duration to run the harness in seconds
    duration_seconds: u64,
    /// Enable network degradation simulation
    enable_degradation: bool,
    /// Network partitioning simulation
    enable_partitioning: bool,
    /// Partition duration in seconds
    partition_duration_seconds: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            peer_count: 20,
            base_rtt_ms: 25.0,
            rtt_variance_ms: 15.0,
            packet_loss_probability: 0.002, // 0.2%
            sync_interval_seconds: 30,
            discovery_interval_seconds: 60,
            duration_seconds: 300, // 5 minutes
            enable_degradation: false,
            enable_partitioning: false,
            partition_duration_seconds: 45,
        }
    }
}

/// Simulated mesh peer
#[derive(Debug, Clone)]
struct MeshPeer {
    id: Uuid,
    address: IpAddr,
    last_seen: Instant,
    rtt_history: Vec<f64>,
    packet_loss_count: u64,
    is_reachable: bool,
}

impl MeshPeer {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: Uuid::new_v4(),
            address: IpAddr::V4(Ipv4Addr::new(
                192,
                168,
                rng.gen_range(1..=254),
                rng.gen_range(1..=254),
            )),
            last_seen: Instant::now(),
            rtt_history: Vec::new(),
            packet_loss_count: 0,
            is_reachable: true,
        }
    }

    fn update_rtt(&mut self, rtt: f64) {
        self.rtt_history.push(rtt);
        if self.rtt_history.len() > 100 {
            self.rtt_history.remove(0);
        }
        self.last_seen = Instant::now();
    }

    fn average_rtt(&self) -> f64 {
        if self.rtt_history.is_empty() {
            0.0
        } else {
            self.rtt_history.iter().sum::<f64>() / self.rtt_history.len() as f64
        }
    }
}

/// Mesh network harness
struct NetworkHarness {
    config: NetworkConfig,
    meter: Meter,
    peers: HashMap<Uuid, MeshPeer>,
    // Metrics
    mesh_sync_histogram: Histogram<f64>,
    peer_discovery_histogram: Histogram<f64>,
    rtt_histogram: Histogram<f64>,
    packet_loss_counter: Counter<u64>,
    sync_operations_counter: Counter<u64>,
    peer_count_gauge: Counter<u64>,
    // State
    running: Arc<AtomicBool>,
    sync_count: Arc<AtomicU64>,
    discovery_count: Arc<AtomicU64>,
}

impl NetworkHarness {
    /// Create a new network harness
    fn new(config: NetworkConfig) -> Result<Self> {
        // Initialize OpenTelemetry meter
        let meter = global::meter("network_harness");
        
        // Create metrics
        let mesh_sync_histogram = meter
            .f64_histogram("mesh_sync_latency")
            .with_description("Mesh network synchronization latency")
            .with_unit("ms")
            .init();
            
        let peer_discovery_histogram = meter
            .f64_histogram("peer_discovery_latency")
            .with_description("Peer discovery operation latency")
            .with_unit("ms")
            .init();
            
        let rtt_histogram = meter
            .f64_histogram("mesh_rtt")
            .with_description("Round trip time between mesh peers")
            .with_unit("ms")
            .init();
            
        let packet_loss_counter = meter
            .u64_counter("mesh_packets_lost_total")
            .with_description("Total packets lost in mesh network")
            .init();
            
        let sync_operations_counter = meter
            .u64_counter("mesh_sync_operations_total")
            .with_description("Total mesh synchronization operations")
            .init();
            
        let peer_count_gauge = meter
            .u64_counter("mesh_peer_count")
            .with_description("Number of active mesh peers")
            .init();

        // Initialize peers
        let mut peers = HashMap::new();
        for _ in 0..config.peer_count {
            let peer = MeshPeer::new();
            peers.insert(peer.id, peer);
        }

        Ok(Self {
            config,
            meter,
            peers,
            mesh_sync_histogram,
            peer_discovery_histogram,
            rtt_histogram,
            packet_loss_counter,
            sync_operations_counter,
            peer_count_gauge,
            running: Arc::new(AtomicBool::new(false)),
            sync_count: Arc::new(AtomicU64::new(0)),
            discovery_count: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Start the network simulation
    async fn run(&self) -> Result<()> {
        info!("Starting network harness with config: {:?}", self.config);
        
        self.running.store(true, Ordering::Relaxed);
        
        let mut sync_timer = interval(Duration::from_secs(self.config.sync_interval_seconds));
        let mut discovery_timer = interval(Duration::from_secs(self.config.discovery_interval_seconds));
        let mut stats_timer = interval(Duration::from_secs(10));
        let mut peer_communication_timer = interval(Duration::from_secs(1));
        
        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(self.config.duration_seconds);
        
        info!(
            "Network simulation started - Peers: {}, Duration: {}s", 
            self.config.peer_count,
            self.config.duration_seconds
        );

        loop {
            tokio::select! {
                _ = sync_timer.tick() => {
                    if Instant::now() >= end_time {
                        break;
                    }
                    self.simulate_mesh_sync().await;
                }
                _ = discovery_timer.tick() => {
                    if Instant::now() >= end_time {
                        break;
                    }
                    self.simulate_peer_discovery().await;
                }
                _ = peer_communication_timer.tick() => {
                    if Instant::now() >= end_time {
                        break;
                    }
                    self.simulate_peer_communication().await;
                }
                _ = stats_timer.tick() => {
                    self.log_statistics().await;
                }
            }
        }
        
        self.running.store(false, Ordering::Relaxed);
        info!("Network harness simulation completed");
        self.log_final_statistics().await;
        
        Ok(())
    }

    /// Simulate mesh network synchronization
    async fn simulate_mesh_sync(&self) {
        let sync_start = Instant::now();
        
        debug!("Starting mesh synchronization");
        
        // Simulate synchronization with multiple peers
        let sync_peer_count = std::cmp::min(5, self.peers.len()); // Sync with up to 5 peers
        let mut total_sync_time = 0.0;
        
        for _ in 0..sync_peer_count {
            let rtt = self.calculate_network_rtt();
            
            // Synchronization involves multiple round trips
            let sync_rounds = rand::thread_rng().gen_range(2..=4);
            let sync_time = rtt * sync_rounds as f64;
            total_sync_time += sync_time;
            
            // Simulate processing time
            tokio::time::sleep(Duration::from_millis((sync_time / 10.0) as u64)).await;
        }
        
        let sync_end = Instant::now();
        let sync_duration = sync_end.duration_since(sync_start);
        let sync_latency_ms = sync_duration.as_secs_f64() * 1000.0;
        
        // Record metrics
        self.mesh_sync_histogram.record(sync_latency_ms, &[
            opentelemetry::KeyValue::new("sync_type", "full"),
            opentelemetry::KeyValue::new("peer_count", sync_peer_count.to_string()),
        ]);
        
        self.sync_operations_counter.add(1, &[]);
        self.sync_count.fetch_add(1, Ordering::Relaxed);
        
        info!("Mesh sync completed: {:.2}ms ({} peers)", sync_latency_ms, sync_peer_count);
    }

    /// Simulate peer discovery
    async fn simulate_peer_discovery(&self) {
        let discovery_start = Instant::now();
        
        debug!("Starting peer discovery");
        
        // Simulate discovery process
        let discovery_attempts = rand::thread_rng().gen_range(3..=8);
        let mut discovered_peers = 0;
        
        for _ in 0..discovery_attempts {
            let rtt = self.calculate_network_rtt();
            
            // Discovery involves broadcast and response
            let discovery_time = rtt * 1.5; // Slightly longer than normal RTT
            
            // Check if peer responds (based on reachability)
            if rand::thread_rng().gen::<f64>() > self.config.packet_loss_probability {
                discovered_peers += 1;
            }
            
            tokio::time::sleep(Duration::from_millis((discovery_time / 20.0) as u64)).await;
        }
        
        let discovery_end = Instant::now();
        let discovery_duration = discovery_end.duration_since(discovery_start);
        let discovery_latency_ms = discovery_duration.as_secs_f64() * 1000.0;
        
        // Record metrics
        self.peer_discovery_histogram.record(discovery_latency_ms, &[
            opentelemetry::KeyValue::new("discovery_method", "broadcast"),
            opentelemetry::KeyValue::new("peers_found", discovered_peers.to_string()),
        ]);
        
        self.discovery_count.fetch_add(1, Ordering::Relaxed);
        
        debug!("Peer discovery completed: {:.2}ms ({} peers found)", discovery_latency_ms, discovered_peers);
    }

    /// Simulate regular peer communication
    async fn simulate_peer_communication(&self) {
        // Simulate communication with random peers
        let communication_count = rand::thread_rng().gen_range(2..=5);
        
        for _ in 0..communication_count {
            let rtt = self.calculate_network_rtt();
            
            // Check for packet loss
            if rand::thread_rng().gen::<f64>() < self.config.packet_loss_probability {
                self.packet_loss_counter.add(1, &[]);
                continue;
            }
            
            // Record RTT
            self.rtt_histogram.record(rtt, &[
                opentelemetry::KeyValue::new("message_type", "data"),
                opentelemetry::KeyValue::new("protocol", "quic"),
            ]);
        }
        
        // Update peer count
        let reachable_peers = self.peers.values().filter(|p| p.is_reachable).count() as u64;
        self.peer_count_gauge.add(reachable_peers, &[]);
    }

    /// Calculate network RTT with variance and conditions
    fn calculate_network_rtt(&self) -> f64 {
        let mut rng = rand::thread_rng();
        
        // Base RTT with random variance
        let base = self.config.base_rtt_ms + 
                   rng.gen_range(-self.config.rtt_variance_ms..self.config.rtt_variance_ms);
        
        // Apply degradation if enabled
        if self.config.enable_degradation {
            let degradation_factor = rng.gen_range(1.0..3.0);
            base * degradation_factor
        } else {
            base.max(1.0) // Ensure positive RTT
        }
    }

    /// Log current statistics
    async fn log_statistics(&self) {
        let syncs = self.sync_count.load(Ordering::Relaxed);
        let discoveries = self.discovery_count.load(Ordering::Relaxed);
        let reachable_peers = self.peers.values().filter(|p| p.is_reachable).count();
        
        info!(
            "Network Stats - Syncs: {}, Discoveries: {}, Reachable peers: {}/{}",
            syncs,
            discoveries,
            reachable_peers,
            self.config.peer_count
        );
    }

    /// Log final statistics
    async fn log_final_statistics(&self) {
        let total_syncs = self.sync_count.load(Ordering::Relaxed);
        let total_discoveries = self.discovery_count.load(Ordering::Relaxed);
        let reachable_peers = self.peers.values().filter(|p| p.is_reachable).count();
        
        info!("=== Network Harness Final Statistics ===");
        info!("Total sync operations: {}", total_syncs);
        info!("Total discovery operations: {}", total_discoveries);
        info!("Final reachable peers: {}/{}", reachable_peers, self.config.peer_count);
        info!("Average sync interval: {}s", self.config.sync_interval_seconds);
        info!("Average discovery interval: {}s", self.config.discovery_interval_seconds);
    }
}

/// Initialize OpenTelemetry with OTLP exporter
fn init_telemetry() -> Result<()> {
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/metrics")
        )
        .build()?;
    
    global::set_meter_provider(meter_provider);
    
    tracing_subscriber::fmt()
        .with_env_filter("network_harness=info")
        .init();
    
    Ok(())
}

/// Load configuration from file or use defaults
fn load_config(config_path: Option<&str>) -> Result<NetworkConfig> {
    match config_path {
        Some(path) => {
            let content = std::fs::read_to_string(path)?;
            let config: NetworkConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        }
        None => Ok(NetworkConfig::default())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("network_harness")
        .about("Network RTT Harness - Simulates mesh network operations and RTT")
        .version("1.0.0")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Configuration file path (YAML)")
        )
        .arg(
            Arg::new("peers")
                .long("peers")
                .short('p')
                .value_name("COUNT")
                .help("Number of mesh peers")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .short('d')
                .value_name("SECONDS")
                .help("Duration to run harness in seconds")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("base-rtt")
                .long("base-rtt")
                .value_name("MS")
                .help("Base RTT in milliseconds")
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new("variance")
                .long("variance")
                .value_name("MS")
                .help("RTT variance in milliseconds")
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new("degradation")
                .long("degradation")
                .help("Enable network degradation simulation")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("partitioning")
                .long("partitioning")
                .help("Enable network partitioning simulation")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Initialize telemetry
    init_telemetry()?;

    // Load configuration
    let mut config = load_config(matches.get_one::<String>("config").map(|s| s.as_str()))?;

    // Override config with command line arguments
    if let Some(peers) = matches.get_one::<u32>("peers") {
        config.peer_count = *peers;
    }
    if let Some(duration) = matches.get_one::<u64>("duration") {
        config.duration_seconds = *duration;
    }
    if let Some(rtt) = matches.get_one::<f64>("base-rtt") {
        config.base_rtt_ms = *rtt;
    }
    if let Some(variance) = matches.get_one::<f64>("variance") {
        config.rtt_variance_ms = *variance;
    }
    if matches.get_flag("degradation") {
        config.enable_degradation = true;
    }
    if matches.get_flag("partitioning") {
        config.enable_partitioning = true;
    }

    // Create and run harness
    let harness = NetworkHarness::new(config)?;
    
    // Set up signal handling
    let running = harness.running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        info!("Received shutdown signal");
        running.store(false, Ordering::Relaxed);
    });

    // Run the simulation
    harness.run().await?;

    // Give time for final metrics export
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    info!("Network harness completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.peer_count, 20);
        assert!(config.base_rtt_ms > 0.0);
        assert!(config.rtt_variance_ms >= 0.0);
    }

    #[test]
    fn test_peer_creation() {
        let peer = MeshPeer::new();
        assert!(peer.is_reachable);
        assert_eq!(peer.rtt_history.len(), 0);
        assert_eq!(peer.packet_loss_count, 0);
    }

    #[test]
    fn test_rtt_calculation() {
        let config = NetworkConfig {
            base_rtt_ms: 50.0,
            rtt_variance_ms: 10.0,
            enable_degradation: false,
            ..Default::default()
        };
        
        let harness = NetworkHarness::new(config).unwrap();
        let rtt = harness.calculate_network_rtt();
        
        // Should be within reasonable range
        assert!(rtt >= 30.0 && rtt <= 70.0);
    }
}
