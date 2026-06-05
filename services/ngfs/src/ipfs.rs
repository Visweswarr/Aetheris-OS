use crate::{
    schema::{IpfsMapV1, IpfsMapEntryV1, ExportOpts, ContentId, EntryKindV1},
    cid::Cid,
    cas::{CasIndex, CasError},
    manifest::{DirManifestBuilder, FileManifestBuilder},
    enc::NgfsEncryption,
};
use std::{string::String, vec::Vec, collections::BTreeMap};
use core::cmp::Ordering;
use serde_cbor;

/// IPFS export service for NGFS
pub struct IpfsExporter {
    cas_index: CasIndex,
    encryption: NgfsEncryption,
}

impl IpfsExporter {
    /// Create a new IPFS exporter
    pub fn new(cas_index: CasIndex, encryption: NgfsEncryption) -> Self {
        Self {
            cas_index,
            encryption,
        }
    }

    /// Export IPFS map for a root NGFS CID
    pub fn export_ipfs_map(&self, root: &Cid, opts: &ExportOpts) -> Result<IpfsMapV1, IpfsExportError> {
        let mut entries = Vec::new();
        let mut visited = BTreeMap::new();
        
        // Start traversal from root
        self.traverse_for_ipfs(root, opts, &mut entries, &mut visited)?;
        
        // Sort entries by ngfs_cid bytes for determinism
        entries.sort_by(|a, b| {
            a.ngfs_cid.cmp(&b.ngfs_cid)
        });
        
        // Apply entry limit if specified
        if let Some(max_entries) = opts.max_entries {
            if entries.len() > max_entries as usize {
                return Err(IpfsExportError::TooManyEntries(entries.len(), max_entries as usize));
            }
        }
        
        Ok(IpfsMapV1 {
            version: 1,
            exported_vclock: self.get_virtual_clock(),
            root_ngfs_cid: root.to_bytes(),
            entries,
        })
    }

    /// Write CAR file for exported content
    pub fn write_car<W: core::fmt::Write>(
        &self, 
        root: &Cid, 
        map: &IpfsMapV1, 
        out: &mut W
    ) -> Result<(), IpfsExportError> {
        // CAR v1 header
        writeln!(out, "CAR v1")?;
        
        // Write each entry as a CAR block
        for entry in &map.entries {
            if let Some(data) = self.get_content_data(&entry.ngfs_cid)? {
                // CAR block format: [length][cid][data]
                let cid_bytes = entry.ipfs_cid.as_bytes();
                let block_header = format!("{} {}\n", data.len(), entry.ipfs_cid);
                out.write_str(&block_header)?;
                out.write_str(&String::from_utf8_lossy(&data))?;
                out.write_str("\n")?;
            }
        }
        
        Ok(())
    }

    /// Traverse NGFS tree and collect IPFS mapping information
    fn traverse_for_ipfs(
        &self,
        cid: &Cid,
        opts: &ExportOpts,
        entries: &mut Vec<IpfsMapEntryV1>,
        visited: &mut BTreeMap<Vec<u8>, bool>,
    ) -> Result<(), IpfsExportError> {
        let cid_bytes = cid.to_bytes();
        
        // Skip if already visited
        if visited.contains_key(&cid_bytes) {
            return Ok(());
        }
        visited.insert(cid_bytes.clone(), true);
        
        // Try to load as directory manifest first
        if let Ok(dir_manifest) = self.load_dir_manifest(cid) {
            let ipfs_cid = self.compute_ipfs_cid_dag_cbor(&dir_manifest)?;
            
            entries.push(IpfsMapEntryV1 {
                ngfs_cid: cid_bytes,
                ipfs_cid,
                kind: EntryKindV1::Directory as u8,
                size: dir_manifest.len() as u64,
            });
            
            // Recursively traverse children
            for entry in &dir_manifest.entries {
                if let Some(child_cid) = entry.cid.as_ref() {
                    self.traverse_for_ipfs(child_cid, opts, entries, visited)?;
                }
            }
            return Ok(());
        }
        
        // Try to load as file manifest
        if let Ok(file_manifest) = self.load_file_manifest(cid) {
            let ipfs_cid = self.compute_ipfs_cid_dag_cbor(&file_manifest)?;
            
            entries.push(IpfsMapEntryV1 {
                ngfs_cid: cid_bytes.clone(),
                ipfs_cid,
                kind: EntryKindV1::File as u8,
                size: file_manifest.total_size,
            });
            
            // Include chunks if requested
            if opts.include_chunks {
                for chunk_info in &file_manifest.chunks {
                    if let Some(chunk_cid) = chunk_info.cid.as_ref() {
                        self.traverse_chunk(chunk_cid, opts, entries, visited)?;
                    }
                }
            }
            return Ok(());
        }
        
        // Try to load as raw chunk
        if opts.include_chunks {
            self.traverse_chunk(cid, opts, entries, visited)?;
        }
        
        Ok(())
    }

