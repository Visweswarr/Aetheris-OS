use std::collections::HashMap;
use std::io::Cursor;
use tempfile::tempdir;
use std::fs;

use ngfs::diff::{DiffEngine, DiffV1, Entry, ModEntry, EntryType, DiffSummary, DiffError};

// Mock CAS implementation for testing
struct MockCasIndex {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl MockCasIndex {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    fn add_chunk(&mut self, cid: Vec<u8>, content: Vec<u8>) {
        self.data.insert(cid, content);
    }
    
    fn read_chunk(&self, cid: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.data.get(cid)
            .cloned()
            .ok_or_else(|| "Chunk not found".into())
    }
}

impl ngfs::cas::CasIndex for MockCasIndex {
    fn read_chunk(&self, cid: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.read_chunk(cid)
    }
    
    fn write_chunk(&mut self, _content: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        unimplemented!("Mock CAS doesn't support writing")
    }
    
    fn has_chunk(&self, cid: &[u8]) -> bool {
        self.data.contains_key(cid)
    }
}

#[test]
fn test_diff_engine_creation() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Engine should be created successfully
    assert!(engine != DiffEngine::new(MockCasIndex::new()));
}

#[test]
fn test_diff_validation() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Create a valid diff
    let valid_diff = DiffV1 {
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
    
    // Valid diff should pass validation
    assert!(engine.validate_diff(&valid_diff).is_ok());
    
    // Invalid version should fail
    let mut invalid_diff = valid_diff.clone();
    invalid_diff.version = 2;
    assert!(engine.validate_diff(&invalid_diff).is_err());
}

#[test]
fn test_path_validation() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Valid paths should pass
    assert!(engine.is_valid_path("/valid/path"));
    assert!(engine.is_valid_path("/file.txt"));
    assert!(engine.is_valid_path("/dir/subdir/file"));
    
    // Invalid paths should fail
    assert!(!engine.is_valid_path("/path/with/../dotdot"));
    assert!(!engine.is_valid_path("/path/with/\0null"));
    assert!(!engine.is_valid_path("//empty//components"));
    assert!(!engine.is_valid_path("relative/path"));
}

#[test]
fn test_sorting_validation() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Sorted entries should pass
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
    
    // Unsorted entries should fail
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

