//! Chaos Engineering for Aetheris OS Networking
//! 
//! This module provides chaos injection capabilities for testing network
//! resilience, including packet loss, latency/jitter injection, and connection churn.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

/// Chaos configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    pub enabled: bool,
    pub seed: Option<u64>,
    pub duration: Duration,
    pub packet_loss_rate: f64, // 0.0 to 1.0
    pub latency_ms: f64,
    pub latency_jitter_ms: f64,
    pub connection_churn_rate: f64, // connections per second
    pub connection_churn_duration: Duration,
    pub bandwidth_limit_mbps: Option<f64>,
    pub cpu_stress_percent: Option<f64>,
    pub memory_stress_mb: Option<f64>,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            seed: Some(42),
            duration: Duration::from_secs(60),
            packet_loss_rate: 0.0,
            latency_ms: 0.0,
            latency_jitter_ms: 0.0,
            connection_churn_rate: 0.0,
            connection_churn_duration: Duration::from_secs(1),
            bandwidth_limit_mbps: None,
            cpu_stress_percent: None,
            memory_stress_mb: None,
        }
    }
}

/// Chaos injection types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosType {
    PacketLoss,
    Latency,
    Jitter,
    ConnectionChurn,
    BandwidthLimit,
    CpuStress,
    MemoryStress,
}

/// Chaos injection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosResult {
    pub chaos_type: ChaosType,
    pub applied: bool,
    pub duration: Duration,
    pub packets_dropped: u64,
    pub packets_delayed: u64,
    pub connections_churned: u64,
    pub error_count: u64,
}

/// Chaos manager for coordinating chaos injections
pub struct ChaosManager {
    config: ChaosConfig,
    active_injections: Arc<RwLock<HashMap<String, ChaosInjection>>>,
    stats: Arc<ChaosStats>,
    rng: Arc<RwLock<ChaosRng>>,
}

/// Chaos injection instance
pub struct ChaosInjection {
    pub id: String,
    pub chaos_type: ChaosType,
    pub config: ChaosConfig,
    pub start_time: Instant,
    pub stats: Arc<ChaosStats>,
    pub rng: Arc<RwLock<ChaosRng>>,
}

/// Chaos statistics
#[derive(Debug, Default)]
pub struct ChaosStats {
    pub packets_dropped: AtomicU64,
    pub packets_delayed: AtomicU64,
    pub connections_churned: AtomicU64,
    pub error_count: AtomicU64,
    pub total_duration: AtomicU64,
}

/// Simple chaos RNG for deterministic results
pub struct ChaosRng {
    state: u64,
}

impl ChaosRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_f64(&mut self) -> f64 {
        // Linear congruential generator
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state >> 16) as f64 / 65536.0
    }

    pub fn next_bool(&mut self, probability: f64) -> bool {
        self.next_f64() < probability
    }
}

impl ChaosManager {
    /// Create a new chaos manager
    pub fn new(config: ChaosConfig) -> Self {
        let rng = Arc::new(RwLock::new(ChaosRng::new(config.seed.unwrap_or(42))));
        
        Self {
            config,
            active_injections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(ChaosStats::default()),
            rng,
        }
    }

    /// Start packet loss injection
    pub async fn start_packet_loss(&self, rate: f64) -> Result<String, Box<dyn std::error::Error>> {
        let injection_id = format!("packet_loss_{}", uuid::Uuid::new_v4());
        
        let injection = ChaosInjection {
            id: injection_id.clone(),
            chaos_type: ChaosType::PacketLoss,
            config: ChaosConfig {
                packet_loss_rate: rate,
                ..self.config.clone()
            },
            start_time: Instant::now(),
            stats: self.stats.clone(),
            rng: self.rng.clone(),
        };
        
        self.active_injections.write().await.insert(injection_id.clone(), injection);
        
        // Start packet loss injection task
        let stats = self.stats.clone();
        let rng = self.rng.clone();
        let duration = self.config.duration;
        
        tokio::spawn(async move {
            let start = Instant::now();
            while start.elapsed() < duration {
                // Simulate packet loss
                let should_drop = {
                    let mut rng = rng.write().await;
                    rng.next_bool(rate)
                };
                
                if should_drop {
                    stats.packets_dropped.fetch_add(1, Ordering::Relaxed);
                }
                
                sleep(Duration::from_millis(1)).await;
            }
        });
        
        Ok(injection_id)
    }

