use std::collections::{BTreeMap, HashMap};
use std::io::{self, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use ciborium::{de, ser};

use crate::schema::{ContentId, DirManifestV1, FileManifestV1};
use crate::cas::CasIndex;

/// Entry types for files and directories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    File = 1,
    Directory = 2,
    Symlink = 3,
    Special = 4,
}

/// Entry for added/removed files/directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub cid_ngfs: Vec<u8>,
    pub size: u64,
    pub entry_type: EntryType,
    pub mode: u16,
    pub mtime: u64,
}

/// Entry for modified files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEntry {
    pub path: String,
    pub old_cid: Vec<u8>,
    pub new_cid: Vec<u8>,
    pub old_size: u64,
    pub new_size: u64,
    pub delta_size: i64,
    pub mode: u16,
    pub mtime: u64,
}

/// Summary statistics for the diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_added: u32,
    pub total_removed: u32,
    pub total_modified: u32,
    pub bytes_added: u64,
    pub bytes_removed: u64,
    pub bytes_delta: i64,
}

/// Complete diff structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffV1 {
    pub version: u16,
    pub timestamp: String,
    pub old_snapshot: Vec<u8>,
    pub new_snapshot: Vec<u8>,
    pub added: Vec<Entry>,
    pub removed: Vec<Entry>,
    pub modified: Vec<ModEntry>,
    pub summary: DiffSummary,
}

/// Diff engine for comparing NGFS snapshots
pub struct DiffEngine {
    cas: CasIndex,
}

impl DiffEngine {
    /// Create a new diff engine
    pub fn new(cas: CasIndex) -> Self {
        Self { cas }
    }

    /// Generate a diff between two snapshots
    pub fn diff_snapshots(&self, old_cid: &[u8], new_cid: &[u8]) -> Result<DiffV1, DiffError> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Load old and new root manifests
        let old_root = self.load_directory_manifest(old_cid)?;
        let new_root = self.load_directory_manifest(new_cid)?;
        
        // Build path maps for both snapshots
        let old_paths = self.build_path_map(&old_root, "/")?;
        let new_paths = self.build_path_map(&new_root, "/")?;
        
        // Generate diff entries
        let (added, removed, modified) = self.compute_differences(&old_paths, &new_paths)?;
        
        // Calculate summary statistics
        let summary = self.calculate_summary(&added, &removed, &modified);
        
        // Sort all entries by path for deterministic output
        let mut added = added;
        let mut removed = removed;
        let mut modified = modified;
        
        added.sort_by(|a, b| a.path.cmp(&b.path));
        removed.sort_by(|a, b| a.path.cmp(&b.path));
        modified.sort_by(|a, b| a.path.cmp(&b.path));
        
