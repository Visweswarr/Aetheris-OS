use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{
    file::FileCaps,
    sched::SchedCaps,
    Error, ErrorExt, WasiFile, WasiDir, WasiCtx,
};

/// WASM module configuration
pub struct WasmConfig {
    /// Default file capabilities
    pub file_caps: FileCaps,
    /// Default scheduling capabilities
    pub sched_caps: SchedCaps,
    /// Allowed file paths
    pub allowed_paths: Vec<String>,
    /// Deny by default
    pub deny_by_default: bool,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            file_caps: FileCaps::empty(),
            sched_caps: SchedCaps::empty(),
            allowed_paths: Vec::new(),
            deny_by_default: true,
        }
    }
}

/// WASM module context
pub struct WasmModule {
    config: WasmConfig,
    wasi_ctx: WasiCtx,
}

impl WasmModule {
    /// Create a new WASM module with the given configuration
    pub fn new(config: WasmConfig) -> Result<Self, Error> {
        let mut builder = WasiCtxBuilder::new();
        
        // Set capabilities based on configuration
        if !config.file_caps.is_empty() {
            builder = builder.inherit_stdio()
                .inherit_args()?
                .inherit_env()?;
            
            // Add file capabilities for allowed paths
            for path in &config.allowed_paths {
                if let Ok(dir) = fs::read_dir(path) {
                    builder = builder.preopened_dir(dir, path)?;
                }
            }
        }
        
        let wasi_ctx = builder.build();
        
        Ok(Self {
            config,
            wasi_ctx,
        })
    }
    
    /// Read a file with capability checking
    pub fn read_file(&self, path: &str) -> Result<String, Error> {
        // Check if file access is allowed
        if !self.is_path_allowed(path) {
            return Err(Error::not_supported("File access denied by capability policy"));
        }
        
        // Check if we have read capabilities
        if !self.config.file_caps.contains(FileCaps::READ) {
            return Err(Error::not_supported("Read capability not granted"));
        }
        
        // Read the file
        let mut file = fs::File::open(path)
            .map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)))?;
        
        Ok(contents)
    }
    
    /// Write to a file with capability checking
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), Error> {
        // Check if file access is allowed
        if !self.is_path_allowed(path) {
            return Err(Error::not_supported("File access denied by capability policy"));
        }
        
        // Check if we have write capabilities
        if !self.config.file_caps.contains(FileCaps::WRITE) {
            return Err(Error::not_supported("Write capability not granted"));
        }
        
        // Write to the file
        let mut file = fs::File::create(path)
            .map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)))?;
        
        Ok(())
    }
    
    /// List directory contents with capability checking
    pub fn list_directory(&self, path: &str) -> Result<Vec<String>, Error> {
        // Check if directory access is allowed
        if !self.is_path_allowed(path) {
            return Err(Error::not_supported("Directory access denied by capability policy"));
        }
        
        // Check if we have read capabilities
        if !self.config.file_caps.contains(FileCaps::READ) {
            return Err(Error::not_supported("Read capability not granted"));
        }
        
        // List directory contents
        let entries = fs::read_dir(path)
            .map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)))?;
        
        let mut files = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(name) = entry.file_name().into_string() {
                    files.push(name);
                }
            }
        }
        
        Ok(files)
    }
    
    /// Check if a path is allowed based on capability policy
    fn is_path_allowed(&self, path: &str) -> bool {
        if self.config.deny_by_default {
            // In deny-by-default mode, only explicitly allowed paths are permitted
            self.config.allowed_paths.iter().any(|allowed| {
                path.starts_with(allowed) || path == allowed
            })
        } else {
            // In allow-by-default mode, only explicitly denied paths are blocked
            !self.config.allowed_paths.iter().any(|denied| {
                path.starts_with(denied) || path == denied
            })
        }
    }
    
    /// Get current capabilities
    pub fn get_capabilities(&self) -> &FileCaps {
        &self.config.file_caps
    }
    
    /// Check if a specific capability is granted
    pub fn has_capability(&self, cap: FileCaps) -> bool {
        self.config.file_caps.contains(cap)
    }
    
    /// Get allowed paths
    pub fn get_allowed_paths(&self) -> &[String] {
        &self.config.allowed_paths
    }
    
    /// Get WASI context
    pub fn get_wasi_context(&self) -> &WasiCtx {
        &self.wasi_ctx
    }
}

