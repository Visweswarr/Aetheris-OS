use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use wasi_common::Error;

mod lib;
use lib::{WasmModule, WasmConfig, WasmManifest, create_module_from_manifest, load_manifest};

/// Main entry point for the WASM module
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [options]", args[0]);
        println!("Commands:");
        println!("  read <file>     - Read a file (requires read capability)");
        println!("  write <file> <content> - Write to a file (requires write capability)");
        println!("  list <dir>      - List directory contents (requires read capability)");
        println!("  capabilities    - Show current capabilities");
        println!("  manifest <file> - Load and apply manifest file");
        println!("  test            - Run capability tests");
        return Ok(());
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "read" => {
            if args.len() < 3 {
                println!("Error: read command requires a file path");
                return Ok(());
            }
            let file_path = &args[2];
            read_file(file_path)?;
        }
        
        "write" => {
            if args.len() < 4 {
                println!("Error: write command requires file path and content");
                return Ok(());
            }
            let file_path = &args[2];
            let content = &args[3];
            write_file(file_path, content)?;
        }
        
        "list" => {
            if args.len() < 3 {
                println!("Error: list command requires a directory path");
                return Ok(());
            }
            let dir_path = &args[2];
            list_directory(dir_path)?;
        }
        
        "capabilities" => {
            show_capabilities()?;
        }
        
        "manifest" => {
            if args.len() < 3 {
                println!("Error: manifest command requires a manifest file path");
                return Ok(());
            }
            let manifest_path = &args[2];
            load_and_apply_manifest(manifest_path)?;
        }
        
        "test" => {
            run_capability_tests()?;
        }
        
        _ => {
            println!("Unknown command: {}", command);
            println!("Use 'help' for usage information");
        }
    }
    
    Ok(())
}

/// Read a file with capability checking
fn read_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Attempting to read file: {}", file_path);
    
    // Create a basic WASM module with read capability
    let mut config = WasmConfig::default();
    config.file_caps = wasi_common::file::FileCaps::READ;
    config.allowed_paths = vec!["/tmp".to_string(), "/home".to_string()];
    config.deny_by_default = true;
    
    let module = WasmModule::new(config)?;
    
    match module.read_file(file_path) {
        Ok(content) => {
            println!("✅ File read successfully!");
            println!("📄 Content (first 100 chars): {}", 
                if content.len() > 100 { 
                    format!("{}...", &content[..100]) 
                } else { 
                    content.clone() 
                }
            );
            println!("📊 File size: {} bytes", content.len());
        }
        Err(e) => {
            println!("❌ Failed to read file: {}", e);
            if e.to_string().contains("denied by capability policy") {
                println!("🚫 Access denied: File path not allowed by capability policy");
            } else if e.to_string().contains("Read capability not granted") {
                println!("🚫 Access denied: Read capability not granted");
            }
        }
    }
    
    Ok(())
}

/// Write to a file with capability checking
fn write_file(file_path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("✍️  Attempting to write to file: {}", file_path);
    
    // Create a basic WASM module with write capability
    let mut config = WasmConfig::default();
    config.file_caps = wasi_common::file::FileCaps::WRITE | wasi_common::file::FileCaps::CREATE;
    config.allowed_paths = vec!["/tmp".to_string()];
    config.deny_by_default = true;
    
    let module = WasmModule::new(config)?;
    
    match module.write_file(file_path, content) {
        Ok(_) => {
            println!("✅ File written successfully!");
            println!("📝 Content written: {} bytes", content.len());
        }
        Err(e) => {
            println!("❌ Failed to write file: {}", e);
            if e.to_string().contains("denied by capability policy") {
                println!("🚫 Access denied: File path not allowed by capability policy");
            } else if e.to_string().contains("Write capability not granted") {
                println!("🚫 Access denied: Write capability not granted");
            }
        }
    }
    
    Ok(())
}

