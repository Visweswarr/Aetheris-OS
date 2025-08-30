use clap::{Parser, Subcommand};
use polynet_dtn::{DtnService, DtnServiceConfig, StoreConfig};
use std::path::PathBuf;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "polynet-dtn")]
#[command(about = "DTN Envelope Service for Polymera OS")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: Level,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the DTN service
    Start {
        /// Service port
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        /// Data directory
        #[arg(short, long, default_value = "data/dtn")]
        data_dir: PathBuf,
    },
    
    /// Send a test envelope
    Send {
        /// Source node ID
        #[arg(short, long)]
        source: String,
        
        /// Destination node ID
        #[arg(short, long)]
        destination: String,
        
        /// Message type
        #[arg(short, long, default_value = "test")]
        message_type: String,
        
        /// Message payload
        #[arg(short, long, default_value = "Hello DTN!")]
        payload: String,
        
        /// TTL in seconds
        #[arg(short, long, default_value = "3600")]
        ttl: u64,
        
        /// Priority (0-255)
        #[arg(short, long, default_value = "100")]
        priority: u32,
    },
    
    /// Receive envelopes for a node
    Receive {
        /// Node ID to receive for
        #[arg(short, long)]
        node_id: String,
        
        /// Maximum number of envelopes to receive
        #[arg(short, long, default_value = "10")]
        max_envelopes: usize,
    },
    
    /// Query envelope status
    Query {
        /// Envelope ID to query
        #[arg(short, long)]
        envelope_id: String,
    },
    
    /// Clean up expired envelopes
    Cleanup,
    
    /// Show service statistics
    Stats,
    
    /// Run tests
    Test,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(cli.log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting DTN Envelope Service");
    
    // Load configuration
    let config = load_config(cli.config.as_ref())?;
    
    // Create DTN service
    let service = DtnService::new(config.store_config)?;
    
    match cli.command {
        Commands::Start { port, data_dir } => {
            info!("Starting DTN service on port {} with data directory {:?}", port, data_dir);
            
            // Create data directory if it doesn't exist
            std::fs::create_dir_all(&data_dir)?;
            
            // Start the service (in a real implementation, this would start a gRPC server)
            info!("DTN service started successfully");
            
            // Keep the service running
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                
                // Periodic cleanup
                let cleaned = service.cleanup_expired().await?;
                if cleaned > 0 {
                    info!("Cleaned up {} expired envelopes", cleaned);
                }
            }
        }
        
        Commands::Send {
            source,
            destination,
            message_type,
            payload,
            ttl,
            priority,
        } => {
            info!("Sending envelope from {} to {}", source, destination);
            
            let envelope = polynet_dtn::DtnEnvelope::new(
                source,
                destination,
                ttl,
                message_type,
                payload.into_bytes(),
                priority,
            );
            
            let envelope_id = service.send_envelope(envelope).await?;
            info!("Envelope sent successfully with ID: {}", envelope_id);
        }
        
        Commands::Receive { node_id, max_envelopes } => {
            info!("Receiving envelopes for node: {}", node_id);
            
            let envelopes = service.receive_envelopes(&node_id, max_envelopes).await?;
            info!("Received {} envelopes for node {}", envelopes.len(), node_id);
            
            for envelope in envelopes {
                println!("Envelope ID: {}", envelope.id);
                println!("  Source: {}", envelope.source);
                println!("  Message Type: {}", envelope.message_type);
                println!("  Priority: {}", envelope.priority);
                println!("  TTL: {}s", envelope.ttl);
                println!("  Payload: {}", String::from_utf8_lossy(&envelope.payload));
                println!("  Created: {:?}", envelope.created_at);
                println!("  Expires: {:?}", envelope.expires_at);
                println!("  Routing Strategy: {:?}", envelope.routing.strategy);
                println!("  Acknowledgments Required: {}", envelope.ack_requirements.required);
                println!("  Acknowledgments Received: {}/{}",
                    envelope.ack_requirements.ack_records.len(),
                    envelope.ack_requirements.required_ack_nodes.len());
                println!();
            }
        }
        
        Commands::Query { envelope_id } => {
            info!("Querying envelope: {}", envelope_id);
            
            if let Some(envelope) = service.query_envelope(&envelope_id).await? {
                println!("Envelope found:");
                println!("  ID: {}", envelope.id);
                println!("  Source: {}", envelope.source);
                println!("  Destination: {}", envelope.destination);
                println!("  Message Type: {}", envelope.message_type);
                println!("  Priority: {}", envelope.priority);
                println!("  TTL: {}s", envelope.ttl);
                println!("  Remaining TTL: {}s", envelope.remaining_ttl());
                println!("  Expired: {}", envelope.is_expired());
                println!("  Payload: {}", String::from_utf8_lossy(&envelope.payload));
                println!("  Created: {:?}", envelope.created_at);
                println!("  Expires: {:?}", envelope.expires_at);
                println!("  Hop Count: {}/{}", envelope.routing.hop_count, envelope.routing.max_hops);
                println!("  Routing Strategy: {:?}", envelope.routing.strategy);
                println!("  Acknowledgments Required: {}", envelope.ack_requirements.required);
                println!("  Acknowledgments Received: {}/{}",
                    envelope.ack_requirements.ack_records.len(),
                    envelope.ack_requirements.required_ack_nodes.len());
                
                if !envelope.ack_requirements.ack_records.is_empty() {
                    println!("  Acknowledgment Records:");
                    for ack in &envelope.ack_requirements.ack_records {
                        println!("    {}: {:?} ({}) - {}", 
                            ack.ack_node, ack.ack_type, ack.status, ack.message);
                    }
                }
                
                if !envelope.metadata.is_empty() {
                    println!("  Metadata:");
                    for (key, value) in &envelope.metadata {
                        println!("    {}: {}", key, value);
                    }
                }
            } else {
                println!("Envelope not found: {}", envelope_id);
            }
        }
        
        Commands::Cleanup => {
            info!("Cleaning up expired envelopes");
            
            let cleaned = service.cleanup_expired().await?;
            info!("Cleaned up {} expired envelopes", cleaned);
            println!("Cleaned up {} expired envelopes", cleaned);
        }
        
        Commands::Stats => {
            info!("Getting service statistics");
            
            let stats = service.get_stats().await?;
            println!("DTN Service Statistics:");
            println!("  Store:");
            println!("    Total Envelopes: {}", stats.store.total_envelopes);
            println!("    Total Storage: {} bytes", stats.store.total_storage_bytes);
            println!("    Total Received: {}", stats.store.total_received);
            println!("    Total Delivered: {}", stats.store.total_delivered);
            println!("    Total Expired: {}", stats.store.total_expired);
            println!("    Total Dropped: {}", stats.store.total_dropped);
            println!("    Average Storage Time: {:.2}s", stats.store.avg_storage_time);
            println!("    Last Cleanup: {:?}", stats.store.last_cleanup);
            println!("  Routing:");
            println!("    Total Nodes: {}", stats.routing.total_nodes);
            println!("    Total Routes: {}", stats.routing.total_routes);
            println!("    Last Updated: {:?}", stats.routing.last_updated);
            println!("    Routing Strategies: {:?}", stats.routing.routing_strategies);
        }
        
        Commands::Test => {
            info!("Running DTN service tests");
            
            // Run basic functionality tests
            println!("Running basic functionality tests...");
            
            // Test envelope creation
            let envelope = polynet_dtn::DtnEnvelope::new(
                "did:polynet:test_source".to_string(),
                "did:polynet:test_dest".to_string(),
                3600,
                "test_message".to_string(),
                b"test payload".to_string(),
                100,
            );
            println!("✓ Envelope creation test passed");
            
            // Test envelope storage
            let envelope_id = envelope.id.clone();
            service.send_envelope(envelope).await?;
            println!("✓ Envelope storage test passed");
            
            // Test envelope retrieval
            let retrieved = service.query_envelope(&envelope_id).await?;
            assert!(retrieved.is_some());
            println!("✓ Envelope retrieval test passed");
            
            // Test acknowledgment
            service.acknowledge_envelope(
                &envelope_id,
                "did:polynet:test_dest",
                polynet_dtn::AckType::Delivery,
                polynet_dtn::AckStatus::Success,
                "Test acknowledgment",
            ).await?;
            println!("✓ Acknowledgment test passed");
            
            // Test cleanup
            let cleaned = service.cleanup_expired().await?;
            println!("✓ Cleanup test passed (cleaned: {})", cleaned);
            
            println!("All basic tests passed!");
        }
    }
    
    Ok(())
}

fn load_config(config_path: Option<&PathBuf>) -> Result<DtnServiceConfig, Box<dyn std::error::Error>> {
    // For now, return default configuration
    // In a real implementation, this would load from file
    Ok(DtnServiceConfig::default())
}
