use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub path: String,
    pub mount_type: String,
    pub capabilities: Vec<String>,
    pub read_only: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub created_at: u64,
    pub modified_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub size: u64,
    pub mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Special,
}

pub struct CapabilityAwareVFS {
    mounts: Arc<Mutex<HashMap<String, MountPoint>>>,
    file_cache: Arc<Mutex<HashMap<String, FileInfo>>>,
    directory_cache: Arc<Mutex<HashMap<String, Vec<DirectoryEntry>>>>,
    virtual_clock: Arc<Mutex<u64>>,
}

impl CapabilityAwareVFS {
    pub fn new() -> Self {
        let mut vfs = Self {
            mounts: Arc::new(Mutex::new(HashMap::new())),
            file_cache: Arc::new(Mutex::new(HashMap::new())),
            directory_cache: Arc::new(Mutex::new(HashMap::new())),
            virtual_clock: Arc::new(Mutex::new(0)),
        };
        
        vfs.initialize_default_mounts();
        vfs
    }

    fn initialize_default_mounts(&mut self) {
        let mut mounts = self.mounts.lock().unwrap();
        
        mounts.insert("/snap".to_string(), MountPoint {
            path: "/snap".to_string(),
            mount_type: "ngfs-snapshot".to_string(),
            capabilities: vec!["snapshot:read".to_string()],
            read_only: true,
            created_at: self.get_virtual_time(),
        });
        
        mounts.insert("/pdv".to_string(), MountPoint {
            path: "/pdv".to_string(),
            mount_type: "personal-data-vault".to_string(),
            capabilities: vec!["vault:read".to_string(), "vault:write".to_string()],
            read_only: false,
            created_at: self.get_virtual_time(),
        });
        
        mounts.insert("/tmp".to_string(), MountPoint {
            path: "/tmp".to_string(),
            mount_type: "temporary".to_string(),
            capabilities: vec!["tmp:read".to_string(), "tmp:write".to_string()],
            read_only: false,
            created_at: self.get_virtual_time(),
        });
        
        mounts.insert("/".to_string(), MountPoint {
            path: "/".to_string(),
            mount_type: "root".to_string(),
            capabilities: vec!["root:read".to_string()],
            read_only: true,
            created_at: self.get_virtual_time(),
        });
    }

    pub fn mount(&self, path: &str, mount_type: &str, capabilities: Vec<String>, read_only: bool) -> Result<(), String> {
        if !self.check_capability(&capabilities, "vfs:mount") {
            return Err("Permission denied: insufficient mount capability".to_string());
        }
        
        let mut mounts = self.mounts.lock().unwrap();
        if mounts.contains_key(path) {
            return Err(format!("Mount point {} already exists", path));
        }
        
        let mount_point = MountPoint {
            path: path.to_string(),
            mount_type: mount_type.to_string(),
            capabilities: capabilities.clone(),
            read_only,
            created_at: self.get_virtual_time(),
        };
        
        mounts.insert(path.to_string(), mount_point);
        Ok(())
    }

    pub fn unmount(&self, path: &str, capabilities: &[String]) -> Result<(), String> {
        if !self.check_capability(capabilities, "vfs:mount") {
            return Err("Permission denied: insufficient unmount capability".to_string());
        }
        
        let mut mounts = self.mounts.lock().unwrap();
        if path == "/" || path == "/snap" || path == "/pdv" || path == "/tmp" {
            return Err("Cannot unmount protected mount points".to_string());
        }
        
        mounts.remove(path);
        Ok(())
    }

    pub fn list_mounts(&self, capabilities: &[String]) -> Result<Vec<MountPoint>, String> {
        if !self.check_capability(capabilities, "vfs:read") {
            return Err("Permission denied: insufficient read capability".to_string());
        }
        
        let mounts = self.mounts.lock().unwrap();
        Ok(mounts.values().cloned().collect())
    }

    pub fn get_mount_point(&self, path: &str) -> Option<MountPoint> {
        let mounts = self.mounts.lock().unwrap();
        
        let mut current_path = Path::new(path);
        while let Some(parent) = current_path.parent() {
            if let Some(mount) = mounts.get(parent.to_str().unwrap_or("")) {
                return Some(mount.clone());
            }
            current_path = parent;
        }
        
        mounts.get("/").cloned()
    }

