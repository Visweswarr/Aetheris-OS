use super::{
    MeshNode, MeshNodeConfig, DidPeerId, TransportConfig, RendezvousConfig,
    default_transport_config, default_rendezvous_config,
};
use libp2p::{PeerId, Multiaddr};
use std::time::Duration;
use tracing::{info, warn, error};

/// Run comprehensive mesh network tests
pub async fn run_mesh_tests(num_nodes: usize, test_duration_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting mesh network tests with {} nodes for {}ms", num_nodes, test_duration_ms);
    
    // Test 1: Basic mesh node creation
    test_mesh_node_creation().await?;
    
    // Test 2: Mesh node configuration
    test_mesh_node_configuration().await?;
    
    // Test 3: Transport layer functionality
    test_transport_functionality().await?;
    
    // Test 4: DID peer ID generation
    test_did_peer_id_generation().await?;
    
    // Test 5: Local mesh formation
    test_local_mesh_formation(num_nodes).await?;
    
    // Test 6: RTT measurement
    test_rtt_measurement().await?;
    
    // Test 7: Peer discovery
    test_peer_discovery().await?;
    
    // Test 8: Connection management
    test_connection_management().await?;
    
    // Test 9: Mesh network stability
    test_mesh_network_stability(num_nodes, test_duration_ms).await?;
    
    info!("All mesh network tests completed successfully!");
    Ok(())
}

/// Test basic mesh node creation
async fn test_mesh_node_creation() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing mesh node creation...");
    
    let did_peer_id = DidPeerId::generate()?;
    let transport_config = default_transport_config();
    let rendezvous_config = default_rendezvous_config();
    
    let mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await?;
    
    assert!(!mesh_node.is_started());
    assert_eq!(mesh_node.local_peer_id(), mesh_node.did_peer_id().peer_id());
    
    info!("✅ Mesh node creation test passed");
    Ok(())
}

/// Test mesh node configuration
async fn test_mesh_node_configuration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing mesh node configuration...");
    
    let config = MeshNodeConfig::default();
    
    assert_eq!(config.mesh.max_connections, 100);
    assert_eq!(config.mesh.connection_timeout, 30);
    assert_eq!(config.mesh.ping_interval, 30);
    assert_eq!(config.mesh.discovery_interval, 300);
    assert!(config.mesh.enable_auto_discovery);
    assert!(config.mesh.enable_auto_cleanup);
    
    info!("✅ Mesh node configuration test passed");
    Ok(())
}

/// Test transport layer functionality
async fn test_transport_functionality() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing transport layer functionality...");
    
    let transport_config = default_transport_config();
    
    assert_eq!(transport_config.bind_addr, "0.0.0.0:0");
    assert!(transport_config.enable_quic);
    assert!(transport_config.enable_tcp);
    assert!(transport_config.enable_noise);
    assert!(transport_config.enable_mdns);
    
    info!("✅ Transport layer functionality test passed");
    Ok(())
}

/// Test DID peer ID generation
async fn test_did_peer_id_generation() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing DID peer ID generation...");
    
    let did_peer_id = DidPeerId::generate()?;
    
    assert!(!did_peer_id.did().is_empty());
    assert!(did_peer_id.did().starts_with("did:polynet:"));
    assert!(!did_peer_id.is_public_only());
    
    info!("✅ DID peer ID generation test passed");
    Ok(())
}

/// Test local mesh formation with specified number of nodes
async fn test_local_mesh_formation(num_nodes: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing local mesh formation with {} nodes...", num_nodes);
    
    if num_nodes < 2 {
        warn!("Skipping mesh formation test - need at least 2 nodes");
        return Ok(());
    }
    
    // Create nodes
    let mut nodes = Vec::new();
    for i in 0..num_nodes {
        info!("Creating node {}", i);
        
        let did_peer_id = DidPeerId::generate()?;
        let transport_config = TransportConfig {
            bind_addr: format!("127.0.0.1:{}", 8080 + i),
            enable_quic: true,
            enable_tcp: true,
            enable_noise: true,
            enable_mdns: true,
            connection_timeout: Some(10),
            keep_alive_interval: Some(5),
        };
        let rendezvous_config = RendezvousConfig {
            server: None,
            namespace: "test-mesh".to_string(),
            ttl: 300,
            discovery_interval: Some(10),
            max_peers: Some(100),
        };
        
        let mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await?;
        nodes.push(mesh_node);
    }
    
    // Start all nodes
    for (i, node) in nodes.iter_mut().enumerate() {
        info!("Starting node {}", i);
        node.start().await?;
        assert!(node.is_started());
    }
    
    // Form mesh by connecting nodes in a ring topology
    for i in 0..num_nodes {
        let current_node = &mut nodes[i];
        let next_node = &nodes[(i + 1) % num_nodes];
        
        let next_peer_id = next_node.local_peer_id();
        let next_addr = format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", 8080 + ((i + 1) % num_nodes), next_peer_id);
        
        info!("Connecting node {} to node {}", i, (i + 1) % num_nodes);
        current_node.connect_to_peer(&next_peer_id.to_string(), &next_addr).await?;
    }
    
    // Wait for connections to stabilize
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Verify mesh formation
    for (i, node) in nodes.iter().enumerate() {
        let connections = node.get_connections().await;
        info!("Node {} has {} connections", i, connections.len());
        
        // Each node should have at least 1 connection (to next node in ring)
        assert!(connections.len() >= 1, "Node {} has insufficient connections", i);
    }
    
    // Stop all nodes
    for (i, node) in nodes.iter_mut().enumerate() {
        info!("Stopping node {}", i);
        node.stop().await?;
        assert!(!node.is_started());
    }
    
    info!("✅ Local mesh formation test passed with {} nodes", num_nodes);
    Ok(())
}

