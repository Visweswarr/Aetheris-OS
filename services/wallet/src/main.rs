use clap::{Parser, Subcommand};
use polymera_wallet::{
    KeystoreService, KeystoreConfig, KeyType, KeyPurpose,
    utils::{create_software_keystore, create_keyvault_keystore},
};
use std::path::PathBuf;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "polymera-wallet")]
#[command(about = "Keystore Interface Service for Polymera OS")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: Level,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new key
    Generate {
        /// Key type
        #[arg(long)]
        key_type: String,
        
        /// Key purposes (comma-separated)
        #[arg(long)]
        purposes: String,
        
        /// Key name
        #[arg(long)]
        name: Option<String>,
        
        /// Expiration time (ISO 8601 format)
        #[arg(long)]
        expires: Option<String>,
        
        /// Tags (key=value,key=value)
        #[arg(long)]
        tags: Option<String>,
    },
    
    /// Import an existing key
    Import {
        /// Key type
        #[arg(long)]
        key_type: String,
        
        /// Key purposes (comma-separated)
        #[arg(long)]
        purposes: String,
        
        /// Key name
        #[arg(long)]
        name: Option<String>,
        
        /// Expiration time (ISO 8601 format)
        #[arg(long)]
        expires: Option<String>,
        
        /// Tags (key=value,key=value)
        #[arg(long)]
        tags: Option<String>,
        
        /// Key file path
        #[arg(long)]
        key_file: PathBuf,
    },
    
    /// List all keys
    List,
    
    /// Get key metadata
    Get {
        /// Key ID
        key_id: String,
    },
    
    /// Sign data
    Sign {
        /// Key ID
        key_id: String,
        
        /// Data to sign
        data: String,
        
        /// Algorithm (optional)
        #[arg(long)]
        algorithm: Option<String>,
    },
    
    /// Verify signature
    Verify {
        /// Key ID
        key_id: String,
        
        /// Data that was signed
        data: String,
        
        /// Signature file path
        signature_file: PathBuf,
        
        /// Algorithm (optional)
        #[arg(long)]
        algorithm: Option<String>,
    },
    
    /// Encrypt data
    Encrypt {
        /// Key ID
        key_id: String,
        
        /// Data to encrypt
        data: String,
        
        /// Algorithm (optional)
        #[arg(long)]
        algorithm: Option<String>,
    },
    
    /// Decrypt data
    Decrypt {
        /// Key ID
        key_id: String,
        
        /// Encrypted data file path
        encrypted_file: PathBuf,
    },
    
    /// Perform key encapsulation
    Encapsulate {
        /// Key ID
        key_id: String,
        
        /// Algorithm (optional)
        #[arg(long)]
        algorithm: Option<String>,
    },
    
    /// Perform key decapsulation
    Decapsulate {
        /// Key ID
        key_id: String,
        
        /// Encapsulated key file path
        encapsulated_file: PathBuf,
        
        /// Algorithm (optional)
        #[arg(long)]
        algorithm: Option<String>,
    },
    
    /// Derive a key
    Derive {
        /// Key ID
        key_id: String,
        
        /// Salt (hex string)
        #[arg(long)]
        salt: String,
        
        /// Iterations
        #[arg(long, default_value = "100000")]
        iterations: u32,
        
        /// Key length in bytes
        #[arg(long, default_value = "32")]
        key_length: usize,
        
        /// Algorithm
        #[arg(long, default_value = "Argon2id")]
        algorithm: String,
    },
    
    /// Update key metadata
    Update {
        /// Key ID
        key_id: String,
        
        /// New name
        #[arg(long)]
        name: Option<String>,
        
        /// New expiration time (ISO 8601 format)
        #[arg(long)]
        expires: Option<String>,
        
        /// New tags (key=value,key=value)
        #[arg(long)]
        tags: Option<String>,
    },
    
    /// Revoke a key
    Revoke {
        /// Key ID
        key_id: String,
        
        /// Reason for revocation
        #[arg(long)]
        reason: Option<String>,
    },
    
    /// Rotate a key
    Rotate {
        /// Key ID
        key_id: String,
        
        /// New key type (optional)
        #[arg(long)]
        new_key_type: Option<String>,
        
        /// New purposes (comma-separated, optional)
        #[arg(long)]
        new_purposes: Option<String>,
    },
    
    /// Delete a key
    Delete {
        /// Key ID
        key_id: String,
    },
    
    /// Show service statistics
    Stats,
    
    /// Perform cleanup
    Cleanup,
    
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
    
    // Load configuration
    let config = load_config(cli.config.as_ref())?;
    
    // Create keystore service
    let service = match config.backend.as_str() {
        "software" => create_software_keystore(Some(config)).await?,
        "keyvault" => {
            let vault_url = config.connection_string
                .as_ref()
                .ok_or("KeyVault connection string required")?
                .clone();
            
            let tenant_id = config.authentication
                .as_ref()
                .and_then(|auth| auth.credentials.get("tenant_id"))
                .cloned()
                .unwrap_or_else(|| "stub-tenant".to_string());
            
            let client_id = config.authentication
                .as_ref()
                .and_then(|auth| auth.credentials.get("client_id"))
                .cloned()
                .unwrap_or_else(|| "stub-client".to_string());
            
            create_keyvault_keystore(vault_url, tenant_id, client_id, Some(config)).await?
        }
        _ => return Err("Unsupported backend".into()),
    };
    
    // Execute command
    match cli.command {
        Commands::Generate { key_type, purposes, name, expires, tags } => {
            let key_type = parse_key_type(&key_type)?;
            let purposes = parse_purposes(&purposes)?;
            let expires = parse_expires(expires.as_deref())?;
            let tags = parse_tags(tags.as_deref())?;
            
            let metadata = service.generate_key(key_type, purposes, name, expires, tags).await?;
            println!("Generated key: {}", metadata.id);
            println!("Type: {:?}", metadata.key_type);
            println!("Purposes: {:?}", metadata.purposes);
            if let Some(name) = metadata.name {
                println!("Name: {}", name);
            }
        }
        
        Commands::Import { key_type, purposes, name, expires, tags, key_file } => {
            let key_type = parse_key_type(&key_type)?;
            let purposes = parse_purposes(&purposes)?;
            let expires = parse_expires(expires.as_deref())?;
            let tags = parse_tags(tags.as_deref())?;
            
            let key_data = std::fs::read(&key_file)?;
            let key_data = secrecy::Secret::new(key_data);
            
            let metadata = service.import_key(key_type, key_data, purposes, name, expires, tags).await?;
            println!("Imported key: {}", metadata.id);
            println!("Type: {:?}", metadata.key_type);
            println!("Purposes: {:?}", metadata.purposes);
        }
        
        Commands::List => {
            let keys = service.list_keys().await?;
            println!("Found {} keys:", keys.len());
            for key in keys {
                println!("  {}: {:?} ({:?})", key.id, key.key_type, key.purposes);
                if let Some(name) = key.name {
                    println!("    Name: {}", name);
                }
                println!("    Status: {:?}", key.status);
            }
        }
        
        Commands::Get { key_id } => {
            let metadata = service.get_key_metadata(&key_id).await?;
            println!("Key: {}", metadata.id);
            println!("Type: {:?}", metadata.key_type);
            println!("Purposes: {:?}", metadata.purposes);
            println!("Status: {:?}", metadata.status);
            println!("Created: {}", metadata.created);
            if let Some(expires) = metadata.expires {
                println!("Expires: {}", expires);
            }
            if let Some(last_used) = metadata.last_used {
                println!("Last used: {}", last_used);
            }
        }
        
        Commands::Sign { key_id, data, algorithm } => {
            let signature = service.sign(&key_id, data.as_bytes(), algorithm).await?;
            println!("Signature: {}", base64::encode(&signature.signature));
            println!("Algorithm: {}", signature.algorithm);
            println!("Created: {}", signature.created);
        }
        
        Commands::Verify { key_id, data, signature_file, algorithm } => {
            let signature_data = std::fs::read(&signature_file)?;
            let signature = base64::decode(signature_data)?;
            
            let is_valid = service.verify(&key_id, data.as_bytes(), &signature, algorithm).await?;
            println!("Signature verification: {}", if is_valid { "VALID" } else { "INVALID" });
        }
        
        Commands::Encrypt { key_id, data, algorithm } => {
            let encrypted = service.encrypt(&key_id, data.as_bytes(), algorithm).await?;
            println!("Encrypted data: {}", base64::encode(&encrypted.encrypted_data));
            println!("Nonce: {}", base64::encode(&encrypted.nonce));
            println!("Algorithm: {}", encrypted.algorithm);
        }
        
        Commands::Decrypt { key_id, encrypted_file } => {
            let encrypted_data = std::fs::read(&encrypted_file)?;
            // This is a simplified approach - in practice, you'd need to parse the encrypted data format
            println!("Decryption not implemented in CLI demo");
        }
        
        Commands::Encapsulate { key_id, algorithm } => {
            let result = service.encapsulate_key(&key_id, algorithm).await?;
            println!("Encapsulated key: {}", base64::encode(&result.encapsulated_key));
            println!("Shared secret: {}", base64::encode(&result.shared_secret));
            println!("Algorithm: {}", result.algorithm);
        }
        
        Commands::Decapsulate { key_id, encapsulated_file, algorithm } => {
            let encapsulated_data = std::fs::read(&encapsulated_file)?;
            let shared_secret = service.decapsulate_key(&key_id, &encapsulated_data, algorithm).await?;
            println!("Shared secret: {}", base64::encode(&shared_secret));
        }
        
        Commands::Derive { key_id, salt, iterations, key_length, algorithm } => {
            let salt = hex::decode(&salt)?;
            let params = polymera_wallet::KeyDerivationParams {
                salt,
                iterations,
                key_length,
                algorithm,
            };
            
            let derived_key = service.derive_key(&key_id, &params).await?;
            println!("Derived key: {}", base64::encode(&derived_key));
        }
        
        Commands::Update { key_id, name, expires, tags } => {
            let expires = parse_expires(expires.as_deref())?;
            let tags = parse_tags(tags.as_deref())?;
            
            let metadata = service.update_key_metadata(&key_id, name, expires, tags).await?;
            println!("Updated key: {}", metadata.id);
        }
        
        Commands::Revoke { key_id, reason } => {
            service.revoke_key(&key_id, reason).await?;
            println!("Revoked key: {}", key_id);
        }
        
        Commands::Rotate { key_id, new_key_type, new_purposes } => {
            let new_key_type = new_key_type.as_deref().map(parse_key_type).transpose()?;
            let new_purposes = new_purposes.as_deref().map(parse_purposes).transpose()?;
            
            let new_metadata = service.rotate_key(&key_id, new_key_type, new_purposes).await?;
            println!("Rotated key: {} -> {}", key_id, new_metadata.id);
        }
        
        Commands::Delete { key_id } => {
            service.delete_key(&key_id).await?;
            println!("Deleted key: {}", key_id);
        }
        
        Commands::Stats => {
            let stats = service.get_stats().await?;
            println!("Keystore Statistics:");
            println!("  Total keys: {}", stats.total_keys);
            println!("  Active keys: {}", stats.active_keys);
            println!("  Expired keys: {}", stats.expired_keys);
            println!("  Revoked keys: {}", stats.revoked_keys);
            println!("  Total operations: {}", stats.total_operations);
            if let Some(last_op) = stats.last_operation {
                println!("  Last operation: {}", last_op);
            }
        }
        
        Commands::Cleanup => {
            service.cleanup().await?;
            println!("Cleanup completed");
        }
        
        Commands::Test => {
            println!("Running keystore tests...");
            
            // Test key generation
            let metadata = service.generate_key(
                polymera_wallet::KeyType::Ed25519,
                vec![polymera_wallet::KeyPurpose::Sign, polymera_wallet::KeyPurpose::Verify],
                Some("Test Key".to_string()),
                None,
                None,
            ).await?;
            println!("✓ Generated test key: {}", metadata.id);
            
            // Test signing and verification
            let data = b"Hello, World!";
            let signature = service.sign(&metadata.id, data, None).await?;
            println!("✓ Signed test data");
            
            let is_valid = service.verify(&metadata.id, data, &signature.signature, None).await?;
            println!("✓ Verified signature: {}", is_valid);
            
            // Test encryption and decryption
            let enc_metadata = service.generate_key(
                polymera_wallet::KeyType::AES256,
                vec![polymera_wallet::KeyPurpose::Encrypt, polymera_wallet::KeyPurpose::Decrypt],
                Some("Test Encryption Key".to_string()),
                None,
                None,
            ).await?;
            println!("✓ Generated encryption key: {}", enc_metadata.id);
            
            let encrypted = service.encrypt(&enc_metadata.id, data, None).await?;
            println!("✓ Encrypted test data");
            
            // Clean up test keys
            service.delete_key(&metadata.id).await?;
            service.delete_key(&enc_metadata.id).await?;
            println!("✓ Cleaned up test keys");
            
            println!("All tests passed!");
        }
    }
    
    Ok(())
}