    /// Traverse a single chunk for IPFS mapping
    fn traverse_chunk(
        &self,
        cid: &Cid,
        opts: &ExportOpts,
        entries: &mut Vec<IpfsMapEntryV1>,
        visited: &mut BTreeMap<Vec<u8>, bool>,
    ) -> Result<(), IpfsExportError> {
        let cid_bytes = cid.to_bytes();
        
        if visited.contains_key(&cid_bytes) {
            return Ok(());
        }
        visited.insert(cid_bytes.clone(), true);
        
        // Get encrypted chunk data
        if let Ok(chunk_data) = self.get_chunk_data(cid) {
            let ipfs_cid = self.compute_ipfs_cid_raw(&chunk_data)?;
            
            entries.push(IpfsMapEntryV1 {
                ngfs_cid: cid_bytes,
                ipfs_cid,
                kind: 2, // Chunk
                size: chunk_data.len() as u64,
            });
        }
        
        Ok(())
    }

    /// Load directory manifest from CAS
    fn load_dir_manifest(&self, cid: &Cid) -> Result<serde_cbor::Value, IpfsExportError> {
        let data = self.cas_index.get_chunk(cid)
            .map_err(|_| IpfsExportError::ChunkNotFound)?;
        
        serde_cbor::from_slice(&data)
            .map_err(|e| IpfsExportError::InvalidManifest(e.to_string()))
    }

    /// Load file manifest from CAS
    fn load_file_manifest(&self, cid: &Cid) -> Result<serde_cbor::Value, IpfsExportError> {
        let data = self.cas_index.get_chunk(cid)
            .map_err(|_| IpfsExportError::ChunkNotFound)?;
        
        serde_cbor::from_slice(&data)
            .map_err(|e| IpfsExportError::InvalidManifest(e.to_string()))
    }

    /// Get chunk data from CAS
    fn get_chunk_data(&self, cid: &Cid) -> Result<Vec<u8>, IpfsExportError> {
        self.cas_index.get_chunk(cid)
            .map_err(|_| IpfsExportError::ChunkNotFound)
    }

    /// Get content data for CAR export
    fn get_content_data(&self, ngfs_cid: &[u8]) -> Result<Option<Vec<u8>>, IpfsExportError> {
        // Try to reconstruct Cid from bytes
        let cid = Cid::from_bytes(ngfs_cid)
            .map_err(|_| IpfsExportError::InvalidCid)?;
        
        // Get the data
        self.get_chunk_data(&cid).map(Some)
    }

    /// Compute IPFS CIDv1 for DAG-CBOR data
    fn compute_ipfs_cid_dag_cbor(&self, data: &serde_cbor::Value) -> Result<String, IpfsExportError> {
        let cbor_bytes = serde_cbor::to_vec(data)
            .map_err(|e| IpfsExportError::SerializationError(e.to_string()))?;
        
        // For now, use a placeholder - in real implementation this would call the C library
        // via FFI to compute the actual CIDv1
        Ok(format!("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"))
    }

    /// Compute IPFS CIDv1 for raw data
    fn compute_ipfs_cid_raw(&self, data: &[u8]) -> Result<String, IpfsExportError> {
        // For now, use a placeholder - in real implementation this would call the C library
        // via FFI to compute the actual CIDv1
        Ok(format!("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"))
    }

    /// Get current virtual clock value
    fn get_virtual_clock(&self) -> u64 {
        // For now, return a placeholder - in real implementation this would get from the service
        1000
    }
}

/// IPFS export errors
#[derive(Debug, thiserror::Error)]
pub enum IpfsExportError {
    #[error("Chunk not found in CAS: {0}")]
    ChunkNotFound,
    
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Too many entries: {0} > {1}")]
    TooManyEntries(usize, usize),
    
    #[error("Invalid CID: {0}")]
    InvalidCid(String),
    
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<core::fmt::Error> for IpfsExportError {
    fn from(err: core::fmt::Error) -> Self {
        IpfsExportError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::MockCasIndex;
    use crate::enc::MockKeyVault;
    
    #[test]
    fn test_export_opts_default() {
        let opts = ExportOpts {
            include_chunks: true,
            generate_car: false,
            max_entries: Some(1000),
            max_car_size: Some(1024 * 1024),
        };
        
        assert!(opts.include_chunks);
        assert!(!opts.generate_car);
        assert_eq!(opts.max_entries, Some(1000));
        assert_eq!(opts.max_car_size, Some(1024 * 1024));
    }
    
    #[test]
    fn test_ipfs_map_entry_ordering() {
        let entry1 = IpfsMapEntryV1 {
            ngfs_cid: vec![1, 2, 3],
            ipfs_cid: "bafy1".to_string(),
            kind: 0,
            size: 100,
        };
        
        let entry2 = IpfsMapEntryV1 {
            ngfs_cid: vec![1, 2, 4],
            ipfs_cid: "bafy2".to_string(),
            kind: 1,
            size: 200,
        };
        
        assert!(entry1.ngfs_cid < entry2.ngfs_cid);
    }
}