/// Test RTT measurement
async fn test_rtt_measurement() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing RTT measurement...");
    
    // Create two nodes for RTT testing
    let did_peer_id_1 = DidPeerId::generate()?;
    let did_peer_id_2 = DidPeerId::generate()?;
    
    let transport_config_1 = TransportConfig {
        bind_addr: "127.0.0.1:8080".to_string(),
        enable_quic: true,
        enable_tcp: true,
        enable_noise: true,
        enable_mdns: true,
        connection_timeout: Some(10),
        keep_alive_interval: Some(5),
    };
    
    let transport_config_2 = TransportConfig {
        bind_addr: "127.0.0.1:8081".to_string(),
        enable_quic: true,
        enable_tcp: true,
        enable_noise: true,
        enable_mdns: true,
        connection_timeout: Some(10),
        keep_alive_interval: Some(5),
    };
    
    let rendezvous_config = RendezvousConfig {
        server: None,
        namespace: "test-rtt".to_string(),
        ttl: 300,
        discovery_interval: Some(10),
        max_peers: Some(100),
    };
    
    let mut node_1 = MeshNode::new(did_peer_id_1, transport_config_1, rendezvous_config.clone()).await?;
    let mut node_2 = MeshNode::new(did_peer_id_2, transport_config_2, rendezvous_config).await?;
    
    // Start nodes
    node_1.start().await?;
    node_2.start().await?;
    
    // Connect nodes
    let peer_id_2 = node_2.local_peer_id();
    let addr_2 = format!("/ip4/127.0.0.1/tcp/8081/p2p/{}", peer_id_2);
    
    node_1.connect_to_peer(&peer_id_2.to_string(), &addr_2).await?;
    
    // Wait for connection to stabilize
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Measure RTT
    let rtt = node_1.ping_peer(&peer_id_2).await?;
    info!("RTT to peer {}: {:?}", peer_id_2, rtt);
    
    // RTT should be reasonable (less than 1 second for local connections)
    assert!(rtt < Duration::from_secs(1), "RTT too high: {:?}", rtt);
    
    // Stop nodes
    node_1.stop().await?;
    node_2.stop().await?;
    
    info!("✅ RTT measurement test passed");
    Ok(())
}

/// Test peer discovery
async fn test_peer_discovery() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing peer discovery...");
    
    let did_peer_id = DidPeerId::generate()?;
    let transport_config = default_transport_config();
    let rendezvous_config = RendezvousConfig {
        server: None,
        namespace: "test-discovery".to_string(),
        ttl: 300,
        discovery_interval: Some(10),
        max_peers: Some(100),
    };
    
    let mut mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await?;
    
    // Start node
    mesh_node.start().await?;
    
    // Wait for discovery to run
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Get peer book stats
    let stats = mesh_node.get_peer_book_stats().await?;
    info!("Peer book stats: {}", stats);
    
    // Stop node
    mesh_node.stop().await?;
    
    info!("✅ Peer discovery test passed");
    Ok(())
}

/// Test connection management
async fn test_connection_management() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing connection management...");
    
    let did_peer_id = DidPeerId::generate()?;
    let transport_config = default_transport_config();
    let rendezvous_config = default_rendezvous_config();
    
    let mut mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await?;
    
    // Start node
    mesh_node.start().await?;
    
    // Get initial connection count
    let initial_connections = mesh_node.get_connections().await;
    assert_eq!(initial_connections.len(), 0);
    
    // Add a bootstrap peer
    mesh_node.add_bootstrap_peer("/ip4/127.0.0.1/tcp/8080/p2p/12D3KooWTestPeer123").await?;
    
    // Verify bootstrap peer was added
    let bootstrap_peers = mesh_node.get_bootstrap_peers();
    assert_eq!(bootstrap_peers.len(), 1);
    
    // Stop node
    mesh_node.stop().await?;
    
    info!("✅ Connection management test passed");
    Ok(())
}

