use clap::{Parser, Subcommand};
use polynet::{
    mesh::MeshNode,
    transport::TransportConfig,
    rendezvous::RendezvousConfig,
    peerbook::PeerBook,
    did::DidPeerId,
};
use std::path::PathBuf;
use tracing::{info, warn, error};

#[derive(Parser)]
#[command(name = "polynet")]
#[command(about = "Polymera OS Mesh Networking Service")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, default_value = "info")]
    log_level: String,
    
    #[arg(long, default_value = "config/polynet.json")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the mesh networking service
    Start {
        #[arg(long, default_value = "0.0.0.0")]
        bind_addr: String,
        
        #[arg(long, default_value = "0")]
        port: u16,
        
        #[arg(long)]
        bootstrap_peers: Vec<String>,
        
        #[arg(long)]
        rendezvous_server: Option<String>,
        
        #[arg(long)]
        did_key: Option<String>,
    },
    
    /// Join an existing mesh network
    Join {
        #[arg(long)]
        peer_id: String,
        
        #[arg(long)]
        address: String,
        
        #[arg(long)]
        did_key: Option<String>,
    },
    
    /// List known peers
    Peers,
    
    /// Ping a specific peer
    Ping {
        #[arg(long)]
        peer_id: String,
    },
    
    /// Run mesh network tests
    Test {
        #[arg(long, default_value = "3")]
        num_nodes: usize,
        
        #[arg(long, default_value = "1000")]
        test_duration_ms: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(format!("polynet={}", cli.log_level))
        .init();
    
    info!("Starting Polymera OS Mesh Networking Service");
    
    match cli.command {
        Commands::Start { bind_addr, port, bootstrap_peers, rendezvous_server, did_key } => {
            start_mesh_service(bind_addr, port, bootstrap_peers, rendezvous_server, did_key).await?;
        }
        Commands::Join { peer_id, address, did_key } => {
            join_mesh_network(peer_id, address, did_key).await?;
        }
        Commands::Peers => {
            list_peers().await?;
        }
        Commands::Ping { peer_id } => {
            ping_peer(peer_id).await?;
        }
        Commands::Test { num_nodes, test_duration_ms } => {
            run_mesh_tests(num_nodes, test_duration_ms).await?;
        }
    }
    
    Ok(())
}

async fn start_mesh_service(
    bind_addr: String,
    port: u16,
    bootstrap_peers: Vec<String>,
    rendezvous_server: Option<String>,
    did_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting mesh networking service on {}:{}", bind_addr, port);
    
    // Generate or load DID-based peer ID
    let peer_id = if let Some(key) = did_key {
        DidPeerId::from_did_key(&key)?
    } else {
        DidPeerId::generate()?
    };
    
    info!("Using peer ID: {}", peer_id);
    
    // Configure transport
    let transport_config = TransportConfig {
        bind_addr: format!("{}:{}", bind_addr, port),
        enable_quic: true,
        enable_tcp: true,
        enable_noise: true,
        enable_mdns: true,
    };
    
    // Configure rendezvous
    let rendezvous_config = RendezvousConfig {
        server: rendezvous_server,
        namespace: "polymera-os".to_string(),
        ttl: 3600, // 1 hour
    };
    
    // Create mesh node
    let mut mesh_node = MeshNode::new(
        peer_id,
        transport_config,
        rendezvous_config,
    ).await?;
    
    // Add bootstrap peers
    for peer in bootstrap_peers {
        info!("Adding bootstrap peer: {}", peer);
        mesh_node.add_bootstrap_peer(&peer).await?;
    }
    
    // Start the mesh node
    info!("Starting mesh node...");
    mesh_node.start().await?;
    
    // Keep the service running
    info!("Mesh networking service is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down mesh networking service...");
    mesh_node.stop().await?;
    
    Ok(())
}

async fn join_mesh_network(
    peer_id: String,
    address: String,
    did_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Joining mesh network at {}:{}", peer_id, address);
    
    // Generate or load DID-based peer ID
    let local_peer_id = if let Some(key) = did_key {
        DidPeerId::from_did_key(&key)?
    } else {
        DidPeerId::generate()?
    };
    
    info!("Local peer ID: {}", local_peer_id);
    
    // Configure transport for joining
    let transport_config = TransportConfig {
        bind_addr: "0.0.0.0:0".to_string(), // Use any available port
        enable_quic: true,
        enable_tcp: true,
        enable_noise: true,
        enable_mdns: true,
    };
    
    // Configure rendezvous
    let rendezvous_config = RendezvousConfig {
        server: None,
        namespace: "polymera-os".to_string(),
        ttl: 3600,
    };
    
    // Create mesh node
    let mut mesh_node = MeshNode::new(
        local_peer_id,
        transport_config,
        rendezvous_config,
    ).await?;
    
    // Connect to the specified peer
    info!("Connecting to peer {} at {}", peer_id, address);
    mesh_node.connect_to_peer(&peer_id, &address).await?;
    
    // Start the mesh node
    mesh_node.start().await?;
    
    // Keep the connection alive
    info!("Connected to mesh network. Press Ctrl+C to disconnect.");
    tokio::signal::ctrl_c().await?;
    
    info!("Disconnecting from mesh network...");
    mesh_node.stop().await?;
    
    Ok(())
}

async fn list_peers() -> Result<(), Box<dyn std::error::Error>> {
    info!("Listing known peers...");
    
    // Load peer book from storage
    let peer_book = PeerBook::load().await?;
    let peers = peer_book.list_peers().await?;
    
    if peers.is_empty() {
        println!("No known peers");
    } else {
        println!("Known peers:");
        for peer in peers {
            println!("  - {} ({})", peer.id, peer.addresses.join(", "));
        }
    }
    
    Ok(())
}

async fn ping_peer(peer_id: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Pinging peer: {}", peer_id);
    
    // This would require a running mesh node
    // For now, just show the command structure
    println!("Ping functionality requires a running mesh node");
    println!("Use 'polynet start' to start the service first");
    
    Ok(())
}

async fn run_mesh_tests(num_nodes: usize, test_duration_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running mesh network tests with {} nodes for {}ms", num_nodes, test_duration_ms);
    
    // Run the mesh test suite
    polynet::tests::run_mesh_tests(num_nodes, test_duration_ms).await?;
    
    Ok(())
}