        Ok(DiffV1 {
            version: 1,
            timestamp,
            old_snapshot: old_cid.to_vec(),
            new_snapshot: new_cid.to_vec(),
            added,
            removed,
            modified,
            summary,
        })
    }

    /// Write diff to output stream
    pub fn write_diff(&self, diff: &DiffV1, out: &mut dyn Write) -> Result<(), DiffError> {
        ser::into_writer(diff, out).map_err(|e| DiffError::SerializationError(e.to_string()))?;
        Ok(())
    }

    /// Load directory manifest from CAS
    fn load_directory_manifest(&self, cid: &[u8]) -> Result<DirManifestV1, DiffError> {
        // This would integrate with the actual CAS system
        // For now, we'll use placeholder logic
        let content = self.cas.read_chunk(cid)
            .map_err(|e| DiffError::CasError(e.to_string()))?;
        
        de::from_reader(&content[..])
            .map_err(|e| DiffError::DeserializationError(e.to_string()))
    }

    /// Build a map of all paths in a snapshot
    fn build_path_map(&self, manifest: &DirManifestV1, base_path: &str) -> Result<HashMap<String, Entry>, DiffError> {
        let mut paths = HashMap::new();
        self.collect_paths(manifest, base_path, &mut paths)?;
        Ok(paths)
    }

    /// Recursively collect all paths from a directory manifest
    fn collect_paths(&self, manifest: &DirManifestV1, base_path: &str, paths: &mut HashMap<String, Entry>) -> Result<(), DiffError> {
        for entry in &manifest.children {
            let entry_path = if base_path == "/" {
                format!("/{}", entry.name)
            } else {
                format!("{}/{}", base_path, entry.name)
            };
            
            let entry_type = match entry.content_type {
                crate::schema::ContentType::Directory => EntryType::Directory,
                crate::schema::ContentType::FileMeta => EntryType::File,
                crate::schema::ContentType::Symlink => EntryType::Symlink,
                _ => EntryType::Special,
            };
            
            let size = match entry_type {
                EntryType::Directory => 0,
                _ => entry.size.unwrap_or(0),
            };
            
            let entry_struct = Entry {
                path: entry_path.clone(),
                cid_ngfs: entry.cid.blake3_hash.to_vec(),
                size,
                entry_type,
                mode: entry.mode.unwrap_or(0o644),
                mtime: entry.mtime_vclock,
            };
            
            paths.insert(entry_path.clone(), entry_struct);
            
            // Recursively process directories
            if entry_type == EntryType::Directory {
                let child_manifest = self.load_directory_manifest(&entry.cid.blake3_hash)?;
                self.collect_paths(&child_manifest, &entry_path, paths)?;
            }
        }
        
        Ok(())
    }

    /// Compute differences between two path maps
    fn compute_differences(
        &self,
        old_paths: &HashMap<String, Entry>,
        new_paths: &HashMap<String, Entry>,
    ) -> Result<(Vec<Entry>, Vec<Entry>, Vec<ModEntry>), DiffError> {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();
        
        // Find added and modified entries
        for (path, new_entry) in new_paths {
            match old_paths.get(path) {
                Some(old_entry) => {
                    // Entry exists in both - check if modified
                    if old_entry.cid_ngfs != new_entry.cid_ngfs || old_entry.size != new_entry.size {
                        let delta_size = new_entry.size as i64 - old_entry.size as i64;
                        let mod_entry = ModEntry {
                            path: path.clone(),
                            old_cid: old_entry.cid_ngfs.clone(),
                            new_cid: new_entry.cid_ngfs.clone(),
                            old_size: old_entry.size,
                            new_size: new_entry.size,
                            delta_size,
                            mode: new_entry.mode,
                            mtime: new_entry.mtime,
                        };
                        modified.push(mod_entry);
                    }
                }
                None => {
                    // Entry only exists in new snapshot
                    added.push(new_entry.clone());
                }
            }
        }
        
        // Find removed entries
        for (path, old_entry) in old_paths {
            if !new_paths.contains_key(path) {
                removed.push(old_entry.clone());
            }
        }
        
        Ok((added, removed, modified))
    }

    /// Calculate summary statistics
    fn calculate_summary(&self, added: &[Entry], removed: &[Entry], modified: &[ModEntry]) -> DiffSummary {
        let total_added = added.len() as u32;
        let total_removed = removed.len() as u32;
        let total_modified = modified.len() as u32;
        
        let bytes_added: u64 = added.iter().map(|e| e.size).sum();
        let bytes_removed: u64 = removed.iter().map(|e| e.size).sum();
        let bytes_delta = bytes_added as i64 - bytes_removed as i64;
        
        DiffSummary {
            total_added,
            total_removed,
            total_modified,
            bytes_added,
            bytes_removed,
            bytes_delta,
        }
    }

    /// Validate a diff structure
    pub fn validate_diff(&self, diff: &DiffV1) -> Result<(), DiffError> {
        // Check version
        if diff.version != 1 {
            return Err(DiffError::ValidationError("Invalid version".to_string()));
        }
        
        // Check path sanity
        for entry in &diff.added {
            if !self.is_valid_path(&entry.path) {
                return Err(DiffError::ValidationError(format!("Invalid path in added: {}", entry.path)));
            }
        }
        
        for entry in &diff.removed {
            if !self.is_valid_path(&entry.path) {
                return Err(DiffError::ValidationError(format!("Invalid path in removed: {}", entry.path)));
            }
        }
        
        for entry in &diff.modified {
            if !self.is_valid_path(&entry.path) {
                return Err(DiffError::ValidationError(format!("Invalid path in modified: {}", entry.path)));
            }
        }
        
        // Check sorting
        if !self.is_sorted(&diff.added) || !self.is_sorted(&diff.removed) || !self.is_sorted(&diff.modified) {
            return Err(DiffError::ValidationError("Entries not properly sorted".to_string()));
        }
        
        // Check size consistency
        let calculated_summary = self.calculate_summary(&diff.added, &diff.removed, &diff.modified);
        if calculated_summary.total_added != diff.summary.total_added ||
           calculated_summary.total_removed != diff.summary.total_removed ||
           calculated_summary.total_modified != diff.summary.total_modified ||
           calculated_summary.bytes_added != diff.summary.bytes_added ||
           calculated_summary.bytes_removed != diff.summary.bytes_removed ||
           calculated_summary.bytes_delta != diff.summary.bytes_delta {
            return Err(DiffError::ValidationError("Summary statistics inconsistent".to_string()));
        }
        
        Ok(())
    }

    /// Check if a path is valid
    fn is_valid_path(&self, path: &str) -> bool {
        // Check for dotdot sequences
        if path.contains("..") {
            return false;
        }
        
        // Check for control characters
        if path.chars().any(|c| c.is_control()) {
            return false;
        }
        
        // Check for empty components
        if path.split('/').any(|component| component.trim().is_empty()) {
            return false;
        }
        
        true
    }

    /// Check if entries are sorted by path
    fn is_sorted(&self, entries: &[Entry]) -> bool {
        for i in 1..entries.len() {
            if entries[i-1].path >= entries[i].path {
                return false;
            }
        }
        true
    }

    /// Generate diff statistics for analysis
    pub fn generate_stats(&self, diff: &DiffV1) -> DiffStats {
        let mut file_changes = FileChanges {
            added_files: 0,
            removed_files: 0,
            modified_files: 0,
            total_files: 0,
        };
        
        let mut directory_changes = DirectoryChanges {
            added_dirs: 0,
            removed_dirs: 0,
            total_dirs: 0,
        };
        
        let mut size_analysis = SizeAnalysis {
            largest_added: None,
            largest_removed: None,
            largest_modified: None,
            avg_file_size: None,
        };
        
        let mut path_analysis = PathAnalysis {
            deepest_path: None,
            most_changed_dir: None,
            path_depth_distribution: Vec::new(),
        };
        
        // Analyze added entries
        for entry in &diff.added {
            match entry.entry_type {
                EntryType::File => {
                    file_changes.added_files += 1;
                    file_changes.total_files += 1;
                    
                    if size_analysis.largest_added.is_none() || 
                       entry.size > size_analysis.largest_added.as_ref().unwrap().size {
                        size_analysis.largest_added = Some(entry.clone());
                    }
                }
                EntryType::Directory => {
                    directory_changes.added_dirs += 1;
                    directory_changes.total_dirs += 1;
                }
                _ => {}
            }
            
            self.analyze_path(&entry.path, &mut path_analysis);
        }
        
        // Analyze removed entries
        for entry in &diff.removed {
            match entry.entry_type {
                EntryType::File => {
                    file_changes.removed_files += 1;
                    file_changes.total_files += 1;
                    
                    if size_analysis.largest_removed.is_none() || 
                       entry.size > size_analysis.largest_removed.as_ref().unwrap().size {
                        size_analysis.largest_removed = Some(entry.clone());
                    }
                }
                EntryType::Directory => {
                    directory_changes.removed_dirs += 1;
                    directory_changes.total_dirs += 1;
                }
                _ => {}
            }
            
            self.analyze_path(&entry.path, &mut path_analysis);
        }
        
        // Analyze modified entries
        for entry in &diff.modified {
            file_changes.modified_files += 1;
            file_changes.total_files += 1;
            
            if size_analysis.largest_modified.is_none() || 
               entry.delta_size.abs() > size_analysis.largest_modified.as_ref().unwrap().delta_size.abs() {
                size_analysis.largest_modified = Some(entry.clone());
            }
            
            self.analyze_path(&entry.path, &mut path_analysis);
        }
        
        // Calculate average file size
        let total_file_size: u64 = diff.added.iter()
            .filter(|e| e.entry_type == EntryType::File)
            .map(|e| e.size)
            .sum::<u64>() + diff.removed.iter()
            .filter(|e| e.entry_type == EntryType::File)
            .map(|e| e.size)
            .sum::<u64>();
        
        if file_changes.total_files > 0 {
            size_analysis.avg_file_size = Some(total_file_size / file_changes.total_files as u64);
        }
        
        DiffStats {
            file_changes,
            directory_changes,
            size_analysis,
            path_analysis,
        }
    }

    /// Analyze path characteristics
    fn analyze_path(&self, path: &str, analysis: &mut PathAnalysis) {
        let depth = path.matches('/').count();
        
        // Update deepest path
        if analysis.deepest_path.is_none() || depth > analysis.deepest_path.as_ref().unwrap().matches('/').count() {
            analysis.deepest_path = Some(path.to_string());
        }
        
        // Update path depth distribution
        while analysis.path_depth_distribution.len() <= depth {
            analysis.path_depth_distribution.push(0);
        }
        analysis.path_depth_distribution[depth] += 1;
    }
}

