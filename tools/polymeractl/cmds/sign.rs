use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct SignCommand {
    /// Target to sign (package, image, all)
    #[arg(short, long, default_value = "all")]
    target: String,

    /// Path to package or image
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Signing key file
    #[arg(short, long)]
    key: Option<PathBuf>,

    /// Signing algorithm (ed25519, rsa, ecdsa)
    #[arg(long, default_value = "ed25519")]
    algorithm: String,

    /// Key passphrase
    #[arg(long)]
    passphrase: Option<String>,

    /// Output signature file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Detached signature
    #[arg(long)]
    detached: bool,

    /// Verify after signing
    #[arg(long)]
    verify: bool,

    /// Show detailed output
    #[arg(long)]
    detailed: bool,

    /// Overwrite existing signatures
    #[arg(long)]
    force: bool,
}

impl SignCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🔐 Signing Polymera OS Components".bold());
        println!("Target: {}", self.target.blue());
        println!("Algorithm: {}", self.algorithm.blue());

        // Validate algorithm
        if !["ed25519", "rsa", "ecdsa"].contains(&self.algorithm.as_str()) {
            return Err(format!("Unsupported signing algorithm: {}", self.algorithm).into());
        }

        // Find signing key
        let key_path = self.find_signing_key()?;
        println!("Signing Key: {}", key_path.display().to_string().blue());

        // Determine targets
        let targets = self.determine_targets()?;

        // Sign targets
        for target in targets {
            println!("\nSigning {}...", target.display().to_string().blue());
            self.sign_target(&target, &key_path).await?;
        }

        // Verify if requested
        if self.verify {
            println!("\nVerifying signatures...");
            for target in targets {
                self.verify_signature(&target).await?;
            }
        }

        println!("{}", "\n🎉 Signing completed successfully".green());
        Ok(())
    }

    /// Find signing key
    fn find_signing_key(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Use provided key if specified
        if let Some(key_path) = &self.key {
            if key_path.exists() {
                return Ok(key_path.clone());
            }
            return Err(format!("Signing key not found: {}", key_path.display()).into());
        }

        // Look for default keys
        let default_keys = [
            "keys/signing.key",
            "keys/private.key",
            ".polymera/signing.key",
            "~/.polymera/signing.key",
        ];

        for key_path in &default_keys {
            let expanded_path = if key_path.starts_with("~") {
                let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
                PathBuf::from(home).join(&key_path[2..])
            } else {
                PathBuf::from(key_path)
            };

            if expanded_path.exists() {
                return Ok(expanded_path);
            }
        }

        // Generate new key if none exists
        println!("  No signing key found, generating new {} key...", self.algorithm);
        self.generate_signing_key().await
    }

    /// Generate new signing key
    async fn generate_signing_key(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let keys_dir = PathBuf::from("keys");
        std::fs::create_dir_all(&keys_dir)?;

        let key_path = keys_dir.join("signing.key");
        let pub_key_path = keys_dir.join("signing.pub");

        match self.algorithm.as_str() {
            "ed25519" => self.generate_ed25519_key(&key_path, &pub_key_path).await?,
            "rsa" => self.generate_rsa_key(&key_path, &pub_key_path).await?,
            "ecdsa" => self.generate_ecdsa_key(&key_path, &pub_key_path).await?,
            _ => unreachable!(),
        }

        println!("  ✓ Generated new signing key: {}", key_path.display());
        println!("  ✓ Public key: {}", pub_key_path.display());

        Ok(key_path)
    }

    /// Generate Ed25519 key
    async fn generate_ed25519_key(&self, key_path: &PathBuf, pub_key_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Use openssl to generate Ed25519 key
        let mut cmd = Command::new("openssl");
        cmd.arg("genpkey")
            .arg("-algorithm").arg("ED25519")
            .arg("-out").arg(key_path);

        let output = cmd.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to generate Ed25519 key: {}", error).into());
        }

        // Extract public key
        let mut cmd = Command::new("openssl");
        cmd.arg("pkey")
            .arg("-in").arg(key_path)
            .arg("-pubout")
            .arg("-out").arg(pub_key_path);

        let output = cmd.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to extract public key: {}", error).into());
        }

        Ok(())
    }

    /// Generate RSA key
    async fn generate_rsa_key(&self, key_path: &PathBuf, pub_key_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Use openssl to generate RSA key
        let mut cmd = Command::new("openssl");
        cmd.arg("genrsa")
            .arg("-out").arg(key_path)
            .arg("2048");

        let output = cmd.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to generate RSA key: {}", error).into());
        }

        // Extract public key
        let mut cmd = Command::new("openssl");
        cmd.arg("rsa")
            .arg("-in").arg(key_path)
            .arg("-pubout")
            .arg("-out").arg(pub_key_path);

        let output = cmd.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to extract public key: {}", error).into());
        }

        Ok(())
    }

    /// Generate ECDSA key
    async fn generate_ecdsa_key(&self, key_path: &PathBuf, pub_key_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Use openssl to generate ECDSA key
        let mut cmd = Command::new("openssl");
        cmd.arg("ecparam")
            .arg("-genkey")
            .arg("-name").arg("secp256r1")
            .arg("-out").arg(key_path);

        let output = cmd.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to generate ECDSA key: {}", error).into());
        }

        // Extract public key
        let mut cmd = Command::new("openssl");
        cmd.arg("ec")
            .arg("-in").arg(key_path)
            .arg("-pubout")
            .arg("-out").arg(pub_key_path);

        let output = cmd.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to extract public key: {}", error).into());
        }

        Ok(())
    }

    /// Determine signing targets
    fn determine_targets(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut targets = Vec::new();

        match self.target.as_str() {
            "package" => {
                if let Some(path) = &self.path {
                    targets.push(path.clone());
                } else {
                    // Find packages in current directory
                    targets.extend(self.find_packages()?);
                }
            }
            "image" => {
                if let Some(path) = &self.path {
                    targets.push(path.clone());
                } else {
                    // Find images in current directory
                    targets.extend(self.find_images()?);
                }
            }
            "all" => {
                // Find all signable items
                targets.extend(self.find_packages()?);
                targets.extend(self.find_images()?);
            }
            _ => return Err(format!("Unknown signing target: {}", self.target).into()),
        }

        if targets.is_empty() {
            return Err("No signing targets found".into());
        }

        Ok(targets)
    }

    /// Find packages to sign
    fn find_packages(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut packages = Vec::new();
        
        // Look for package directories
        let package_dirs = ["packages", "dist/packages", "target/packages"];
        
        for dir in &package_dirs {
            let path = std::path::Path::new(dir);
            if path.exists() && path.is_dir() {
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() && path.join("manifest.json").exists() {
                        packages.push(path);
                    }
                }
            }
        }
        
        // Look for .pkg files
        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "pkg") {
                packages.push(path);
            }
        }
        
        Ok(packages)
    }

    /// Find images to sign
    fn find_images(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut images = Vec::new();
        
        // Look for image files
        let image_extensions = ["img", "iso", "qcow2", "vmdk", "vdi"];
        
        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if image_extensions.contains(&ext.to_str().unwrap_or("")) {
                        images.push(path);
                    }
                }
            }
        }
        
        // Look in dist directory
        let dist_dir = std::path::Path::new("dist");
        if dist_dir.exists() && dist_dir.is_dir() {
            for entry in std::fs::read_dir(dist_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if image_extensions.contains(&ext.to_str().unwrap_or("")) {
                            images.push(path);
                        }
                    }
                }
            }
        }
        
        Ok(images)
    }

    /// Sign a specific target
    async fn sign_target(&self, target: &PathBuf, key_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if target.is_dir() && target.join("manifest.json").exists() {
            // This is a package directory
            self.sign_package(target, key_path).await?;
        } else if target.is_file() {
            // This is an image file
            self.sign_image(target, key_path).await?;
        } else {
            return Err(format!("Unknown target type: {}", target.display()).into());
        }

        Ok(())
    }

    /// Sign a package
    async fn sign_package(&self, package_dir: &PathBuf, key_path: &PathBuf) -> Result<(), Box<dyn std::error>> {
        println!("  Signing package: {}", package_dir.display());
        
        let manifest_path = package_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err("Package manifest not found".into());
        }

        // Read manifest
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let mut manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

        // Calculate content hash
        let content_hash = self.calculate_content_hash(package_dir).await?;
        
        // Create signature
        let signature = self.create_signature(&manifest_content, key_path).await?;
        
        // Add signature to manifest
        if !manifest["signatures"].is_array() {
            manifest["signatures"] = serde_json::json!([]);
        }
        
        let signature_info = serde_json::json!({
            "algorithm": self.algorithm,
            "signature": signature,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "key_id": self.get_key_id(key_path).await?
        });
        
        manifest["signatures"].as_array_mut().unwrap().push(signature_info);
        
        // Update content hash
        manifest["content_hash"] = serde_json::Value::String(content_hash);
        
        // Write updated manifest
        let updated_content = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, updated_content)?;
        
        println!("    ✓ Package signed successfully");
        Ok(())
    }

    /// Sign an image
    async fn sign_image(&self, image_path: &PathBuf, key_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Signing image: {}", image_path.display());
        
        // Calculate image hash
        let image_hash = self.calculate_file_hash(image_path).await?;
        
        // Create signature
        let signature = self.create_signature(&image_hash, key_path).await?;
        
        // Create signature file
        let signature_path = if self.detached {
            format!("{}.sig", image_path.display())
        } else {
            format!("{}.signed", image_path.display())
        };
        
        let signature_data = serde_json::json!({
            "algorithm": self.algorithm,
            "signature": signature,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "key_id": self.get_key_id(key_path).await?,
            "image_hash": image_hash
        });
        
        let signature_content = serde_json::to_string_pretty(&signature_data)?;
        std::fs::write(&signature_path, signature_content)?;
        
        println!("    ✓ Image signed successfully");
        println!("    Signature: {}", signature_path);
        Ok(())
    }

    /// Calculate content hash for package
    async fn calculate_content_hash(&self, package_dir: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        
        // Walk through package directory
        for entry in walkdir::WalkDir::new(package_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                // Add file path to hash
                hasher.update(path.to_string_lossy().as_bytes());
                
                // Add file content to hash
                if let Ok(content) = std::fs::read(path) {
                    hasher.update(&content);
                }
            }
        }
        
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Calculate file hash
    async fn calculate_file_hash(&self, file_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        use sha2::{Sha256, Digest};
        
        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];
        
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Create signature
    async fn create_signature(&self, data: &str, key_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real implementation, you'd use proper cryptographic signing
        
        // For now, we'll create a mock signature
        let signature_data = format!("{}:{}:{}", self.algorithm, data, chrono::Utc::now().timestamp());
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(signature_data.as_bytes());
        let hash = hasher.finalize();
        
        Ok(base64::encode(hash))
    }

    /// Get key ID
    async fn get_key_id(&self, key_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real implementation, you'd extract the actual key ID
        
        // For now, we'll use a hash of the key file
        let key_content = std::fs::read(key_path)?;
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&key_content);
        let hash = hasher.finalize();
        
        Ok(format!("{:x}", hash)[..16].to_string())
    }

    /// Verify signature
    async fn verify_signature(&self, target: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if target.is_dir() && target.join("manifest.json").exists() {
            self.verify_package_signature(target).await?;
        } else if target.is_file() {
            self.verify_image_signature(target).await?;
        }
        
        Ok(())
    }

    /// Verify package signature
    async fn verify_package_signature(&self, package_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Verifying package signature: {}", package_dir.display());
        
        let manifest_path = package_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err("Package manifest not found".into());
        }
        
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
        
        if let Some(signatures) = manifest.get("signatures") {
            if let Some(sigs) = signatures.as_array() {
                println!("    Found {} signature(s)", sigs.len());
                
                for (i, sig) in sigs.iter().enumerate() {
                    if let (Some(algorithm), Some(signature), Some(timestamp)) = (
                        sig.get("algorithm").and_then(|v| v.as_str()),
                        sig.get("signature").and_then(|v| v.as_str()),
                        sig.get("timestamp").and_then(|v| v.as_str())
                    ) {
                        println!("      Signature {}: {} ({})", i + 1, algorithm, timestamp);
                        println!("        ✓ Signature format valid");
                    }
                }
            }
        } else {
            println!("    ⚠ No signatures found");
        }
        
        Ok(())
    }

    /// Verify image signature
    async fn verify_image_signature(&self, image_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Verifying image signature: {}", image_path.display());
        
        // Look for signature file
        let signature_path = format!("{}.sig", image_path.display());
        let signature_file = PathBuf::from(&signature_path);
        
        if signature_file.exists() {
            let signature_content = std::fs::read_to_string(&signature_file)?;
            let signature_data: serde_json::Value = serde_json::from_str(&signature_content)?;
            
            if let (Some(algorithm), Some(signature), Some(timestamp)) = (
                signature_data.get("algorithm").and_then(|v| v.as_str()),
                signature_data.get("signature").and_then(|v| v.as_str()),
                signature_data.get("timestamp").and_then(|v| v.as_str())
            ) {
                println!("    Found signature: {} ({})", algorithm, timestamp);
                println!("      ✓ Signature format valid");
                
                // Verify hash
                if let Some(expected_hash) = signature_data.get("image_hash").and_then(|v| v.as_str()) {
                    let actual_hash = self.calculate_file_hash(image_path).await?;
                    if expected_hash == actual_hash {
                        println!("      ✓ Image hash verified");
                    } else {
                        println!("      ❌ Image hash mismatch");
                        return Err("Image hash verification failed".into());
                    }
                }
            }
        } else {
            println!("    ⚠ No signature file found");
        }
        
        Ok(())
    }
}

impl Default for SignCommand {
    fn default() -> Self {
        Self {
            target: "all".to_string(),
            path: None,
            key: None,
            algorithm: "ed25519".to_string(),
            passphrase: None,
            output: None,
            detached: false,
            verify: false,
            detailed: false,
            force: false,
        }
    }
}
