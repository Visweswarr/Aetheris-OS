//! NGFS v1 - Content-Addressed Filesystem with DID-Bound Encryption
//! 
//! This crate provides the core implementation of NGFS v1, including:
//! - Content addressing with Blake3/IPFS compatibility
//! - CAS (Content-Addressed Storage) with segment-based storage
//! - Deterministic schemas with CBOR encoding
//! - Crash-safe indexing and recovery
//! - Envelope encryption with XChaCha20-Poly1305

pub mod schema;
pub mod cid;
pub mod cas;
pub mod enc;
pub mod manifest;
pub mod mount;
pub mod resolve;
pub mod read;
pub mod snapshot;
pub mod ipfs;

// Re-export main types for convenience
pub use schema::*;
pub use cid::{Cid, cid_from_bytes, verify_chunk, CidError};
pub use cas::{CasSegmentStore, CasIndex, CasError, CasStats};
pub use enc::{NgfsEncryption, EncEnvelopeV1, AssociatedData, Dek, Kek, Nonce, EncError, VirtualClock, KeyVaultClient, DefaultVirtualClock};
pub use manifest::{DirManifestBuilder, FileManifestBuilder, build_dir_manifest, build_file_manifest, compute_cid, verify_proof, ManifestError};
pub use mount::{MountTable, Mount, MountError};
pub use resolve::{PathResolver, NodeRef, ResolveError};
pub use read::{FileReader, ReadError};
pub use snapshot::{SnapshotBuilder, SnapshotError};
pub use ipfs::{IpfsExporter, IpfsExportError, ExportOpts, IpfsMapV1, IpfsMapEntryV1};

/// NGFS service configuration
#[derive(Debug, Clone)]
pub struct NgfsConfig {
    /// Base path for storage
    pub storage_path: String,
    /// Maximum chunk size
    pub max_chunk_size: usize,
    /// Segment size
    pub segment_size: u64,
    /// Enable IPFS compatibility
    pub ipfs_compat: bool,
    /// Enable DID encryption
    pub did_encryption: bool,
    /// Mount salt for nonce generation
    pub mount_salt: [u8; 16],
}

impl Default for NgfsConfig {
    fn default() -> Self {
        Self {
            storage_path: "/var/lib/ngfs".to_string(),
            max_chunk_size: cas::NGFS_MAX_CHUNK_SIZE,
            segment_size: cas::NGFS_SEGMENT_SIZE,
            ipfs_compat: true,
            did_encryption: true,
            mount_salt: [0u8; 16],
        }
    }
}

/// NGFS service instance
pub struct NgfsService {
    /// CAS storage backend
    cas_store: CasSegmentStore,
    /// Encryption service
    encryption: NgfsEncryption,
    /// Mount table
    mount_table: MountTable,
    /// Path resolver
    path_resolver: PathResolver,
    /// File reader
    file_reader: FileReader,
    /// Snapshot builder
    snapshot_builder: SnapshotBuilder,
    /// IPFS exporter
    ipfs_exporter: IpfsExporter,
    /// Service configuration
    config: NgfsConfig,
}

impl NgfsService {
    /// Create a new NGFS service
    pub fn new(
        config: NgfsConfig,
        keyvault: Box<dyn KeyVaultClient>,
    ) -> Result<Self, CasError> {
        let cas_store = CasSegmentStore::new(&config.storage_path)?;
        
        let virtual_clock = Box::new(DefaultVirtualClock::new(config.mount_salt));
        let encryption = NgfsEncryption::new(virtual_clock, keyvault);
        
        let cas_index = CasIndex::new(&config.storage_path);
        let path_resolver = PathResolver::new(cas_index.clone());
        let file_reader = FileReader::new(cas_index.clone(), encryption.clone());
        let snapshot_builder = SnapshotBuilder::new();
        let ipfs_exporter = IpfsExporter::new(cas_index.clone(), encryption.clone());
        
        Ok(Self {
            cas_store,
            encryption,
            mount_table: MountTable::new(),
            path_resolver,
            file_reader,
            snapshot_builder,
            ipfs_exporter,
            config,
        })
    }

