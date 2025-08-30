use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimConfig {
    pub language: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimCall {
    pub function: String,
    pub args: Vec<String>,
    pub language: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimResult {
    pub success: bool,
    pub result: String,
    pub error: Option<String>,
    pub timestamp: u64,
}

pub struct PolyglotShims {
    shims: Arc<Mutex<HashMap<String, ShimConfig>>>,
    call_history: Arc<Mutex<Vec<ShimCall>>>,
    results_cache: Arc<Mutex<HashMap<String, ShimResult>>>,
}

impl PolyglotShims {
    pub fn new() -> Self {
        let mut shims = Self {
            shims: Arc::new(Mutex::new(HashMap::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
            results_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        
        shims.initialize_default_shims();
        shims
    }

    fn initialize_default_shims(&mut self) {
        let mut shims = self.shims.lock().unwrap();
        
        shims.insert("libc".to_string(), ShimConfig {
            language: "C".to_string(),
            version: "2.37".to_string(),
            capabilities: vec!["posix:basic".to_string(), "filesystem:read".to_string()],
            enabled: true,
        });
        
        shims.insert("go".to_string(), ShimConfig {
            language: "Go".to_string(),
            version: "1.21".to_string(),
            capabilities: vec!["posix:basic".to_string(), "filesystem:read".to_string(), "filesystem:write".to_string()],
            enabled: true,
        });
        
        shims.insert("rust".to_string(), ShimConfig {
            language: "Rust".to_string(),
            version: "1.75".to_string(),
            capabilities: vec!["posix:basic".to_string(), "filesystem:read".to_string(), "filesystem:write".to_string()],
            enabled: true,
        });
        
        shims.insert("node".to_string(), ShimConfig {
            language: "Node.js".to_string(),
            version: "20.0".to_string(),
            capabilities: vec!["posix:basic".to_string(), "filesystem:read".to_string(), "filesystem:write".to_string()],
            enabled: true,
        });
        
        shims.insert("wasi".to_string(), ShimConfig {
            language: "WASI".to_string(),
            version: "0.2.0".to_string(),
            capabilities: vec!["posix:basic".to_string(), "filesystem:read".to_string(), "filesystem:write".to_string()],
            enabled: true,
        });
    }

    pub fn call_shim(&self, language: &str, function: &str, args: Vec<String>, capabilities: &[String]) -> Result<ShimResult, String> {
        let shim = self.get_shim(language)?;
        
        if !shim.enabled {
            return Err(format!("Shim {} is not enabled", language));
        }
        
        if !self.check_capabilities(&shim.capabilities, capabilities) {
            return Err("Insufficient capabilities for this shim".to_string());
        }
        
        let call = ShimCall {
            function: function.to_string(),
            args: args.clone(),
            language: language.to_string(),
            timestamp: self.get_virtual_time(),
        };
        
        self.call_history.lock().unwrap().push(call);
        
        let result = self.execute_shim_call(language, function, &args);
        self.results_cache.lock().unwrap().insert(
            format!("{}:{}:{}", language, function, args.join(",")),
            result.clone()
        );
        
        Ok(result)
    }

    fn execute_shim_call(&self, language: &str, function: &str, args: &[String]) -> ShimResult {
        match language {
            "libc" => self.execute_libc_call(function, args),
            "go" => self.execute_go_call(function, args),
            "rust" => self.execute_rust_call(function, args),
            "node" => self.execute_node_call(function, args),
            "wasi" => self.execute_wasi_call(function, args),
            _ => ShimResult {
                success: false,
                result: "".to_string(),
                error: Some(format!("Unknown language: {}", language)),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn execute_libc_call(&self, function: &str, args: &[String]) -> ShimResult {
        match function {
            "open" => {
                if args.len() >= 2 {
                    let path = &args[0];
                    let mode = &args[1];
                    ShimResult {
                        success: true,
                        result: format!("fd:3, path:{}, mode:{}", path, mode),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("open: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "read" => {
                if args.len() >= 2 {
                    let fd = &args[0];
                    let size: usize = args[1].parse().unwrap_or(0);
                    ShimResult {
                        success: true,
                        result: format!("read {} bytes from fd {}", size, fd),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("read: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "write" => {
                if args.len() >= 2 {
                    let fd = &args[0];
                    let data = &args[1];
                    ShimResult {
                        success: true,
                        result: format!("wrote {} bytes to fd {}", data.len(), fd),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("write: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "close" => {
                if args.len() >= 1 {
                    let fd = &args[0];
                    ShimResult {
                        success: true,
                        result: format!("closed fd {}", fd),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("close: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            _ => ShimResult {
                success: false,
                result: "".to_string(),
                error: Some(format!("Unsupported libc function: {}", function)),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn execute_go_call(&self, function: &str, args: &[String]) -> ShimResult {
        match function {
            "OpenFile" => {
                if args.len() >= 2 {
                    let path = &args[0];
                    let mode = &args[1];
                    ShimResult {
                        success: true,
                        result: format!("Go file opened: {}, mode: {}", path, mode),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("OpenFile: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "ReadFile" => {
                if args.len() >= 1 {
                    let path = &args[0];
                    ShimResult {
                        success: true,
                        result: format!("Go file read: {}", path),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("ReadFile: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            _ => ShimResult {
                success: false,
                result: "".to_string(),
                error: Some(format!("Unsupported Go function: {}", function)),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn execute_rust_call(&self, function: &str, args: &[String]) -> ShimResult {
        match function {
            "File::open" => {
                if args.len() >= 1 {
                    let path = &args[0];
                    ShimResult {
                        success: true,
                        result: format!("Rust file opened: {}", path),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("File::open: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "read_to_string" => {
                if args.len() >= 1 {
                    let path = &args[0];
                    ShimResult {
                        success: true,
                        result: format!("Rust file read: {}", path),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("read_to_string: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            _ => ShimResult {
                success: false,
                result: "".to_string(),
                error: Some(format!("Unsupported Rust function: {}", function)),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn execute_node_call(&self, function: &str, args: &[String]) -> ShimResult {
        match function {
            "fs.open" => {
                if args.len() >= 2 {
                    let path = &args[0];
                    let mode = &args[1];
                    ShimResult {
                        success: true,
                        result: format!("Node.js file opened: {}, mode: {}", path, mode),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("fs.open: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "fs.readFile" => {
                if args.len() >= 1 {
                    let path = &args[0];
                    ShimResult {
                        success: true,
                        result: format!("Node.js file read: {}", path),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("fs.readFile: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            _ => ShimResult {
                success: false,
                result: "".to_string(),
                error: Some(format!("Unsupported Node.js function: {}", function)),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn execute_wasi_call(&self, function: &str, args: &[String]) -> ShimResult {
        match function {
            "fd_open" => {
                if args.len() >= 2 {
                    let path = &args[0];
                    let flags = &args[1];
                    ShimResult {
                        success: true,
                        result: format!("WASI fd opened: {}, flags: {}", path, flags),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("fd_open: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            "fd_read" => {
                if args.len() >= 2 {
                    let fd = &args[0];
                    let size = &args[1];
                    ShimResult {
                        success: true,
                        result: format!("WASI fd read: {}, size: {}", fd, size),
                        error: None,
                        timestamp: self.get_virtual_time(),
                    }
                } else {
                    ShimResult {
                        success: false,
                        result: "".to_string(),
                        error: Some("fd_read: insufficient arguments".to_string()),
                        timestamp: self.get_virtual_time(),
                    }
                }
            },
            _ => ShimResult {
                success: false,
                result: "".to_string(),
                error: Some(format!("Unsupported WASI function: {}", function)),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn get_shim(&self, language: &str) -> Result<ShimConfig, String> {
        let shims = self.shims.lock().unwrap();
        shims.get(language)
            .cloned()
            .ok_or_else(|| format!("Shim not found for language: {}", language))
    }

    fn check_capabilities(&self, required: &[String], provided: &[String]) -> bool {
        required.iter().all(|req| {
            provided.iter().any(|prov| {
                prov == req || prov == "all" || prov == "root:all"
            })
        })
    }

    fn get_virtual_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn get_shim_count(&self) -> usize {
        self.shims.lock().unwrap().len()
    }

    pub fn get_call_count(&self) -> usize {
        self.call_history.lock().unwrap().len()
    }

    pub fn get_enabled_shims(&self) -> Vec<String> {
        let shims = self.shims.lock().unwrap();
        shims.iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn enable_shim(&self, language: &str) -> Result<(), String> {
        let mut shims = self.shims.lock().unwrap();
        if let Some(shim) = shims.get_mut(language) {
            shim.enabled = true;
            Ok(())
        } else {
            Err(format!("Shim not found: {}", language))
        }
    }

    pub fn disable_shim(&self, language: &str) -> Result<(), String> {
        let mut shims = self.shims.lock().unwrap();
        if let Some(shim) = shims.get_mut(language) {
            shim.enabled = false;
            Ok(())
        } else {
            Err(format!("Shim not found: {}", language))
        }
    }

    pub fn get_call_history(&self) -> Vec<ShimCall> {
        self.call_history.lock().unwrap().clone()
    }

    pub fn clear_cache(&self) {
        self.results_cache.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shims_creation() {
        let shims = PolyglotShims::new();
        assert_eq!(shims.get_shim_count(), 5);
        assert_eq!(shims.get_call_count(), 0);
    }

    #[test]
    fn test_libc_open() {
        let shims = PolyglotShims::new();
        let result = shims.call_shim("libc", "open", 
            vec!["/tmp/test.txt".to_string(), "r".to_string()],
            &["posix:basic".to_string()]
        ).unwrap();
        
        assert!(result.success);
        assert!(result.result.contains("fd:3"));
    }

    #[test]
    fn test_go_open_file() {
        let shims = PolyglotShims::new();
        let result = shims.call_shim("go", "OpenFile", 
            vec!["/tmp/test.txt".to_string(), "r".to_string()],
            &["posix:basic".to_string()]
        ).unwrap();
        
        assert!(result.success);
        assert!(result.result.contains("Go file opened"));
    }

    #[test]
    fn test_rust_file_open() {
        let shims = PolyglotShims::new();
        let result = shims.call_shim("rust", "File::open", 
            vec!["/tmp/test.txt".to_string()],
            &["posix:basic".to_string()]
        ).unwrap();
        
        assert!(result.success);
        assert!(result.result.contains("Rust file opened"));
    }

    #[test]
    fn test_node_fs_open() {
        let shims = PolyglotShims::new();
        let result = shims.call_shim("node", "fs.open", 
            vec!["/tmp/test.txt".to_string(), "r".to_string()],
            &["posix:basic".to_string()]
        ).unwrap();
        
        assert!(result.success);
        assert!(result.result.contains("Node.js file opened"));
    }

    #[test]
    fn test_wasi_fd_open() {
        let shims = PolyglotShims::new();
        let result = shims.call_shim("wasi", "fd_open", 
            vec!["/tmp/test.txt".to_string(), "0".to_string()],
            &["posix:basic".to_string()]
        ).unwrap();
        
        assert!(result.success);
        assert!(result.result.contains("WASI fd opened"));
    }

    #[test]
    fn test_insufficient_capabilities() {
        let shims = PolyglotShims::new();
        let result = shims.call_shim("libc", "open", 
            vec!["/tmp/test.txt".to_string(), "r".to_string()],
            &["network:connect".to_string()]
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_disable_shim() {
        let shims = PolyglotShims::new();
        
        shims.disable_shim("libc").unwrap();
        let enabled = shims.get_enabled_shims();
        assert!(!enabled.contains(&"libc".to_string()));
        
        shims.enable_shim("libc").unwrap();
        let enabled = shims.get_enabled_shims();
        assert!(enabled.contains(&"libc".to_string()));
    }
}
