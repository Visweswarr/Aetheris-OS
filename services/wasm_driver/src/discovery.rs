//! Driver Discovery Module
//!
//! Scans for and loads WASM driver modules.
//! Requirement: 12.2 - Driver discovery and loading

use std::fs;
use std::path::{Path, PathBuf};

/// Driver manifest (parsed from driver metadata)
#[derive(Debug, Clone)]
pub struct DriverManifest {
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub supported_devices: Vec<String>,
    pub entry_point: String,
}

/// Discovery service
pub struct DriverDiscovery {
    search_paths: Vec<PathBuf>,
}

impl DriverDiscovery {
    pub fn new() -> Self {
        Self {
            search_paths: vec![
                PathBuf::from("/drivers"),
                PathBuf::from("/usr/lib/polymera/drivers"),
            ],
        }
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Scan for available drivers
    pub fn scan(&self) -> Vec<DiscoveredDriver> {
        let mut drivers = Vec::new();

        for path in &self.search_paths {
            if path.exists() && path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let file_path = entry.path();
                        if file_path.extension().map_or(false, |e| e == "wasm") {
                            if let Some(driver) = self.parse_driver(&file_path) {
                                drivers.push(driver);
                            }
                        }
                    }
                }
            }
        }

        drivers
    }

    /// Parse driver metadata from WASM file
    fn parse_driver(&self, path: &Path) -> Option<DiscoveredDriver> {
        let name = path.file_stem()?.to_string_lossy().to_string();

        Some(DiscoveredDriver {
            name,
            path: path.to_path_buf(),
            manifest: None, // Would parse custom section in WASM
        })
    }
}

/// Discovered driver info
#[derive(Debug, Clone)]
pub struct DiscoveredDriver {
    pub name: String,
    pub path: PathBuf,
    pub manifest: Option<DriverManifest>,
}