    /// Store encrypted data and return CID
    pub fn store_encrypted(
        &mut self,
        kid: String,
        plaintext: &[u8],
        metadata: std::collections::BTreeMap<String, String>,
    ) -> Result<Cid, Box<dyn std::error::Error>> {
        // Generate a random DEK for this chunk
        let dek = Dek::random();
        
        // Create associated data
        let ad = AssociatedData::for_chunk(
            schema::ngfs_schema_hash(),
            kid.clone(),
            EncryptionAlg::XChaCha20Poly1305,
            plaintext.len() as u64,
            metadata,
        );
        
        // Encrypt the data
        let envelope = self.encryption.seal_envelope(kid, dek, &ad, plaintext)?;
        
        // Store the encrypted envelope
        let envelope_bytes = serde_cbor::to_vec(&envelope)?;
        let cid = self.cas_store.append_chunk(&envelope_bytes)?;
        
        Ok(cid)
    }

    /// Store data and return CID (unencrypted, for testing)
    pub fn store(&mut self, data: &[u8]) -> Result<Cid, CasError> {
        self.cas_store.append_chunk(data)
    }

    /// Retrieve and decrypt data by CID
    pub fn retrieve_encrypted(
        &mut self,
        cid: &Cid,
        kid: &str,
        metadata: std::collections::BTreeMap<String, String>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Retrieve the encrypted envelope
        let envelope_bytes = self.cas_store.read_chunk(cid)?;
        let envelope: EncEnvelopeV1 = serde_cbor::from_slice(&envelope_bytes)?;
        
        // Create associated data for decryption
        let ad = AssociatedData::for_chunk(
            schema::ngfs_schema_hash(),
            kid.to_string(),
            EncryptionAlg::XChaCha20Poly1305,
            envelope.enc_data.len() as u64,
            metadata,
        );
        
        // Decrypt the data
        let plaintext = self.encryption.open_envelope(kid, &envelope, &ad)?;
        
        Ok(plaintext)
    }

    /// Retrieve data by CID (unencrypted, for testing)
    pub fn retrieve(&mut self, cid: &Cid) -> Result<Vec<u8>, CasError> {
        self.cas_store.read_chunk(cid)
    }

    /// Get service statistics
    pub fn get_stats(&self) -> CasStats {
        self.cas_store.get_stats()
    }

    /// Flush all data to disk
    pub fn flush(&mut self) -> Result<(), CasError> {
        self.cas_store.flush()
    }

    /// Recover from crash
    pub fn recover(&mut self) -> Result<(), CasError> {
        self.cas_store.recover()
    }

    /// Mount a read-only filesystem
    pub fn mount_ro(&mut self, mount_point: &str, options: MountOptionsV1) -> Result<(), MountError> {
        self.mount_table.mount(mount_point, options)
    }

    /// Unmount a filesystem
    pub fn unmount(&mut self, mount_point: &str) -> Option<Mount> {
        self.mount_table.unmount(mount_point)
    }

    /// List all mounts
    pub fn list_mounts(&self) -> Vec<(&String, &Mount)> {
        self.mount_table.list_mounts()
    }

    /// Resolve a path to a node
    pub fn resolve_path(&self, mount_point: &str, path: &str) -> Result<NodeRef, ResolveError> {
        let mount = self.mount_table.get_mount(mount_point)
            .ok_or_else(|| ResolveError::NotFound(format!("mount not found: {}", mount_point)))?;
        
        self.path_resolver.resolve_path(&mount.root, path)
    }

    /// Read file data
    pub fn read_file(&self, file_cid: &Cid, offset: u64, len: usize) -> Result<Vec<u8>, ReadError> {
        self.file_reader.read_file(file_cid, offset, len)
    }

    /// Get file information
    pub fn get_file_info(&self, file_cid: &Cid) -> Result<(u64, Vec<(Cid, u32)>), ReadError> {
        self.file_reader.get_file_info(file_cid)
    }

    /// Create a snapshot
    pub fn create_snapshot(&self, mount_point: &str, signer_did: &str) -> Result<SnapshotV1, SnapshotError> {
        let mount = self.mount_table.get_mount(mount_point)
            .ok_or_else(|| SnapshotError::MountNotFound(mount_point.to_string()))?;
        
        self.snapshot_builder.build_snapshot(mount_point, &mount.root, signer_did)
    }

    /// Verify a snapshot
    pub fn verify_snapshot(&self, snapshot: &SnapshotV1) -> Result<bool, SnapshotError> {
        self.snapshot_builder.verify_snapshot(snapshot)
    }

    /// Export IPFS map for a root CID
    pub fn export_ipfs_map(&self, root: &Cid, opts: &ExportOpts) -> Result<IpfsMapV1, IpfsExportError> {
        self.ipfs_exporter.export_ipfs_map(root, opts)
    }