    /// Start latency injection
    pub async fn start_latency(&self, latency_ms: f64, jitter_ms: f64) -> Result<String, Box<dyn std::error::Error>> {
        let injection_id = format!("latency_{}", uuid::Uuid::new_v4());
        
        let injection = ChaosInjection {
            id: injection_id.clone(),
            chaos_type: ChaosType::Latency,
            config: ChaosConfig {
                latency_ms,
                latency_jitter_ms: jitter_ms,
                ..self.config.clone()
            },
            start_time: Instant::now(),
            stats: self.stats.clone(),
            rng: self.rng.clone(),
        };
        
        self.active_injections.write().await.insert(injection_id.clone(), injection);
        
        // Start latency injection task
        let stats = self.stats.clone();
        let rng = self.rng.clone();
        let duration = self.config.duration;
        
        tokio::spawn(async move {
            let start = Instant::now();
            while start.elapsed() < duration {
                // Simulate latency
                let jitter = {
                    let mut rng = rng.write().await;
                    (rng.next_f64() - 0.5) * 2.0 * jitter_ms
                };
                
                let total_latency = latency_ms + jitter;
                if total_latency > 0.0 {
                    sleep(Duration::from_millis(total_latency as u64)).await;
                    stats.packets_delayed.fetch_add(1, Ordering::Relaxed);
                }
                
                sleep(Duration::from_millis(1)).await;
            }
        });
        
        Ok(injection_id)
    }

    /// Start connection churn injection
    pub async fn start_connection_churn(&self, rate: f64) -> Result<String, Box<dyn std::error::Error>> {
        let injection_id = format!("connection_churn_{}", uuid::Uuid::new_v4());
        
        let injection = ChaosInjection {
            id: injection_id.clone(),
            chaos_type: ChaosType::ConnectionChurn,
            config: ChaosConfig {
                connection_churn_rate: rate,
                ..self.config.clone()
            },
            start_time: Instant::now(),
            stats: self.stats.clone(),
            rng: self.rng.clone(),
        };
        
        self.active_injections.write().await.insert(injection_id.clone(), injection);
        
        // Start connection churn injection task
        let stats = self.stats.clone();
        let rng = self.rng.clone();
        let duration = self.config.duration;
        let churn_duration = self.config.connection_churn_duration;
        
        tokio::spawn(async move {
            let start = Instant::now();
            while start.elapsed() < duration {
                // Simulate connection churn
                let should_churn = {
                    let mut rng = rng.write().await;
                    rng.next_bool(rate / 1000.0) // Convert to per-millisecond probability
                };
                
                if should_churn {
                    stats.connections_churned.fetch_add(1, Ordering::Relaxed);
                    
                    // Simulate connection churn duration
                    sleep(churn_duration).await;
                }
                
                sleep(Duration::from_millis(1)).await;
            }
        });
        
        Ok(injection_id)
    }

    /// Start bandwidth limiting
    pub async fn start_bandwidth_limit(&self, limit_mbps: f64) -> Result<String, Box<dyn std::error::Error>> {
        let injection_id = format!("bandwidth_limit_{}", uuid::Uuid::new_v4());
        
        let injection = ChaosInjection {
            id: injection_id.clone(),
            chaos_type: ChaosType::BandwidthLimit,
            config: ChaosConfig {
                bandwidth_limit_mbps: Some(limit_mbps),
                ..self.config.clone()
            },
            start_time: Instant::now(),
            stats: self.stats.clone(),
            rng: self.rng.clone(),
        };
        
        self.active_injections.write().await.insert(injection_id.clone(), injection);
        
        // Start bandwidth limiting task
        let duration = self.config.duration;
        
        tokio::spawn(async move {
            let start = Instant::now();
            while start.elapsed() < duration {
                // Simulate bandwidth limiting
                let bytes_per_second = (limit_mbps * 1024.0 * 1024.0) / 8.0;
                let sleep_duration = Duration::from_secs_f64(1.0 / (bytes_per_second / 1024.0));
                sleep(sleep_duration).await;
            }
        });
        
        Ok(injection_id)
    }