/// List directory contents with capability checking
fn list_directory(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Attempting to list directory: {}", dir_path);
    
    // Create a basic WASM module with read capability
    let mut config = WasmConfig::default();
    config.file_caps = wasi_common::file::FileCaps::READ | wasi_common::file::FileCaps::READDIR;
    config.allowed_paths = vec!["/tmp".to_string(), "/home".to_string()];
    config.deny_by_default = true;
    
    let module = WasmModule::new(config)?;
    
    match module.list_directory(dir_path) {
        Ok(entries) => {
            println!("✅ Directory listed successfully!");
            println!("📋 Contents ({} items):", entries.len());
            for (i, entry) in entries.iter().enumerate() {
                println!("  {}. {}", i + 1, entry);
            }
        }
        Err(e) => {
            println!("❌ Failed to list directory: {}", e);
            if e.to_string().contains("denied by capability policy") {
                println!("🚫 Access denied: Directory path not allowed by capability policy");
            } else if e.to_string().contains("Read capability not granted") {
                println!("🚫 Access denied: Read capability not granted");
            }
        }
    }
    
    Ok(())
}

/// Show current capabilities
fn show_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Current WASM Module Capabilities");
    println!("====================================");
    
    // Create a basic WASM module
    let config = WasmConfig::default();
    let module = WasmModule::new(config)?;
    
    let caps = module.get_capabilities();
    let allowed_paths = module.get_allowed_paths();
    
    println!("📁 File Capabilities:");
    println!("  READ: {}", module.has_capability(wasi_common::file::FileCaps::READ));
    println!("  WRITE: {}", module.has_capability(wasi_common::file::FileCaps::WRITE));
    println!("  CREATE: {}", module.has_capability(wasi_common::file::FileCaps::CREATE));
    println!("  DELETE: {}", module.has_capability(wasi_common::file::FileCaps::UNLINK));
    println!("  READDIR: {}", module.has_capability(wasi_common::file::FileCaps::READDIR));
    
    println!("\n🛣️  Path Access Policy:");
    println!("  Deny by default: {}", module.config.deny_by_default);
    if !allowed_paths.is_empty() {
        println!("  Allowed paths:");
        for path in allowed_paths {
            println!("    - {}", path);
        }
    } else {
        println!("  No paths explicitly allowed/denied");
    }
    
    println!("\n🔧 WASI Context:");
    println!("  Context created: {}", module.get_wasi_context().is_some());
    
    Ok(())
}

/// Load and apply a manifest file
fn load_and_apply_manifest(manifest_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Loading manifest: {}", manifest_path);
    
    let manifest = load_manifest(manifest_path)?;
    
    println!("✅ Manifest loaded successfully!");
    println!("📄 Module: {} v{}", manifest.name, manifest.version);
    println!("🔐 Required capabilities: {:?}", manifest.required_caps);
    println!("🛣️  Allowed paths: {:?}", manifest.allowed_paths);
    println!("🚫 Denied paths: {:?}", manifest.denied_paths);
    println!("⚙️  Deny by default: {}", manifest.deny_by_default);
    
    // Create module from manifest
    let module = create_module_from_manifest(&manifest)?;
    
    println!("\n🚀 Module created from manifest!");
    println!("🔐 Active capabilities: {:?}", module.get_capabilities());
    println!("🛣️  Active paths: {:?}", module.get_allowed_paths());
    
    Ok(())
}

