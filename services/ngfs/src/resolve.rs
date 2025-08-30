use crate::cas::CasIndex;
use crate::manifest::{DirManifestV1, FileManifestV1};
use crate::schema::{Cid, EntryKindV1, EntryV1};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum NodeRef {
    File { cid: Cid, size: u64 },
    Dir { cid: Cid },
    Symlink { target: String },
}

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("Path not found: {0}")]
    NotFound(String),
    #[error("Invalid path component: {0}")]
    InvalidComponent(String),
    #[error("Not a directory: {0}")]
    NotDirectory(String),
    #[error("Not a file: {0}")]
    NotFile(String),
    #[error("CAS error: {0}")]
    CasError(String),
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
}

pub struct PathResolver {
    cas: CasIndex,
}

impl PathResolver {
    pub fn new(cas: CasIndex) -> Self {
        Self { cas }
    }

    pub fn resolve_path(&self, root_cid: &Cid, abs_path: &str) -> Result<NodeRef, ResolveError> {
        let path = Path::new(abs_path);
        
        if !path.is_absolute() {
            return Err(ResolveError::InvalidComponent("path must be absolute".to_string()));
        }

        if path == Path::new("/") {
            return self.resolve_dir_manifest(root_cid);
        }

        let mut current_cid = root_cid.clone();
        let mut components = path.components().peekable();

        while let Some(component) = components.next() {
            match component {
                Component::RootDir => continue,
                Component::Normal(name) => {
                    let name_str = self.normalize_name(name.to_string_lossy())?;
                    
                    if components.peek().is_some() {
                        current_cid = self.resolve_dir_component(&current_cid, &name_str)?;
                    } else {
                        return self.resolve_final_component(&current_cid, &name_str);
                    }
                }
                Component::ParentDir => {
                    return Err(ResolveError::InvalidComponent(".. not allowed".to_string()));
                }
                Component::CurDir => {
                    return Err(ResolveError::InvalidComponent(". not allowed".to_string()));
                }
                Component::Prefix(_) => {
                    return Err(ResolveError::InvalidComponent("prefix not allowed".to_string()));
                }
            }
        }

        Err(ResolveError::NotFound("unexpected end of path".to_string()))
    }

    fn normalize_name(&self, name: std::borrow::Cow<str>) -> Result<String, ResolveError> {
        let name = name.to_string();
        
        if name.is_empty() {
            return Err(ResolveError::InvalidComponent("empty component".to_string()));
        }
        
        if name.contains('\0') {
            return Err(ResolveError::InvalidComponent("NUL in component".to_string()));
        }
        
        if name == "." || name == ".." {
            return Err(ResolveError::InvalidComponent(format!("invalid component: {}", name)));
        }
        
        if name.contains('/') {
            return Err(ResolveError::InvalidComponent("slash in component".to_string()));
        }
        
        if name.len() > 255 {
            return Err(ResolveError::InvalidComponent("component too long".to_string()));
        }

        Ok(name)
    }

    fn resolve_dir_component(&self, dir_cid: &Cid, name: &str) -> Result<Cid, ResolveError> {
        let manifest = self.resolve_dir_manifest(dir_cid)?;
        
        for entry in &manifest.entries {
            if entry.name == name {
                match entry.kind {
                    EntryKindV1::Dir => return Ok(entry.cid.clone()),
                    EntryKindV1::File => {
                        return Err(ResolveError::NotDirectory(format!("{} is a file", name)));
                    }
                    EntryKindV1::Symlink => {
                        return Err(ResolveError::NotDirectory(format!("{} is a symlink", name)));
                    }
                }
            }
        }
        
        Err(ResolveError::NotFound(format!("directory entry not found: {}", name)))
    }

    fn resolve_final_component(&self, dir_cid: &Cid, name: &str) -> Result<NodeRef, ResolveError> {
        let manifest = self.resolve_dir_manifest(dir_cid)?;
        
        for entry in &manifest.entries {
            if entry.name == name {
                match entry.kind {
                    EntryKindV1::Dir => {
                        return Ok(NodeRef::Dir { cid: entry.cid.clone() });
                    }
                    EntryKindV1::File => {
                        let file_manifest = self.resolve_file_manifest(&entry.cid)?;
                        return Ok(NodeRef::File {
                            cid: entry.cid.clone(),
                            size: file_manifest.total_size,
                        });
                    }
                    EntryKindV1::Symlink => {
                        return Ok(NodeRef::Symlink {
                            target: entry.name.clone(),
                        });
                    }
                }
            }
        }
        
        Err(ResolveError::NotFound(format!("entry not found: {}", name)))
    }

