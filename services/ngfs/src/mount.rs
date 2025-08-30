use crate::schema::{Cid, MountOptionsV1};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Mount {
    pub root: Cid,
    pub salt: [u8; 16],
    pub vclock: u64,
}

#[derive(Debug)]
pub struct MountTable {
    mounts: HashMap<String, Mount>,
}

#[derive(Error, Debug)]
pub enum MountError {
    #[error("Invalid mount point: {0}")]
    InvalidMountPoint(String),
    #[error("Mount point already exists: {0}")]
    AlreadyMounted(String),
    #[error("Mount point overlaps with existing mount: {0}")]
    Overlap(String),
    #[error("Root CID not found: {0}")]
    RootNotFound(String),
}

impl MountTable {
    pub fn new() -> Self {
        Self {
            mounts: HashMap::new(),
        }
    }

    pub fn mount(&mut self, mount_point: &str, options: MountOptionsV1) -> Result<(), MountError> {
        self.validate_mount_point(mount_point)?;
        self.check_overlap(mount_point)?;

        let mount = Mount {
            root: options.root_cid,
            salt: options.salt,
            vclock: options.vclock_base,
        };

        self.mounts.insert(mount_point.to_string(), mount);
        Ok(())
    }

    pub fn unmount(&mut self, mount_point: &str) -> Option<Mount> {
        self.mounts.remove(mount_point)
    }

    pub fn get_mount(&self, mount_point: &str) -> Option<&Mount> {
        self.mounts.get(mount_point)
    }

    pub fn list_mounts(&self) -> Vec<(&String, &Mount)> {
        self.mounts.iter().collect()
    }

    fn validate_mount_point(&self, mount_point: &str) -> Result<(), MountError> {
        if mount_point.is_empty() {
            return Err(MountError::InvalidMountPoint("empty mount point".to_string()));
        }

        if !mount_point.starts_with('/') {
            return Err(MountError::InvalidMountPoint(
                "mount point must be absolute".to_string(),
            ));
        }

        if !mount_point.starts_with("/ro/") {
            return Err(MountError::InvalidMountPoint(
                "mount point must be under /ro/".to_string(),
            ));
        }

        let path = Path::new(mount_point);
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();
                if name_str == "." || name_str == ".." {
                    return Err(MountError::InvalidMountPoint(
                        "mount point contains . or ..".to_string(),
                    ));
                }
                if name_str.contains('\0') {
                    return Err(MountError::InvalidMountPoint(
                        "mount point contains NUL".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn check_overlap(&self, mount_point: &str) -> Result<(), MountError> {
        let new_path = Path::new(mount_point);
        
        for (existing_mp, _) in &self.mounts {
            let existing_path = Path::new(existing_mp);
            
            if new_path.starts_with(existing_path) || existing_path.starts_with(new_path) {
                return Err(MountError::Overlap(existing_mp.clone()));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::MountOptionsV1;

    fn test_mount_options() -> MountOptionsV1 {
        MountOptionsV1 {
            root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
            salt: [1; 16],
            vclock_base: 1000,
        }
    }

    #[test]
    fn test_valid_mount_point() {
        let mut table = MountTable::new();
        let result = table.mount("/ro/test", test_mount_options());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_mount_point_relative() {
        let mut table = MountTable::new();
        let result = table.mount("test", test_mount_options());
        assert!(matches!(result, Err(MountError::InvalidMountPoint(_))));
    }

    #[test]
    fn test_invalid_mount_point_not_ro() {
        let mut table = MountTable::new();
        let result = table.mount("/tmp/test", test_mount_options());
        assert!(matches!(result, Err(MountError::InvalidMountPoint(_))));
    }

    #[test]
    fn test_mount_overlap() {
        let mut table = MountTable::new();
        table.mount("/ro/test", test_mount_options()).unwrap();
        
        let result = table.mount("/ro/test/sub", test_mount_options());
        assert!(matches!(result, Err(MountError::Overlap(_))));
    }

    #[test]
    fn test_mount_already_exists() {
        let mut table = MountTable::new();
        table.mount("/ro/test", test_mount_options()).unwrap();
        
        let result = table.mount("/ro/test", test_mount_options());
        assert!(matches!(result, Err(MountError::AlreadyMounted(_))));
    }

    #[test]
    fn test_list_mounts() {
        let mut table = MountTable::new();
        table.mount("/ro/test1", test_mount_options()).unwrap();
        table.mount("/ro/test2", test_mount_options()).unwrap();
        
        let mounts = table.list_mounts();
        assert_eq!(mounts.len(), 2);
    }
}