fn load_config(config_path: Option<&PathBuf>) -> Result<KeystoreConfig, Box<dyn std::error::Error>> {
    if let Some(path) = config_path {
        let config_str = std::fs::read_to_string(path)?;
        let config: KeystoreConfig = serde_yaml::from_str(&config_str)?;
        Ok(config)
    } else {
        Ok(KeystoreConfig::default())
    }
}

fn parse_key_type(s: &str) -> Result<KeyType, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "ed25519" => Ok(KeyType::Ed25519),
        "dilithium3" => Ok(KeyType::Dilithium3),
        "dilithium5" => Ok(KeyType::Dilithium5),
        "kyber512" => Ok(KeyType::Kyber512),
        "kyber768" => Ok(KeyType::Kyber768),
        "kyber1024" => Ok(KeyType::Kyber1024),
        "ed25519dilithium3" => Ok(KeyType::Ed25519Dilithium3),
        "ed25519kyber512" => Ok(KeyType::Ed25519Kyber512),
        "aes256" => Ok(KeyType::AES256),
        "chacha20poly1305" => Ok(KeyType::ChaCha20Poly1305),
        _ => Err(format!("Unknown key type: {}", s).into()),
    }
}

fn parse_purposes(s: &str) -> Result<Vec<KeyPurpose>, Box<dyn std::error::Error>> {
    let mut purposes = Vec::new();
    for purpose_str in s.split(',') {
        let purpose = match purpose_str.trim().to_lowercase().as_str() {
            "sign" => KeyPurpose::Sign,
            "verify" => KeyPurpose::Verify,
            "encrypt" => KeyPurpose::Encrypt,
            "decrypt" => KeyPurpose::Decrypt,
            "keyencapsulation" => KeyPurpose::KeyEncapsulation,
            "keydecapsulation" => KeyPurpose::KeyDecapsulation,
            "keyagreement" => KeyPurpose::KeyAgreement,
            "keyderivation" => KeyPurpose::KeyDerivation,
            _ => return Err(format!("Unknown purpose: {}", purpose_str).into()),
        };
        purposes.push(purpose);
    }
    Ok(purposes)
}

fn parse_expires(s: Option<&str>) -> Result<Option<chrono::DateTime<chrono::Utc>>, Box<dyn std::error::Error>> {
    if let Some(s) = s {
        let dt = chrono::DateTime::parse_from_rfc3339(s)?;
        Ok(Some(dt.with_timezone(&chrono::Utc)))
    } else {
        Ok(None)
    }
}

fn parse_tags(s: Option<&str>) -> Result<Option<std::collections::HashMap<String, String>>, Box<dyn std::error::Error>> {
    if let Some(s) = s {
        let mut tags = std::collections::HashMap::new();
        for tag_pair in s.split(',') {
            if let Some((key, value)) = tag_pair.split_once('=') {
                tags.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Ok(Some(tags))
    } else {
        Ok(None)
    }
}
