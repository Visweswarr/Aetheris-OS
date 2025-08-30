use super::envelope::{DtnEnvelope, RoutingStrategy, GeographicConstraints, NetworkConstraints};
use super::store::DtnStore;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tracing::{debug, info, warn, error};

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("Routing failed: {0}")]
    RoutingFailed(String),
    
    #[error("No route found: {0}")]
    NoRouteFound(String),
    
    #[error("Routing constraints violated: {0}")]
    ConstraintsViolated(String),
    
    #[error("Maximum hops exceeded: {0}")]
    MaxHopsExceeded(String),
    
    #[error("Invalid routing strategy: {0}")]
    InvalidStrategy(String),
}

#[derive(Debug, Clone)]
pub struct RoutingNode {
    pub node_id: String,
    pub capabilities: HashSet<String>,
    pub location: Option<GeographicLocation>,
    pub interfaces: Vec<NetworkInterface>,
    pub last_seen: SystemTime,
    pub storage_capacity: u64,
    pub available_storage: u64,
}

#[derive(Debug, Clone)]
pub struct GeographicLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub accuracy: f64,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub interface_type: String,
    pub address: String,
    pub capabilities: HashSet<String>,
    pub bandwidth: u64,
    pub status: InterfaceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceStatus {
    Up,
    Down,
    Error,
}

#[derive(Debug, Clone)]
pub struct RoutingTable {
    pub routes: HashMap<String, Route>,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub destination: String,
    pub next_hop: String,
    pub cost: u64,
    pub hops: u32,
    pub last_used: SystemTime,
    pub reliability: f64,
}

pub struct DtnRouter {
    routing_table: RoutingTable,
    known_nodes: HashMap<String, RoutingNode>,
    routing_strategies: HashMap<RoutingStrategy, Box<dyn RoutingStrategyImpl>>,
}

impl DtnRouter {
    /// Create a new DTN router
    pub fn new() -> Self {
        let mut router = Self {
            routing_table: RoutingTable {
                routes: HashMap::new(),
                last_updated: SystemTime::now(),
            },
            known_nodes: HashMap::new(),
            routing_strategies: HashMap::new(),
        };
        
        // Register routing strategies
        router.register_routing_strategy(RoutingStrategy::Flood, Box::new(FloodRoutingStrategy));
        router.register_routing_strategy(RoutingStrategy::Directed, Box::new(DirectedRoutingStrategy));
        router.register_routing_strategy(RoutingStrategy::Opportunistic, Box::new(OpportunisticRoutingStrategy));
        router.register_routing_strategy(RoutingStrategy::Epidemic, Box::new(EpidemicRoutingStrategy));
        router.register_routing_strategy(RoutingStrategy::SprayAndWait, Box::new(SprayAndWaitRoutingStrategy));
        
        router
    }
    
    /// Register a routing strategy
    pub fn register_routing_strategy(
        &mut self,
        strategy: RoutingStrategy,
        implementation: Box<dyn RoutingStrategyImpl>,
    ) {
        self.routing_strategies.insert(strategy, implementation);
    }
    
    /// Add a known node
    pub fn add_node(&mut self, node: RoutingNode) {
        self.known_nodes.insert(node.node_id.clone(), node);
        self.update_routing_table();
    }
    
    /// Remove a known node
    pub fn remove_node(&mut self, node_id: &str) {
        self.known_nodes.remove(node_id);
        self.update_routing_table();
    }
    
    /// Route an envelope
    pub async fn route_envelope(
        &self,
        envelope: &DtnEnvelope,
        store: &DtnStore,
    ) -> Result<Vec<RoutingDecision>, RoutingError> {
        let strategy = envelope.routing.strategy;
        
        if let Some(strategy_impl) = self.routing_strategies.get(&strategy) {
            strategy_impl.route(envelope, store, &self.known_nodes, &self.routing_table).await
        } else {
            Err(RoutingError::InvalidStrategy(
                format!("Unknown routing strategy: {:?}", strategy)
            ))
        }
    }
    
