use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tonic::{transport::Server, Request, Response, Status};
use tonic_health::{health_server::{Health, HealthServer}, HealthCheckResponse, HealthCheckResponse_ServingStatus};
use tonic_reflection::server::Builder as ReflectionBuilder;
use uuid::Uuid;

use health::health_service_server::{HealthService, HealthServiceServer};
use health::health_monitoring_service_server::{HealthMonitoringService, HealthMonitoringServiceServer};
use health::{
    HealthCheckRequest, HealthCheckResponse, HealthStatus, Severity, HealthCheckDetail,
    ServiceMetadata, Dependency, DependencyDAG, DependencyHealthSummary,
    HealthAggregationRequest, HealthAggregationResponse, SystemHealthSummary,
    HealthCheckStats, HealthAlert, AlertAcknowledgment, HealthMetrics,
    MonitoringConfig, RetryConfig, CircuitBreakerConfig
};

mod health; // Generated from proto

/// Health service configuration
#[derive(Debug, Clone)]
pub struct HealthServiceConfig {
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Service instance ID
    pub instance_id: String,
    /// Service environment
    pub environment: String,
    /// Service region
    pub region: String,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Dependency check interval
    pub dependency_check_interval: Duration,
    /// Alert thresholds
    pub alert_thresholds: HashMap<String, f64>,
    /// Health score weights
    pub health_score_weights: HashMap<String, f64>,
}

impl Default for HealthServiceConfig {
    fn default() -> Self {
        let mut alert_thresholds = HashMap::new();
        alert_thresholds.insert("health_score".to_string(), 70.0);
        alert_thresholds.insert("response_time_ms".to_string(), 1000.0);
        alert_thresholds.insert("error_rate_eps".to_string(), 0.1);

        let mut health_score_weights = HashMap::new();
        health_score_weights.insert("self_health".to_string(), 0.4);
        health_score_weights.insert("dependency_health".to_string(), 0.4);
        health_score_weights.insert("performance_metrics".to_string(), 0.2);

        Self {
            service_name: "health-service".to_string(),
            service_version: "0.1.0".to_string(),
            instance_id: Uuid::new_v4().to_string(),
            environment: "development".to_string(),
            region: "local".to_string(),
            health_check_interval: Duration::from_secs(30),
            dependency_check_interval: Duration::from_secs(60),
            alert_thresholds,
            health_score_weights,
        }
    }
}

/// Health service implementation
#[derive(Debug)]
pub struct HealthServiceImpl {
    config: HealthServiceConfig,
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
    dependencies: Arc<RwLock<HashMap<String, Vec<Dependency>>>>,
    health_scores: Arc<RwLock<HashMap<String, f64>>>,
    health_history: Arc<RwLock<HashMap<String, VecDeque<HealthCheckResponse>>>>,
    alerts: Arc<RwLock<Vec<HealthAlert>>>,
    stats: Arc<RwLock<HealthCheckStats>>,
    alert_sender: mpsc::Sender<HealthAlert>,
}