    /// Start CPU stress
    pub async fn start_cpu_stress(&self, stress_percent: f64) -> Result<String, Box<dyn std::error::Error>> {
        let injection_id = format!("cpu_stress_{}", uuid::Uuid::new_v4());
        
        let injection = ChaosInjection {
            id: injection_id.clone(),
            chaos_type: ChaosType::CpuStress,
            config: ChaosConfig {
                cpu_stress_percent: Some(stress_percent),
                ..self.config.clone()
            },
            start_time: Instant::now(),
            stats: self.stats.clone(),
            rng: self.rng.clone(),
        };
        
        self.active_injections.write().await.insert(injection_id.clone(), injection);
        
        // Start CPU stress task
        let duration = self.config.duration;
        
        tokio::spawn(async move {
            let start = Instant::now();
            while start.elapsed() < duration {
                // Simulate CPU stress
                let work_duration = Duration::from_millis((stress_percent * 10.0) as u64);
                let sleep_duration = Duration::from_millis(((100.0 - stress_percent) * 10.0) as u64);
                
                // Do some work
                let work_start = Instant::now();
                while work_start.elapsed() < work_duration {
                    // Busy wait
                }
                
                sleep(sleep_duration).await;
            }
        });
        
        Ok(injection_id)
    }

    /// Start memory stress
    pub async fn start_memory_stress(&self, stress_mb: f64) -> Result<String, Box<dyn std::error::Error>> {
        let injection_id = format!("memory_stress_{}", uuid::Uuid::new_v4());
        
        let injection = ChaosInjection {
            id: injection_id.clone(),
            chaos_type: ChaosType::MemoryStress,
            config: ChaosConfig {
                memory_stress_mb: Some(stress_mb),
                ..self.config.clone()
            },
            start_time: Instant::now(),
            stats: self.stats.clone(),
            rng: self.rng.clone(),
        };
        
        self.active_injections.write().await.insert(injection_id.clone(), injection);
        
        // Start memory stress task
        let duration = self.config.duration;
        
        tokio::spawn(async move {
            let start = Instant::now();
            let mut memory_chunks = Vec::new();
            
            // Allocate memory chunks
            let chunk_size = 1024 * 1024; // 1MB chunks
            let num_chunks = (stress_mb as usize).min(1000); // Limit to 1GB max
            
            for _ in 0..num_chunks {
                memory_chunks.push(vec![0u8; chunk_size]);
            }
            
            while start.elapsed() < duration {
                // Touch memory to keep it in RAM
                for chunk in &mut memory_chunks {
                    chunk[0] = chunk[0].wrapping_add(1);
                }
                
                sleep(Duration::from_millis(100)).await;
            }
            
            // Clean up
            drop(memory_chunks);
        });
        
        Ok(injection_id)
    }

    /// Stop a chaos injection
    pub async fn stop_injection(&self, injection_id: &str) -> Result<ChaosResult, Box<dyn std::error::Error>> {
        let injection = self.active_injections.write().await.remove(injection_id)
            .ok_or("Injection not found")?;
        
        let duration = injection.start_time.elapsed();
        let packets_dropped = self.stats.packets_dropped.load(Ordering::Relaxed);
        let packets_delayed = self.stats.packets_delayed.load(Ordering::Relaxed);
        let connections_churned = self.stats.connections_churned.load(Ordering::Relaxed);
        let error_count = self.stats.error_count.load(Ordering::Relaxed);
        
        Ok(ChaosResult {
            chaos_type: injection.chaos_type,
            applied: true,
            duration,
            packets_dropped,
            packets_delayed,
            connections_churned,
            error_count,
        })
    }

    /// Stop all chaos injections
    pub async fn stop_all_injections(&self) -> Vec<ChaosResult> {
        let mut results = Vec::new();
        let injection_ids: Vec<String> = self.active_injections.read().await.keys().cloned().collect();
        
        for injection_id in injection_ids {
            if let Ok(result) = self.stop_injection(&injection_id).await {
                results.push(result);
            }
        }
        
        results
    }

    /// Get chaos statistics
    pub fn get_stats(&self) -> ChaosStatsSnapshot {
        ChaosStatsSnapshot {
            packets_dropped: self.stats.packets_dropped.load(Ordering::Relaxed),
            packets_delayed: self.stats.packets_delayed.load(Ordering::Relaxed),
            connections_churned: self.stats.connections_churned.load(Ordering::Relaxed),
            error_count: self.stats.error_count.load(Ordering::Relaxed),
            total_duration: self.stats.total_duration.load(Ordering::Relaxed),
        }
    }