    /// Update routing table
    fn update_routing_table(&mut self) {
        // Simple routing table update - in practice this would be more sophisticated
        self.routing_table.last_updated = SystemTime::now();
        
        // Clear old routes
        self.routing_table.routes.clear();
        
        // Generate routes based on known nodes
        for (node_id, node) in &self.known_nodes {
            // Simple direct route for now
            let route = Route {
                destination: node_id.clone(),
                next_hop: node_id.clone(),
                cost: 1,
                hops: 1,
                last_used: SystemTime::now(),
                reliability: 1.0,
            };
            self.routing_table.routes.insert(node_id.clone(), route);
        }
    }
    
    /// Check if routing constraints are satisfied
    pub fn check_constraints(
        &self,
        envelope: &DtnEnvelope,
        node: &RoutingNode,
    ) -> Result<bool, RoutingError> {
        // Check geographic constraints
        if let Some(geo_constraints) = &envelope.routing.geo_constraints {
            if let Some(node_location) = &node.location {
                if !self.check_geographic_constraints(geo_constraints, node_location) {
                    return Ok(false);
                }
            }
        }
        
        // Check network constraints
        if let Some(net_constraints) = &envelope.routing.net_constraints {
            if !self.check_network_constraints(net_constraints, node) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Check geographic constraints
    fn check_geographic_constraints(
        &self,
        constraints: &GeographicConstraints,
        location: &GeographicLocation,
    ) -> bool {
        // Check latitude bounds
        if location.latitude < constraints.min_latitude || location.latitude > constraints.max_latitude {
            return false;
        }
        
        // Check longitude bounds
        if location.longitude < constraints.min_longitude || location.longitude > constraints.max_longitude {
            return false;
        }
        
        // Check distance constraint (simplified)
        // In practice, you would calculate actual distance
        true
    }
    
    /// Check network constraints
    fn check_network_constraints(
        &self,
        constraints: &NetworkConstraints,
        node: &RoutingNode,
    ) -> bool {
        // Check required capabilities
        for required_capability in &constraints.required_capabilities {
            if !node.capabilities.contains(required_capability) {
                return false;
            }
        }
        
        // Check bandwidth
        let max_bandwidth = node.interfaces.iter()
            .filter(|iface| iface.status == InterfaceStatus::Up)
            .map(|iface| iface.bandwidth)
            .max()
            .unwrap_or(0);
        
        if max_bandwidth < constraints.min_bandwidth {
            return false;
        }
        
        // Check security level (simplified)
        // In practice, you would check actual security capabilities
        true
    }
    
    /// Get routing statistics
    pub fn get_routing_stats(&self) -> RoutingStats {
        RoutingStats {
            total_nodes: self.known_nodes.len(),
            total_routes: self.routing_table.routes.len(),
            last_updated: self.routing_table.last_updated,
            routing_strategies: self.routing_strategies.keys().cloned().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub next_hop: String,
    pub cost: u64,
    pub reliability: f64,
    pub strategy: RoutingStrategy,
    pub constraints_satisfied: bool,
}

#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_nodes: usize,
    pub total_routes: usize,
    pub last_updated: SystemTime,
    pub routing_strategies: Vec<RoutingStrategy>,
}

/// Trait for routing strategy implementations
#[async_trait::async_trait]
pub trait RoutingStrategyImpl: Send + Sync {
    async fn route(
        &self,
        envelope: &DtnEnvelope,
        store: &DtnStore,
        known_nodes: &HashMap<String, RoutingNode>,
        routing_table: &RoutingTable,
    ) -> Result<Vec<RoutingDecision>, RoutingError>;
}

/// Flood routing strategy - send to all neighbors
pub struct FloodRoutingStrategy;

#[async_trait::async_trait]
impl RoutingStrategyImpl for FloodRoutingStrategy {
    async fn route(
        &self,
        envelope: &DtnEnvelope,
        _store: &DtnStore,
        known_nodes: &HashMap<String, RoutingNode>,
        _routing_table: &RoutingTable,
    ) -> Result<Vec<RoutingDecision>, RoutingError> {
        let mut decisions = Vec::new();
        
        for (node_id, node) in known_nodes {
            // Skip source node
            if node_id == &envelope.source {
                continue;
            }
            
            // Check if node can handle the envelope
            if node.available_storage > 0 {
                let decision = RoutingDecision {
                    next_hop: node_id.clone(),
                    cost: 1,
                    reliability: 0.8, // Flood has lower reliability
                    strategy: RoutingStrategy::Flood,
                    constraints_satisfied: true, // Simplified
                };
                decisions.push(decision);
            }
        }
        
        Ok(decisions)
    }
}

/// Directed routing strategy - find direct path to destination
pub struct DirectedRoutingStrategy;

#[async_trait::async_trait]
impl RoutingStrategyImpl for DirectedRoutingStrategy {
    async fn route(
        &self,
        envelope: &DtnEnvelope,
        _store: &DtnStore,
        known_nodes: &HashMap<String, RoutingNode>,
        routing_table: &RoutingTable,
    ) -> Result<Vec<RoutingDecision>, RoutingError> {
        let mut decisions = Vec::new();
        
        // Check if destination is directly known
        if let Some(dest_node) = known_nodes.get(&envelope.destination) {
            if dest_node.available_storage > 0 {
                let decision = RoutingDecision {
                    next_hop: envelope.destination.clone(),
                    cost: 1,
                    reliability: 0.95, // Direct routing has high reliability
                    strategy: RoutingStrategy::Directed,
                    constraints_satisfied: true,
                };
                decisions.push(decision);
                return Ok(decisions);
            }
        }
        
        // Try to find route through routing table
        if let Some(route) = routing_table.routes.get(&envelope.destination) {
            if let Some(next_hop_node) = known_nodes.get(&route.next_hop) {
                if next_hop_node.available_storage > 0 {
                    let decision = RoutingDecision {
                        next_hop: route.next_hop.clone(),
                        cost: route.cost,
                        reliability: route.reliability,
                        strategy: RoutingStrategy::Directed,
                        constraints_satisfied: true,
                    };
                    decisions.push(decision);
                }
            }
        }
        
        Ok(decisions)
    }
}

/// Opportunistic routing strategy - forward when opportunity arises
pub struct OpportunisticRoutingStrategy;

#[async_trait::async_trait]
impl RoutingStrategyImpl for OpportunisticRoutingStrategy {
    async fn route(
        &self,
        envelope: &DtnEnvelope,
        _store: &DtnStore,
        known_nodes: &HashMap<String, RoutingNode>,
        _routing_table: &RoutingTable,
    ) -> Result<Vec<RoutingDecision>, RoutingError> {
        let mut decisions = Vec::new();
        
        // Find nodes that might be closer to destination
        for (node_id, node) in known_nodes {
            // Skip source node
            if node_id == &envelope.source {
                continue;
            }
            
            // Check if node has available storage
            if node.available_storage > 0 {
                // Simple opportunistic decision based on available storage
                let reliability = if node.available_storage > 1024 * 1024 { 0.9 } else { 0.7 };
                
                let decision = RoutingDecision {
                    next_hop: node_id.clone(),
                    cost: 2, // Opportunistic routing has higher cost
                    reliability,
                    strategy: RoutingStrategy::Opportunistic,
                    constraints_satisfied: true,
                };
                decisions.push(decision);
            }
        }
        
        Ok(decisions)
    }
}

/// Epidemic routing strategy - spread to all nodes
pub struct EpidemicRoutingStrategy;

#[async_trait::async_trait]
impl RoutingStrategyImpl for EpidemicRoutingStrategy {
    async fn route(
        &self,
        envelope: &DtnEnvelope,
        _store: &DtnStore,
        known_nodes: &HashMap<String, RoutingNode>,
        _routing_table: &RoutingTable,
    ) -> Result<Vec<RoutingDecision>, RoutingError> {
        let mut decisions = Vec::new();
        
        // Epidemic routing spreads to all available nodes
        for (node_id, node) in known_nodes {
            // Skip source node
            if node_id == &envelope.source {
                continue;
            }
            
            // Check if node can handle the envelope
            if node.available_storage > 0 {
                let decision = RoutingDecision {
                    next_hop: node_id.clone(),
                    cost: 1,
                    reliability: 0.85, // Epidemic has moderate reliability
                    strategy: RoutingStrategy::Epidemic,
                    constraints_satisfied: true,
                };
                decisions.push(decision);
            }
        }
        
        Ok(decisions)
    }
}

/// Spray and wait routing strategy - limited copies
pub struct SprayAndWaitRoutingStrategy;

#[async_trait::async_trait]
impl RoutingStrategyImpl for SprayAndWaitRoutingStrategy {
    async fn route(
        &self,
        envelope: &DtnEnvelope,
        _store: &DtnStore,
        known_nodes: &HashMap<String, RoutingNode>,
        _routing_table: &RoutingTable,
    ) -> Result<Vec<RoutingDecision>, RoutingError> {
        let mut decisions = Vec::new();
        
        // Spray and wait limits the number of copies
        let max_copies = 3; // Limit to 3 copies
        
        let mut available_nodes: Vec<_> = known_nodes.iter()
            .filter(|(node_id, node)| {
                node_id != &envelope.source && node.available_storage > 0
            })
            .collect();
        
        // Sort by available storage (prefer nodes with more storage)
        available_nodes.sort_by(|a, b| b.1.available_storage.cmp(&a.1.available_storage));
        
        // Take only the best nodes up to max_copies
        for (node_id, node) in available_nodes.into_iter().take(max_copies) {
            let decision = RoutingDecision {
                next_hop: node_id.clone(),
                cost: 2, // Spray and wait has moderate cost
                reliability: 0.9, // High reliability due to limited copies
                strategy: RoutingStrategy::SprayAndWait,
                constraints_satisfied: true,
            };
            decisions.push(decision);
        }
        
        Ok(decisions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::envelope::{DtnEnvelope, GeographicConstraints, NetworkConstraints};
    
    #[tokio::test]
    async fn test_router_creation() {
        let router = DtnRouter::new();
        let stats = router.get_routing_stats();
        
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_routes, 0);
        assert!(!stats.routing_strategies.is_empty());
    }
    
    #[tokio::test]
    async fn test_add_node() {
        let mut router = DtnRouter::new();
        
        let node = RoutingNode {
            node_id: "did:polynet:test".to_string(),
            capabilities: HashSet::new(),
            location: None,
            interfaces: Vec::new(),
            last_seen: SystemTime::now(),
            storage_capacity: 1024 * 1024,
            available_storage: 512 * 1024,
        };
        
        router.add_node(node);
        
        let stats = router.get_routing_stats();
        assert_eq!(stats.total_nodes, 1);
        assert_eq!(stats.total_routes, 1);
    }
    
    #[tokio::test]
    async fn test_flood_routing() {
        let router = DtnRouter::new();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Create a mock store (simplified)
        let store = super::super::store::DtnStore::new(
            super::super::store::StoreConfig::default()
        ).unwrap();
        
        let decisions = router.route_envelope(&envelope, &store).await.unwrap();
        
        // With no known nodes, should return empty decisions
        assert!(decisions.is_empty());
    }
    
    #[test]
    fn test_geographic_constraints() {
        let router = DtnRouter::new();
        
        let constraints = GeographicConstraints {
            min_latitude: -90.0,
            max_latitude: 90.0,
            min_longitude: -180.0,
            max_longitude: 180.0,
            max_distance: 1000,
        };
        
        let location = GeographicLocation {
            latitude: 45.0,
            longitude: -75.0,
            altitude: 100.0,
            accuracy: 10.0,
            timestamp: SystemTime::now(),
        };
        
        let satisfied = router.check_geographic_constraints(&constraints, &location);
        assert!(satisfied);
    }
    
    #[test]
    fn test_network_constraints() {
        let router = DtnRouter::new();
        
        let constraints = NetworkConstraints {
            required_capabilities: vec!["storage".to_string(), "routing".to_string()],
            min_bandwidth: 1000,
            max_latency: 100,
            min_security_level: super::super::envelope::SecurityLevel::Medium,
        };
        
        let mut capabilities = HashSet::new();
        capabilities.insert("storage".to_string());
        capabilities.insert("routing".to_string());
        
        let node = RoutingNode {
            node_id: "did:polynet:test".to_string(),
            capabilities,
            location: None,
            interfaces: vec![NetworkInterface {
                interface_type: "ethernet".to_string(),
                address: "192.168.1.1".to_string(),
                capabilities: HashSet::new(),
                bandwidth: 10000,
                status: InterfaceStatus::Up,
            }],
            last_seen: SystemTime::now(),
            storage_capacity: 1024 * 1024,
            available_storage: 512 * 1024,
        };
        
        let satisfied = router.check_network_constraints(&constraints, &node);
        assert!(satisfied);
    }
}