impl HealthServiceImpl {
    /// Create a new health service
    pub fn new(config: HealthServiceConfig) -> (Self, mpsc::Receiver<HealthAlert>) {
        let (alert_sender, alert_receiver) = mpsc::channel(100);
        
        let service = Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            health_scores: Arc::new(RwLock::new(HashMap::new())),
            health_history: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(HealthCheckStats {
                total_checks: 0,
                successful_checks: 0,
                failed_checks: 0,
                average_duration_ms: 0.0,
                success_rate: 1.0,
                failure_rate: 0.0,
                last_check: Some(SystemTime::now().into()),
            })),
            alert_sender,
        };
        
        (service, alert_receiver)
    }
    
    /// Calculate health score based on various factors
    fn calculate_health_score(&self, service_id: &str, details: &[HealthCheckDetail]) -> f64 {
        let mut score = 100.0;
        
        // Base score from health status
        for detail in details {
            match detail.status() {
                HealthStatus::Healthy => score -= 0.0,
                HealthStatus::Degraded => score -= 10.0,
                HealthStatus::Unhealthy => score -= 30.0,
                HealthStatus::Critical => score -= 60.0,
                HealthStatus::Maintenance => score -= 5.0,
                HealthStatus::Unknown => score -= 20.0,
                _ => score -= 15.0,
            }
        }
        
        // Apply dependency health
        if let Ok(dependencies) = self.dependencies.read() {
            if let Some(deps) = dependencies.get(service_id) {
                let dep_score: f64 = deps.iter()
                    .map(|dep| dep.health_score)
                    .sum::<f64>() / deps.len() as f64;
                score = score * 0.7 + dep_score * 0.3;
            }
        }
        
        // Apply weights from configuration
        if let Some(weight) = self.config.health_score_weights.get("self_health") {
            score *= *weight;
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Check if health score triggers an alert
    fn check_alert_thresholds(&self, service_id: &str, health_score: f64) -> Option<HealthAlert> {
        if let Some(threshold) = self.config.alert_thresholds.get("health_score") {
            if health_score < *threshold {
                let severity = if health_score < 30.0 {
                    Severity::Critical
                } else if health_score < 50.0 {
                    Severity::High
                } else {
                    Severity::Medium
                };
                
                return Some(HealthAlert {
                    alert_id: Uuid::new_v4().to_string(),
                    severity: severity.into(),
                    message: format!("Health score {} below threshold {}", health_score, threshold),
                    service_id: service_id.to_string(),
                    timestamp: Some(SystemTime::now().into()),
                    status: "active".to_string(),
                    description: format!("Service {} health score {} is below the alert threshold of {}", 
                                      service_id, health_score, threshold),
                    metadata: HashMap::new(),
                    related_checks: Vec::new(),
                    acknowledgment: None,
                });
            }
        }
        
        None
    }
    
    /// Build dependency DAG for a service
    fn build_dependency_dag(&self, service_id: &str) -> DependencyDAG {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut direct_deps = Vec::new();
        let mut transitive_deps = Vec::new();
        let mut critical_path = Vec::new();
        
        // Get direct dependencies
        if let Ok(dependencies) = self.dependencies.read() {
            if let Some(deps) = dependencies.get(service_id) {
                direct_deps = deps.clone();
                for dep in deps {
                    queue.push_back(dep.service_id.clone());
                    visited.insert(dep.service_id.clone());
                }
            }
        }
        
        // Find transitive dependencies
        while let Some(dep_id) = queue.pop_front() {
            if let Ok(dependencies) = self.dependencies.read() {
                if let Some(deps) = dependencies.get(&dep_id) {
                    for dep in deps {
                        if !visited.contains(&dep.service_id) {
                            transitive_deps.push(dep.clone());
                            queue.push_back(dep.service_id.clone());
                            visited.insert(dep.service_id.clone());
                        }
                    }
                }
            }
        }
        
        // Calculate health summary
        let total_count = direct_deps.len() + transitive_deps.len();
        let healthy_count = direct_deps.iter().filter(|d| d.status() == HealthStatus::Healthy).count() +
                          transitive_deps.iter().filter(|d| d.status() == HealthStatus::Healthy).count();
        let degraded_count = direct_deps.iter().filter(|d| d.status() == HealthStatus::Degraded).count() +
                           transitive_deps.iter().filter(|d| d.status() == HealthStatus::Degraded).count();
        let unhealthy_count = direct_deps.iter().filter(|d| d.status() == HealthStatus::Unhealthy).count() +
                            transitive_deps.iter().filter(|d| d.status() == HealthStatus::Unhealthy).count();
        let critical_count = direct_deps.iter().filter(|d| d.status() == HealthStatus::Critical).count() +
                           transitive_deps.iter().filter(|d| d.status() == HealthStatus::Critical).count();
        
        let overall_score = if total_count > 0 {
            (healthy_count as f64 * 100.0 + degraded_count as f64 * 70.0 + 
             unhealthy_count as f64 * 30.0 + critical_count as f64 * 0.0) / total_count as f64
        } else {
            100.0
        };
        
        // Generate DOT graph
        let dot_graph = self.generate_dot_graph(service_id, &direct_deps, &transitive_deps);
        
        DependencyDAG {
            service_id: service_id.to_string(),
            dependencies: direct_deps,
            transitive_dependencies: transitive_deps,
            dot_graph,
            max_depth: self.calculate_max_depth(service_id),
            total_dependencies: total_count as i32,
            critical_path,
            health_summary: Some(DependencyHealthSummary {
                total_count: total_count as i32,
                healthy_count: healthy_count as i32,
                degraded_count: degraded_count as i32,
                unhealthy_count: unhealthy_count as i32,
                critical_count: critical_count as i32,
                overall_score,
                critical_dependencies: Vec::new(),
                failed_dependencies: Vec::new(),
            }),
        }
    }
    
    /// Generate DOT format graph for dependencies
    fn generate_dot_graph(&self, service_id: &str, direct_deps: &[Dependency], transitive_deps: &[Dependency]) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");
        dot.push_str(&format!("  \"{}\" [shape=box, style=filled, fillcolor=lightblue];\n", service_id));
        
        // Direct dependencies
        for dep in direct_deps {
            let color = match dep.status() {
                HealthStatus::Healthy => "green",
                HealthStatus::Degraded => "yellow",
                HealthStatus::Unhealthy => "orange",
                HealthStatus::Critical => "red",
                _ => "gray",
            };
            dot.push_str(&format!("  \"{}\" [shape=ellipse, style=filled, fillcolor={}];\n", dep.service_id, color));
            dot.push_str(&format!("  \"{}\" -> \"{}\" [style=bold];\n", service_id, dep.service_id));
        }
        
        // Transitive dependencies
        for dep in transitive_deps {
            let color = match dep.status() {
                HealthStatus::Healthy => "green",
                HealthStatus::Degraded => "yellow",
                HealthStatus::Unhealthy => "orange",
                HealthStatus::Critical => "red",
                _ => "gray",
            };
            dot.push_str(&format!("  \"{}\" [shape=ellipse, style=filled, fillcolor={}];\n", dep.service_id, color));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// Calculate maximum dependency depth
    fn calculate_max_depth(&self, service_id: &str) -> i32 {
        let mut max_depth = 0;
        let mut visited = HashSet::new();
        
        fn dfs(dependencies: &HashMap<String, Vec<Dependency>>, service_id: &str, depth: i32, max_depth: &mut i32, visited: &mut HashSet<String>) {
            if visited.contains(service_id) {
                return;
            }
            visited.insert(service_id.to_string());
            *max_depth = (*max_depth).max(depth);
            
            if let Some(deps) = dependencies.get(service_id) {
                for dep in deps {
                    dfs(dependencies, &dep.service_id, depth + 1, max_depth, visited);
                }
            }
        }
        
        if let Ok(dependencies) = self.dependencies.read() {
            dfs(&dependencies, service_id, 0, &mut max_depth, &mut visited);
        }
        
        max_depth
    }
    
    /// Update health statistics
    fn update_stats(&self, success: bool, duration_ms: i64) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_checks += 1;
            if success {
                stats.successful_checks += 1;
            } else {
                stats.failed_checks += 1;
            }
            
            // Update average duration
            let total_duration = stats.average_duration_ms * (stats.total_checks - 1) as f64 + duration_ms as f64;
            stats.average_duration_ms = total_duration / stats.total_checks as f64;
            
            // Update success/failure rates
            stats.success_rate = stats.successful_checks as f64 / stats.total_checks as f64;
            stats.failure_rate = stats.failed_checks as f64 / stats.total_checks as f64;
            
            stats.last_check = Some(SystemTime::now().into());
        }
    }
}

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn health_check(&self, request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        let start_time = Instant::now();
        let req = request.into_inner();
        
        let service_id = req.service_id;
        let include_details = req.include_details;
        
        // Perform health checks
        let mut details = Vec::new();
        
        // Self health check
        details.push(HealthCheckDetail {
            check_name: "self_health".to_string(),
            status: HealthStatus::Healthy.into(),
            description: "Service is running and responding".to_string(),
            duration_ms: 0,
            error_message: String::new(),
            data: HashMap::new(),
            severity: Severity::Low.into(),
            timestamp: Some(SystemTime::now().into()),
        });
        
        // Dependency health check
        if let Ok(dependencies) = self.dependencies.read() {
            if let Some(deps) = dependencies.get(&service_id) {
                for dep in deps {
                    details.push(HealthCheckDetail {
                        check_name: format!("dependency_{}", dep.name),
                        status: dep.status,
                        description: format!("Dependency {} health check", dep.name),
                        duration_ms: 0,
                        error_message: if dep.status() == HealthStatus::Healthy { String::new() } else { "Dependency unhealthy".to_string() },
                        data: HashMap::new(),
                        severity: if dep.status() == HealthStatus::Critical { Severity::Critical } else { Severity::Medium }.into(),
                        timestamp: Some(SystemTime::now().into()),
                    });
                }
            }
        }
        
        // Calculate health score
        let health_score = self.calculate_health_score(&service_id, &details);
        
        // Check for alerts
        if let Some(alert) = self.check_alert_thresholds(&service_id, health_score) {
            let _ = self.alert_sender.send(alert).await;
        }
        
        // Update health score
        if let Ok(mut scores) = self.health_scores.write() {
            scores.insert(service_id.clone(), health_score);
        }
        
        // Update health history
        if let Ok(mut history) = self.health_history.write() {
            let entry = history.entry(service_id.clone()).or_insert_with(VecDeque::new);
            entry.push_back(HealthCheckResponse {
                service_id: service_id.clone(),
                status: if health_score >= 90.0 { HealthStatus::Healthy } else if health_score >= 70.0 { HealthStatus::Degraded } else if health_score >= 50.0 { HealthStatus::Unhealthy } else { HealthStatus::Critical }.into(),
                health_score,
                timestamp: Some(SystemTime::now().into()),
                duration_ms: start_time.elapsed().as_millis() as i64,
                details: if include_details { details.clone() } else { Vec::new() },
                error_message: String::new(),
                metadata: Some(ServiceMetadata {
                    name: self.config.service_name.clone(),
                    version: self.config.service_version.clone(),
                    instance_id: self.config.instance_id.clone(),
                    environment: self.config.environment.clone(),
                    region: self.config.region.clone(),
                    tags: Vec::new(),
                    owner: "Polymera OS Team".to_string(),
                    description: "Health monitoring service".to_string(),
                    contact: "health@polymera-os.org".to_string(),
                    documentation_url: "https://docs.polymera-os.org/health".to_string(),
                }),
            });
            
            // Keep only last 100 entries
            if entry.len() > 100 {
                entry.pop_front();
            }
        }
        
        // Update statistics
        self.update_stats(true, start_time.elapsed().as_millis() as i64);
        
        let response = HealthCheckResponse {
            service_id,
            status: if health_score >= 90.0 { HealthStatus::Healthy } else if health_score >= 70.0 { HealthStatus::Degraded } else if health_score >= 50.0 { HealthStatus::Unhealthy } else { HealthStatus::Critical }.into(),
            health_score,
            timestamp: Some(SystemTime::now().into()),
            duration_ms: start_time.elapsed().as_millis() as i64,
            details: if include_details { details } else { Vec::new() },
            error_message: String::new(),
            metadata: Some(ServiceMetadata {
                name: self.config.service_name.clone(),
                version: self.config.service_version.clone(),
                instance_id: self.config.instance_id.clone(),
                environment: self.config.environment.clone(),
                region: self.config.region.clone(),
                tags: Vec::new(),
                owner: "Polymera OS Team".to_string(),
                description: "Health monitoring service".to_string(),
                contact: "health@polymera-os.org".to_string(),
                documentation_url: "https://docs.polymera-os.org/health".to_string(),
            }),
        };
        
        Ok(Response::new(response))
    }
    
    async fn get_health_details(&self, request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        // Same as health_check but always include details
        let mut req = request.into_inner();
        req.include_details = true;
        self.health_check(Request::new(req)).await
    }
    
    async fn get_dependencies(&self, request: Request<HealthCheckRequest>) -> Result<Response<DependencyDAG>, Status> {
        let req = request.into_inner();
        let service_id = req.service_id;
        
        let dag = self.build_dependency_dag(&service_id);
        Ok(Response::new(dag))
    }
    
    async fn aggregate_health(&self, request: Request<HealthAggregationRequest>) -> Result<Response<HealthAggregationResponse>, Status> {
        let req = request.into_inner();
        let service_ids = req.service_ids;
        let include_dependencies = req.include_dependencies;
        let include_details = req.include_details;
        
        let mut service_health = Vec::new();
        let mut dependency_dags = Vec::new();
        let mut total_services = 0;
        let mut healthy_services = 0;
        let mut degraded_services = 0;
        let mut unhealthy_services = 0;
        let mut critical_services = 0;
        let mut maintenance_services = 0;
        let mut total_score = 0.0;
        
        for service_id in service_ids {
            // Perform health check for each service
            let health_req = HealthCheckRequest {
                service_id: service_id.clone(),
                check_type: "aggregation".to_string(),
                parameters: HashMap::new(),
                timeout_seconds: 5,
                include_details,
            };
            
            if let Ok(response) = self.health_check(Request::new(health_req)).await {
                let health_response = response.into_inner();
                service_health.push(health_response.clone());
                
                total_services += 1;
                match health_response.status() {
                    HealthStatus::Healthy => healthy_services += 1,
                    HealthStatus::Degraded => degraded_services += 1,
                    HealthStatus::Unhealthy => unhealthy_services += 1,
                    HealthStatus::Critical => critical_services += 1,
                    HealthStatus::Maintenance => maintenance_services += 1,
                    _ => {},
                }
                
                total_score += health_response.health_score;
                
                // Get dependencies if requested
                if include_dependencies {
                    let dep_req = HealthCheckRequest {
                        service_id: service_id.clone(),
                        check_type: "dependencies".to_string(),
                        parameters: HashMap::new(),
                        timeout_seconds: 5,
                        include_details: false,
                    };
                    
                    if let Ok(dep_response) = self.get_dependencies(Request::new(dep_req)).await {
                        dependency_dags.push(dep_response.into_inner());
                    }
                }
            }
        }
        
        let system_score = if total_services > 0 { total_score / total_services as f64 } else { 0.0 };
        
        let response = HealthAggregationResponse {
            overall_status: if system_score >= 90.0 { HealthStatus::Healthy } else if system_score >= 70.0 { HealthStatus::Degraded } else if system_score >= 50.0 { HealthStatus::Unhealthy } else { HealthStatus::Critical }.into(),
            overall_score: system_score,
            timestamp: Some(SystemTime::now().into()),
            service_health,
            dependency_dags,
            system_summary: Some(SystemHealthSummary {
                total_services,
                healthy_services,
                degraded_services,
                unhealthy_services,
                critical_services,
                maintenance_services,
                system_score,
                critical_services: Vec::new(),
                failed_services: Vec::new(),
            }),
            stats: Some(self.stats.read().unwrap().clone()),
        };
        
        Ok(Response::new(response))
    }
    
    async fn get_system_health(&self, _request: Request<google::protobuf::Empty>) -> Result<Response<SystemHealthSummary>, Status> {
        // Get all registered services
        let services = self.services.read().unwrap();
        let service_ids: Vec<String> = services.keys().cloned().collect();
        drop(services);
        
        // Aggregate health for all services
        let req = HealthAggregationRequest {
            service_ids,
            include_dependencies: true,
            include_details: false,
            timeout_seconds: 10,
            concurrency_limit: 10,
        };
        
        let response = self.aggregate_health(Request::new(req)).await?;
        let agg_response = response.into_inner();
        
        Ok(Response::new(agg_response.system_summary.unwrap()))
    }
    
    async fn get_health_metrics(&self, request: Request<HealthCheckRequest>) -> Result<Response<HealthMetrics>, Status> {
        let req = request.into_inner();
        let service_id = req.service_id;
        
        // Generate mock metrics for demonstration
        let metrics = HealthMetrics {
            service_id,
            timestamp: Some(SystemTime::now().into()),
            cpu_usage_percent: 25.5,
            memory_usage_percent: 45.2,
            disk_usage_percent: 30.1,
            network_io_bps: 1024.0 * 1024.0, // 1 MB/s
            request_rate_rps: 150.0,
            error_rate_eps: 0.5,
            response_time_ms: 45.0,
            active_connections: 25,
            queue_depth: 5,
            custom_metrics: {
                let mut map = HashMap::new();
                map.insert("cache_hit_rate".to_string(), 95.5);
                map.insert("database_connections".to_string(), 10.0);
                map
            },
        };
        
        Ok(Response::new(metrics))
    }
    
    async fn stream_health_updates(&self, request: Request<HealthCheckRequest>) -> Result<Response<tonic::codec::Streaming<HealthCheckResponse>>, Status> {
        // Implementation for streaming health updates
        unimplemented!("Streaming health updates not implemented yet");
    }
    
    async fn stream_health_alerts(&self, _request: Request<google::protobuf::Empty>) -> Result<Response<tonic::codec::Streaming<HealthAlert>>, Status> {
        // Implementation for streaming health alerts
        unimplemented!("Streaming health alerts not implemented yet");
    }
    
    async fn acknowledge_alert(&self, request: Request<AlertAcknowledgment>) -> Result<Response<google::protobuf::Empty>, Status> {
        let ack = request.into_inner();
        
        // Update alert status
        if let Ok(mut alerts) = self.alerts.write() {
            for alert in alerts.iter_mut() {
                if alert.alert_id == ack.acknowledged_by {
                    alert.status = "acknowledged".to_string();
                    alert.acknowledgment = Some(ack);
                    break;
                }
            }
        }
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
    
    async fn get_health_stats(&self, _request: Request<google::protobuf::Empty>) -> Result<Response<HealthCheckStats>, Status> {
        let stats = self.stats.read().unwrap().clone();
        Ok(Response::new(stats))
    }
}