/// Run capability tests
fn run_capability_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running Capability Tests");
    println!("============================");
    
    // Test 1: No capabilities
    println!("\n📋 Test 1: No capabilities");
    let config = WasmConfig::default();
    let module = WasmModule::new(config)?;
    
    println!("  Reading /tmp/test.txt...");
    match module.read_file("/tmp/test.txt") {
        Ok(_) => println!("    ❌ Unexpected success!"),
        Err(e) => println!("    ✅ Expected failure: {}", e),
    }
    
    // Test 2: Read capability only
    println!("\n📋 Test 2: Read capability only");
    let mut config = WasmConfig::default();
    config.file_caps = wasi_common::file::FileCaps::READ;
    config.allowed_paths = vec!["/tmp".to_string()];
    let module = WasmModule::new(config)?;
    
    println!("  Reading /tmp/test.txt...");
    match module.read_file("/tmp/test.txt") {
        Ok(_) => println!("    ✅ Success with read capability"),
        Err(e) => println!("    ❌ Unexpected failure: {}", e),
    }
    
    println!("  Writing to /tmp/test.txt...");
    match module.write_file("/tmp/test.txt", "test") {
        Ok(_) => println!("    ❌ Unexpected success!"),
        Err(e) => println!("    ✅ Expected failure: {}", e),
    }
    
    // Test 3: Read and write capabilities
    println!("\n📋 Test 3: Read and write capabilities");
    let mut config = WasmConfig::default();
    config.file_caps = wasi_common::file::FileCaps::READ | wasi_common::file::FileCaps::WRITE;
    config.allowed_paths = vec!["/tmp".to_string()];
    let module = WasmModule::new(config)?;
    
    println!("  Writing to /tmp/test.txt...");
    match module.write_file("/tmp/test.txt", "Hello, WASM!") {
        Ok(_) => println!("    ✅ Success with write capability"),
        Err(e) => println!("    ❌ Unexpected failure: {}", e),
    }
    
    println!("  Reading /tmp/test.txt...");
    match module.read_file("/tmp/test.txt") {
        Ok(content) => println!("    ✅ Success with read capability: {}", content),
        Err(e) => println!("    ❌ Unexpected failure: {}", e),
    }
    
    // Test 4: Path restrictions
    println!("\n📋 Test 4: Path restrictions");
    let mut config = WasmConfig::default();
    config.file_caps = wasi_common::file::FileCaps::READ;
    config.allowed_paths = vec!["/tmp".to_string()];
    config.deny_by_default = true;
    let module = WasmModule::new(config)?;
    
    println!("  Reading /tmp/test.txt (allowed)...");
    match module.read_file("/tmp/test.txt") {
        Ok(_) => println!("    ✅ Success: Path is allowed"),
        Err(e) => println!("    ❌ Unexpected failure: {}", e),
    }
    
    println!("  Reading /etc/passwd (denied)...");
    match module.read_file("/etc/passwd") {
        Ok(_) => println!("    ❌ Unexpected success!"),
        Err(e) => println!("    ✅ Expected failure: {}", e),
    }
    
    println!("\n🎉 All capability tests completed!");
    
    Ok(())
}

/// Create a test file for testing
fn create_test_file() -> Result<(), Box<dyn std::error::Error>> {
    let test_content = "Hello, WASM World!\nThis is a test file for capability testing.\n";
    
    // Create /tmp directory if it doesn't exist
    if !Path::new("/tmp").exists() {
        fs::create_dir_all("/tmp")?;
    }
    
    // Write test file
    let mut file = fs::File::create("/tmp/test.txt")?;
    file.write_all(test_content.as_bytes())?;
    
    println!("✅ Test file created: /tmp/test.txt");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_operations() {
        // Create test file
        create_test_file().unwrap();
        
        // Test read operation
        let mut config = WasmConfig::default();
        config.file_caps = wasi_common::file::FileCaps::READ;
        config.allowed_paths = vec!["/tmp".to_string()];
        
        let module = WasmModule::new(config).unwrap();
        
        // Should succeed
        assert!(module.read_file("/tmp/test.txt").is_ok());
        
        // Should fail (path not allowed)
        assert!(module.read_file("/etc/passwd").is_err());
    }
    
    #[test]
    fn test_capability_checks() {
        let config = WasmConfig::default();
        let module = WasmModule::new(config).unwrap();
        
        // No capabilities by default
        assert!(!module.has_capability(wasi_common::file::FileCaps::READ));
        assert!(!module.has_capability(wasi_common::file::FileCaps::WRITE));
    }
}