    pub fn open_file(&self, path: &str, mode: &str, capabilities: &[String]) -> Result<FileInfo, String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if mode.contains('w') && mount_point.read_only {
            return Err("Mount point is read-only".to_string());
        }
        
        let required_cap = if mode.contains('w') { "write" } else { "read" };
        if !self.check_capability(capabilities, &format!("{}:{}", mount_point.mount_type, required_cap)) {
            return Err("Permission denied: insufficient capability".to_string());
        }
        
        let file_info = FileInfo {
            path: path.to_string(),
            size: 1024,
            mode: 0o644,
            uid: 1000,
            gid: 1000,
            created_at: self.get_virtual_time(),
            modified_at: self.get_virtual_time(),
        };
        
        self.file_cache.lock().unwrap().insert(path.to_string(), file_info.clone());
        Ok(file_info)
    }

    pub fn read_file(&self, path: &str, offset: u64, size: usize, capabilities: &[String]) -> Result<Vec<u8>, String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if !self.check_capability(capabilities, &format!("{}:read", mount_point.mount_type)) {
            return Err("Permission denied: insufficient read capability".to_string());
        }
        
        let file_info = self.file_cache.lock().unwrap()
            .get(path)
            .ok_or("File not found")?;
        
        if offset >= file_info.size {
            return Ok(Vec::new());
        }
        
        let read_size = size.min((file_info.size - offset) as usize);
        let mock_data = vec![b'A'; read_size];
        Ok(mock_data)
    }

    pub fn write_file(&self, path: &str, offset: u64, data: &[u8], capabilities: &[String]) -> Result<usize, String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if mount_point.read_only {
            return Err("Mount point is read-only".to_string());
        }
        
        if !self.check_capability(capabilities, &format!("{}:write", mount_point.mount_type)) {
            return Err("Permission denied: insufficient write capability".to_string());
        }
        
        let mut file_cache = self.file_cache.lock().unwrap();
        if let Some(file_info) = file_cache.get_mut(path) {
            file_info.modified_at = self.get_virtual_time();
            file_info.size = file_info.size.max(offset + data.len() as u64);
        } else {
            let new_file_info = FileInfo {
                path: path.to_string(),
                size: offset + data.len() as u64,
                mode: 0o644,
                uid: 1000,
                gid: 1000,
                created_at: self.get_virtual_time(),
                modified_at: self.get_virtual_time(),
            };
            file_cache.insert(path.to_string(), new_file_info);
        }
        
        Ok(data.len())
    }

    pub fn stat_file(&self, path: &str, capabilities: &[String]) -> Result<FileInfo, String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if !self.check_capability(capabilities, &format!("{}:read", mount_point.mount_type)) {
            return Err("Permission denied: insufficient read capability".to_string());
        }
        
        let file_cache = self.file_cache.lock().unwrap();
        if let Some(file_info) = file_cache.get(path) {
            Ok(file_info.clone())
        } else {
            Err("File not found".to_string())
        }
    }

    pub fn list_directory(&self, path: &str, capabilities: &[String]) -> Result<Vec<DirectoryEntry>, String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if !self.check_capability(capabilities, &format!("{}:read", mount_point.mount_type)) {
            return Err("Permission denied: insufficient read capability".to_string());
        }
        
        let directory_cache = self.directory_cache.lock().unwrap();
        if let Some(entries) = directory_cache.get(path) {
            Ok(entries.clone())
        } else {
            let default_entries = vec![
                DirectoryEntry {
                    name: ".".to_string(),
                    entry_type: EntryType::Directory,
                    size: 0,
                    mode: 0o755,
                },
                DirectoryEntry {
                    name: "..".to_string(),
                    entry_type: EntryType::Directory,
                    size: 0,
                    mode: 0o755,
                },
            ];
            Ok(default_entries)
        }
    }

    pub fn create_directory(&self, path: &str, mode: u32, capabilities: &[String]) -> Result<(), String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if mount_point.read_only {
            return Err("Mount point is read-only".to_string());
        }
        
        if !self.check_capability(capabilities, &format!("{}:write", mount_point.mount_type)) {
            return Err("Permission denied: insufficient write capability".to_string());
        }
        
        let mut directory_cache = self.directory_cache.lock().unwrap();
        let parent_path = Path::new(path).parent().unwrap_or(Path::new("/")).to_str().unwrap();
        
        if let Some(parent_entries) = directory_cache.get_mut(parent_path) {
            parent_entries.push(DirectoryEntry {
                name: Path::new(path).file_name().unwrap().to_str().unwrap().to_string(),
                entry_type: EntryType::Directory,
                size: 0,
                mode,
            });
        }
        
        Ok(())
    }

    pub fn remove_file(&self, path: &str, capabilities: &[String]) -> Result<(), String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if mount_point.read_only {
            return Err("Mount point is read-only".to_string());
        }
        
        if !self.check_capability(capabilities, &format!("{}:write", mount_point.mount_type)) {
            return Err("Permission denied: insufficient write capability".to_string());
        }
        
        self.file_cache.lock().unwrap().remove(path);
        Ok(())
    }

    pub fn remove_directory(&self, path: &str, capabilities: &[String]) -> Result<(), String> {
        let mount_point = self.get_mount_point(path).ok_or("No mount point found")?;
        
        if mount_point.read_only {
            return Err("Mount point is read-only".to_string());
        }
        
        if !self.check_capability(capabilities, &format!("{}:write", mount_point.mount_type)) {
            return Err("Permission denied: insufficient write capability".to_string());
        }
        
        let mut directory_cache = self.directory_cache.lock().unwrap();
        directory_cache.remove(path);
        
        let parent_path = Path::new(path).parent().unwrap_or(Path::new("/")).to_str().unwrap();
        if let Some(parent_entries) = directory_cache.get_mut(parent_path) {
            parent_entries.retain(|entry| entry.name != Path::new(path).file_name().unwrap().to_str().unwrap());
        }
        
        Ok(())
    }

    fn check_capability(&self, capabilities: &[String], required: &str) -> bool {
        capabilities.iter().any(|cap| cap == required || cap == "all" || cap == "root:all")
    }

    fn get_virtual_time(&self) -> u64 {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock += 1;
        *clock
    }

    pub fn get_mount_count(&self) -> usize {
        self.mounts.lock().unwrap().len()
    }

    pub fn get_file_count(&self) -> usize {
        self.file_cache.lock().unwrap().len()
    }

    pub fn get_directory_count(&self) -> usize {
        self.directory_cache.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_creation() {
        let vfs = CapabilityAwareVFS::new();
        assert_eq!(vfs.get_mount_count(), 4);
        assert_eq!(vfs.get_file_count(), 0);
    }

    #[test]
    fn test_default_mounts() {
        let vfs = CapabilityAwareVFS::new();
        let mounts = vfs.list_mounts(&["vfs:read".to_string()]).unwrap();
        
        let mount_paths: Vec<String> = mounts.iter().map(|m| m.path.clone()).collect();
        assert!(mount_paths.contains(&"/snap".to_string()));
        assert!(mount_paths.contains(&"/pdv".to_string()));
        assert!(mount_paths.contains(&"/tmp".to_string()));
        assert!(mount_paths.contains(&"/".to_string()));
    }

    #[test]
    fn test_open_file() {
        let vfs = CapabilityAwareVFS::new();
        let file_info = vfs.open_file("/tmp/test.txt", "r", &["tmp:read".to_string()]).unwrap();
        assert_eq!(file_info.path, "/tmp/test.txt");
        assert_eq!(file_info.size, 1024);
    }

    #[test]
    fn test_open_file_denied() {
        let vfs = CapabilityAwareVFS::new();
        let result = vfs.open_file("/tmp/test.txt", "r", &["network:connect".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_file() {
        let vfs = CapabilityAwareVFS::new();
        let data = b"Hello, World!";
        let written = vfs.write_file("/tmp/test.txt", 0, data, &["tmp:write".to_string()]).unwrap();
        assert_eq!(written, data.len());
        
        let file_info = vfs.stat_file("/tmp/test.txt", &["tmp:read".to_string()]).unwrap();
        assert_eq!(file_info.size, data.len() as u64);
    }

    #[test]
    fn test_readonly_mount() {
        let vfs = CapabilityAwareVFS::new();
        let result = vfs.write_file("/snap/test.txt", 0, b"test", &["snapshot:read".to_string()]);
        assert!(result.is_err());
    }
}