    /// Check if a packet should be dropped
    pub async fn should_drop_packet(&self) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        let mut rng = self.rng.write().await;
        rng.next_bool(self.config.packet_loss_rate)
    }

    /// Get latency to inject
    pub async fn get_latency_to_inject(&self) -> Duration {
        if !self.config.enabled || self.config.latency_ms <= 0.0 {
            return Duration::from_secs(0);
        }
        
        let mut rng = self.rng.write().await;
        let jitter = (rng.next_f64() - 0.5) * 2.0 * self.config.latency_jitter_ms;
        let total_latency = self.config.latency_ms + jitter;
        
        Duration::from_millis(total_latency.max(0.0) as u64)
    }

    /// Check if a connection should be churned
    pub async fn should_churn_connection(&self) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        let mut rng = self.rng.write().await;
        rng.next_bool(self.config.connection_churn_rate / 1000.0)
    }
}

/// Chaos statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosStatsSnapshot {
    pub packets_dropped: u64,
    pub packets_delayed: u64,
    pub connections_churned: u64,
    pub error_count: u64,
    pub total_duration: u64,
}

/// Chaos injection wrapper for network operations
pub struct ChaosWrapper {
    manager: Arc<ChaosManager>,
}

impl ChaosWrapper {
    /// Create a new chaos wrapper
    pub fn new(manager: Arc<ChaosManager>) -> Self {
        Self { manager }
    }

    /// Wrap a network operation with chaos injection
    pub async fn wrap_operation<F, T>(&self, operation: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        // Check if packet should be dropped
        if self.manager.should_drop_packet().await {
            self.manager.stats.packets_dropped.fetch_add(1, Ordering::Relaxed);
            return Err("Packet dropped by chaos injection".into());
        }
        
        // Inject latency
        let latency = self.manager.get_latency_to_inject().await;
        if !latency.is_zero() {
            sleep(latency).await;
            self.manager.stats.packets_delayed.fetch_add(1, Ordering::Relaxed);
        }
        
        // Execute the operation
        match operation.await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.manager.stats.error_count.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }
}

/// Global chaos manager instance
static mut GLOBAL_CHAOS_MANAGER: Option<Arc<ChaosManager>> = None;

/// Initialize the global chaos manager
pub fn init_global_chaos_manager(config: ChaosConfig) -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(ChaosManager::new(config));
    unsafe {
        GLOBAL_CHAOS_MANAGER = Some(manager);
    }
    Ok(())
}

/// Get the global chaos manager
pub fn get_global_chaos_manager() -> Option<Arc<ChaosManager>> {
    unsafe { GLOBAL_CHAOS_MANAGER.clone() }
}

/// Check if a packet should be dropped with the global chaos manager
pub async fn should_drop_packet() -> bool {
    if let Some(manager) = get_global_chaos_manager() {
        manager.should_drop_packet().await
    } else {
        false
    }
}

/// Get latency to inject with the global chaos manager
pub async fn get_latency_to_inject() -> Duration {
    if let Some(manager) = get_global_chaos_manager() {
        manager.get_latency_to_inject().await
    } else {
        Duration::from_secs(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_manager() {
        let config = ChaosConfig {
            enabled: true,
            seed: Some(42),
            duration: Duration::from_secs(1),
            packet_loss_rate: 0.1,
            latency_ms: 10.0,
            latency_jitter_ms: 5.0,
            connection_churn_rate: 1.0,
            connection_churn_duration: Duration::from_millis(100),
            ..Default::default()
        };
        
        let manager = ChaosManager::new(config);
        
        // Start packet loss injection
        let injection_id = manager.start_packet_loss(0.1).await.unwrap();
        
        // Wait a bit
        sleep(Duration::from_millis(100)).await;
        
        // Stop injection
        let result = manager.stop_injection(&injection_id).await.unwrap();
        assert!(result.applied);
        assert_eq!(result.chaos_type, ChaosType::PacketLoss);
    }

    #[tokio::test]
    async fn test_chaos_rng() {
        let mut rng = ChaosRng::new(42);
        
        // Test deterministic behavior
        let val1 = rng.next_f64();
        let val2 = rng.next_f64();
        
        assert_ne!(val1, val2);
        assert!(val1 >= 0.0 && val1 <= 1.0);
        assert!(val2 >= 0.0 && val2 <= 1.0);
    }

    #[tokio::test]
    async fn test_chaos_wrapper() {
        let config = ChaosConfig {
            enabled: true,
            packet_loss_rate: 0.0, // No packet loss for this test
            latency_ms: 1.0,
            ..Default::default()
        };
        
        let manager = Arc::new(ChaosManager::new(config));
        let wrapper = ChaosWrapper::new(manager);
        
        // Test successful operation
        let result = wrapper.wrap_operation(async {
            Ok::<i32, Box<dyn std::error::Error>>(42)
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