    /// Write CAR file for exported content
    pub fn write_car<W: core::fmt::Write>(
        &self, 
        root: &Cid, 
        map: &IpfsMapV1, 
        out: &mut W
    ) -> Result<(), IpfsExportError> {
        self.ipfs_exporter.write_car(root, map, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::collections::BTreeMap;
    
    struct MockKeyVault;
    impl KeyVaultClient for MockKeyVault {
        fn derive_kek_for_kid(&self, _kid: &str) -> Result<Kek, EncError> {
            // Return a fixed test key
            Ok(Kek([42u8; 32]))
        }
    }

    #[test]
    fn test_ngfs_service_creation() {
        let temp_dir = tempdir().unwrap();
        let config = NgfsConfig {
            storage_path: temp_dir.path().to_string_lossy().to_string(),
            mount_salt: [1u8; 16],
            ..Default::default()
        };
        
        let keyvault = Box::new(MockKeyVault);
        let service = NgfsService::new(config, keyvault);
        assert!(service.is_ok());
    }

    #[test]
    fn test_ngfs_store_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let config = NgfsConfig {
            storage_path: temp_dir.path().to_string_lossy().to_string(),
            mount_salt: [1u8; 16],
            ..Default::default()
        };
        
        let keyvault = Box::new(MockKeyVault);
        let mut service = NgfsService::new(config, keyvault).unwrap();
        
        let test_data = b"NGFS service test data";
        let cid = service.store(test_data).unwrap();
        
        let retrieved_data = service.retrieve(&cid).unwrap();
        assert_eq!(test_data, retrieved_data.as_slice());
    }

    #[test]
    fn test_ngfs_encrypted_store_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let config = NgfsConfig {
            storage_path: temp_dir.path().to_string_lossy().to_string(),
            mount_salt: [1u8; 16],
            ..Default::default()
        };
        
        let keyvault = Box::new(MockKeyVault);
        let mut service = NgfsService::new(config, keyvault).unwrap();
        
        let test_data = b"NGFS encrypted test data";
        let mut metadata = BTreeMap::new();
        metadata.insert("test".to_string(), "value".to_string());
        
        let cid = service.store_encrypted("test-key".to_string(), test_data, metadata.clone()).unwrap();
        
        let retrieved_data = service.retrieve_encrypted(&cid, "test-key", metadata).unwrap();
        assert_eq!(test_data, retrieved_data.as_slice());
    }

    #[test]
    fn test_ngfs_stats() {
        let temp_dir = tempdir().unwrap();
        let config = NgfsConfig {
            storage_path: temp_dir.path().to_string_lossy().to_string(),
            mount_salt: [1u8; 16],
            ..Default::default()
        };
        
        let keyvault = Box::new(MockKeyVault);
        let mut service = NgfsService::new(config, keyvault).unwrap();
        
        // Store some data
        let data1 = b"Stats test data 1";
        let data2 = b"Stats test data 2";
        
        service.store(data1).unwrap();
        service.store(data2).unwrap();
        
        let stats = service.get_stats();
        assert_eq!(stats.total_chunks, 2);
        assert!(stats.total_segments >= 1);
    }

    #[test]
    fn test_ngfs_mount_and_resolve() {
        let temp_dir = tempdir().unwrap();
        let config = NgfsConfig {
            storage_path: temp_dir.path().to_string_lossy().to_string(),
            mount_salt: [1u8; 16],
            ..Default::default()
        };
        
        let keyvault = Box::new(MockKeyVault);
        let mut service = NgfsService::new(config, keyvault).unwrap();
        
        let mount_options = MountOptionsV1 {
            root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
            salt: [1u8; 16],
            vclock_base: 1000,
        };
        
        // Mount a filesystem
        let result = service.mount_ro("/ro/test", mount_options);
        assert!(result.is_ok());
        
        // List mounts
        let mounts = service.list_mounts();
        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].0, "/ro/test");
    }

    #[test]
    fn test_ngfs_snapshot() {
        let temp_dir = tempdir().unwrap();
        let config = NgfsConfig {
            storage_path: temp_dir.path().to_string_lossy().to_string(),
            mount_salt: [1u8; 16],
            ..Default::default()
        };
        
        let keyvault = Box::new(MockKeyVault);
        let service = NgfsService::new(config, keyvault).unwrap();
        
        let mount_options = MountOptionsV1 {
            root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
            salt: [1u8; 16],
            vclock_base: 1000,
        };
        
        // Create a snapshot
        let snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
        assert_eq!(snapshot.signer_did, "did:key:test");
        assert!(!snapshot.snap_id.is_empty());
        
        // Verify the snapshot
        let is_valid = service.verify_snapshot(&snapshot).unwrap();
        assert!(is_valid);
    }
}
