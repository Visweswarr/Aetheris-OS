use clap::{Parser, Subcommand};
use polymera_identity::{IdentityService, KeyType, KeyPurpose};
use std::path::PathBuf;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "polymera-identity")]
#[command(about = "DID Identity Service for Polymera OS")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: Level,
    
    /// Data directory
    #[arg(short, long, default_value = "data/identity")]
    data_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new DID
    Create {
        /// DID method (e.g., polynet)
        #[arg(short, long)]
        method: String,
        
        /// Key types to generate
        #[arg(short, long, value_delimiter = ',', default_value = "ed25519")]
        key_types: Vec<String>,
        
        /// Key purposes
        #[arg(short, long, value_delimiter = ',', default_value = "authentication")]
        purposes: Vec<String>,
    },
    
    /// Resolve a DID
    Resolve {
        /// DID to resolve
        #[arg(short, long)]
        did: String,
    },
    
    /// Rotate keys for a DID
    Rotate {
        /// DID to rotate keys for
        #[arg(short, long)]
        did: String,
        
        /// Key IDs to rotate
        #[arg(short, long, value_delimiter = ',')]
        key_ids: Vec<String>,
        
        /// New key types
        #[arg(short, long, value_delimiter = ',')]
        new_key_types: Vec<String>,
        
        /// New key purposes
        #[arg(short, long, value_delimiter = ',')]
        new_purposes: Vec<String>,
    },
    
    /// List all DIDs
    List,
    
    /// Get DID keys
    Keys {
        /// DID to get keys for
        #[arg(short, long)]
        did: String,
    },
    
    /// Deactivate a DID
    Deactivate {
        /// DID to deactivate
        #[arg(short, long)]
        did: String,
        
        /// Reason for deactivation
        #[arg(short, long)]
        reason: Option<String>,
    },
    
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
    
    info!("Starting DID Identity Service");
    
    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&cli.data_dir)?;
    
    // Create identity service
    let service = IdentityService::new(cli.data_dir).await?;
    
    match cli.command {
        Commands::Create { method, key_types, purposes } => {
            info!("Creating new DID with method: {}, key types: {:?}, purposes: {:?}", 
                  method, key_types, purposes);
            
            // Parse key types
            let parsed_key_types: Vec<KeyType> = key_types.iter()
                .map(|kt| match kt.as_str() {
                    "ed25519" => KeyType::Ed25519,
                    "dilithium3" => KeyType::Dilithium3,
                    "dilithium5" => KeyType::Dilithium5,
                    "kyber512" => KeyType::Kyber512,
                    "kyber768" => KeyType::Kyber768,
                    "kyber1024" => KeyType::Kyber1024,
                    "ed25519-dilithium3" => KeyType::Ed25519Dilithium3,
                    "ed25519-kyber512" => KeyType::Ed25519Kyber512,
                    _ => {
                        eprintln!("Unknown key type: {}", kt);
                        std::process::exit(1);
                    }
                })
                .collect();
            
            // Parse purposes
            let parsed_purposes: Vec<KeyPurpose> = purposes.iter()
                .map(|p| match p.as_str() {
                    "authentication" => KeyPurpose::Authentication,
                    "assertion" => KeyPurpose::Assertion,
                    "key-agreement" => KeyPurpose::KeyAgreement,
                    "key-encapsulation" => KeyPurpose::KeyEncapsulation,
                    "capability-invocation" => KeyPurpose::CapabilityInvocation,
                    "capability-delegation" => KeyPurpose::CapabilityDelegation,
                    _ => {
                        eprintln!("Unknown key purpose: {}", p);
                        std::process::exit(1);
                    }
                })
                .collect();
            
            // Create DID
            let (did, doc) = service.create_did(&method, parsed_key_types, parsed_purposes).await?;
            
            println!("Created DID: {}", did);
            println!("Version ID: {}", doc.version_id);
            println!("Verification Methods: {}", doc.verification_methods.len());
            println!("Authentication Keys: {}", doc.authentication.len());
            println!("Assertion Keys: {}", doc.assertion_method.len());
            println!("Key Agreement Keys: {}", doc.key_agreement.len());
        }
        
        Commands::Resolve { did } => {
            info!("Resolving DID: {}", did);
            
            if let Some(doc) = service.resolve_did(&did).await? {
                println!("DID Document:");
                println!("  ID: {}", doc.id);
                println!("  Version: {}", doc.version_id);
                println!("  Created: {}", doc.created);
                println!("  Updated: {}", doc.updated);
                println!("  Verification Methods: {}", doc.verification_methods.len());
                println!("  Authentication: {}", doc.authentication.len());
                println!("  Assertion: {}", doc.assertion_method.len());
                println!("  Key Agreement: {}", doc.key_agreement.len());
                println!("  Key Encapsulation: {}", doc.key_encapsulation.len());
                println!("  Capability Invocation: {}", doc.capability_invocation.len());
                println!("  Capability Delegation: {}", doc.capability_delegation.len());
                
                if !doc.verification_methods.is_empty() {
                    println!("  Verification Methods:");
                    for vm in &doc.verification_methods {
                        println!("    {}: {} ({})", vm.id, vm.key_type, vm.controller);
                    }
                }
            } else {
                println!("DID not found: {}", did);
            }
        }
        
        Commands::Rotate { did, key_ids, new_key_types, new_purposes } => {
            info!("Rotating keys for DID: {}", did);
            
            // Parse new key types
            let parsed_new_key_types: Vec<KeyType> = new_key_types.iter()
                .map(|kt| match kt.as_str() {
                    "ed25519" => KeyType::Ed25519,
                    "dilithium3" => KeyType::Dilithium3,
                    "dilithium5" => KeyType::Dilithium5,
                    "kyber512" => KeyType::Kyber512,
                    "kyber768" => KeyType::Kyber768,
                    "kyber1024" => KeyType::Kyber1024,
                    "ed25519-dilithium3" => KeyType::Ed25519Dilithium3,
                    "ed25519-kyber512" => KeyType::Ed25519Kyber512,
                    _ => {
                        eprintln!("Unknown key type: {}", kt);
                        std::process::exit(1);
                    }
                })
                .collect();
            
            // Parse new purposes
            let parsed_new_purposes: Vec<KeyPurpose> = new_purposes.iter()
                .map(|p| match p.as_str() {
                    "authentication" => KeyPurpose::Authentication,
                    "assertion" => KeyPurpose::Assertion,
                    "key-agreement" => KeyPurpose::KeyAgreement,
                    "key-encapsulation" => KeyPurpose::KeyEncapsulation,
                    "capability-invocation" => KeyPurpose::CapabilityInvocation,
                    "capability-delegation" => KeyPurpose::CapabilityDelegation,
                    _ => {
                        eprintln!("Unknown key purpose: {}", p);
                        std::process::exit(1);
                    }
                })
                .collect();
            
            // Rotate keys
            let updated_doc = service.rotate_keys(&did, key_ids, parsed_new_key_types, parsed_new_purposes).await?;
            
            println!("Successfully rotated keys for DID: {}", did);
            println!("New Version ID: {}", updated_doc.version_id);
            println!("Updated: {}", updated_doc.updated);
            println!("Verification Methods: {}", updated_doc.verification_methods.len());
        }
        
        Commands::List => {
            info!("Listing all DIDs");
            
            let dids = service.list_dids().await?;
            
            if dids.is_empty() {
                println!("No DIDs found");
            } else {
                println!("Found {} DIDs:", dids.len());
                for (i, did) in dids.iter().enumerate() {
                    println!("  {}. {}", i + 1, did);
                }
            }
        }
        
        Commands::Keys { did } => {
            info!("Getting keys for DID: {}", did);
            
            let keys = service.get_did_keys(&did).await?;
            
            if keys.is_empty() {
                println!("No keys found for DID: {}", did);
            } else {
                println!("Found {} keys for DID {}:", keys.len(), did);
                for (i, key) in keys.iter().enumerate() {
                    println!("  {}. Key ID: {}", i + 1, key.id);
                    println!("     Type: {:?}", key.key_type);
                    println!("     Created: {}", key.created);
                    println!("     Revoked: {}", key.revoked);
                    println!("     Purposes: {:?}", key.purpose);
                    println!();
                }
            }
        }
        
        Commands::Deactivate { did, reason } => {
            info!("Deactivating DID: {} (reason: {:?})", did, reason);
            
            service.deactivate_did(&did, reason).await?;
            
            println!("Successfully deactivated DID: {}", did);
        }
        
        Commands::Stats => {
            info!("Getting service statistics");
            
            let stats = service.get_stats().await?;
            println!("DID Identity Service Statistics:");
            println!("  Total DIDs: {}", stats.store.total_dids);
            println!("  Total Keys: {}", stats.store.total_keys);
            println!("  Total Documents: {}", stats.store.total_documents);
            println!("  Last Cleanup: {:?}", stats.store.last_cleanup);
        }
        
        Commands::Test => {
            info!("Running DID identity service tests");
            
            // Run basic functionality tests
            println!("Running basic functionality tests...");
            
            // Test DID creation
            let (did, doc) = service.create_did(
                "polynet",
                vec![KeyType::Ed25519],
                vec![KeyPurpose::Authentication],
            ).await?;
            println!("✓ DID creation test passed: {}", did);
            
            // Test DID resolution
            let resolved = service.resolve_did(&did).await?;
            assert!(resolved.is_some());
            println!("✓ DID resolution test passed");
            
            // Test key retrieval
            let keys = service.get_did_keys(&did).await?;
            assert_eq!(keys.len(), 1);
            println!("✓ Key retrieval test passed");
            
            // Test key rotation
            let key_ids: Vec<String> = keys.iter().map(|k| k.id.clone()).collect();
            let updated_doc = service.rotate_keys(
                &did,
                key_ids,
                vec![KeyType::Dilithium3],
                vec![KeyPurpose::Authentication],
            ).await?;
            assert_ne!(updated_doc.version_id, doc.version_id);
            println!("✓ Key rotation test passed");
            
            // Test DID deactivation
            service.deactivate_did(&did, Some("Testing deactivation".to_string())).await?;
            println!("✓ DID deactivation test passed");
            
            println!("All basic tests passed!");
        }
    }
    
    Ok(())
}