#[tonic::async_trait]
impl HealthMonitoringService for HealthServiceImpl {
    async fn register_service(&self, request: Request<ServiceMetadata>) -> Result<Response<google::protobuf::Empty>, Status> {
        let metadata = request.into_inner();
        let service_id = metadata.instance_id.clone();
        
        if let Ok(mut services) = self.services.write() {
            services.insert(service_id, metadata);
        }
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
    
    async fn unregister_service(&self, request: Request<ServiceMetadata>) -> Result<Response<google::protobuf::Empty>, Status> {
        let metadata = request.into_inner();
        let service_id = metadata.instance_id;
        
        if let Ok(mut services) = self.services.write() {
            services.remove(&service_id);
        }
        
        if let Ok(mut dependencies) = self.dependencies.write() {
            dependencies.remove(&service_id);
        }
        
        if let Ok(mut scores) = self.health_scores.write() {
            scores.remove(&service_id);
        }
        
        if let Ok(mut history) = self.health_history.write() {
            history.remove(&service_id);
        }
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
    
    async fn update_health_status(&self, request: Request<HealthCheckResponse>) -> Result<Response<google::protobuf::Empty>, Status> {
        let health_response = request.into_inner();
        let service_id = health_response.service_id.clone();
        
        // Update health score
        if let Ok(mut scores) = self.health_scores.write() {
            scores.insert(service_id.clone(), health_response.health_score);
        }
        
        // Update health history
        if let Ok(mut history) = self.health_history.write() {
            let entry = history.entry(service_id).or_insert_with(VecDeque::new);
            entry.push_back(health_response);
            
            // Keep only last 100 entries
            if entry.len() > 100 {
                entry.pop_front();
            }
        }
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
    
    async fn report_dependency_health(&self, request: Request<Dependency>) -> Result<Response<google::protobuf::Empty>, Status> {
        let dependency = request.into_inner();
        let service_id = dependency.service_id.clone();
        
        if let Ok(mut dependencies) = self.dependencies.write() {
            let deps = dependencies.entry(service_id).or_insert_with(Vec::new);
            
            // Update existing dependency or add new one
            if let Some(existing_dep) = deps.iter_mut().find(|d| d.service_id == dependency.service_id) {
                *existing_dep = dependency;
            } else {
                deps.push(dependency);
            }
        }
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
    
    async fn report_health_alert(&self, request: Request<HealthAlert>) -> Result<Response<google::protobuf::Empty>, Status> {
        let alert = request.into_inner();
        
        // Store alert
        if let Ok(mut alerts) = self.alerts.write() {
            alerts.push(alert.clone());
            
            // Keep only last 1000 alerts
            if alerts.len() > 1000 {
                alerts.remove(0);
            }
        }
        
        // Send alert to alert channel
        let _ = self.alert_sender.send(alert).await;
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
    
    async fn get_monitoring_config(&self, _request: Request<google::protobuf::Empty>) -> Result<Response<MonitoringConfig>, Status> {
        let config = MonitoringConfig {
            health_check_interval_seconds: self.config.health_check_interval.as_secs() as i32,
            health_check_timeout_seconds: 5,
            dependency_check_interval_seconds: self.config.dependency_check_interval.as_secs() as i32,
            alert_thresholds: self.config.alert_thresholds.clone(),
            default_retry_config: Some(RetryConfig {
                max_retries: 3,
                initial_delay_ms: 1000,
                max_delay_ms: 10000,
                delay_multiplier: 2.0,
                retryable_errors: vec!["timeout".to_string(), "connection_error".to_string()],
            }),
            default_circuit_breaker_config: Some(CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 2,
                timeout_ms: 30000,
                minimum_request_count: 10,
            }),
            health_score_weights: self.config.health_score_weights.clone(),
            monitoring_enabled: true,
            alerting_enabled: true,
            metrics_collection_enabled: true,
        };
        
        Ok(Response::new(config))
    }
    
    async fn update_monitoring_config(&self, request: Request<MonitoringConfig>) -> Result<Response<google::protobuf::Empty>, Status> {
        let config = request.into_inner();
        
        // Update configuration
        self.config.health_check_interval = Duration::from_secs(config.health_check_interval_seconds as u64);
        self.config.dependency_check_interval = Duration::from_secs(config.dependency_check_interval_seconds as u64);
        self.config.alert_thresholds = config.alert_thresholds;
        self.config.health_score_weights = config.health_score_weights;
        
        Ok(Response::new(google::protobuf::Empty {}))
    }
}

/// Health check service for gRPC health checks
#[derive(Debug, Default)]
pub struct HealthCheckService;

#[tonic::async_trait]
impl Health for HealthCheckService {
    async fn check(&self, _request: Request<tonic_health::HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: HealthCheckResponse_ServingStatus::Serving.into(),
        }))
    }
    
    async fn watch(&self, _request: Request<tonic_health::HealthCheckRequest>) -> Result<Response<tonic::codec::Streaming<HealthCheckResponse>>, Status> {
        unimplemented!("Watch not implemented yet");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    // Create health service
    let config = HealthServiceConfig::default();
    let (health_service, mut alert_receiver) = HealthServiceImpl::new(config);
    
    // Start alert processing
    let alert_service = health_service.clone();
    tokio::spawn(async move {
        while let Some(alert) = alert_receiver.recv().await {
            tracing::warn!("Health Alert: {} - {}", alert.severity().as_str_name(), alert.message);
            
            // Here you would typically send alerts to external systems
            // like PagerDuty, Slack, email, etc.
        }
    });
    
    let addr: SocketAddr = "[::1]:50052".parse()?;
    
    // Create reflection service
    let reflection_service = ReflectionBuilder::configure()
        .register_grpc_reflection_service()
        .build()?;
    
    tracing::info!("🚀 Health Service starting on {}", addr);
    
    // Start server
    Server::builder()
        .add_service(HealthServiceServer::new(health_service))
        .add_service(HealthMonitoringServiceServer::new(health_service))
        .add_service(HealthServer::new(HealthCheckService))
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    
    Ok(())
}
