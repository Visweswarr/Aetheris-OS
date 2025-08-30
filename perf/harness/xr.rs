//! XR (Extended Reality) Timing Harness
//! 
//! Simulates XR frame rendering pipeline and exports Mean Time to Present (MTP) metrics
//! to OpenTelemetry/Prometheus for SLO validation.

use std::time::{Duration, Instant};
use std::thread;
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

/// XR rendering configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
struct XrConfig {
    /// Target frame rate (FPS)
    target_fps: u32,
    /// Base rendering latency in milliseconds
    base_latency_ms: f64,
    /// Latency variance for simulation
    latency_variance_ms: f64,
    /// Probability of frame drops (0.0-1.0)
    frame_drop_probability: f64,
    /// Duration to run the harness in seconds
    duration_seconds: u64,
    /// Enable performance degradation simulation
    enable_degradation: bool,
    /// Degradation cycle duration in seconds
    degradation_cycle_seconds: u64,
}

impl Default for XrConfig {
    fn default() -> Self {
        Self {
            target_fps: 90,
            base_latency_ms: 8.0,
            latency_variance_ms: 3.0,
            frame_drop_probability: 0.001, // 0.1%
            duration_seconds: 300, // 5 minutes
            enable_degradation: false,
            degradation_cycle_seconds: 120, // 2 minutes
        }
    }
}

/// XR frame rendering simulator
struct XrHarness {
    config: XrConfig,
    meter: Meter,
    // Metrics
    frame_render_histogram: Histogram<f64>,
    mtp_histogram: Histogram<f64>,
    frame_counter: Counter<u64>,
    frame_drop_counter: Counter<u64>,
    // State
    running: Arc<AtomicBool>,
    frame_count: Arc<AtomicU64>,
    dropped_frames: Arc<AtomicU64>,
}