/// WASM module manifest
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmManifest {
    /// Module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Required capabilities
    pub required_caps: Vec<String>,
    /// Allowed file paths
    pub allowed_paths: Vec<String>,
    /// Denied file paths
    pub denied_paths: Vec<String>,
    /// Deny by default flag
    pub deny_by_default: bool,
    /// Environment variables
    pub environment: std::collections::HashMap<String, String>,
    /// Command line arguments
    pub args: Vec<String>,
}

impl Default for WasmManifest {
    fn default() -> Self {
        Self {
            name: "hello-wasm".to_string(),
            version: "0.1.0".to_string(),
            required_caps: vec![],
            allowed_paths: vec![],
            denied_paths: vec![],
            deny_by_default: true,
            environment: std::collections::HashMap::new(),
            args: vec![],
        }
    }
}

/// Load manifest from file
pub fn load_manifest(path: &str) -> Result<WasmManifest, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let manifest: WasmManifest = serde_json::from_str(&content)?;
    Ok(manifest)
}

/// Save manifest to file
pub fn save_manifest(manifest: &WasmManifest, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(path, content)?;
    Ok(())
}

/// Create a WASM module from manifest
pub fn create_module_from_manifest(manifest: &WasmManifest) -> Result<WasmModule, Error> {
    let mut config = WasmConfig::default();
    
    // Set deny by default from manifest
    config.deny_by_default = manifest.deny_by_default;
    
    // Set allowed paths
    if manifest.deny_by_default {
        config.allowed_paths = manifest.allowed_paths.clone();
    } else {
        config.allowed_paths = manifest.denied_paths.clone();
    }
    
    // Set capabilities based on required caps
    for cap_str in &manifest.required_caps {
        match cap_str.as_str() {
            "read" => config.file_caps |= FileCaps::READ,
            "write" => config.file_caps |= FileCaps::WRITE,
            "create" => config.file_caps |= FileCaps::CREATE,
            "truncate" => config.file_caps |= FileCaps::TRUNCATE,
            "seek" => config.file_caps |= FileCaps::SEEK,
            "tell" => config.file_caps |= FileCaps::TELL,
            "sync" => config.file_caps |= FileCaps::SYNC,
            "datasync" => config.file_caps |= FileCaps::DATASYNC,
            "readdir" => config.file_caps |= FileCaps::READDIR,
            "link" => config.file_caps |= FileCaps::LINK,
            "unlink" => config.file_caps |= FileCaps::UNLINK,
            "rename" => config.file_caps |= FileCaps::RENAME,
            "symlink" => config.file_caps |= FileCaps::SYMLINK,
            "path_create_file" => config.file_caps |= FileCaps::PATH_CREATE_FILE,
            "path_create_dir" => config.file_caps |= FileCaps::PATH_CREATE_DIR,
            "path_link" => config.file_caps |= FileCaps::PATH_LINK,
            "path_open" => config.file_caps |= FileCaps::PATH_OPEN,
            "path_readlink" => config.file_caps |= FileCaps::PATH_READLINK,
            "path_rename" => config.file_caps |= FileCaps::PATH_RENAME,
            "path_symlink" => config.file_caps |= FileCaps::PATH_SYMLINK,
            "path_unlink_file" => config.file_caps |= FileCaps::PATH_UNLINK_FILE,
            "path_remove_dir" => config.file_caps |= FileCaps::PATH_REMOVE_DIR,
            "path_filestat_get" => config.file_caps |= FileCaps::PATH_FILESTAT_GET,
            "path_filestat_set_times" => config.file_caps |= FileCaps::PATH_FILESTAT_SET_TIMES,
            "path_filestat_set_size" => config.file_caps |= FileCaps::PATH_FILESTAT_SET_SIZE,
            "fd_read" => config.file_caps |= FileCaps::FD_READ,
            "fd_write" => config.file_caps |= FileCaps::FD_WRITE,
            "fd_seek" => config.file_caps |= FileCaps::FD_SEEK,
            "fd_tell" => config.file_caps |= FileCaps::FD_TELL,
            "fd_sync" => config.file_caps |= FileCaps::FD_SYNC,
            "fd_datasync" => config.file_caps |= FileCaps::FD_DATASYNC,
            "fd_readdir" => config.file_caps |= FileCaps::FD_READDIR,
            "fd_link" => config.file_caps |= FileCaps::FD_LINK,
            "fd_unlink" => config.file_caps |= FileCaps::FD_UNLINK,
            "fd_rename" => config.file_caps |= FileCaps::FD_RENAME,
            "fd_symlink" => config.file_caps |= FileCaps::FD_SYMLINK,
            "fd_create_file" => config.file_caps |= FileCaps::FD_CREATE_FILE,
            "fd_create_dir" => config.file_caps |= FileCaps::FD_CREATE_DIR,
            "fd_open" => config.file_caps |= FileCaps::FD_OPEN,
            "fd_readlink" => config.file_caps |= FileCaps::FD_READLINK,
            "fd_remove_dir" => config.file_caps |= FileCaps::FD_REMOVE_DIR,
            "fd_filestat_get" => config.file_caps |= FileCaps::FD_FILESTAT_GET,
            "fd_filestat_set_times" => config.file_caps |= FileCaps::FD_FILESTAT_SET_TIMES,
            "fd_filestat_set_size" => config.file_caps |= FileCaps::FD_FILESTAT_SET_SIZE,
            _ => {
                // Unknown capability, log warning but continue
                eprintln!("Warning: Unknown capability '{}'", cap_str);
            }
        }
    }
    
    WasmModule::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_wasm_config_default() {
        let config = WasmConfig::default();
        assert!(config.deny_by_default);
        assert!(config.allowed_paths.is_empty());
        assert!(config.file_caps.is_empty());
    }
    
    #[test]
    fn test_wasm_manifest_default() {
        let manifest = WasmManifest::default();
        assert_eq!(manifest.name, "hello-wasm");
        assert_eq!(manifest.version, "0.1.0");
        assert!(manifest.deny_by_default);
        assert!(manifest.required_caps.is_empty());
        assert!(manifest.allowed_paths.is_empty());
    }
    
    #[test]
    fn test_path_allowed_deny_by_default() {
        let mut config = WasmConfig::default();
        config.deny_by_default = true;
        config.allowed_paths = vec!["/tmp".to_string(), "/home".to_string()];
        
        let module = WasmModule::new(config).unwrap();
        
        assert!(module.is_path_allowed("/tmp/test.txt"));
        assert!(module.is_path_allowed("/home/user/file.txt"));
        assert!(!module.is_path_allowed("/etc/passwd"));
        assert!(!module.is_path_allowed("/var/log/system.log"));
    }
    
    #[test]
    fn test_path_allowed_allow_by_default() {
        let mut config = WasmConfig::default();
        config.deny_by_default = false;
        config.allowed_paths = vec!["/etc".to_string(), "/var".to_string()];
        
        let module = WasmModule::new(config).unwrap();
        
        assert!(!module.is_path_allowed("/etc/passwd"));
        assert!(!module.is_path_allowed("/var/log/system.log"));
        assert!(module.is_path_allowed("/tmp/test.txt"));
        assert!(module.is_path_allowed("/home/user/file.txt"));
    }
    
    #[test]
    fn test_file_capabilities() {
        let mut config = WasmConfig::default();
        config.file_caps = FileCaps::READ | FileCaps::WRITE;
        config.allowed_paths = vec!["/tmp".to_string()];
        
        let module = WasmModule::new(config).unwrap();
        
        assert!(module.has_capability(FileCaps::READ));
        assert!(module.has_capability(FileCaps::WRITE));
        assert!(!module.has_capability(FileCaps::CREATE));
    }
    
    #[test]
    fn test_manifest_loading() {
        let temp_dir = tempdir().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");
        
        let manifest = WasmManifest {
            name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            required_caps: vec!["read".to_string(), "write".to_string()],
            allowed_paths: vec!["/tmp".to_string()],
            denied_paths: vec![],
            deny_by_default: true,
            environment: std::collections::HashMap::new(),
            args: vec![],
        };
        
        save_manifest(&manifest, manifest_path.to_str().unwrap()).unwrap();
        
        let loaded_manifest = load_manifest(manifest_path.to_str().unwrap()).unwrap();
        
        assert_eq!(loaded_manifest.name, "test-module");
        assert_eq!(loaded_manifest.version, "1.0.0");
        assert_eq!(loaded_manifest.required_caps, vec!["read", "write"]);
        assert_eq!(loaded_manifest.allowed_paths, vec!["/tmp"]);
    }
    
    #[test]
    fn test_module_from_manifest() {
        let manifest = WasmManifest {
            name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            required_caps: vec!["read".to_string()],
            allowed_paths: vec!["/tmp".to_string()],
            denied_paths: vec![],
            deny_by_default: true,
            environment: std::collections::HashMap::new(),
            args: vec![],
        };
        
        let module = create_module_from_manifest(&manifest).unwrap();
        
        assert!(module.has_capability(FileCaps::READ));
        assert!(!module.has_capability(FileCaps::WRITE));
        assert!(module.is_path_allowed("/tmp/test.txt"));
        assert!(!module.is_path_allowed("/etc/passwd"));
    }
}