    fn resolve_dir_manifest(&self, cid: &Cid) -> Result<DirManifestV1, ResolveError> {
        let data = self.cas.retrieve(cid)
            .map_err(|e| ResolveError::CasError(format!("failed to retrieve dir manifest: {}", e)))?;
        
        serde_cbor::from_slice(&data)
            .map_err(|e| ResolveError::InvalidManifest(format!("invalid dir manifest CBOR: {}", e)))
    }

    fn resolve_file_manifest(&self, cid: &Cid) -> Result<FileManifestV1, ResolveError> {
        let data = self.cas.retrieve(cid)
            .map_err(|e| ResolveError::CasError(format!("failed to retrieve file manifest: {}", e)))?;
        
        serde_cbor::from_slice(&data)
            .map_err(|e| ResolveError::InvalidManifest(format!("invalid file manifest CBOR: {}", e)))
    }

    pub fn list_directory(&self, dir_cid: &Cid) -> Result<Vec<EntryV1>, ResolveError> {
        let manifest = self.resolve_dir_manifest(dir_cid)?;
        Ok(manifest.entries)
    }

    pub fn get_file_info(&self, file_cid: &Cid) -> Result<(u64, Vec<(Cid, u32)>), ResolveError> {
        let manifest = self.resolve_file_manifest(file_cid)?;
        Ok((manifest.total_size, manifest.chunks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::CasIndex;
    use crate::manifest::{DirManifestV1, FileManifestV1};
    use crate::schema::{EntryKindV1, EntryV1};
    use std::collections::HashMap;

    fn create_test_cas() -> CasIndex {
        let mut cas = CasIndex::new("test_cas".into());
        
        let file_manifest = FileManifestV1 {
            version: 1,
            chunks: vec![("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(), 100)],
            total_size: 100,
            algo: "blake3".to_string(),
        };
        
        let dir_manifest = DirManifestV1 {
            version: 1,
            entries: vec![
                EntryV1 {
                    name: "file.txt".to_string(),
                    kind: EntryKindV1::File,
                    cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                    size: Some(100),
                    mode: Some(0o644),
                    xattrs: None,
                },
                EntryV1 {
                    name: "subdir".to_string(),
                    kind: EntryKindV1::Dir,
                    cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                    size: None,
                    mode: Some(0o755),
                    xattrs: None,
                },
            ],
        };
        
        let file_data = serde_cbor::to_vec(&file_manifest).unwrap();
        let dir_data = serde_cbor::to_vec(&dir_manifest).unwrap();
        
        cas.store(&file_data).unwrap();
        cas.store(&dir_data).unwrap();
        
        cas
    }

    #[test]
    fn test_resolve_root() {
        let cas = create_test_cas();
        let resolver = PathResolver::new(cas);
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        
        let result = resolver.resolve_path(&root_cid.to_string(), "/");
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_file() {
        let cas = create_test_cas();
        let resolver = PathResolver::new(cas);
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        
        let result = resolver.resolve_path(&root_cid.to_string(), "/file.txt");
        assert!(matches!(result, Ok(NodeRef::File { .. })));
    }

    #[test]
    fn test_resolve_subdir() {
        let cas = create_test_cas();
        let resolver = PathResolver::new(cas);
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        
        let result = resolver.resolve_path(&root_cid.to_string(), "/subdir");
        assert!(matches!(result, Ok(NodeRef::Dir { .. })));
    }

    #[test]
    fn test_invalid_path_relative() {
        let cas = create_test_cas();
        let resolver = PathResolver::new(cas);
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        
        let result = resolver.resolve_path(&root_cid.to_string(), "file.txt");
        assert!(matches!(result, Err(ResolveError::InvalidComponent(_))));
    }

    #[test]
    fn test_invalid_path_parent() {
        let cas = create_test_cas();
        let resolver = PathResolver::new(cas);
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        
        let result = resolver.resolve_path(&root_cid.to_string(), "/../file.txt");
        assert!(matches!(result, Err(ResolveError::InvalidComponent(_))));
    }
}