/// Test mesh network stability over time
async fn test_mesh_network_stability(num_nodes: usize, test_duration_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing mesh network stability with {} nodes for {}ms", num_nodes, test_duration_ms);
    
    if num_nodes < 2 {
        warn!("Skipping stability test - need at least 2 nodes");
        return Ok(());
    }
    
    // Create and start nodes
    let mut nodes = Vec::new();
    for i in 0..num_nodes {
        let did_peer_id = DidPeerId::generate()?;
        let transport_config = TransportConfig {
            bind_addr: format!("127.0.0.1:{}", 8080 + i),
            enable_quic: true,
            enable_tcp: true,
            enable_noise: true,
            enable_mdns: true,
            connection_timeout: Some(10),
            keep_alive_interval: Some(5),
        };
        let rendezvous_config = RendezvousConfig {
            server: None,
            namespace: "test-stability".to_string(),
            ttl: 300,
            discovery_interval: Some(10),
            max_peers: Some(100),
        };
        
        let mut mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await?;
        mesh_node.start().await?;
        nodes.push(mesh_node);
    }
    
    // Form mesh
    for i in 0..num_nodes {
        let current_node = &mut nodes[i];
        let next_node = &nodes[(i + 1) % num_nodes];
        
        let next_peer_id = next_node.local_peer_id();
        let next_addr = format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", 8080 + ((i + 1) % num_nodes), next_peer_id);
        
        current_node.connect_to_peer(&next_peer_id.to_string(), &next_addr).await?;
    }
    
    // Monitor stability over time
    let start_time = std::time::Instant::now();
    let test_duration = Duration::from_millis(test_duration_ms);
    let mut stability_checks = 0;
    
    while start_time.elapsed() < test_duration {
        // Check connection stability
        for (i, node) in nodes.iter().enumerate() {
            let connections = node.get_connections().await;
            let peer_count = node.get_peer_count().await;
            
            if stability_checks % 10 == 0 {
                info!("Stability check {}: Node {} has {} connections, {} peers", 
                    stability_checks, i, connections.len(), peer_count);
            }
            
            // Each node should maintain at least 1 connection
            assert!(connections.len() >= 1, "Node {} lost all connections", i);
        }
        
        stability_checks += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    info!("Completed {} stability checks over {:?}", stability_checks, test_duration);
    
    // Stop all nodes
    for (i, node) in nodes.iter_mut().enumerate() {
        info!("Stopping node {}", i);
        node.stop().await?;
    }
    
    info!("✅ Mesh network stability test passed");
    Ok(())
}

/// Run a quick smoke test
pub async fn run_smoke_test() -> Result<(), Box<dyn std::error::Error>> {
    info!("Running mesh networking smoke test...");
    
    // Test basic functionality
    test_mesh_node_creation().await?;
    test_did_peer_id_generation().await?;
    
    // Test minimal mesh formation
    test_local_mesh_formation(2).await?;
    
    info!("✅ Smoke test passed");
    Ok(())
}

/// Run performance benchmarks
pub async fn run_performance_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    info!("Running mesh networking performance benchmarks...");
    
    let start_time = std::time::Instant::now();
    
    // Benchmark node creation
    let creation_start = std::time::Instant::now();
    for _ in 0..10 {
        let did_peer_id = DidPeerId::generate()?;
        let transport_config = default_transport_config();
        let rendezvous_config = default_rendezvous_config();
        
        let _mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await?;
    }
    let creation_time = creation_start.elapsed();
    info!("Node creation benchmark: 10 nodes in {:?} ({:?} per node)", 
        creation_time, creation_time / 10);
    
    // Benchmark mesh formation
    let formation_start = std::time::Instant::now();
    test_local_mesh_formation(3).await?;
    let formation_time = formation_start.elapsed();
    info!("Mesh formation benchmark: 3-node mesh in {:?}", formation_time);
    
    let total_time = start_time.elapsed();
    info!("Total benchmark time: {:?}", total_time);
    
    info!("✅ Performance benchmarks completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_smoke_test() {
        run_smoke_test().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_mesh_tests() {
        run_mesh_tests(2, 1000).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_performance_benchmarks() {
        run_performance_benchmarks().await.unwrap();
    }
}
