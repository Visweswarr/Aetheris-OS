use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct SbomCommand {
    /// Action to perform (generate, validate, merge, diff)
    #[arg(short, long, default_value = "generate")]
    action: String,

    /// Source directory or file
    #[arg(short, long, default_value = ".")]
    source: PathBuf,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// SBOM format (spdx, cyclonedx, swid)
    #[arg(short, long, default_value = "spdx")]
    format: String,

    /// SBOM version
    #[arg(long)]
    version: Option<String>,

    /// Include dependencies
    #[arg(long)]
    dependencies: bool,

    /// Include licenses
    #[arg(long)]
    licenses: bool,

    /// Include vulnerabilities
    #[arg(long)]
    vulnerabilities: bool,

    /// Validate against schema
    #[arg(long)]
    validate: bool,

    /// Show detailed output
    #[arg(long)]
    detailed: bool,

    /// Output format (json, xml, yaml, text)
    #[arg(long, default_value = "json")]
    output_format: String,
}

impl SbomCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "📊 Managing Software Bill of Materials".bold());
        println!("Action: {}", self.action.blue());
        println!("Source: {}", self.source.display().to_string().blue());
        println!("Format: {}", self.format.blue());

        // Validate action
        if !["generate", "validate", "merge", "diff"].contains(&self.action.as_str()) {
            return Err(format!("Unknown action: {}", self.action).into());
        }

        // Validate format
        if !["spdx", "cyclonedx", "swid"].contains(&self.format.as_str()) {
            return Err(format!("Unsupported SBOM format: {}", self.format).into());
        }

        // Execute action
        match self.action.as_str() {
            "generate" => self.generate_sbom().await?,
            "validate" => self.validate_sbom().await?,
            "merge" => self.merge_sbom().await?,
            "diff" => self.diff_sbom().await?,
            _ => unreachable!(),
        }

        println!("{}", "✅ SBOM operation completed successfully".green());
        Ok(())
    }

    /// Generate SBOM
    async fn generate_sbom(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating SBOM...");

        // Determine output path
        let output_path = self.output.clone().unwrap_or_else(|| {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("sbom_{}.{}", timestamp, self.output_format))
        });

        // Generate SBOM based on format
        match self.format.as_str() {
            "spdx" => self.generate_spdx_sbom(&output_path).await?,
            "cyclonedx" => self.generate_cyclonedx_sbom(&output_path).await?,
            "swid" => self.generate_swid_sbom(&output_path).await?,
            _ => unreachable!(),
        }

        println!("  ✓ SBOM generated successfully");
        println!("  Output: {}", output_path.display().to_string().blue());

        // Validate if requested
        if self.validate {
            self.validate_generated_sbom(&output_path).await?;
        }

        Ok(())
    }

    /// Generate SPDX SBOM
    async fn generate_spdx_sbom(&self, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating SPDX SBOM...");

        let mut sbom = serde_json::json!({
            "SPDXID": "SPDXRef-DOCUMENT",
            "spdxVersion": "SPDX-2.3",
            "name": "Polymera OS SBOM",
            "documentNamespace": format!("https://polymera-os.org/sbom/{}", chrono::Utc::now().timestamp()),
            "creationInfo": {
                "creators": ["Tool: polymeractl"],
                "created": chrono::Utc::now().to_rfc3339()
            },
            "packages": [],
            "relationships": []
        });

        // Scan source directory for components
        let components = self.scan_components().await?;
        
        // Add components to SBOM
        for component in components {
            let package = serde_json::json!({
                "SPDXID": format!("SPDXRef-Package-{}", component.name.replace("-", "_")),
                "name": component.name,
                "versionInfo": component.version,
                "packageFileName": component.path,
                "packageVerificationCode": {
                    "packageVerificationCodeValue": component.checksum
                },
                "licenseConcluded": component.license,
                "licenseDeclared": component.license,
                "copyrightText": component.copyright,
                "description": component.description
            });
            
            sbom["packages"].as_array_mut().unwrap().push(package);
        }

        // Write SBOM to file
        let content = serde_json::to_string_pretty(&sbom)?;
        std::fs::write(output_path, content)?;

        Ok(())
    }

    /// Generate CycloneDX SBOM
    async fn generate_cyclonedx_sbom(&self, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating CycloneDX SBOM...");

        let mut sbom = serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "version": 1,
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "tools": [{
                    "vendor": "Polymera OS",
                    "name": "polymeractl",
                    "version": env!("CARGO_PKG_VERSION")
                }]
            },
            "components": []
        });

        // Scan source directory for components
        let components = self.scan_components().await?;
        
        // Add components to SBOM
        for component in components {
            let comp = serde_json::json!({
                "type": "library",
                "name": component.name,
                "version": component.version,
                "purl": format!("pkg:polymera/{}@{}", component.name, component.version),
                "description": component.description,
                "licenses": [{
                    "license": {
                        "id": component.license
                    }
                }]
            });
            
            sbom["components"].as_array_mut().unwrap().push(comp);
        }

        // Write SBOM to file
        let content = serde_json::to_string_pretty(&sbom)?;
        std::fs::write(output_path, content)?;

        Ok(())
    }

    /// Generate SWID SBOM
    async fn generate_swid_sbom(&self, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating SWID SBOM...");

        let mut sbom = serde_json::json!({
            "swid": {
                "tagId": format!("polymera-os-{}", chrono::Utc::now().timestamp()),
                "name": "Polymera OS",
                "version": "1.0.0",
                "versionScheme": "semver",
                "tagVersion": 1,
                "softwareLicenses": [{
                    "name": "MIT OR Apache-2.0"
                }],
                "entities": [{
                    "name": "Polymera OS Team",
                    "role": "softwareCreator"
                }],
                "links": [],
                "softwareMeta": {
                    "generator": "polymeractl",
                    "generatorVersion": env!("CARGO_PKG_VERSION")
                }
            }
        });

        // Scan source directory for components
        let components = self.scan_components().await?;
        
        // Add components to SBOM
        for component in components {
            let link = serde_json::json!({
                "href": component.path,
                "rel": "component",
                "artifact": {
                    "name": component.name,
                    "version": component.version
                }
            });
            
            sbom["swid"]["links"].as_array_mut().unwrap().push(link);
        }

        // Write SBOM to file
        let content = serde_json::to_string_pretty(&sbom)?;
        std::fs::write(output_path, content)?;

        Ok(())
    }

    /// Scan components in source directory
    async fn scan_components(&self) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
        let mut components = Vec::new();

        // Look for Rust crates
        if self.source.join("Cargo.toml").exists() {
            components.extend(self.scan_rust_crates().await?);
        }

        // Look for Node.js packages
        if self.source.join("package.json").exists() {
            components.extend(self.scan_node_packages().await?);
        }

        // Look for Python packages
        if self.source.join("pyproject.toml").exists() || self.source.join("setup.py").exists() {
            components.extend(self.scan_python_packages().await?);
        }

        // Look for Go modules
        if self.source.join("go.mod").exists() {
            components.extend(self.scan_go_modules().await?);
        }

        // Look for binary files
        components.extend(self.scan_binary_files().await?);

        println!("    Found {} components", components.len());
        Ok(components)
    }

    /// Scan Rust crates
    async fn scan_rust_crates(&self) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
        let mut components = Vec::new();

        // Read Cargo.toml
        let cargo_content = std::fs::read_to_string(self.source.join("Cargo.toml"))?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_content)?;

        if let Some(package) = cargo_toml.get("package") {
            let name = package.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let version = package.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
            let description = package.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let license = package.get("license").and_then(|v| v.as_str()).unwrap_or("MIT");

            // Calculate checksum
            let checksum = self.calculate_file_checksum(&self.source.join("Cargo.toml")).await?;

            components.push(Component {
                name: name.to_string(),
                version: version.to_string(),
                path: "Cargo.toml".to_string(),
                checksum,
                license: license.to_string(),
                copyright: "".to_string(),
                description: description.to_string(),
            });
        }

        // Scan dependencies
        if self.dependencies {
            if let Some(dependencies) = cargo_toml.get("dependencies") {
                for (dep_name, dep_info) in dependencies.as_table().unwrap_or(&toml::map::Map::new()) {
                    let version = if let Some(version) = dep_info.get("version") {
                        version.as_str().unwrap_or("0.0.0")
                    } else if let Some(version) = dep_info.as_str() {
                        version
                    } else {
                        "0.0.0"
                    };

                    components.push(Component {
                        name: dep_name.clone(),
                        version: version.to_string(),
                        path: format!("dependency:{}", dep_name),
                        checksum: "".to_string(),
                        license: "Unknown".to_string(),
                        copyright: "".to_string(),
                        description: "".to_string(),
                    });
                }
            }
        }

        Ok(components)
    }

    /// Scan Node.js packages
    async fn scan_node_packages(&self) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
        let mut components = Vec::new();

        // Read package.json
        let package_content = std::fs::read_to_string(self.source.join("package.json"))?;
        let package_json: serde_json::Value = serde_json::from_str(&package_content)?;

        if let Some(name) = package_json.get("name").and_then(|v| v.as_str()) {
            let version = package_json.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
            let description = package_json.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let license = package_json.get("license").and_then(|v| v.as_str()).unwrap_or("MIT");

            // Calculate checksum
            let checksum = self.calculate_file_checksum(&self.source.join("package.json")).await?;

            components.push(Component {
                name: name.to_string(),
                version: version.to_string(),
                path: "package.json".to_string(),
                checksum,
                license: license.to_string(),
                copyright: "".to_string(),
                description: description.to_string(),
            });
        }

        // Scan dependencies
        if self.dependencies {
            if let Some(dependencies) = package_json.get("dependencies") {
                if let Some(deps) = dependencies.as_object() {
                    for (dep_name, dep_info) in deps {
                        let version = if let Some(version) = dep_info.as_str() {
                            version
                        } else {
                            "0.0.0"
                        };

                        components.push(Component {
                            name: dep_name.clone(),
                            version: version.to_string(),
                            path: format!("dependency:{}", dep_name),
                            checksum: "".to_string(),
                            license: "Unknown".to_string(),
                            copyright: "".to_string(),
                            description: "".to_string(),
                        });
                    }
                }
            }
        }

        Ok(components)
    }

    /// Scan Python packages
    async fn scan_python_packages(&self) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
        let mut components = Vec::new();

        // Try pyproject.toml first
        if self.source.join("pyproject.toml").exists() {
            let pyproject_content = std::fs::read_to_string(self.source.join("pyproject.toml"))?;
            let pyproject: toml::Value = toml::from_str(&pyproject_content)?;

            if let Some(project) = pyproject.get("project") {
                let name = project.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                let version = project.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
                let description = project.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let license = project.get("license").and_then(|v| v.as_str()).unwrap_or("MIT");

                let checksum = self.calculate_file_checksum(&self.source.join("pyproject.toml")).await?;

                components.push(Component {
                    name: name.to_string(),
                    version: version.to_string(),
                    path: "pyproject.toml".to_string(),
                    checksum,
                    license: license.to_string(),
                    copyright: "".to_string(),
                    description: description.to_string(),
                });
            }
        }

        // Try setup.py
        if self.source.join("setup.py").exists() {
            let checksum = self.calculate_file_checksum(&self.source.join("setup.py")).await?;

            components.push(Component {
                name: "python-package".to_string(),
                version: "0.0.0".to_string(),
                path: "setup.py".to_string(),
                checksum,
                license: "MIT".to_string(),
                copyright: "".to_string(),
                description: "Python package".to_string(),
            });
        }

        Ok(components)
    }

    /// Scan Go modules
    async fn scan_go_modules(&self) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
        let mut components = Vec::new();

        // Read go.mod
        let go_mod_content = std::fs::read_to_string(self.source.join("go.mod"))?;
        
        // Parse module name
        for line in go_mod_content.lines() {
            if line.starts_with("module ") {
                let module_name = line.split_whitespace().nth(1).unwrap_or("unknown");
                let checksum = self.calculate_file_checksum(&self.source.join("go.mod")).await?;

                components.push(Component {
                    name: module_name.to_string(),
                    version: "0.0.0".to_string(),
                    path: "go.mod".to_string(),
                    checksum,
                    license: "MIT".to_string(),
                    copyright: "".to_string(),
                    description: "Go module".to_string(),
                });
                break;
            }
        }

        // Scan dependencies
        if self.dependencies {
            for line in go_mod_content.lines() {
                if line.starts_with("\t") && line.contains(" v") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let dep_name = parts[0];
                        let version = parts[1];

                        components.push(Component {
                            name: dep_name.to_string(),
                            version: version.to_string(),
                            path: format!("dependency:{}", dep_name),
                            checksum: "".to_string(),
                            license: "Unknown".to_string(),
                            copyright: "".to_string(),
                            description: "".to_string(),
                        });
                    }
                }
            }
        }

        Ok(components)
    }

    /// Scan binary files
    async fn scan_binary_files(&self) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
        let mut components = Vec::new();

        // Look for common binary files
        let binary_extensions = ["exe", "bin", "so", "dylib", "dll"];
        
        for entry in walkdir::WalkDir::new(&self.source)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if binary_extensions.contains(&ext.to_str().unwrap_or("")) {
                        let name = path.file_name().unwrap().to_string_lossy().to_string();
                        let checksum = self.calculate_file_checksum(path).await?;

                        components.push(Component {
                            name,
                            version: "1.0.0".to_string(),
                            path: path.to_string_lossy().to_string(),
                            checksum,
                            license: "Unknown".to_string(),
                            copyright: "".to_string(),
                            description: "Binary file".to_string(),
                        });
                    }
                }
            }
        }

        Ok(components)
    }

    /// Calculate file checksum
    async fn calculate_file_checksum(&self, file_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
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

    /// Validate SBOM
    async fn validate_sbom(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Validating SBOM...");

        if !self.source.exists() {
            return Err("Source file not found".into());
        }

        // Read SBOM file
        let content = std::fs::read_to_string(&self.source)?;
        
        // Validate based on format
        match self.format.as_str() {
            "spdx" => self.validate_spdx_sbom(&content).await?,
            "cyclonedx" => self.validate_cyclonedx_sbom(&content).await?,
            "swid" => self.validate_swid_sbom(&content).await?,
            _ => unreachable!(),
        }

        println!("  ✓ SBOM validation passed");
        Ok(())
    }

    /// Validate SPDX SBOM
    async fn validate_spdx_sbom(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sbom: serde_json::Value = serde_json::from_str(content)?;
        
        // Check required fields
        let required_fields = ["SPDXID", "spdxVersion", "name", "documentNamespace", "creationInfo"];
        for field in &required_fields {
            if !sbom.get(field).is_some() {
                return Err(format!("Missing required SPDX field: {}", field).into());
            }
        }

        // Check version
        if let Some(version) = sbom.get("spdxVersion") {
            if version.as_str() != Some("SPDX-2.3") {
                return Err("Unsupported SPDX version".into());
            }
        }

        Ok(())
    }

    /// Validate CycloneDX SBOM
    async fn validate_cyclonedx_sbom(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sbom: serde_json::Value = serde_json::from_str(content)?;
        
        // Check required fields
        let required_fields = ["bomFormat", "specVersion", "version", "metadata"];
        for field in &required_fields {
            if !sbom.get(field).is_some() {
                return Err(format!("Missing required CycloneDX field: {}", field).into());
            }
        }

        // Check format
        if let Some(format) = sbom.get("bomFormat") {
            if format.as_str() != Some("CycloneDX") {
                return Err("Invalid CycloneDX format".into());
            }
        }

        Ok(())
    }

    /// Validate SWID SBOM
    async fn validate_swid_sbom(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sbom: serde_json::Value = serde_json::from_str(content)?;
        
        // Check required fields
        if let Some(swid) = sbom.get("swid") {
            let required_fields = ["tagId", "name", "version"];
            for field in &required_fields {
                if !swid.get(field).is_some() {
                    return Err(format!("Missing required SWID field: {}", field).into());
                }
            }
        } else {
            return Err("Missing SWID root element".into());
        }

        Ok(())
    }

    /// Validate generated SBOM
    async fn validate_generated_sbom(&self, sbom_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Validating generated SBOM...");
        
        let content = std::fs::read_to_string(sbom_path)?;
        self.validate_sbom().await?;
        
        println!("    ✓ Generated SBOM is valid");
        Ok(())
    }

    /// Merge SBOMs
    async fn merge_sbom(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Merging SBOMs...");
        // Implementation for merging multiple SBOMs
        println!("  ⚠ SBOM merging not yet implemented");
        Ok(())
    }

    /// Diff SBOMs
    async fn diff_sbom(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Diffing SBOMs...");
        // Implementation for comparing SBOMs
        println!("  ⚠ SBOM diffing not yet implemented");
        Ok(())
    }
}

/// Component information
#[derive(Debug)]
struct Component {
    name: String,
    version: String,
    path: String,
    checksum: String,
    license: String,
    copyright: String,
    description: String,
}

impl Default for SbomCommand {
    fn default() -> Self {
        Self {
            action: "generate".to_string(),
            source: PathBuf::from("."),
            output: None,
            format: "spdx".to_string(),
            version: None,
            dependencies: true,
            licenses: true,
            vulnerabilities: false,
            validate: false,
            detailed: false,
            output_format: "json".to_string(),
        }
    }
}