/// Statistics structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChanges {
    pub added_files: u32,
    pub removed_files: u32,
    pub modified_files: u32,
    pub total_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryChanges {
    pub added_dirs: u32,
    pub removed_dirs: u32,
    pub total_dirs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeAnalysis {
    pub largest_added: Option<Entry>,
    pub largest_removed: Option<Entry>,
    pub largest_modified: Option<ModEntry>,
    pub avg_file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathAnalysis {
    pub deepest_path: Option<String>,
    pub most_changed_dir: Option<String>,
    pub path_depth_distribution: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub file_changes: FileChanges,
    pub directory_changes: DirectoryChanges,
    pub size_analysis: SizeAnalysis,
    pub path_analysis: PathAnalysis,
}

/// Error types for diff operations
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("CAS error: {0}")]
    CasError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::MockCasIndex;
    
    #[test]
    fn test_diff_validation() {
        let cas = MockCasIndex::new();
        let engine = DiffEngine::new(cas);
        
        let diff = DiffV1 {
            version: 1,
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            old_snapshot: vec![1, 2, 3],
            new_snapshot: vec![4, 5, 6],
            added: vec![],
            removed: vec![],
            modified: vec![],
            summary: DiffSummary {
                total_added: 0,
                total_removed: 0,
                total_modified: 0,
                bytes_added: 0,
                bytes_removed: 0,
                bytes_delta: 0,
            },
        };
        
        assert!(engine.validate_diff(&diff).is_ok());
    }
    
    #[test]
    fn test_path_validation() {
        let cas = MockCasIndex::new();
        let engine = DiffEngine::new(cas);
        
        assert!(engine.is_valid_path("/valid/path"));
        assert!(!engine.is_valid_path("/path/with/../dotdot"));
        assert!(!engine.is_valid_path("/path/with/\0null"));
        assert!(!engine.is_valid_path("//empty//components"));
    }
    
    #[test]
    fn test_sorting_validation() {
        let cas = MockCasIndex::new();
        let engine = DiffEngine::new(cas);
        
        let sorted_entries = vec![
            Entry {
                path: "/a".to_string(),
                cid_ngfs: vec![1],
                size: 0,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
            Entry {
                path: "/b".to_string(),
                cid_ngfs: vec![2],
                size: 0,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
        ];
        
        assert!(engine.is_sorted(&sorted_entries));
        
        let unsorted_entries = vec![
            Entry {
                path: "/b".to_string(),
                cid_ngfs: vec![2],
                size: 0,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
            Entry {
                path: "/a".to_string(),
                cid_ngfs: vec![1],
                size: 0,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
        ];
        
        assert!(!engine.is_sorted(&unsorted_entries));
    }
}