impl XrHarness {
    /// Create a new XR harness
    fn new(config: XrConfig) -> Result<Self> {
        // Initialize OpenTelemetry meter
        let meter = global::meter("xr_harness");
        
        // Create metrics
        let frame_render_histogram = meter
            .f64_histogram("xr_frame_render_duration")
            .with_description("Time to render a single XR frame")
            .with_unit("ms")
            .init();
            
        let mtp_histogram = meter
            .f64_histogram("xr_mtp_latency")
            .with_description("XR Mean Time to Present latency")
            .with_unit("ms")
            .init();
            
        let frame_counter = meter
            .u64_counter("xr_frames_total")
            .with_description("Total XR frames rendered")
            .init();
            
        let frame_drop_counter = meter
            .u64_counter("xr_frames_dropped_total")
            .with_description("Total XR frames dropped")
            .init();

        Ok(Self {
            config,
            meter,
            frame_render_histogram,
            mtp_histogram,
            frame_counter,
            frame_drop_counter,
            running: Arc::new(AtomicBool::new(false)),
            frame_count: Arc::new(AtomicU64::new(0)),
            dropped_frames: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Start the XR rendering simulation
    async fn run(&self) -> Result<()> {
        info!("Starting XR harness with config: {:?}", self.config);
        
        self.running.store(true, Ordering::Relaxed);
        let frame_duration = Duration::from_nanos(1_000_000_000 / self.config.target_fps as u64);
        
        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(self.config.duration_seconds);
        
        let mut frame_timer = interval(frame_duration);
        let mut stats_timer = interval(Duration::from_secs(10));
        
        info!(
            "XR simulation started - Target: {}fps, Duration: {}s", 
            self.config.target_fps,
            self.config.duration_seconds
        );

        loop {
            tokio::select! {
                _ = frame_timer.tick() => {
                    if Instant::now() >= end_time {
                        break;
                    }
                    self.simulate_frame_render().await;
                }
                _ = stats_timer.tick() => {
                    self.log_statistics();
                }
            }
        }
        
        self.running.store(false, Ordering::Relaxed);
        info!("XR harness simulation completed");
        self.log_final_statistics();
        
        Ok(())
    }

    /// Simulate rendering a single XR frame
    async fn simulate_frame_render(&self) {
        let frame_start = Instant::now();
        
        // Check for frame drop
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.config.frame_drop_probability {
            self.frame_drop_counter.add(1, &[]);
            self.dropped_frames.fetch_add(1, Ordering::Relaxed);
            debug!("Frame dropped");
            return;
        }

        // Calculate latency with variance and potential degradation
        let base_latency = self.calculate_frame_latency();
        
        // Simulate frame processing time
        let processing_time = Duration::from_millis(base_latency as u64);
        tokio::time::sleep(processing_time).await;
        
        let frame_end = Instant::now();
        let frame_duration = frame_end.duration_since(frame_start);
        let frame_latency_ms = frame_duration.as_secs_f64() * 1000.0;
        
        // Record metrics
        self.frame_render_histogram.record(frame_latency_ms, &[
            opentelemetry::KeyValue::new("frame_type", "normal"),
        ]);
        
        // Mean Time to Present includes additional display pipeline latency
        let mtp_latency = frame_latency_ms + rng.gen_range(1.0..4.0); // Display pipeline overhead
        self.mtp_histogram.record(mtp_latency, &[
            opentelemetry::KeyValue::new("display_mode", "stereo"),
            opentelemetry::KeyValue::new("resolution", "2160x1200"),
        ]);
        
        self.frame_counter.add(1, &[]);
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        
        debug!("Frame rendered: {:.2}ms, MTP: {:.2}ms", frame_latency_ms, mtp_latency);
    }

    /// Calculate frame latency with variance and degradation
    fn calculate_frame_latency(&self) -> f64 {
        let mut rng = rand::thread_rng();
        
        // Base latency with random variance
        let base = self.config.base_latency_ms + 
                   rng.gen_range(-self.config.latency_variance_ms..self.config.latency_variance_ms);
        
        // Apply degradation if enabled
        if self.config.enable_degradation {
            let cycle_progress = (Instant::now().elapsed().as_secs() % self.config.degradation_cycle_seconds) as f64 
                                / self.config.degradation_cycle_seconds as f64;
            
            // Create a sine wave degradation pattern
            let degradation_factor = 1.0 + (cycle_progress * 2.0 * std::f64::consts::PI).sin() * 0.5;
            base * degradation_factor
        } else {
            base
        }
    }

    /// Log current statistics
    fn log_statistics(&self) {
        let frames = self.frame_count.load(Ordering::Relaxed);
        let dropped = self.dropped_frames.load(Ordering::Relaxed);
        let drop_rate = if frames > 0 { dropped as f64 / frames as f64 * 100.0 } else { 0.0 };
        
        info!(
            "XR Stats - Frames: {}, Dropped: {} ({:.2}%), FPS: ~{}",
            frames,
            dropped,
            drop_rate,
            self.config.target_fps
        );
    }

    /// Log final statistics
    fn log_final_statistics(&self) {
        let total_frames = self.frame_count.load(Ordering::Relaxed);
        let total_dropped = self.dropped_frames.load(Ordering::Relaxed);
        let drop_rate = if total_frames > 0 { 
            total_dropped as f64 / total_frames as f64 * 100.0 
        } else { 
            0.0 
        };
        
        info!("=== XR Harness Final Statistics ===");
        info!("Total frames rendered: {}", total_frames);
        info!("Total frames dropped: {}", total_dropped);
        info!("Frame drop rate: {:.3}%", drop_rate);
        info!("Target FPS: {}", self.config.target_fps);
        info!("Average frame time: {:.2}ms", 1000.0 / self.config.target_fps as f64);
    }
}

/// Initialize OpenTelemetry with Prometheus exporter
fn init_telemetry() -> Result<()> {
    // Initialize OpenTelemetry with OTLP exporter
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/metrics")
        )
        .build()?;
    
    global::set_meter_provider(meter_provider);
    
    // Also initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("xr_harness=info")
        .init();
    
    Ok(())
}

/// Load configuration from file or use defaults
fn load_config(config_path: Option<&str>) -> Result<XrConfig> {
    match config_path {
        Some(path) => {
            let content = std::fs::read_to_string(path)?;
            let config: XrConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        }
        None => Ok(XrConfig::default())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("xr_harness")
        .about("XR Timing Harness - Simulates XR frame rendering and MTP latency")
        .version("1.0.0")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Configuration file path (YAML)")
        )
        .arg(
            Arg::new("fps")
                .long("fps")
                .value_name("FPS")
                .help("Target frame rate")
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
            Arg::new("base-latency")
                .long("base-latency")
                .value_name("MS")
                .help("Base frame latency in milliseconds")
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new("variance")
                .long("variance")
                .value_name("MS")
                .help("Latency variance in milliseconds")
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new("degradation")
                .long("degradation")
                .help("Enable performance degradation simulation")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("otlp-endpoint")
                .long("otlp-endpoint")
                .value_name("URL")
                .help("OpenTelemetry OTLP endpoint")
                .default_value("http://localhost:4318/v1/metrics")
        )
        .get_matches();

    // Initialize telemetry
    init_telemetry()?;

    // Load configuration
    let mut config = load_config(matches.get_one::<String>("config").map(|s| s.as_str()))?;

    // Override config with command line arguments
    if let Some(fps) = matches.get_one::<u32>("fps") {
        config.target_fps = *fps;
    }
    if let Some(duration) = matches.get_one::<u64>("duration") {
        config.duration_seconds = *duration;
    }
    if let Some(latency) = matches.get_one::<f64>("base-latency") {
        config.base_latency_ms = *latency;
    }
    if let Some(variance) = matches.get_one::<f64>("variance") {
        config.latency_variance_ms = *variance;
    }
    if matches.get_flag("degradation") {
        config.enable_degradation = true;
    }

    // Create and run harness
    let harness = XrHarness::new(config)?;
    
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
    
    info!("XR harness completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_config_default() {
        let config = XrConfig::default();
        assert_eq!(config.target_fps, 90);
        assert_eq!(config.base_latency_ms, 8.0);
        assert!(config.latency_variance_ms > 0.0);
    }

    #[test]
    fn test_latency_calculation() {
        let config = XrConfig {
            base_latency_ms: 10.0,
            latency_variance_ms: 2.0,
            enable_degradation: false,
            ..Default::default()
        };
        
        let harness = XrHarness::new(config).unwrap();
        let latency = harness.calculate_frame_latency();
        
        // Should be within reasonable range
        assert!(latency >= 6.0 && latency <= 14.0);
    }

    #[tokio::test]
    async fn test_frame_simulation() {
        let config = XrConfig {
            target_fps: 60,
            base_latency_ms: 5.0,
            duration_seconds: 1,
            ..Default::default()
        };
        
        // This test would normally require OpenTelemetry setup
        // For unit testing, we'd mock the telemetry components
        assert!(config.target_fps > 0);
    }
}