#[test]
fn test_diff_statistics_generation() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    let diff = DiffV1 {
        version: 1,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
        old_snapshot: vec![1, 2, 3],
        new_snapshot: vec![4, 5, 6],
        added: vec![
            Entry {
                path: "/file1.txt".to_string(),
                cid_ngfs: vec![10],
                size: 1024,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
            Entry {
                path: "/dir1".to_string(),
                cid_ngfs: vec![11],
                size: 0,
                entry_type: EntryType::Directory,
                mode: 0o755,
                mtime: 0,
            },
        ],
        removed: vec![
            Entry {
                path: "/file2.txt".to_string(),
                cid_ngfs: vec![20],
                size: 512,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
        ],
        modified: vec![
            ModEntry {
                path: "/file3.txt".to_string(),
                old_cid: vec![30],
                new_cid: vec![31],
                old_size: 256,
                new_size: 512,
                delta_size: 256,
                mode: 0o644,
                mtime: 0,
            },
        ],
        summary: DiffSummary {
            total_added: 2,
            total_removed: 1,
            total_modified: 1,
            bytes_added: 1024,
            bytes_removed: 512,
            bytes_delta: 512,
        },
    };
    
    let stats = engine.generate_stats(&diff);
    
    // Check file changes
    assert_eq!(stats.file_changes.added_files, 1);
    assert_eq!(stats.file_changes.removed_files, 1);
    assert_eq!(stats.file_changes.modified_files, 1);
    assert_eq!(stats.file_changes.total_files, 3);
    
    // Check directory changes
    assert_eq!(stats.directory_changes.added_dirs, 1);
    assert_eq!(stats.directory_changes.removed_dirs, 0);
    assert_eq!(stats.directory_changes.total_dirs, 1);
    
    // Check size analysis
    assert!(stats.size_analysis.largest_added.is_some());
    assert!(stats.size_analysis.largest_removed.is_some());
    assert!(stats.size_analysis.largest_modified.is_some());
    assert_eq!(stats.size_analysis.avg_file_size, Some(896)); // (1024 + 512 + 256) / 3
    
    // Check path analysis
    assert!(stats.path_analysis.deepest_path.is_some());
    assert_eq!(stats.path_analysis.path_depth_distribution.len(), 2); // depth 0 and 1
}

#[test]
fn test_diff_serialization() {
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
    
    // Test writing to a buffer
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    
    // This would test the actual serialization
    // For now, we'll just verify the diff structure is valid
    assert!(engine.validate_diff(&diff).is_ok());
}

#[test]
fn test_diff_error_handling() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Test with invalid diff
    let invalid_diff = DiffV1 {
        version: 1,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
        old_snapshot: vec![1, 2, 3],
        new_snapshot: vec![4, 5, 6],
        added: vec![
            Entry {
                path: "/invalid/../path".to_string(), // Invalid path
                cid_ngfs: vec![10],
                size: 1024,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
        ],
        removed: vec![],
        modified: vec![],
        summary: DiffSummary {
            total_added: 1,
            total_removed: 0,
            total_modified: 0,
            bytes_added: 1024,
            bytes_removed: 0,
            bytes_delta: 1024,
        },
    };
    
    // Invalid diff should fail validation
    assert!(engine.validate_diff(&invalid_diff).is_err());
}

#[test]
fn test_diff_summary_consistency() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Create a diff with inconsistent summary
    let inconsistent_diff = DiffV1 {
        version: 1,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
        old_snapshot: vec![1, 2, 3],
        new_snapshot: vec![4, 5, 6],
        added: vec![
            Entry {
                path: "/file1.txt".to_string(),
                cid_ngfs: vec![10],
                size: 1024,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
        ],
        removed: vec![],
        modified: vec![],
        summary: DiffSummary {
            total_added: 0, // Inconsistent: should be 1
            total_removed: 0,
            total_modified: 0,
            bytes_added: 1024,
            bytes_removed: 0,
            bytes_delta: 1024,
        },
    };
    
    // Inconsistent diff should fail validation
    assert!(engine.validate_diff(&inconsistent_diff).is_err());
}

#[test]
fn test_diff_entry_types() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Test different entry types
    let diff_with_types = DiffV1 {
        version: 1,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
        old_snapshot: vec![1, 2, 3],
        new_snapshot: vec![4, 5, 6],
        added: vec![
            Entry {
                path: "/file.txt".to_string(),
                cid_ngfs: vec![10],
                size: 1024,
                entry_type: EntryType::File,
                mode: 0o644,
                mtime: 0,
            },
            Entry {
                path: "/dir".to_string(),
                cid_ngfs: vec![11],
                size: 0,
                entry_type: EntryType::Directory,
                mode: 0o755,
                mtime: 0,
            },
            Entry {
                path: "/link".to_string(),
                cid_ngfs: vec![12],
                size: 0,
                entry_type: EntryType::Symlink,
                mode: 0o777,
                mtime: 0,
            },
        ],
        removed: vec![],
        modified: vec![],
        summary: DiffSummary {
            total_added: 3,
            total_removed: 0,
            total_modified: 0,
            bytes_added: 1024,
            bytes_removed: 0,
            bytes_delta: 1024,
        },
    };
    
    // Valid diff should pass validation
    assert!(engine.validate_diff(&diff_with_types).is_ok());
    
    let stats = engine.generate_stats(&diff_with_types);
    
    // Check that different types are counted correctly
    assert_eq!(stats.file_changes.added_files, 1);
    assert_eq!(stats.directory_changes.added_dirs, 1);
    // Symlinks are not counted as files or directories
}

#[test]
fn test_diff_performance_limits() {
    let cas = MockCasIndex::new();
    let engine = DiffEngine::new(cas);
    
    // Create a diff with many entries (but within reasonable limits)
    let mut many_entries = Vec::new();
    for i in 0..1000 {
        many_entries.push(Entry {
            path: format!("/file{}.txt", i),
            cid_ngfs: vec![i as u8],
            size: 1024,
            entry_type: EntryType::File,
            mode: 0o644,
            mtime: 0,
        });
    }
    
    let large_diff = DiffV1 {
        version: 1,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
        old_snapshot: vec![1, 2, 3],
        new_snapshot: vec![4, 5, 6],
        added: many_entries,
        removed: vec![],
        modified: vec![],
        summary: DiffSummary {
            total_added: 1000,
            total_removed: 0,
            total_modified: 0,
            bytes_added: 1000 * 1024,
            bytes_removed: 0,
            bytes_delta: 1000 * 1024,
        },
    };
    
    // Large but valid diff should pass validation
    assert!(engine.validate_diff(&large_diff).is_ok());
    
    // Check that statistics are generated correctly
    let stats = engine.generate_stats(&large_diff);
    assert_eq!(stats.file_changes.total_files, 1000);
    assert_eq!(stats.size_analysis.avg_file_size, Some(1024));
}
